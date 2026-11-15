use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{Shell, Surface};

const MAX_PROFILE_BYTES: u64 = 64 * 1024;
const MAX_WORKSTREAMS: usize = 16;
const MAX_BINDINGS: usize = 16;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceProfile {
    pub version: u8,
    pub name: String,
    pub surfaces: Vec<Surface>,
    pub agora: Option<AgoraProfile>,
    pub chronos: Option<ChronosProfile>,
    pub vellum: Option<VellumProfile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgoraProfile {
    pub socket: PathBuf,
    pub participant: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChronosProfile {
    pub workstreams: Vec<WorkstreamProfile>,
    #[serde(default)]
    pub moira: Vec<MoiraBinding>,
    #[serde(default)]
    pub daimon: Vec<DaimonBinding>,
    pub planning: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkstreamProfile {
    pub name: String,
    pub directory: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MoiraBinding {
    pub workstream: String,
    pub socket: PathBuf,
    pub scope: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DaimonBinding {
    pub workstream: String,
    pub socket: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VellumProfile {
    pub socket: PathBuf,
    pub credential: Option<PathBuf>,
}

impl WorkspaceProfile {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_PROFILE_BYTES {
            return Err("workspace profile exceeds 64 KiB".into());
        }
        let bytes = fs::read(path)?;
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        Self::parse(&bytes, base)
    }

    pub fn parse(bytes: &[u8], base: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() as u64 > MAX_PROFILE_BYTES {
            return Err("workspace profile exceeds 64 KiB".into());
        }
        let mut profile: Self = serde_json::from_slice(bytes)?;
        profile.validate()?;
        if let Some(agora) = &mut profile.agora {
            agora.socket = resolve(base, &agora.socket);
        }
        if let Some(chronos) = &mut profile.chronos {
            for workstream in &mut chronos.workstreams {
                workstream.directory = resolve(base, &workstream.directory);
            }
            for binding in &mut chronos.moira {
                binding.socket = resolve(base, &binding.socket);
            }
            for binding in &mut chronos.daimon {
                binding.socket = resolve(base, &binding.socket);
            }
            if let Some(path) = &mut chronos.planning {
                *path = resolve(base, path);
            }
        }
        if let Some(vellum) = &mut profile.vellum {
            vellum.socket = resolve(base, &vellum.socket);
            if let Some(path) = &mut vellum.credential {
                *path = resolve(base, path);
            }
        }
        Ok(profile)
    }

    pub fn shell(&self) -> Result<Shell, &'static str> {
        Shell::new(self.surfaces.clone())
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.version != 1 {
            return Err("unsupported workspace profile version");
        }
        validate_text(&self.name, "workspace name")?;
        self.shell()?;
        if let Some(agora) = &self.agora {
            if !self.surfaces.contains(&Surface::Agora) {
                return Err("Agora configuration requires the Agora surface");
            }
            validate_path(&agora.socket)?;
            validate_text(&agora.participant, "Agora participant")?;
        }
        if self.surfaces.contains(&Surface::Workspace)
            && self.chronos.is_none()
            && self.vellum.is_none()
        {
            return Err("workspace surface requires Chronos or Vellum configuration");
        }
        if let Some(chronos) = &self.chronos {
            if chronos.workstreams.is_empty() || chronos.workstreams.len() > MAX_WORKSTREAMS {
                return Err("Chronos workstream count is outside the supported range");
            }
            if chronos.moira.len() > MAX_BINDINGS || chronos.daimon.len() > MAX_BINDINGS {
                return Err("Chronos peer binding count is outside the supported range");
            }
            for (index, workstream) in chronos.workstreams.iter().enumerate() {
                validate_text(&workstream.name, "workstream name")?;
                validate_path(&workstream.directory)?;
                if chronos.workstreams[..index]
                    .iter()
                    .any(|other| other.name == workstream.name)
                {
                    return Err("Chronos workstream names must be unique");
                }
            }
            for (index, binding) in chronos.moira.iter().enumerate() {
                validate_text(&binding.workstream, "Moira workstream")?;
                validate_text(&binding.scope, "Moira scope")?;
                validate_path(&binding.socket)?;
                validate_binding(
                    &binding.workstream,
                    chronos
                        .workstreams
                        .iter()
                        .map(|workstream| &workstream.name),
                )?;
                if chronos.moira[..index]
                    .iter()
                    .any(|other| other.workstream == binding.workstream)
                {
                    return Err("Moira workstream bindings must be unique");
                }
            }
            for (index, binding) in chronos.daimon.iter().enumerate() {
                validate_text(&binding.workstream, "Daimon workstream")?;
                validate_path(&binding.socket)?;
                validate_binding(
                    &binding.workstream,
                    chronos
                        .workstreams
                        .iter()
                        .map(|workstream| &workstream.name),
                )?;
                if chronos.daimon[..index]
                    .iter()
                    .any(|other| other.workstream == binding.workstream)
                {
                    return Err("Daimon workstream bindings must be unique");
                }
            }
            if let Some(path) = &chronos.planning {
                validate_path(path)?;
            }
        }
        if let Some(vellum) = &self.vellum {
            validate_path(&vellum.socket)?;
            if let Some(path) = &vellum.credential {
                validate_path(path)?;
            }
        }
        Ok(())
    }
}

fn validate_binding<'a>(
    binding: &str,
    workstreams: impl Iterator<Item = &'a String>,
) -> Result<(), &'static str> {
    if workstreams.into_iter().any(|name| name == binding) {
        Ok(())
    } else {
        Err("peer binding references an unknown Chronos workstream")
    }
}

fn validate_path(path: &Path) -> Result<(), &'static str> {
    if path.as_os_str().is_empty() {
        Err("workspace profile contains an empty path")
    } else {
        Ok(())
    }
}

fn validate_text(value: &str, field: &'static str) -> Result<(), &'static str> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(match field {
            "workspace name" => "workspace name is invalid",
            "workstream name" => "workstream name is invalid",
            "Moira workstream" => "Moira workstream name is invalid",
            "Moira scope" => "Moira scope is invalid",
            "Agora participant" => "Agora participant is invalid",
            _ => "Daimon workstream name is invalid",
        });
    }
    Ok(())
}

fn resolve(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_is_versioned_bounded_and_resolves_relative_paths() {
        let bytes = br#"{
          "version": 1,
          "name": "Local federation",
          "surfaces": ["overview", "workspace", "agora"],
          "agora": {"socket":"agora.sock", "participant":"alice"},
          "chronos": {
            "workstreams": [{"name":"Atlas", "directory":"chronos"}],
            "moira": [{"workstream":"Atlas", "socket":"moira.sock", "scope":"atlas"}],
            "daimon": [],
            "planning": "plan.json"
          },
          "vellum": {"socket":"vellum.sock", "credential":null}
        }"#;
        let profile =
            WorkspaceProfile::parse(bytes, Path::new("/tmp/workbench")).expect("valid profile");
        assert_eq!(profile.surfaces[2], Surface::Agora);
        assert_eq!(
            profile.agora.as_ref().unwrap().socket,
            Path::new("/tmp/workbench/agora.sock")
        );
        assert_eq!(
            profile.chronos.as_ref().unwrap().workstreams[0].directory,
            Path::new("/tmp/workbench/chronos")
        );
        assert_eq!(
            profile.vellum.as_ref().unwrap().socket,
            Path::new("/tmp/workbench/vellum.sock")
        );
    }

    #[test]
    fn profile_refuses_unknown_versions_duplicates_and_authority_flags() {
        let unknown = br#"{"version":2,"name":"x","surfaces":["overview"],"agora":null,"chronos":null,"vellum":null}"#;
        assert!(WorkspaceProfile::parse(unknown, Path::new(".")).is_err());
        let duplicate = br#"{"version":1,"name":"x","surfaces":["agora","agora"],"agora":null,"chronos":null,"vellum":null}"#;
        assert!(WorkspaceProfile::parse(duplicate, Path::new(".")).is_err());
        let authority = br#"{"version":1,"name":"x","surfaces":["overview"],"agora":null,"chronos":null,"vellum":null,"allow_writes":true}"#;
        assert!(WorkspaceProfile::parse(authority, Path::new(".")).is_err());
    }

    #[test]
    fn profile_refuses_ambiguous_or_unresolved_peer_bindings() {
        let duplicate_workstream = br#"{
          "version":1,"name":"x","surfaces":["workspace"],"agora":null,
          "chronos":{"workstreams":[{"name":"x","directory":"a"},{"name":"x","directory":"b"}],"moira":[],"daimon":[],"planning":null},
          "vellum":null
        }"#;
        assert!(WorkspaceProfile::parse(duplicate_workstream, Path::new(".")).is_err());
        let missing_workstream = br#"{
          "version":1,"name":"x","surfaces":["workspace"],"agora":null,
          "chronos":{"workstreams":[{"name":"x","directory":"a"}],"moira":[{"workstream":"y","socket":"m.sock","scope":"s"}],"daimon":[],"planning":null},
          "vellum":null
        }"#;
        assert!(WorkspaceProfile::parse(missing_workstream, Path::new(".")).is_err());
        let hidden_agora = br#"{
          "version":1,"name":"x","surfaces":["overview"],
          "agora":{"socket":"a.sock","participant":"alice"},
          "chronos":null,"vellum":null
        }"#;
        assert!(WorkspaceProfile::parse(hidden_agora, Path::new(".")).is_err());
    }
}
