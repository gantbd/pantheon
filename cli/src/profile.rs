use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use workbench_tui::{Shell, Surface};

const MAX_PROFILE_BYTES: u64 = 64 * 1024;
const MAX_WORKSTREAMS: usize = 16;
const MAX_BINDINGS: usize = 16;
const MAX_REQUIRED_CAPABILITIES: usize = 16;
const MAX_CAPABILITY_BYTES: usize = 64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceProfile {
    pub version: u8,
    pub name: String,
    pub surfaces: Vec<Surface>,
    pub agora: Option<AgoraProfile>,
    pub moira: Option<MoiraProfile>,
    pub daimon: Option<DaimonProfile>,
    pub hermes: Option<HermesProfile>,
    pub aegis: Option<AegisProfile>,
    pub grounding: Option<GroundingProfile>,
    pub chronos: Option<ChronosProfile>,
    pub vellum: Option<VellumProfile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgoraProfile {
    pub socket: PathBuf,
    pub participant: String,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MoiraProfile {
    pub socket: PathBuf,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DaimonProfile {
    pub socket: PathBuf,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HermesProfile {
    #[serde(default)]
    pub deployment: DeploymentMode,
    pub socket: PathBuf,
    pub stream: Option<String>,
    pub consumer: Option<String>,
    pub host: Option<HermesHostProfile>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HermesHostProfile {
    pub executable: PathBuf,
    pub directory: PathBuf,
    pub mode: HermesOpenMode,
    pub effective_uid: u32,
    pub effective_gid: u32,
    pub subject: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HermesOpenMode {
    Create,
    Open,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AegisProfile {
    #[serde(default)]
    pub deployment: DeploymentMode,
    pub socket: PathBuf,
    pub host: Option<AegisHostProfile>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AegisHostProfile {
    pub executable: PathBuf,
    pub directory: PathBuf,
    pub mode: AegisOpenMode,
    pub authority: String,
    pub effective_uid: u32,
    pub effective_gid: u32,
    pub principal: String,
    pub session_namespace: String,
    pub role: AegisRole,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AegisOpenMode {
    Create,
    Open,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AegisRole {
    Administrator,
    Principal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GroundingProfile {
    pub endpoint: SocketAddr,
    pub token: Option<PathBuf>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChronosProfile {
    #[serde(default)]
    pub deployment: DeploymentMode,
    pub workstreams: Vec<WorkstreamProfile>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub moira: Vec<MoiraBinding>,
    #[serde(default)]
    pub daimon: Vec<DaimonBinding>,
    pub planning: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMode {
    #[default]
    Attach,
    Hosted,
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
    #[serde(default)]
    pub deployment: DeploymentMode,
    pub socket: PathBuf,
    pub credential: Option<PathBuf>,
    #[serde(default)]
    pub roots: Vec<PathBuf>,
    #[serde(default)]
    pub requires: Vec<String>,
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
        if let Some(moira) = &mut profile.moira {
            moira.socket = resolve(base, &moira.socket);
        }
        if let Some(daimon) = &mut profile.daimon {
            daimon.socket = resolve(base, &daimon.socket);
        }
        if let Some(hermes) = &mut profile.hermes {
            hermes.socket = resolve(base, &hermes.socket);
            if let Some(host) = &mut hermes.host {
                host.executable = resolve(base, &host.executable);
                host.directory = resolve(base, &host.directory);
            }
        }
        if let Some(aegis) = &mut profile.aegis {
            aegis.socket = resolve(base, &aegis.socket);
            if let Some(host) = &mut aegis.host {
                host.executable = resolve(base, &host.executable);
                host.directory = resolve(base, &host.directory);
            }
        }
        if let Some(grounding) = &mut profile.grounding
            && let Some(token) = &mut grounding.token
        {
            *token = resolve(base, token);
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
            for root in &mut vellum.roots {
                *root = resolve(base, root);
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
            validate_required_capabilities(&agora.requires)?;
        }
        if let Some(moira) = &self.moira {
            if !self.surfaces.contains(&Surface::Moira) {
                return Err("Moira configuration requires the Moira surface");
            }
            validate_path(&moira.socket)?;
            validate_required_capabilities(&moira.requires)?;
        }
        if let Some(daimon) = &self.daimon {
            if !self.surfaces.contains(&Surface::Daimon) {
                return Err("Daimon configuration requires the Daimon surface");
            }
            validate_path(&daimon.socket)?;
            validate_required_capabilities(&daimon.requires)?;
        }
        if let Some(hermes) = &self.hermes {
            if !self.surfaces.contains(&Surface::Hermes) {
                return Err("Hermes configuration requires the Hermes surface");
            }
            validate_path(&hermes.socket)?;
            match (hermes.deployment, &hermes.host) {
                (DeploymentMode::Hosted, Some(host)) => {
                    validate_path(&host.executable)?;
                    validate_path(&host.directory)?;
                    if host.subject.len() != 32
                        || !host
                            .subject
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                    {
                        return Err(
                            "hosted Hermes subject must be 32 lowercase hexadecimal digits",
                        );
                    }
                }
                (DeploymentMode::Hosted, None) => {
                    return Err("hosted Hermes requires an explicit host configuration");
                }
                (DeploymentMode::Attach, Some(_)) => {
                    return Err("attached Hermes cannot declare a host configuration");
                }
                (DeploymentMode::Attach, None) => {}
            }
            if let Some(stream) = &hermes.stream
                && hermes_tui::parse_stream_id(stream).is_none()
            {
                return Err("Hermes stream must be 32 hexadecimal digits");
            }
            if let Some(consumer) = &hermes.consumer {
                if hermes.stream.is_none() {
                    return Err("Hermes consumer requires a stream");
                }
                if hermes_tui::parse_consumer_id(consumer).is_none() {
                    return Err("Hermes consumer must be 32 hexadecimal digits");
                }
            }
            validate_required_capabilities(&hermes.requires)?;
        }
        if let Some(aegis) = &self.aegis {
            if !self.surfaces.contains(&Surface::Aegis) {
                return Err("Aegis configuration requires the Aegis surface");
            }
            validate_path(&aegis.socket)?;
            match (aegis.deployment, &aegis.host) {
                (DeploymentMode::Hosted, Some(host)) => {
                    validate_path(&host.executable)?;
                    validate_path(&host.directory)?;
                    validate_lower_hex(&host.authority, 32, "hosted Aegis authority")?;
                    validate_lower_hex(&host.principal, 32, "hosted Aegis principal")?;
                    validate_lower_hex(
                        &host.session_namespace,
                        16,
                        "hosted Aegis session namespace",
                    )?;
                }
                (DeploymentMode::Hosted, None) => {
                    return Err("hosted Aegis requires an explicit host configuration");
                }
                (DeploymentMode::Attach, Some(_)) => {
                    return Err("attached Aegis cannot declare a host configuration");
                }
                (DeploymentMode::Attach, None) => {}
            }
            validate_required_capabilities(&aegis.requires)?;
        }
        if let Some(grounding) = &self.grounding {
            if !self.surfaces.contains(&Surface::Grounding) {
                return Err("Grounding configuration requires the Grounding surface");
            }
            if !grounding.endpoint.ip().is_loopback() {
                return Err("Grounding endpoint must be loopback");
            }
            if let Some(token) = &grounding.token {
                validate_path(token)?;
            }
            validate_required_capabilities(&grounding.requires)?;
        }
        if self.surfaces.contains(&Surface::Workspace)
            && self.chronos.is_none()
            && self.vellum.is_none()
        {
            return Err("workspace surface requires Chronos or Vellum configuration");
        }
        if let Some(chronos) = &self.chronos {
            validate_required_capabilities(&chronos.requires)?;
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
            validate_required_capabilities(&vellum.requires)?;
            if let Some(path) = &vellum.credential {
                validate_path(path)?;
            }
            if vellum.roots.len() > 16 {
                return Err("Vellum root count exceeds the supported range");
            }
            for root in &vellum.roots {
                validate_path(root)?;
            }
            if vellum.deployment == DeploymentMode::Hosted && vellum.roots.is_empty() {
                return Err("hosted Vellum requires at least one workspace root");
            }
            if vellum.deployment == DeploymentMode::Attach && !vellum.roots.is_empty() {
                return Err("attached Vellum cannot declare hosted workspace roots");
            }
        }
        Ok(())
    }
}

fn validate_required_capabilities(values: &[String]) -> Result<(), &'static str> {
    if values.len() > MAX_REQUIRED_CAPABILITIES {
        return Err("required capability count exceeds the supported range");
    }
    for (index, value) in values.iter().enumerate() {
        if value.is_empty()
            || value.len() > MAX_CAPABILITY_BYTES
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.')
        {
            return Err("required capability identifier is invalid");
        }
        if values[..index].contains(value) {
            return Err("required capabilities must be unique");
        }
    }
    Ok(())
}

fn validate_lower_hex(value: &str, length: usize, field: &'static str) -> Result<(), &'static str> {
    if value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(match field {
            "hosted Aegis authority" => {
                "hosted Aegis authority must be 32 lowercase hexadecimal digits"
            }
            "hosted Aegis principal" => {
                "hosted Aegis principal must be 32 lowercase hexadecimal digits"
            }
            _ => "hosted Aegis session namespace must be 16 lowercase hexadecimal digits",
        })
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
          "agora": {"socket":"agora.sock", "participant":"alice", "requires":["rooms.read.v2"]},
          "chronos": {
            "workstreams": [{"name":"Atlas", "directory":"chronos"}],
            "requires": ["tasks.observe.v1"],
            "moira": [{"workstream":"Atlas", "socket":"moira.sock", "scope":"atlas"}],
            "daimon": [],
            "planning": "plan.json"
          },
          "vellum": {"socket":"vellum.sock", "credential":null, "requires":["workspace.inventory.v1"]}
        }"#;
        let profile =
            WorkspaceProfile::parse(bytes, Path::new("/tmp/workbench")).expect("valid profile");
        assert_eq!(profile.surfaces[2], Surface::Agora);
        assert_eq!(profile.agora.as_ref().unwrap().requires, ["rooms.read.v2"]);
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
    fn profile_accepts_exact_opaque_hermes_stream_and_consumer_selectors() {
        let valid = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"socket":"hermes.sock","stream":"05050505050505050505050505050505","consumer":"06060606060606060606060606060606"}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let hermes = profile.hermes.unwrap();
        assert_eq!(
            hermes.stream.as_deref(),
            Some("05050505050505050505050505050505")
        );
        assert_eq!(
            hermes.consumer.as_deref(),
            Some("06060606060606060606060606060606")
        );
        let invalid = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"socket":"hermes.sock","stream":"not-an-opaque-id"}
        }"#;
        assert!(WorkspaceProfile::parse(invalid, Path::new(".")).is_err());
        let consumer_without_stream = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"socket":"hermes.sock","stream":null,"consumer":"06060606060606060606060606060606"}
        }"#;
        assert!(WorkspaceProfile::parse(consumer_without_stream, Path::new(".")).is_err());
    }

    #[test]
    fn hosted_hermes_requires_explicit_owner_inputs_and_canonical_subject() {
        let valid = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"deployment":"hosted","socket":"hermes.sock","stream":null,"consumer":null,
            "host":{"executable":"bin/hermes-daemon","directory":"ledger","mode":"create",
              "effective_uid":501,"effective_gid":20,"subject":"07070707070707070707070707070707"}}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let host = profile.hermes.unwrap().host.unwrap();
        assert_eq!(
            host.executable,
            Path::new("/tmp/workbench/bin/hermes-daemon")
        );
        assert_eq!(host.directory, Path::new("/tmp/workbench/ledger"));

        let missing_host = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"deployment":"hosted","socket":"hermes.sock","stream":null,"consumer":null}
        }"#;
        assert!(WorkspaceProfile::parse(missing_host, Path::new(".")).is_err());
        let attached_host = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"socket":"hermes.sock","stream":null,"consumer":null,
            "host":{"executable":"hermes-daemon","directory":"ledger","mode":"open",
              "effective_uid":501,"effective_gid":20,"subject":"07070707070707070707070707070707"}}
        }"#;
        assert!(WorkspaceProfile::parse(attached_host, Path::new(".")).is_err());
    }

    #[test]
    fn hosted_aegis_requires_explicit_owner_inputs_and_canonical_authority_facts() {
        let valid = br#"{
          "version":1,"name":"Aegis","surfaces":["aegis"],
          "aegis":{"deployment":"hosted","socket":"aegis.sock",
            "host":{"executable":"bin/aegis-daemon","directory":"authority","mode":"create",
              "authority":"01010101010101010101010101010101",
              "effective_uid":501,"effective_gid":20,
              "principal":"02020202020202020202020202020202",
              "session_namespace":"0303030303030303","role":"administrator"}}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let host = profile.aegis.unwrap().host.unwrap();
        assert_eq!(
            host.executable,
            Path::new("/tmp/workbench/bin/aegis-daemon")
        );
        assert_eq!(host.directory, Path::new("/tmp/workbench/authority"));
        assert_eq!(host.role, AegisRole::Administrator);

        let missing_host = br#"{
          "version":1,"name":"Aegis","surfaces":["aegis"],
          "aegis":{"deployment":"hosted","socket":"aegis.sock"}
        }"#;
        assert!(WorkspaceProfile::parse(missing_host, Path::new(".")).is_err());
        let invalid_authority = br#"{
          "version":1,"name":"Aegis","surfaces":["aegis"],
          "aegis":{"deployment":"hosted","socket":"aegis.sock",
            "host":{"executable":"aegis-daemon","directory":"authority","mode":"open",
              "authority":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
              "effective_uid":501,"effective_gid":20,
              "principal":"02020202020202020202020202020202",
              "session_namespace":"0303030303030303","role":"principal"}}
        }"#;
        assert!(WorkspaceProfile::parse(invalid_authority, Path::new(".")).is_err());
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

    #[test]
    fn profile_refuses_invalid_or_duplicate_capability_requirements() {
        let invalid = br#"{
          "version":1,"name":"x","surfaces":["agora"],
          "agora":{"socket":"a.sock","participant":"alice","requires":["Rooms Read"]},
          "chronos":null,"vellum":null
        }"#;
        assert!(WorkspaceProfile::parse(invalid, Path::new(".")).is_err());
        let duplicate = br#"{
          "version":1,"name":"x","surfaces":["agora"],
          "agora":{"socket":"a.sock","participant":"alice","requires":["rooms.read.v2","rooms.read.v2"]},
          "chronos":null,"vellum":null
        }"#;
        assert!(WorkspaceProfile::parse(duplicate, Path::new(".")).is_err());
    }

    #[test]
    fn profile_requires_hosted_vellum_roots_and_rejects_attached_roots() {
        let hosted_without_roots = br#"{
          "version":1,"name":"x","surfaces":["workspace"],"agora":null,"chronos":null,
          "vellum":{"deployment":"hosted","socket":"v.sock","credential":null,"roots":[]}
        }"#;
        assert!(WorkspaceProfile::parse(hosted_without_roots, Path::new(".")).is_err());
        let attached_with_roots = br#"{
          "version":1,"name":"x","surfaces":["workspace"],"agora":null,"chronos":null,
          "vellum":{"deployment":"attach","socket":"v.sock","credential":null,"roots":["a"]}
        }"#;
        assert!(WorkspaceProfile::parse(attached_with_roots, Path::new(".")).is_err());
    }
}
