//! Headless presentation of Tellus-owned, Dendrite-derived SME plans.

use serde::Deserialize;
use serde_json::{Value, json};
use std::{ffi::OsString, net::SocketAddr, path::PathBuf, time::Duration};
use tellus::{GroundingDepthV1, SmeEntryKindV1, SmePlanPort, SmePlanRequestV1, SmePlanStatusV1};
use tellus_local_client::{ClientError, LocalPort};

pub const CAPABILITY: &str = "tellus.sme-plan.v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestDocumentV1 {
    version: u8,
    fold_load_id: String,
    lattice_load_id: String,
    tags: Vec<String>,
    depth: String,
    max_roles: u8,
    max_surface_bytes: u32,
}

pub fn plan(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = required_text(&mut args, "SME plan requires ENDPOINT REQUEST_JSON")?
        .parse::<SocketAddr>()?;
    let document = PathBuf::from(required(
        &mut args,
        "SME plan requires ENDPOINT REQUEST_JSON",
    )?);
    let token = match args.next() {
        Some(flag) if flag == "--token-file" => {
            let path = PathBuf::from(required(&mut args, "--token-file requires PATH")?);
            if std::fs::metadata(&path)?.len() > 8_192 {
                return Err("Tellus token file exceeds 8 KiB".into());
            }
            Some(std::fs::read_to_string(path)?.trim_end().to_owned())
        }
        Some(_) => return Err("expected --token-file PATH after REQUEST_JSON".into()),
        None => None,
    };
    no_more(&mut args)?;
    let request = read_request(&document)?;
    let mut port = match LocalPort::new(endpoint, token, Duration::from_secs(5)) {
        Ok(port) => port,
        Err(error) => return transport_failure(error),
    };
    match port.plan_smes(&request) {
        Ok(plan) => {
            let status = match plan.status {
                SmePlanStatusV1::Current => "current",
                SmePlanStatusV1::ReviewRequired => "review_required",
                SmePlanStatusV1::Unmatched => "unmatched",
            };
            emit(json!({
                "version": 1, "peer": "tellus", "capability": CAPABILITY,
                "complete": plan.complete, "status": status,
                "ensemble_question": tellus::SME_ENSEMBLE_QUESTION,
                "next_action": plan.next_action,
                "fold": source_json(&plan.fold), "lattice": source_json(&plan.lattice),
                "tags": plan.tags,
                "roles": plan.roles.into_iter().map(|role| json!({
                    "role": role.role, "ref": role.reference,
                    "authority_sha256": hex(&role.authority_sha256),
                    "surface_sha256": hex(&role.surface_sha256),
                    "depth": role.depth.as_str(), "surface_version": role.surface_version,
                    "entries": role.entries.into_iter().map(|entry| json!({
                        "id": entry.id, "kind": entry_kind(entry.kind), "text": entry.text,
                    })).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
                "orthogonality": {
                    "all_orthogonal": plan.orthogonality.all_orthogonal,
                    "sufficient_roles": plan.orthogonality.sufficient_roles,
                    "pairs": plan.orthogonality.pairs.into_iter().map(|pair| json!({
                        "role1": pair.role1, "role2": pair.role2,
                        "orthogonal": pair.orthogonal,
                        "facet1": pair.facet1, "facet2": pair.facet2,
                    })).collect::<Vec<_>>(),
                },
                "boundary": tellus::SME_ADVISORY_BOUNDARY,
            }))
        }
        Err(error) => transport_failure(error),
    }
}

fn read_request(path: &PathBuf) -> Result<SmePlanRequestV1, Box<dyn std::error::Error>> {
    if std::fs::metadata(path)?.len() > 16 * 1024 {
        return Err("SME plan request exceeds 16 KiB".into());
    }
    let document: RequestDocumentV1 = serde_json::from_slice(&std::fs::read(path)?)?;
    if document.version != 1 {
        return Err("SME plan document version must be 1".into());
    }
    let depth = match document.depth.as_str() {
        "ambient" => GroundingDepthV1::Ambient,
        "activated" => GroundingDepthV1::Activated,
        "reference" => GroundingDepthV1::Reference,
        _ => return Err("SME plan depth must be ambient, activated, or reference".into()),
    };
    Ok(SmePlanRequestV1 {
        fold_load_id: document.fold_load_id,
        lattice_load_id: document.lattice_load_id,
        tags: document.tags,
        depth,
        max_roles: document.max_roles,
        max_surface_bytes: document.max_surface_bytes,
    })
}

fn source_json(source: &tellus::SmePlanSourceV1) -> Value {
    json!({"load_id": source.load_id, "sha256": hex(&source.sha256)})
}

const fn entry_kind(kind: SmeEntryKindV1) -> &'static str {
    match kind {
        SmeEntryKindV1::Guidance => "guidance",
        SmeEntryKindV1::Question => "question",
        SmeEntryKindV1::Hazard => "hazard",
        SmeEntryKindV1::Probe => "probe",
        SmeEntryKindV1::Reference => "reference",
    }
}

fn transport_failure(value: ClientError) -> Result<(), Box<dyn std::error::Error>> {
    let (status, next_action) = match value {
        ClientError::NotSent(_) => (
            "not_sent",
            "verify the selected loopback Tellus endpoint and retry the read-only request",
        ),
        ClientError::OutcomeUnknown(_) => (
            "outcome_unknown",
            "the operation is read-only; re-observe after checking endpoint health",
        ),
        ClientError::InvalidResponse => (
            "invalid_reply",
            "stop admission and inspect Tellus/Dendrite version and provenance compatibility",
        ),
        ClientError::Refused(_) => (
            "refused",
            "verify Tellus authentication and the exact bounded request",
        ),
    };
    emit(json!({
        "version": 1, "peer": "tellus", "capability": CAPABILITY,
        "complete": false, "status": status, "detail": format!("{value:?}"),
        "next_action": next_action,
        "boundary": "no frontier session should be admitted from missing, refused, or invalid SME evidence",
    }))?;
    Err(std::io::Error::other("Tellus SME plan unavailable").into())
}

fn emit(value: Value) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn required(
    args: &mut impl Iterator<Item = OsString>,
    message: &'static str,
) -> Result<OsString, Box<dyn std::error::Error>> {
    args.next().ok_or_else(|| message.into())
}

fn required_text(
    args: &mut impl Iterator<Item = OsString>,
    message: &'static str,
) -> Result<String, Box<dyn std::error::Error>> {
    required(args, message)?
        .into_string()
        .map_err(|_| message.into())
}

fn no_more(args: &mut impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    if args.next().is_some() {
        return Err("unexpected SME plan argument".into());
    }
    Ok(())
}
