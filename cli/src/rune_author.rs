//! Bounded Pantheon adapter for the deployment-owned Rune author component.
//!
//! The adapter owns neither Rune semantics nor publication. It admits one
//! human-selected, digest-pinned process profile, writes one exact request to
//! stdin, validates one closed owner response, and reports the real
//! `process_contract` confinement level. No caller selects an executable,
//! argument, child environment, workspace binding, or publication target.

use serde::Deserialize;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

#[allow(dead_code)]
mod generated {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../design/generated/pantheon_rune_author_capability.rs"
    ));
}

use generated::{
    PantheonRuneAuthorCapabilityAdvanced, PantheonRuneAuthorCapabilityCandidate,
    PantheonRuneAuthorCapabilityChooser, PantheonRuneAuthorCapabilityEvaluator,
    PantheonRuneAuthorCapabilityObligation, PantheonRuneAuthorCapabilityObservation,
    PantheonRuneAuthorCapabilityObserver, PantheonRuneAuthorCapabilitySignal,
    PantheonRuneAuthorCapabilityState,
};

pub const CAPABILITY: &str = "rune.author.v1";
const MAX_PROFILE_BYTES: u64 = 32 * 1024;
const MAX_REQUEST_BYTES: u64 = 256 * 1024;
const MAX_STDOUT_BYTES: usize = 16 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 64 * 1024;
const MAX_EXECUTABLE_BYTES: u64 = 256 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(45);
const BOUNDARY: &str = "private Rune author preparation only; process_contract is not an operating-system sandbox; no prose inference, caller-selected executable, shell, workspace mutation, publication, approval, task mutation, go-no-go, or rollout authority";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileV1 {
    version: u8,
    confinement: String,
    rune_author: ToolProfile,
    scry: ToolProfile,
    smith: ToolProfile,
    rustc: ToolProfile,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolProfile {
    path: PathBuf,
    sha256: String,
}

#[derive(Clone, Debug)]
struct QualifiedTool {
    path: PathBuf,
    sha256: String,
}

#[derive(Debug)]
struct QualifiedProfile {
    rune_author: QualifiedTool,
    scry: QualifiedTool,
    smith: QualifiedTool,
    rustc: QualifiedTool,
}

#[derive(Debug)]
struct ProcessResult {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    duration_ms: u128,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OwnerOutcome {
    Questions,
    AuthorRefusal,
    Projection,
}

#[derive(Debug)]
struct AdapterRefusal {
    stage: &'static str,
    detail: Box<AdapterRefusalDetail>,
}

#[derive(Debug)]
struct AdapterRefusalDetail {
    reason: String,
    request_sha256: Option<String>,
    author_sha256: Option<String>,
    stderr_sha256: Option<String>,
    stderr_bytes: usize,
    invoked: bool,
}

pub fn run(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let profile_path = PathBuf::from(required(
        &mut args,
        "rune-author requires RUNE_PROFILE_JSON REQUEST_JSON",
    )?);
    let request_path = PathBuf::from(required(
        &mut args,
        "rune-author requires RUNE_PROFILE_JSON REQUEST_JSON",
    )?);
    if args.next().is_some() {
        return Err("unexpected rune-author argument".into());
    }

    match execute(&profile_path, &request_path) {
        Ok(value) => {
            println!("{}", serde_json::to_string_pretty(&value)?);
            Ok(())
        }
        Err(refusal) => {
            let value = refusal_json(&refusal);
            println!("{}", serde_json::to_string_pretty(&value)?);
            Err(std::io::Error::other("Rune author adapter refused").into())
        }
    }
}

/// Reuses the exact bounded owner adapter without projecting its private
/// packet through stdout. Composition callers receive the same closed success
/// or refusal document; they gain no executable selection or publication
/// authority from this function.
pub(crate) fn execute_private(profile_path: &Path, request_path: &Path) -> Result<Value, Value> {
    execute(profile_path, request_path).map_err(|refusal| refusal_json(&refusal))
}

fn execute(profile_path: &Path, request_path: &Path) -> Result<Value, AdapterRefusal> {
    let profile = qualify_profile(profile_path)?;
    let request = read_bounded_regular(request_path, MAX_REQUEST_BYTES, "request")?;
    let request_sha256 = sha256(&request);
    let invoked = advance(
        PantheonRuneAuthorCapabilityState::INITIAL,
        PantheonRuneAuthorCapabilitySignal::AuthorInvoked,
    )
    .map_err(|reason| {
        refusal(
            "controller",
            reason,
            Some(&request_sha256),
            Some(&profile),
            false,
        )
    })?;

    let result = invoke(&profile, &request).map_err(|mut refusal| {
        refusal.detail.request_sha256 = Some(request_sha256.clone());
        refusal.detail.author_sha256 = Some(profile.rune_author.sha256.clone());
        refusal.detail.invoked = true;
        refusal
    })?;

    if let Err(reason) = requalify_profile(&profile) {
        return Err(refusal_with_process(
            "tool_identity_changed",
            reason,
            &request_sha256,
            &profile,
            &result,
        ));
    }

    let response: Value = serde_json::from_slice(&result.stdout).map_err(|error| {
        refusal_with_process(
            "invalid_response",
            format!("Rune author response is not closed JSON: {error}"),
            &request_sha256,
            &profile,
            &result,
        )
    })?;
    let outcome =
        validate_owner_response(&response, result.status.code(), &request_sha256, &profile)
            .map_err(|reason| {
                refusal_with_process(
                    "invalid_response",
                    reason,
                    &request_sha256,
                    &profile,
                    &result,
                )
            })?;

    let signal = match outcome {
        OwnerOutcome::Questions => PantheonRuneAuthorCapabilitySignal::QuestionsReturned,
        OwnerOutcome::AuthorRefusal => PantheonRuneAuthorCapabilitySignal::AuthorRefusalReturned,
        OwnerOutcome::Projection => PantheonRuneAuthorCapabilitySignal::ProjectionPacketReturned,
    };
    let terminal = advance(invoked.state, signal).map_err(|reason| {
        refusal_with_process("controller", reason, &request_sha256, &profile, &result)
    })?;

    Ok(json!({
        "version": 1,
        "peer": "rune",
        "capability": CAPABILITY,
        "status": response["status"],
        "complete": outcome == OwnerOutcome::Projection,
        "machine_state": state_name(terminal.state),
        "signal": signal_name(terminal.signal),
        "request_sha256": request_sha256,
        "confinement": confinement_json(),
        "toolchain": toolchain_json(&profile),
        "observation": observation_json(&result, outcome, request.len()),
        "owner_outcome": response,
        "next_action": match outcome {
            OwnerOutcome::Questions => "answer the exact typed questions through accountable review, then submit a complete new request revision",
            OwnerOutcome::AuthorRefusal => "follow the Rune-owned next action without deleting or weakening an obligation",
            OwnerOutcome::Projection => "present the exact private packet for human review; separately authorize any revision-bound Vellum publication",
        },
        "boundary": BOUNDARY,
    }))
}

fn qualify_profile(path: &Path) -> Result<QualifiedProfile, AdapterRefusal> {
    let bytes = read_bounded_regular(path, MAX_PROFILE_BYTES, "profile")?;
    let profile: ProfileV1 = serde_json::from_slice(&bytes).map_err(|error| {
        refusal(
            "profile",
            format!("invalid closed Rune profile: {error}"),
            None,
            None,
            false,
        )
    })?;
    if profile.version != 1 {
        return Err(refusal(
            "profile",
            "Rune profile version must be 1",
            None,
            None,
            false,
        ));
    }
    if profile.confinement != "process_contract" {
        return Err(refusal(
            "profile",
            "version one admits only explicit process_contract confinement",
            None,
            None,
            false,
        ));
    }
    Ok(QualifiedProfile {
        rune_author: qualify_tool(profile.rune_author, "rune_author")?,
        scry: qualify_tool(profile.scry, "scry")?,
        smith: qualify_tool(profile.smith, "smith")?,
        rustc: qualify_tool(profile.rustc, "rustc")?,
    })
}

fn qualify_tool(value: ToolProfile, field: &'static str) -> Result<QualifiedTool, AdapterRefusal> {
    if !value.path.is_absolute() {
        return Err(refusal(
            "profile",
            format!("{field} path must be absolute"),
            None,
            None,
            false,
        ));
    }
    if !valid_digest(&value.sha256) {
        return Err(refusal(
            "profile",
            format!("{field} sha256 must be a lowercase sha256 digest"),
            None,
            None,
            false,
        ));
    }
    let path = value.path.canonicalize().map_err(|error| {
        refusal(
            "profile",
            format!("resolve {field}: {error}"),
            None,
            None,
            false,
        )
    })?;
    let metadata = fs::metadata(&path).map_err(|error| {
        refusal(
            "profile",
            format!("inspect {field}: {error}"),
            None,
            None,
            false,
        )
    })?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_EXECUTABLE_BYTES {
        return Err(refusal(
            "profile",
            format!("{field} must be a non-empty regular file no larger than 256 MiB"),
            None,
            None,
            false,
        ));
    }
    let actual = file_sha256(&path).map_err(|reason| {
        refusal(
            "profile",
            format!("read {field}: {reason}"),
            None,
            None,
            false,
        )
    })?;
    if actual != value.sha256 {
        return Err(refusal(
            "profile",
            format!("{field} bytes do not match the human-owned profile digest"),
            None,
            None,
            false,
        ));
    }
    Ok(QualifiedTool {
        path,
        sha256: actual,
    })
}

fn requalify_profile(profile: &QualifiedProfile) -> Result<(), String> {
    for (name, tool) in [
        ("rune_author", &profile.rune_author),
        ("scry", &profile.scry),
        ("smith", &profile.smith),
        ("rustc", &profile.rustc),
    ] {
        let actual = file_sha256(&tool.path).map_err(|error| format!("read {name}: {error}"))?;
        if actual != tool.sha256 {
            return Err(format!("{name} identity changed during the transaction"));
        }
    }
    Ok(())
}

fn invoke(profile: &QualifiedProfile, request: &[u8]) -> Result<ProcessResult, AdapterRefusal> {
    let started = Instant::now();
    let mut child = Command::new(&profile.rune_author.path)
        .env_clear()
        .env("RUNE_AUTHOR_SCRY", &profile.scry.path)
        .env("RUNE_AUTHOR_SMITH", &profile.smith.path)
        .env("RUNE_AUTHOR_RUSTC", &profile.rustc.path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            refusal(
                "spawn",
                format!("start Rune author: {error}"),
                None,
                None,
                true,
            )
        })?;

    let mut stdin = child.stdin.take().ok_or_else(|| {
        refusal(
            "transport",
            "Rune author stdin is unavailable",
            None,
            None,
            true,
        )
    })?;
    stdin.write_all(request).map_err(|error| {
        refusal(
            "transport",
            format!("write exact request to Rune author: {error}"),
            None,
            None,
            true,
        )
    })?;
    drop(stdin);

    let stdout = child.stdout.take().ok_or_else(|| {
        refusal(
            "transport",
            "Rune author stdout is unavailable",
            None,
            None,
            true,
        )
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        refusal(
            "transport",
            "Rune author stderr is unavailable",
            None,
            None,
            true,
        )
    })?;
    let stdout_reader = thread::spawn(move || bounded_read(stdout, MAX_STDOUT_BYTES));
    let stderr_reader = thread::spawn(move || bounded_read(stderr, MAX_STDERR_BYTES));

    let deadline = Instant::now() + TIMEOUT;
    let (status, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status, false),
            Ok(None) if Instant::now() >= deadline => {
                child.kill().map_err(|error| {
                    refusal(
                        "timeout",
                        format!("stop timed-out Rune author: {error}"),
                        None,
                        None,
                        true,
                    )
                })?;
                let status = child.wait().map_err(|error| {
                    refusal(
                        "timeout",
                        format!("reap timed-out Rune author: {error}"),
                        None,
                        None,
                        true,
                    )
                })?;
                break (status, true);
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(error) => {
                return Err(refusal(
                    "observe",
                    format!("observe Rune author: {error}"),
                    None,
                    None,
                    true,
                ));
            }
        }
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| refusal("transport", "stdout reader panicked", None, None, true))?
        .map_err(|reason| refusal("transport", reason, None, None, true))?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| refusal("transport", "stderr reader panicked", None, None, true))?
        .map_err(|reason| refusal("transport", reason, None, None, true))?;
    if timed_out {
        return Err(AdapterRefusal {
            stage: "timeout",
            detail: Box::new(AdapterRefusalDetail {
                reason: "Rune author exceeded the fixed 45-second deadline".into(),
                request_sha256: None,
                author_sha256: None,
                stderr_sha256: Some(sha256(&stderr)),
                stderr_bytes: stderr.len(),
                invoked: true,
            }),
        });
    }
    Ok(ProcessResult {
        status,
        stdout,
        stderr,
        duration_ms: started.elapsed().as_millis(),
    })
}

fn validate_owner_response(
    value: &Value,
    exit_code: Option<i32>,
    request_sha256: &str,
    profile: &QualifiedProfile,
) -> Result<OwnerOutcome, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "Rune author response must be an object".to_owned())?;
    if object.get("version").and_then(Value::as_u64) != Some(1) {
        return Err("Rune author response version must be 1".into());
    }
    let status = object
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| "Rune author response status is absent".to_owned())?;
    match status {
        "questions_pending" => {
            closed_exact(object, QUESTIONS_FIELDS)?;
            require_exit(exit_code, 0, status)?;
            require_text(object, "request_sha256", request_sha256)?;
            require_text(object, "machine_state", "questions_pending")?;
            require_text(object, "signal", "clarification-required")?;
            nonempty_text(object, "boundary")?;
            let questions = object
                .get("questions")
                .and_then(Value::as_array)
                .filter(|questions| !questions.is_empty())
                .ok_or_else(|| "questions_pending requires nonempty questions".to_owned())?;
            for question in questions {
                let question = question
                    .as_object()
                    .ok_or_else(|| "each question must be an object".to_owned())?;
                closed_exact(question, &["path", "category", "question"])?;
                for field in ["path", "category", "question"] {
                    nonempty_text(question, field)?;
                }
            }
            Ok(OwnerOutcome::Questions)
        }
        "projection_qualified" => {
            closed_exact(object, PROJECTION_FIELDS)?;
            require_exit(exit_code, 0, status)?;
            require_text(object, "request_sha256", request_sha256)?;
            require_text(object, "machine_state", "projection_qualified")?;
            require_text(object, "signal", "projection-prepared")?;
            require_text(object, "scry_sha256", &profile.scry.sha256)?;
            require_text(object, "smith_sha256", &profile.smith.sha256)?;
            require_text(object, "rustc_sha256", &profile.rustc.sha256)?;
            let source = nonempty_text(object, "canonical_candidate")?;
            let typed_ir = nonempty_text(object, "typed_ir")?;
            let projection = nonempty_text(object, "projection")?;
            nonempty_text(object, "next_action")?;
            nonempty_text(object, "boundary")?;
            require_text(
                object,
                "canonical_source_sha256",
                &sha256(source.as_bytes()),
            )?;
            require_text(
                object,
                "scry_ir_transport_sha256",
                &sha256(typed_ir.as_bytes()),
            )?;
            require_text(object, "projection_sha256", &sha256(projection.as_bytes()))?;
            let typed_ir_identity = nonempty_text(object, "typed_ir_sha256")?;
            for (field, expected) in [
                (
                    "source_sha256",
                    nonempty_text(object, "canonical_source_sha256")?,
                ),
                ("typed_ir_sha256", typed_ir_identity),
                ("smith_sha256", profile.smith.sha256.as_str()),
            ] {
                let needle = format!("{field}: \"{expected}\"");
                if !projection.contains(&needle) {
                    return Err(format!("Rust projection lineage omits exact {field}"));
                }
            }
            let compatibility = object
                .get("compatibility")
                .and_then(Value::as_object)
                .ok_or_else(|| "projection compatibility is absent".to_owned())?;
            closed_exact(compatibility, COMPATIBILITY_FIELDS)?;
            if compatibility.get("version").and_then(Value::as_u64) != Some(1) {
                return Err("projection compatibility version must be 1".into());
            }
            require_text(compatibility, "status", "complete")?;
            require_text(compatibility, "target", "rust")?;
            for field in ["actor", "boundary", "source"] {
                nonempty_text(compatibility, field)?;
            }
            require_nonempty_text_array(compatibility, "preserved")?;
            if compatibility
                .get("unsupported")
                .and_then(Value::as_array)
                .is_none_or(|values| !values.is_empty())
            {
                return Err("projection compatibility has unsupported obligations".into());
            }
            let diagnostics = object
                .get("diagnostics")
                .and_then(Value::as_object)
                .ok_or_else(|| "projection diagnostics are absent".to_owned())?;
            closed_exact(
                diagnostics,
                &["scry", "smith_compatibility", "smith_projection"],
            )?;
            for field in ["scry", "smith_compatibility", "smith_projection"] {
                require_text_value(diagnostics, field)?;
            }
            Ok(OwnerOutcome::Projection)
        }
        "scry_refused" => {
            closed_exact(object, SCRY_REFUSAL_FIELDS)?;
            require_exit(exit_code, 3, status)?;
            require_text(object, "request_sha256", request_sha256)?;
            require_text(object, "machine_state", "refused")?;
            require_text(object, "signal", "scry-refusal-recorded")?;
            require_text(object, "scry_sha256", &profile.scry.sha256)?;
            require_digest(object, "candidate_sha256")?;
            require_exit_metadata(object)?;
            for field in ["diagnostic", "next_action", "boundary"] {
                nonempty_text(object, field)?;
            }
            Ok(OwnerOutcome::AuthorRefusal)
        }
        "smith_refused" => {
            closed_exact(object, SMITH_REFUSAL_FIELDS)?;
            require_exit(exit_code, 4, status)?;
            require_text(object, "request_sha256", request_sha256)?;
            require_text(object, "machine_state", "refused")?;
            require_text(object, "signal", "projection-refusal-recorded")?;
            require_text(object, "scry_sha256", &profile.scry.sha256)?;
            require_text(object, "smith_sha256", &profile.smith.sha256)?;
            require_text(object, "rustc_sha256", &profile.rustc.sha256)?;
            require_digest(object, "canonical_source_sha256")?;
            require_digest(object, "scry_ir_transport_sha256")?;
            require_exit_metadata(object)?;
            for field in ["stage", "diagnostic", "next_action", "boundary"] {
                nonempty_text(object, field)?;
            }
            let compatibility = object
                .get("compatibility")
                .and_then(Value::as_object)
                .ok_or_else(|| "Smith refusal compatibility is absent".to_owned())?;
            closed_exact(compatibility, COMPATIBILITY_FIELDS)?;
            Ok(OwnerOutcome::AuthorRefusal)
        }
        "refused" => {
            closed_exact(object, &["version", "status", "path", "reason", "boundary"])?;
            require_exit(exit_code, 2, status)?;
            for field in ["path", "reason", "boundary"] {
                nonempty_text(object, field)?;
            }
            Ok(OwnerOutcome::AuthorRefusal)
        }
        _ => Err(format!("unsupported Rune author status {status:?}")),
    }
}

const QUESTIONS_FIELDS: &[&str] = &[
    "version",
    "status",
    "request_sha256",
    "machine_state",
    "signal",
    "questions",
    "boundary",
];
const PROJECTION_FIELDS: &[&str] = &[
    "version",
    "status",
    "request_sha256",
    "canonical_source_sha256",
    "scry_ir_transport_sha256",
    "typed_ir_sha256",
    "scry_sha256",
    "smith_sha256",
    "rustc_sha256",
    "projection_sha256",
    "machine_state",
    "signal",
    "compatibility",
    "canonical_candidate",
    "typed_ir",
    "projection",
    "diagnostics",
    "next_action",
    "boundary",
];
const SCRY_REFUSAL_FIELDS: &[&str] = &[
    "version",
    "status",
    "request_sha256",
    "candidate_sha256",
    "machine_state",
    "signal",
    "scry_sha256",
    "exit_code",
    "diagnostic",
    "timed_out",
    "next_action",
    "boundary",
];
const SMITH_REFUSAL_FIELDS: &[&str] = &[
    "version",
    "status",
    "stage",
    "request_sha256",
    "canonical_source_sha256",
    "scry_ir_transport_sha256",
    "scry_sha256",
    "smith_sha256",
    "rustc_sha256",
    "machine_state",
    "signal",
    "exit_code",
    "diagnostic",
    "timed_out",
    "compatibility",
    "next_action",
    "boundary",
];
const COMPATIBILITY_FIELDS: &[&str] = &[
    "actor",
    "boundary",
    "preserved",
    "source",
    "status",
    "target",
    "unsupported",
    "version",
];

fn closed_exact(object: &Map<String, Value>, allowed: &[&str]) -> Result<(), String> {
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!(
                "Rune author response contains unknown field {key:?}"
            ));
        }
    }
    for field in allowed {
        if !object.contains_key(*field) {
            return Err(format!(
                "Rune author response omits required field {field:?}"
            ));
        }
    }
    Ok(())
}

fn require_digest(object: &Map<String, Value>, field: &str) -> Result<(), String> {
    let value = nonempty_text(object, field)?;
    if valid_digest(value) {
        Ok(())
    } else {
        Err(format!(
            "Rune author {field} must be a lowercase SHA-256 digest"
        ))
    }
}

fn require_exit_metadata(object: &Map<String, Value>) -> Result<(), String> {
    if !object.get("timed_out").is_some_and(Value::is_boolean) {
        return Err("Rune author timed_out must be boolean".into());
    }
    if !object
        .get("exit_code")
        .is_some_and(|value| value.is_null() || value.as_i64().is_some())
    {
        return Err("Rune author exit_code must be an integer or null".into());
    }
    Ok(())
}

fn require_nonempty_text_array(object: &Map<String, Value>, field: &str) -> Result<(), String> {
    let values = object
        .get(field)
        .and_then(Value::as_array)
        .filter(|values| !values.is_empty())
        .ok_or_else(|| format!("Rune author {field} must be a nonempty array"))?;
    if values
        .iter()
        .all(|value| value.as_str().is_some_and(|value| !value.is_empty()))
    {
        Ok(())
    } else {
        Err(format!("Rune author {field} entries must be nonempty text"))
    }
}

fn require_exit(actual: Option<i32>, expected: i32, status: &str) -> Result<(), String> {
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(format!(
            "Rune author status {status:?} requires exit {expected}, got {actual:?}"
        ))
    }
}

fn require_text(object: &Map<String, Value>, field: &str, expected: &str) -> Result<(), String> {
    let actual = object.get(field).and_then(Value::as_str);
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(format!(
            "Rune author {field} does not match the exact transaction"
        ))
    }
}

fn nonempty_text<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Rune author {field} must be nonempty text"))
}

fn require_text_value<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Rune author {field} must be text"))
}

fn read_bounded_regular(
    path: &Path,
    max_bytes: u64,
    label: &'static str,
) -> Result<Vec<u8>, AdapterRefusal> {
    let metadata = fs::metadata(path).map_err(|error| {
        refusal(
            label,
            format!("inspect {label}: {error}"),
            None,
            None,
            false,
        )
    })?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > max_bytes {
        return Err(refusal(
            label,
            format!("{label} must be a non-empty regular file within {max_bytes} bytes"),
            None,
            None,
            false,
        ));
    }
    fs::read(path)
        .map_err(|error| refusal(label, format!("read {label}: {error}"), None, None, false))
}

fn bounded_read(mut input: impl Read, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    input
        .by_ref()
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read bounded process output: {error}"))?;
    if bytes.len() > limit {
        return Err("Rune author output exceeded its bounded channel".into());
    }
    Ok(bytes)
}

fn file_sha256(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|bytes| sha256(&bytes))
        .map_err(|error| error.to_string())
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn refusal(
    stage: &'static str,
    reason: impl Into<String>,
    request_sha256: Option<&str>,
    profile: Option<&QualifiedProfile>,
    invoked: bool,
) -> AdapterRefusal {
    AdapterRefusal {
        stage,
        detail: Box::new(AdapterRefusalDetail {
            reason: reason.into(),
            request_sha256: request_sha256.map(ToOwned::to_owned),
            author_sha256: profile.map(|profile| profile.rune_author.sha256.clone()),
            stderr_sha256: None,
            stderr_bytes: 0,
            invoked,
        }),
    }
}

fn refusal_with_process(
    stage: &'static str,
    reason: impl Into<String>,
    request_sha256: &str,
    profile: &QualifiedProfile,
    result: &ProcessResult,
) -> AdapterRefusal {
    AdapterRefusal {
        stage,
        detail: Box::new(AdapterRefusalDetail {
            reason: reason.into(),
            request_sha256: Some(request_sha256.to_owned()),
            author_sha256: Some(profile.rune_author.sha256.clone()),
            stderr_sha256: Some(sha256(&result.stderr)),
            stderr_bytes: result.stderr.len(),
            invoked: true,
        }),
    }
}

fn refusal_json(refusal: &AdapterRefusal) -> Value {
    let state = if refusal.detail.invoked {
        advance(
            PantheonRuneAuthorCapabilityState::AuthorInvoked,
            PantheonRuneAuthorCapabilitySignal::AdapterRefused,
        )
    } else {
        advance(
            PantheonRuneAuthorCapabilityState::INITIAL,
            PantheonRuneAuthorCapabilitySignal::AdapterRefused,
        )
    };
    let (machine_state, signal) = match state {
        Ok(value) => (state_name(value.state), signal_name(value.signal)),
        Err(_) => ("adapter_refused", "adapter-refused"),
    };
    json!({
        "version": 1,
        "peer": "rune",
        "capability": CAPABILITY,
        "status": "adapter_refused",
        "complete": false,
        "machine_state": machine_state,
        "signal": signal,
        "stage": refusal.stage,
        "reason": refusal.detail.reason,
        "request_sha256": refusal.detail.request_sha256,
        "author_sha256": refusal.detail.author_sha256,
        "stderr_sha256": refusal.detail.stderr_sha256,
        "stderr_bytes": refusal.detail.stderr_bytes,
        "confinement": confinement_json(),
        "observation": {
            "schema": "rune.author.observation.v1",
            "classification": "adapter_refusal",
            "requests": if refusal.detail.invoked { 1 } else { 0 },
            "adapter_errors": 1,
            "saturation": {
                "request_limit_bytes": MAX_REQUEST_BYTES,
                "stdout_limit_bytes": MAX_STDOUT_BYTES,
                "stderr_limit_bytes": MAX_STDERR_BYTES,
                "deadline_ms": TIMEOUT.as_millis(),
            },
            "publication": "inline_private_result_only",
            "oculus_published": false,
        },
        "next_action": "correct the profile, transport, or response-integrity failure; do not reinterpret it as a Rune semantic verdict or retry an unknown mutation",
        "boundary": BOUNDARY,
    })
}

fn confinement_json() -> Value {
    json!({
        "mode": "process_contract",
        "os_sandbox": false,
        "environment_cleared": true,
        "caller_selects_executable": false,
        "intentional_workspace_binding": false,
        "intentional_network_capability": false,
        "nonclaim": "the child still executes as Pantheon's operating-system principal; omitted bindings and cleared environment do not revoke ambient filesystem or network access available to that principal",
    })
}

fn toolchain_json(profile: &QualifiedProfile) -> Value {
    json!({
        "rune_author_sha256": profile.rune_author.sha256,
        "scry_sha256": profile.scry.sha256,
        "smith_sha256": profile.smith.sha256,
        "rustc_sha256": profile.rustc.sha256,
    })
}

fn observation_json(result: &ProcessResult, outcome: OwnerOutcome, request_bytes: usize) -> Value {
    json!({
        "schema": "rune.author.observation.v1",
        "classification": match outcome {
            OwnerOutcome::Questions => "questions",
            OwnerOutcome::AuthorRefusal => "owner_refusal",
            OwnerOutcome::Projection => "projection",
        },
        "requests": 1,
        "adapter_errors": 0,
        "duration_ms": result.duration_ms,
        "traffic": {
            "request_bytes": request_bytes,
            "stdout_bytes": result.stdout.len(),
            "stderr_bytes": result.stderr.len(),
        },
        "saturation": {
            "request_limit_bytes": MAX_REQUEST_BYTES,
            "stdout_limit_bytes": MAX_STDOUT_BYTES,
            "stderr_limit_bytes": MAX_STDERR_BYTES,
            "deadline_ms": TIMEOUT.as_millis(),
        },
        "publication": "inline_private_result_only",
        "oculus_published": false,
    })
}

struct Gate {
    signal: PantheonRuneAuthorCapabilitySignal,
}

impl PantheonRuneAuthorCapabilityEvaluator for Gate {
    fn evaluate(&mut self, obligation: &PantheonRuneAuthorCapabilityObligation) -> bool {
        obligation.signal == pascal_signal(self.signal)
    }
}

impl PantheonRuneAuthorCapabilityChooser for Gate {
    fn choose(
        &mut self,
        _state: PantheonRuneAuthorCapabilityState,
        eligible: &[PantheonRuneAuthorCapabilityCandidate],
    ) -> Option<usize> {
        (!eligible.is_empty()).then_some(0)
    }
}

impl PantheonRuneAuthorCapabilityObserver for Gate {
    fn observe(&mut self, _observation: &PantheonRuneAuthorCapabilityObservation) {}
}

fn advance(
    state: PantheonRuneAuthorCapabilityState,
    signal: PantheonRuneAuthorCapabilitySignal,
) -> Result<PantheonRuneAuthorCapabilityAdvanced, String> {
    let mut evaluator = Gate { signal };
    let mut chooser = Gate { signal };
    let mut observer = Gate { signal };
    state
        .advance(&mut evaluator, &mut chooser, &mut observer)
        .map_err(|error| format!("forged capability controller refused {signal:?}: {error:?}"))
}

const fn pascal_signal(signal: PantheonRuneAuthorCapabilitySignal) -> &'static str {
    match signal {
        PantheonRuneAuthorCapabilitySignal::AdapterRefused => "AdapterRefused",
        PantheonRuneAuthorCapabilitySignal::AuthorInvoked => "AuthorInvoked",
        PantheonRuneAuthorCapabilitySignal::AuthorRefusalReturned => "AuthorRefusalReturned",
        PantheonRuneAuthorCapabilitySignal::ProjectionPacketReturned => "ProjectionPacketReturned",
        PantheonRuneAuthorCapabilitySignal::QuestionsReturned => "QuestionsReturned",
    }
}

const fn state_name(state: PantheonRuneAuthorCapabilityState) -> &'static str {
    match state {
        PantheonRuneAuthorCapabilityState::RequestReceived => "request_received",
        PantheonRuneAuthorCapabilityState::AuthorInvoked => "author_invoked",
        PantheonRuneAuthorCapabilityState::QuestionsReturned => "questions_returned",
        PantheonRuneAuthorCapabilityState::AuthorRefusalReturned => "author_refusal_returned",
        PantheonRuneAuthorCapabilityState::ProjectionPacketReturned => "projection_packet_returned",
        PantheonRuneAuthorCapabilityState::AdapterRefused => "adapter_refused",
    }
}

const fn signal_name(signal: PantheonRuneAuthorCapabilitySignal) -> &'static str {
    match signal {
        PantheonRuneAuthorCapabilitySignal::AdapterRefused => "adapter-refused",
        PantheonRuneAuthorCapabilitySignal::AuthorInvoked => "author-invoked",
        PantheonRuneAuthorCapabilitySignal::AuthorRefusalReturned => "author-refusal-returned",
        PantheonRuneAuthorCapabilitySignal::ProjectionPacketReturned => {
            "projection-packet-returned"
        }
        PantheonRuneAuthorCapabilitySignal::QuestionsReturned => "questions-returned",
    }
}

fn required(
    args: &mut impl Iterator<Item = OsString>,
    message: &'static str,
) -> Result<OsString, Box<dyn std::error::Error>> {
    args.next().ok_or_else(|| message.into())
}
