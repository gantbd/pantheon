//! Closed pre-publication contract for attributed verification evidence.

use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{ffi::OsString, path::PathBuf};

use crate::private_document::{self, PrivateDocumentError};

pub const CAPABILITY: &str = "verification.result-envelope.validate.v1";
const MAX_REQUEST_BYTES: u64 = 64 * 1024;
const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VerificationDocumentV1 {
    version: u8,
    operation_id: String,
    task: TaskV1,
    source: SourceV1,
    profile: ProfileV1,
    execution: ExecutionV1,
    result: ResultV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskV1 {
    key: String,
    incarnation: u64,
    attempt: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceV1 {
    owner: String,
    workspace: u64,
    document: u64,
    revision: u64,
    bytes: usize,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileV1 {
    semantic_capability: String,
    digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionV1 {
    owner: String,
    execution_id: String,
    evidence_digest: String,
    output_digest: String,
    output_bytes: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultV1 {
    disposition: String,
    exit_code: Option<i32>,
    duration_ms: u64,
}

pub fn validate(
    mut args: impl Iterator<Item = OsString>,
) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "verification result validation requires PRIVATE_REQUEST_JSON";
    let request_path = PathBuf::from(args.next().ok_or(usage)?);
    if args.next().is_some() {
        return Err(usage.into());
    }
    let bytes = match private_document::read(&request_path, MAX_REQUEST_BYTES) {
        Ok(bytes) => bytes,
        Err(error) => return private_document_failure(error),
    };
    let document: VerificationDocumentV1 = match serde_json::from_slice(&bytes) {
        Ok(document) => document,
        Err(error) => {
            return failure(
                "request_refused",
                &format!("request document is not closed version-1 JSON: {error}"),
                "provide only the documented verification envelope fields; unknown or duplicate semantics require a new contract version",
            );
        }
    };
    validate_document(&document)?;
    let request_digest = format!("sha256:{:x}", Sha256::digest(&bytes));
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "component": "verification",
            "capability": CAPABILITY,
            "complete": true,
            "valid": true,
            "status": "validated_not_published",
            "operation_id": document.operation_id,
            "task": {
                "key": document.task.key,
                "incarnation": document.task.incarnation,
                "attempt": document.task.attempt,
            },
            "source": {
                "owner": document.source.owner,
                "workspace": document.source.workspace,
                "document": document.source.document,
                "revision": document.source.revision,
                "bytes": document.source.bytes,
                "sha256": document.source.sha256,
            },
            "profile": {
                "semantic_capability": document.profile.semantic_capability,
                "digest": document.profile.digest,
            },
            "execution": {
                "owner": document.execution.owner,
                "execution_id": document.execution.execution_id,
                "evidence_digest": document.execution.evidence_digest,
                "output_digest": document.execution.output_digest,
                "output_bytes": document.execution.output_bytes,
            },
            "result": {
                "disposition": document.result.disposition,
                "exit_code": document.result.exit_code,
                "duration_ms": document.result.duration_ms,
            },
            "request_digest": request_digest,
            "published": false,
            "next_action": "submit this exact digest and facts only through a qualified Moira verification-result composition, then observe its owner receipt before proposing any Chronos transition",
            "boundary": "closed semantic validation only; supplied owner references remain unverified until their owners are re-observed, and validation grants no execution, publication, task settlement, acceptance, release, or rollout authority",
        }))?
    );
    Ok(())
}

fn validate_document(document: &VerificationDocumentV1) -> Result<(), Box<dyn std::error::Error>> {
    let disposition_valid = match (
        document.result.disposition.as_str(),
        document.result.exit_code,
    ) {
        ("passed", Some(0)) => true,
        ("failed", Some(code)) => code != 0,
        ("cancelled" | "timed_out" | "uncertain", None) => true,
        _ => false,
    };
    if document.version != 1
        || !nonzero_hex_16(&document.operation_id)
        || !nonzero_hex_16(&document.task.key)
        || document.task.incarnation == 0
        || document.task.attempt == 0
        || document.source.owner != "vellum"
        || document.source.workspace == 0
        || document.source.document == 0
        || !digest(&document.source.sha256)
        || document.profile.semantic_capability != "verification.test.run.v1"
        || !digest(&document.profile.digest)
        || document.execution.owner != "daimon"
        || !nonzero_hex_16(&document.execution.execution_id)
        || !digest(&document.execution.evidence_digest)
        || !digest(&document.execution.output_digest)
        || document.execution.output_bytes > MAX_OUTPUT_BYTES
        || !disposition_valid
    {
        return failure(
            "request_refused",
            "verification facts violate the closed version-1 attribution contract",
            "supply nonzero exact task/source/execution identities, Vellum source and Daimon evidence digests, the governed test capability, bounded output, and a disposition-consistent exit code",
        );
    }
    Ok(())
}

fn nonzero_hex_16(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        && value.bytes().any(|byte| byte != b'0')
}

fn digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        && value[7..].bytes().any(|byte| byte != b'0')
}

fn private_document_failure<T>(
    error: PrivateDocumentError,
) -> Result<T, Box<dyn std::error::Error>> {
    failure(
        "request_refused",
        &format!("private verification request document {error}"),
        "stage one immutable owner-private regular request document of at most 65536 bytes with no symbolic or hard links",
    )
}

fn failure<T>(
    status: &'static str,
    detail: &str,
    next_action: &'static str,
) -> Result<T, Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "component": "verification",
            "capability": CAPABILITY,
            "complete": false,
            "valid": false,
            "status": status,
            "detail": detail,
            "next_action": next_action,
            "boundary": "no owner connection, execution, publication, task mutation, filesystem expansion, or authority grant occurred",
        }))?
    );
    Err(std::io::Error::other("verification result validation refused").into())
}
