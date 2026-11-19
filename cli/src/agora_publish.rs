//! Durable, actor-bound Agora publication for independently admitted agents.

use std::{
    ffi::OsString,
    fs::OpenOptions,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    time::Duration,
};

use agora_actor_publication_port::{
    ActorPublicationIdV1, ActorPublicationIntentV1, ActorPublicationObservationV1,
    ActorPublicationRequestV1, ActorPublicationWireClientV1,
};
use agora_local_client::{LocalCallErrorV1, ScheduledLocalClientV1};
use agora_wire::WireLimits;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

pub const EXECUTE_CAPABILITY: &str = "messages.actor-publication.execute.v1";
pub const OBSERVE_CAPABILITY: &str = "messages.actor-publication.observe.v1";
const MAX_REQUEST_BYTES: u64 = 64 * 1_024;
const MAX_BODY_BYTES: usize = 16 * 1_024;
const MAX_ADDRESSEES: usize = 64;
const MAX_ID_BYTES: usize = 128;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicationDocumentV1 {
    version: u8,
    operation_id: String,
    room_id: String,
    expected_revision: u64,
    logical_time: u64,
    body: String,
    addressed_to: Vec<String>,
}

struct LoadedPublication {
    request: ActorPublicationRequestV1,
    operation_id: String,
    room_id: String,
    expected_revision: u64,
    logical_time: u64,
    body_bytes: usize,
    addressed_count: usize,
    request_digest: String,
}

pub fn execute(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "Agora actor publication requires SOCKET PARTICIPANT REQUEST_JSON 'PUBLISH AGORA ACTOR MESSAGE OPERATION … REQUEST SHA256 …'";
    let socket = PathBuf::from(required(&mut args, usage)?);
    let participant = required_text(&mut args, usage)?;
    let request_path = PathBuf::from(required(&mut args, usage)?);
    let confirmation = required_text(&mut args, usage)?;
    no_more(&mut args, usage)?;
    let loaded = load_publication(&request_path, EXECUTE_CAPABILITY)?;
    let required = required_confirmation(&loaded);
    if confirmation != required {
        return failure(
            EXECUTE_CAPABILITY,
            "confirmation_refused",
            "exact actor-publication confirmation required",
            Some(&required),
            "review the exact private request document and use the returned confirmation; no owner connection was attempted",
        );
    }
    let facts = publication_facts(&loaded);
    let mut client = client(socket, participant);
    match client.publish_actor_wire(loaded.request) {
        Ok(Ok(receipt)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "version": 1,
                    "peer": "agora",
                    "capability": EXECUTE_CAPABILITY,
                    "complete": true,
                    "status": "accepted",
                    "request": facts,
                    "receipt": {
                        "revision": receipt.revision,
                        "message_id": hex(&receipt.message_id),
                        "message_sequence": receipt.message_sequence,
                    },
                    "boundary": "one exact actor-bound durable publication; the operation identity grants no room, membership, moderation, task, provider, shell, filesystem, or retry authority",
                }))?
            );
            Ok(())
        }
        Ok(Err(refusal)) => failure(
            EXECUTE_CAPABILITY,
            "refused",
            &format!("Agora refused actor publication: {refusal:?}"),
            None,
            "observe the exact operation before any retry; revise intent only under a fresh operation identity",
        ),
        Err(error) => transport_failure(EXECUTE_CAPABILITY, error, true),
    }
}

pub fn observe(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "Agora actor-publication observation requires SOCKET PARTICIPANT REQUEST_JSON";
    let socket = PathBuf::from(required(&mut args, usage)?);
    let participant = required_text(&mut args, usage)?;
    let request_path = PathBuf::from(required(&mut args, usage)?);
    no_more(&mut args, usage)?;
    let loaded = load_publication(&request_path, OBSERVE_CAPABILITY)?;
    let facts = publication_facts(&loaded);
    let mut client = client(socket, participant);
    match client.observe_actor_wire(loaded.request) {
        Ok(Ok(ActorPublicationObservationV1::Accepted(receipt))) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "version": 1,
                    "peer": "agora",
                    "capability": OBSERVE_CAPABILITY,
                    "complete": true,
                    "status": "accepted",
                    "request": facts,
                    "receipt": {
                        "revision": receipt.revision,
                        "message_id": hex(&receipt.message_id),
                        "message_sequence": receipt.message_sequence,
                    },
                    "boundary": "owner-retained evidence for one exact actor-bound publication identity; observation grants no publication or retry authority",
                }))?
            );
            Ok(())
        }
        Ok(Ok(ActorPublicationObservationV1::Absent)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "version": 1,
                    "peer": "agora",
                    "capability": OBSERVE_CAPABILITY,
                    "complete": true,
                    "status": "absent",
                    "request": facts,
                    "next_action": "execute the exact same request only if the intent remains desired and its separately admitted publication capability is current",
                    "boundary": "Agora authoritatively retains every accepted actor-publication identity and backpressures before mutation when full; absence is not publication authority",
                }))?
            );
            Ok(())
        }
        Ok(Err(refusal)) => failure(
            OBSERVE_CAPABILITY,
            "refused",
            &format!("Agora refused actor-publication observation: {refusal:?}"),
            None,
            "verify the exact operation, request intent, authenticated participant, and owner endpoint",
        ),
        Err(error) => transport_failure(OBSERVE_CAPABILITY, error, false),
    }
}

fn load_publication(
    path: &Path,
    capability: &'static str,
) -> Result<LoadedPublication, Box<dyn std::error::Error>> {
    let mut file = match OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW)
        .open(path)
    {
        Ok(file) => file,
        Err(error) => {
            return failure(
                capability,
                "request_refused",
                &format!("private request document unavailable: {error}"),
                None,
                "stage one private regular request document and retry before connecting to Agora",
            );
        }
    };
    let metadata = file.metadata()?;
    if !metadata.file_type().is_file()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.nlink() != 1
        || metadata.len() == 0
        || metadata.len() > MAX_REQUEST_BYTES
    {
        return failure(
            capability,
            "request_refused",
            "request document must be one nonempty private regular file of at most 65536 bytes",
            None,
            "stage the exact request with owner-only permissions and no hard or symbolic links",
        );
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.by_ref()
        .take(MAX_REQUEST_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 != metadata.len() {
        return failure(
            capability,
            "request_refused",
            "request document changed while it was being read",
            None,
            "restage an immutable private request document",
        );
    }
    let request_digest = hex(&Sha256::digest(&bytes));
    let document: PublicationDocumentV1 = match serde_json::from_slice(&bytes) {
        Ok(document) => document,
        Err(error) => {
            return failure(
                capability,
                "request_refused",
                &format!("request document is not bounded version-1 JSON: {error}"),
                None,
                "provide only version, operation_id, room_id, expected_revision, logical_time, body, and addressed_to",
            );
        }
    };
    validate_document(&document, capability)?;
    let operation_bytes = match parse_lower_hex_16(&document.operation_id) {
        Ok(operation) => operation,
        Err(_) => {
            return failure(
                capability,
                "request_refused",
                "operation_id must be exactly 32 lowercase hexadecimal characters",
                None,
                "issue one fresh nonzero opaque actor-publication identity",
            );
        }
    };
    let operation = match ActorPublicationIdV1::new(operation_bytes) {
        Ok(operation) => operation,
        Err(_) => {
            return failure(
                capability,
                "request_refused",
                "operation_id must not be the zero identity",
                None,
                "issue one fresh nonzero opaque actor-publication identity",
            );
        }
    };
    let loaded = LoadedPublication {
        request: ActorPublicationRequestV1 {
            id: operation,
            intent: ActorPublicationIntentV1::new(
                document.room_id.clone(),
                document.expected_revision,
                document.logical_time,
                document.body.clone(),
                document.addressed_to.clone(),
            ),
        },
        operation_id: document.operation_id,
        room_id: document.room_id,
        expected_revision: document.expected_revision,
        logical_time: document.logical_time,
        body_bytes: document.body.len(),
        addressed_count: document.addressed_to.len(),
        request_digest,
    };
    Ok(loaded)
}

fn validate_document(
    document: &PublicationDocumentV1,
    capability: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    let valid_id = |value: &str| {
        !value.is_empty() && value.len() <= MAX_ID_BYTES && !value.chars().any(char::is_control)
    };
    if document.version != 1
        || !valid_id(&document.room_id)
        || document.body.is_empty()
        || document.body.len() > MAX_BODY_BYTES
        || document.addressed_to.len() > MAX_ADDRESSEES
        || document.addressed_to.iter().any(|value| !valid_id(value))
        || document
            .addressed_to
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return failure(
            capability,
            "request_refused",
            "request facts exceed the bounded actor-publication contract",
            None,
            "use version 1, a bounded room, 1..=16384 body bytes, and at most 64 strictly sorted unique bounded addressees",
        );
    }
    Ok(())
}

fn client(socket: PathBuf, participant: String) -> ScheduledLocalClientV1 {
    ScheduledLocalClientV1::new(
        socket,
        participant,
        WireLimits::reference(),
        Duration::from_secs(5),
    )
}

fn publication_facts(loaded: &LoadedPublication) -> serde_json::Value {
    json!({
        "operation_id": loaded.operation_id,
        "room": loaded.room_id,
        "expected_revision": loaded.expected_revision,
        "logical_time": loaded.logical_time,
        "body_bytes": loaded.body_bytes,
        "addressed_count": loaded.addressed_count,
        "request_digest": format!("sha256:{}", loaded.request_digest),
    })
}

fn required_confirmation(loaded: &LoadedPublication) -> String {
    format!(
        "PUBLISH AGORA ACTOR MESSAGE OPERATION {} REQUEST SHA256 {}",
        loaded.operation_id, loaded.request_digest
    )
}

fn transport_failure(
    capability: &'static str,
    error: LocalCallErrorV1,
    mutation: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = match error {
        LocalCallErrorV1::NotSent => "not_sent",
        LocalCallErrorV1::OutcomeUnknown => "outcome_unknown",
        LocalCallErrorV1::InvalidResponse => "invalid_reply",
    };
    let next_action = if mutation {
        "do not retry publication; observe the exact operation and intent until Agora reports accepted or authoritative absent"
    } else {
        "retry only the read-only observation against the same owner, participant, operation, and intent"
    };
    failure(
        capability,
        status,
        &format!("Agora actor-publication transport failed: {error:?}"),
        None,
        next_action,
    )
}

fn failure<T>(
    capability: &'static str,
    status: &'static str,
    detail: &str,
    required_confirmation: Option<&str>,
    next_action: &'static str,
) -> Result<T, Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "agora",
            "capability": capability,
            "complete": false,
            "status": status,
            "detail": detail,
            "required_confirmation": required_confirmation,
            "next_action": next_action,
            "boundary": "no aggregate Agora, alternate-room, ordinary Speak, blind retry, moderation, task, shell, filesystem, provider, or execution fallback is permitted",
        }))?
    );
    Err(std::io::Error::other("Agora actor publication unavailable").into())
}

fn parse_lower_hex_16(value: &str) -> Result<[u8; 16], Box<dyn std::error::Error>> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value))
    {
        return Err("Agora operation identity must be exactly 32 lowercase hex characters".into());
    }
    let mut result = [0_u8; 16];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)?;
    }
    Ok(result)
}

fn hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut result, "{byte:02x}").expect("writing to String cannot fail");
    }
    result
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_identity_is_exact_lowercase_nonzero() {
        assert_eq!(parse_lower_hex_16(&"01".repeat(16)).unwrap(), [1; 16]);
        assert!(parse_lower_hex_16(&"AB".repeat(16)).is_err());
        assert!(ActorPublicationIdV1::new(parse_lower_hex_16(&"00".repeat(16)).unwrap()).is_err());
    }
}
