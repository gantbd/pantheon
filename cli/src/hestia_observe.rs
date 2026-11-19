//! Exact Hestia owner observations for bounded agent tool profiles.

use hestia_local_client::LocalClient;
use hestia_protocol::{ArtifactIdV1, RuntimeIdV1};
use serde_json::{Value, json};
use std::{ffi::OsString, path::PathBuf};

use crate::hestia::{artifact_observation_projection, runtime_observation_projection};

pub const RUNTIME_CAPABILITY: &str = "runtimes.observe.v1";
pub const ARTIFACT_CAPABILITY: &str = "artifacts.observe.v1";
pub const CATALOG_CAPABILITY: &str = "artifacts.catalog.v1";
const MAX_CATALOG_ITEMS: usize = 128;

pub fn runtime(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "Hestia runtime observation requires SOCKET RUNTIME_HEX";
    let socket = PathBuf::from(required(&mut args, usage)?);
    let runtime_text = required_text(&mut args, usage)?;
    no_more(&mut args, usage)?;
    let runtime = RuntimeIdV1(parse_lower_hex_16(&runtime_text)?);
    match LocalClient::new(socket).observe(runtime) {
        Ok(Some(observation)) if observation.runtime == runtime => emit_success(
            RUNTIME_CAPABILITY,
            json!({
                "runtime": runtime_text,
                "observation": runtime_observation_projection(&observation),
            }),
            "one exact Hestia runtime observation; no artifact acquisition, placement, lifecycle, provider qualification, inference, task, shell, path, environment, or process authority is transferred",
        ),
        Ok(Some(_)) => failure(
            RUNTIME_CAPABILITY,
            "invalid_reply",
            "Hestia returned a different runtime identity",
            "stop agent admission and inspect owner protocol compatibility",
        ),
        Ok(None) => failure(
            RUNTIME_CAPABILITY,
            "not_found",
            "Hestia does not own the exact runtime identity",
            "select a current owner-issued runtime identity; do not enumerate or substitute",
        ),
        Err(error) => transport_failure(RUNTIME_CAPABILITY, error),
    }
}

pub fn artifact(
    mut args: impl Iterator<Item = OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "Hestia artifact observation requires SOCKET ARTIFACT_ID";
    let socket = PathBuf::from(required(&mut args, usage)?);
    let artifact_text = required_text(&mut args, usage)?;
    no_more(&mut args, usage)?;
    let artifact = ArtifactIdV1::new(artifact_text.clone())?;
    match LocalClient::new(socket).observe_artifact(artifact) {
        Ok(Some(observation)) if observation.entry.artifact.as_str() == artifact_text => {
            emit_success(
                ARTIFACT_CAPABILITY,
                json!({
                    "artifact": artifact_text,
                    "observation": artifact_observation_projection(&observation),
                }),
                "one exact Hestia artifact observation; registry locator, source, storage path, downloader, acquisition, lifecycle, provider, inference, and task authority remain unavailable",
            )
        }
        Ok(Some(_)) => failure(
            ARTIFACT_CAPABILITY,
            "invalid_reply",
            "Hestia returned a different artifact identity",
            "stop agent admission and inspect owner protocol compatibility",
        ),
        Ok(None) => failure(
            ARTIFACT_CAPABILITY,
            "not_found",
            "Hestia does not own the exact artifact identity",
            "select a current owner-issued artifact identity; do not infer a registry locator",
        ),
        Err(error) => transport_failure(ARTIFACT_CAPABILITY, error),
    }
}

pub fn catalog(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "Hestia artifact catalogue requires SOCKET";
    let socket = PathBuf::from(required(&mut args, usage)?);
    no_more(&mut args, usage)?;
    match LocalClient::new(socket).catalog_artifacts() {
        Ok(catalog) if catalog.len() <= MAX_CATALOG_ITEMS => emit_success(
            CATALOG_CAPABILITY,
            json!({
                "count": catalog.len(),
                "catalog": catalog.iter().map(artifact_observation_projection).collect::<Vec<_>>(),
            }),
            "bounded Hestia owner catalogue only; registry locators, sources, storage paths, downloaders, acquisition, lifecycle, provider, inference, and task authority remain unavailable",
        ),
        Ok(catalog) => failure(
            CATALOG_CAPABILITY,
            "response_limit",
            &format!(
                "Hestia returned {} artifacts; the bounded agent limit is {MAX_CATALOG_ITEMS}",
                catalog.len()
            ),
            "have the operator narrow the deployment catalogue; no partial catalogue is emitted",
        ),
        Err(error) => transport_failure(CATALOG_CAPABILITY, error),
    }
}

fn emit_success(
    capability: &'static str,
    evidence: Value,
    boundary: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "hestia",
            "capability": capability,
            "complete": true,
            "status": "current",
            "evidence": evidence,
            "boundary": boundary,
        }))?
    );
    Ok(())
}

fn transport_failure(
    capability: &'static str,
    error: hestia_local_client::LocalClientError,
) -> Result<(), Box<dyn std::error::Error>> {
    failure(
        capability,
        "owner_unavailable",
        &format!("Hestia observation failed: {error:?}"),
        "verify the deployment-fixed Hestia owner socket and retry the read-only observation",
    )
}

fn failure(
    capability: &'static str,
    status: &'static str,
    detail: &str,
    next_action: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "hestia",
            "capability": capability,
            "complete": false,
            "status": status,
            "detail": detail,
            "next_action": next_action,
            "boundary": "no aggregate observation, owner enumeration, mutation, download, lifecycle, provider, inference, shell, path, environment, or process fallback is permitted",
        }))?
    );
    Err(std::io::Error::other("Hestia observation unavailable").into())
}

fn required(
    args: &mut impl Iterator<Item = OsString>,
    usage: &'static str,
) -> Result<OsString, Box<dyn std::error::Error>> {
    args.next().ok_or_else(|| usage.into())
}

fn required_text(
    args: &mut impl Iterator<Item = OsString>,
    usage: &'static str,
) -> Result<String, Box<dyn std::error::Error>> {
    required(args, usage)?
        .into_string()
        .map_err(|_| usage.into())
}

fn no_more(
    args: &mut impl Iterator<Item = OsString>,
    usage: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.next().is_some() {
        Err(usage.into())
    } else {
        Ok(())
    }
}

fn parse_lower_hex_16(value: &str) -> Result<[u8; 16], Box<dyn std::error::Error>> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value))
    {
        return Err("Hestia runtime must be exactly 32 lowercase hex characters".into());
    }
    let mut result = [0_u8; 16];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_identity_is_exact_lowercase_hex() {
        assert_eq!(parse_lower_hex_16(&"01".repeat(16)).unwrap(), [1; 16]);
        assert!(parse_lower_hex_16(&"AB".repeat(16)).is_err());
        assert!(parse_lower_hex_16("01").is_err());
    }
}
