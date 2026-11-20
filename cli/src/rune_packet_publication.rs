//! Two-phase, revision-bound publication of one qualified Rune packet through
//! the sovereign Vellum owner.

use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use vellum_sdk_client::{
    protocol::{
        DurableOperationRefV1, MAX_WORKSPACE_EDIT_PAYLOAD_BYTES_V1, ServiceAccessErrorV1,
        WorkspaceDocumentEditV1, WorkspaceDocumentIdV1, WorkspaceReplacementV1,
    },
    transport::{
        AuthenticatedUnixConnector, FrameExchange, RemoteTransport, TransportError, TransportLimits,
    },
    workspace::{WorkspaceClientErrorV1, WorkspaceClientV1},
};
use vellum_tui::{DurableSaveCommandV1, run_durable_save_v1};

#[allow(dead_code)]
mod generated {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../design/generated/pantheon_rune_packet_publication.rs"
    ));
}

use generated::{
    PantheonRunePacketPublicationAdvanced, PantheonRunePacketPublicationCandidate,
    PantheonRunePacketPublicationChooser, PantheonRunePacketPublicationEvaluator,
    PantheonRunePacketPublicationObligation, PantheonRunePacketPublicationObservation,
    PantheonRunePacketPublicationObserver, PantheonRunePacketPublicationSignal,
    PantheonRunePacketPublicationState,
};

pub const CAPABILITY: &str = "rune.packet-publication.v1";
const MAX_PROFILE_BYTES: u64 = 32 * 1024;
const MAX_APPROVAL_BYTES: u64 = 32 * 1024;
const BOUNDARY: &str = "one qualified private Rune packet and one deployment-pinned pre-admitted Vellum document; no membership, arbitrary target, shell, agent-tool, design acceptance, go-no-go, or rollout authority";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileV1 {
    version: u8,
    rune_profile: PathBuf,
    vellum_socket: PathBuf,
    vellum_credential: PathBuf,
    vellum_document: u64,
    max_packet_bytes: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApprovalV1 {
    version: u8,
    decision: String,
    confirmation: String,
    packet_sha256: String,
    workspace: u64,
    document: u64,
    target_revision: u64,
    target_sha256: String,
    replacement_sha256: String,
    patch_start: u64,
    patch_end: u64,
    inserted_sha256: String,
    durable_operation: String,
}

enum Connector {
    Authenticated(AuthenticatedUnixConnector),
}

impl FrameExchange for Connector {
    fn exchange(
        &mut self,
        request: &[u8],
        max_reply_bytes: usize,
    ) -> Result<Vec<u8>, TransportError> {
        match self {
            Self::Authenticated(connector) => connector.exchange(request, max_reply_bytes),
        }
    }
}

type Transport = RemoteTransport<Connector>;

struct Prepared {
    profile_sha256: String,
    packet: String,
    packet_sha256: String,
    workspace: u64,
    document: WorkspaceDocumentIdV1,
    target_revision: u64,
    target_bytes: usize,
    target_sha256: String,
    replacement_sha256: String,
    patch: crate::vellum_patch::MinimalPatch,
    inserted_sha256: String,
    operation: DurableOperationRefV1,
    operation_text: String,
    confirmation: String,
    rune_owner_requests: u8,
    client: Option<WorkspaceClientV1<Transport>>,
}

struct ConfirmationBinding<'a> {
    workspace: u64,
    document: u64,
    revision: u64,
    target_sha256: &'a str,
    replacement_sha256: &'a str,
    patch_start: usize,
    patch_end: usize,
    inserted_sha256: &'a str,
}

#[derive(Debug)]
struct Failure {
    stage: &'static str,
    reason: String,
    next_action: &'static str,
}

pub fn run(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let mode = text(required(&mut args)?)?;
    let profile = PathBuf::from(required(&mut args)?);
    let request = PathBuf::from(required(&mut args)?);
    let operation = text(required(&mut args)?)?;
    let approval = match mode.as_str() {
        "preview" => None,
        "publish" => Some(PathBuf::from(required(&mut args)?)),
        _ => return Err("Rune packet publication mode must be preview or publish".into()),
    };
    if args.next().is_some() {
        return Err("unexpected Rune packet publication argument".into());
    }

    let outcome = match execute(&mode, &profile, &request, &operation, approval.as_deref()) {
        Ok(value) => {
            println!("{}", serde_json::to_string_pretty(&value)?);
            return Ok(());
        }
        Err(failure) => failure_json(failure),
    };
    println!("{}", serde_json::to_string_pretty(&outcome)?);
    Err(std::io::Error::other("Rune packet publication refused").into())
}

fn execute(
    mode: &str,
    profile_path: &Path,
    request_path: &Path,
    operation_text: &str,
    approval_path: Option<&Path>,
) -> Result<Value, Failure> {
    let started = Instant::now();
    let prepared = prepare(profile_path, request_path, operation_text)?;
    execute_prepared(mode, profile_path, prepared, approval_path, started)
}

fn execute_prepared(
    mode: &str,
    profile_path: &Path,
    mut prepared: Prepared,
    approval_path: Option<&Path>,
    started: Instant,
) -> Result<Value, Failure> {
    let preview = advance(
        PantheonRunePacketPublicationState::INITIAL,
        PantheonRunePacketPublicationSignal::PreviewReturned,
    )?;
    if mode == "preview" {
        let outcome = preview_json(&prepared, preview, false, started.elapsed().as_millis());
        prepared
            .client
            .take()
            .expect("prepared Vellum client")
            .disconnect()
            .map_err(|failure| Failure {
                stage: "vellum_disconnect",
                reason: format!("Vellum preview release refused: {:?}", failure.error()),
                next_action: "inspect Vellum client capacity before preparing another publication",
            })?;
        return Ok(outcome);
    }

    let approval_path = approval_path.ok_or(Failure {
        stage: "approval",
        reason: "publish requires one private approval document".into(),
        next_action: "write the exact preview approval template to a mode-0600 regular file",
    })?;
    validate_approval(approval_path, &prepared)?;
    requalify_profile(profile_path, &prepared.profile_sha256)?;
    let armed = advance(
        preview.state,
        PantheonRunePacketPublicationSignal::PublicationArmed,
    )?;
    let expected_revision = prepared.target_revision.checked_add(1).ok_or(Failure {
        stage: "target",
        reason: "target revision has no representable successor".into(),
        next_action: "stop before mutation and inspect the Vellum owner",
    })?;

    let edit = WorkspaceDocumentEditV1 {
        document: prepared.document,
        expected_revision: prepared.target_revision,
        replacements: vec![WorkspaceReplacementV1 {
            start: u64::try_from(prepared.patch.start).map_err(|_| Failure {
                stage: "target",
                reason: "Vellum patch start is not representable".into(),
                next_action: "use a bounded pre-admitted packet document",
            })?,
            end: u64::try_from(prepared.patch.end).map_err(|_| Failure {
                stage: "target",
                reason: "Vellum patch end is not representable".into(),
                next_action: "use a bounded pre-admitted packet document",
            })?,
            inserted: prepared.patch.inserted.clone(),
        }],
    };
    let receipt = match prepared
        .client
        .as_mut()
        .expect("prepared Vellum client")
        .workspace_edit(vec![edit])
    {
        Ok(receipt) => receipt,
        Err(WorkspaceClientErrorV1::Access(ServiceAccessErrorV1::OutcomeUnknown)) => {
            return edit_unknown_json(&prepared, armed.state, "owner reported outcome unknown");
        }
        Err(error) => {
            return Err(Failure {
                stage: "vellum_edit",
                reason: format!("Vellum owner refused packet edit: {error:?}"),
                next_action: "refresh exact target evidence and prepare a new human decision; do not retry stale or uncertain work",
            });
        }
    };
    let Some(edited) = receipt
        .documents
        .iter()
        .find(|value| value.document == prepared.document)
    else {
        return edit_unknown_json(
            &prepared,
            armed.state,
            "accepted owner receipt omitted the exact document",
        );
    };
    if receipt.workspace.get() != prepared.workspace
        || edited.snapshot.revision != expected_revision
        || edited.snapshot.text.as_ref() != prepared.packet
    {
        return edit_unknown_json(
            &prepared,
            armed.state,
            "accepted owner receipt did not reproduce the exact packet successor",
        );
    }
    let in_memory = advance(
        armed.state,
        PantheonRunePacketPublicationSignal::EditAccepted,
    )?;
    let transport = match prepared
        .client
        .take()
        .expect("prepared Vellum client")
        .disconnect()
    {
        Ok(transport) => transport,
        Err(failure) => {
            return persistence_pending_json(
                &prepared,
                in_memory.state,
                expected_revision,
                "client_release_failed",
                failure.error(),
            );
        }
    };
    let durable = run_durable_save_v1(
        transport,
        DurableSaveCommandV1::Execute,
        prepared.operation,
        prepared.document.get(),
        expected_revision,
    );
    let (status, disconnect) = match durable {
        Ok(report) => (report.outcome.status(), report.disconnect.status()),
        Err(_) => ("unavailable", "unknown"),
    };
    let is_durable = status == "durable";
    let terminal = advance(
        in_memory.state,
        if is_durable {
            PantheonRunePacketPublicationSignal::DurablePublished
        } else {
            PantheonRunePacketPublicationSignal::PersistencePending
        },
    )?;
    Ok(json!({
        "version": 1,
        "capability": CAPABILITY,
        "status": if is_durable { "durable_published" } else { "persistence_pending" },
        "complete": is_durable,
        "machine_state": state_name(terminal.state),
        "signal": signal_name(terminal.signal),
        "packet_sha256": prepared.packet_sha256,
        "workspace": prepared.workspace,
        "document": prepared.document.get(),
        "prior_revision": prepared.target_revision,
        "published_revision": expected_revision,
        "durable_operation": prepared.operation_text,
        "durable_outcome": status,
        "disconnect": disconnect,
        "next_action": if is_durable {
            "retain this exact receipt as publication evidence; downstream acceptance and execution remain separate decisions"
        } else {
            "lookup the exact durable operation through Vellum; do not issue a new save identity or restyle the dirty edit as refusal"
        },
        "boundary": BOUNDARY,
    }))
}

pub(crate) struct PublicationProfileBinding {
    pub rune_profile: PathBuf,
    pub sha256: String,
}

pub(crate) fn publication_profile_binding(
    profile_path: &Path,
) -> Result<PublicationProfileBinding, Value> {
    let profile_bytes =
        read_regular(profile_path, MAX_PROFILE_BYTES, "profile").map_err(failure_json)?;
    let profile_sha256 = sha256(&profile_bytes);
    let profile: ProfileV1 = serde_json::from_slice(&profile_bytes).map_err(|error| {
        failure_json(Failure {
            stage: "profile",
            reason: format!("invalid closed publication profile: {error}"),
            next_action: "supply the deployment-owned version-one publication profile",
        })
    })?;
    validate_profile(&profile).map_err(failure_json)?;
    Ok(PublicationProfileBinding {
        rune_profile: profile.rune_profile,
        sha256: profile_sha256,
    })
}

pub(crate) fn execute_qualified(
    mode: &str,
    profile_path: &Path,
    exact_request: &[u8],
    author: &Value,
    expected_profile_sha256: &str,
    operation_text: &str,
    approval_path: Option<&Path>,
) -> Result<Value, Value> {
    let started = Instant::now();
    let prepared = prepare_qualified(
        profile_path,
        exact_request,
        author,
        expected_profile_sha256,
        operation_text,
    )
    .map_err(failure_json)?;
    execute_prepared(mode, profile_path, prepared, approval_path, started).map_err(failure_json)
}

fn edit_unknown_json(
    prepared: &Prepared,
    state: PantheonRunePacketPublicationState,
    detail: &str,
) -> Result<Value, Failure> {
    let unknown = advance(
        state,
        PantheonRunePacketPublicationSignal::EditOutcomeUnknown,
    )?;
    Ok(json!({
        "version": 1,
        "capability": CAPABILITY,
        "status": "edit_outcome_unknown",
        "complete": false,
        "machine_state": state_name(unknown.state),
        "signal": signal_name(unknown.signal),
        "classification": detail,
        "packet_sha256": prepared.packet_sha256,
        "workspace": prepared.workspace,
        "document": prepared.document.get(),
        "target_revision": prepared.target_revision,
        "durable_operation": prepared.operation_text,
        "next_action": "authoritatively re-observe the Vellum document before any new publication; do not retry this edit or send a durable save",
        "boundary": BOUNDARY,
    }))
}

fn persistence_pending_json(
    prepared: &Prepared,
    state: PantheonRunePacketPublicationState,
    published_revision: u64,
    durable_outcome: &str,
    detail: impl std::fmt::Debug,
) -> Result<Value, Failure> {
    let terminal = advance(
        state,
        PantheonRunePacketPublicationSignal::PersistencePending,
    )?;
    Ok(json!({
        "version": 1,
        "capability": CAPABILITY,
        "status": "persistence_pending",
        "complete": false,
        "machine_state": state_name(terminal.state),
        "signal": signal_name(terminal.signal),
        "packet_sha256": prepared.packet_sha256,
        "workspace": prepared.workspace,
        "document": prepared.document.get(),
        "prior_revision": prepared.target_revision,
        "published_revision": published_revision,
        "durable_operation": prepared.operation_text,
        "durable_outcome": durable_outcome,
        "detail": format!("{detail:?}"),
        "next_action": "lookup the exact durable operation through Vellum if it was sent; otherwise preserve the dirty edit and obtain owner-guided recovery without inventing a new save identity",
        "boundary": BOUNDARY,
    }))
}

fn prepare(
    profile_path: &Path,
    request_path: &Path,
    operation_text: &str,
) -> Result<Prepared, Failure> {
    let (profile, profile_sha256) = load_profile(profile_path)?;
    let author = crate::rune_author::execute_private(&profile.rune_profile, request_path)
        .map_err(|value| Failure {
            stage: "rune_author",
            reason: format!("bounded Rune owner refused: {}", compact_status(&value)),
            next_action: "follow the Rune-owned next action; do not publish an incomplete or refused projection",
        })?;
    prepare_from_author(profile, profile_sha256, author, 1, operation_text)
}

fn prepare_qualified(
    profile_path: &Path,
    exact_request: &[u8],
    author: &Value,
    expected_profile_sha256: &str,
    operation_text: &str,
) -> Result<Prepared, Failure> {
    let (profile, profile_sha256) = load_profile(profile_path)?;
    if profile_sha256 != expected_profile_sha256 {
        return Err(Failure {
            stage: "profile_binding",
            reason: "publication profile changed during in-memory composition".into(),
            next_action: "restart from blueprint preparation with one exact deployment-owned publication profile",
        });
    }
    let request_sha256 = sha256(exact_request);
    if author.get("request_sha256").and_then(Value::as_str) != Some(request_sha256.as_str()) {
        return Err(Failure {
            stage: "request_binding",
            reason: "qualified Rune result does not bind the exact revised request bytes".into(),
            next_action: "recompute blueprint qualification from the exact current answers; do not reconstruct or substitute request bytes",
        });
    }
    prepare_from_author(profile, profile_sha256, author.clone(), 0, operation_text)
}

fn load_profile(profile_path: &Path) -> Result<(ProfileV1, String), Failure> {
    let profile_bytes = read_regular(profile_path, MAX_PROFILE_BYTES, "profile")?;
    let profile_sha256 = sha256(&profile_bytes);
    let profile: ProfileV1 = serde_json::from_slice(&profile_bytes).map_err(|error| Failure {
        stage: "profile",
        reason: format!("invalid closed publication profile: {error}"),
        next_action: "supply the deployment-owned version-one publication profile",
    })?;
    validate_profile(&profile)?;
    Ok((profile, profile_sha256))
}

fn prepare_from_author(
    profile: ProfileV1,
    profile_sha256: String,
    author: Value,
    rune_owner_requests: u8,
    operation_text: &str,
) -> Result<Prepared, Failure> {
    let operation = parse_operation(operation_text)?;
    if author.get("status").and_then(Value::as_str) != Some("projection_qualified")
        || author.get("complete").and_then(Value::as_bool) != Some(true)
    {
        return Err(Failure {
            stage: "rune_author",
            reason: format!(
                "Rune owner did not qualify a projection: {}",
                compact_status(&author)
            ),
            next_action: "resolve the exact owner questions or refusal before publication",
        });
    }
    let packet_value = json!({
        "schema": "pantheon.rune.projection-packet.v1",
        "version": 1,
        "request_sha256": author["request_sha256"],
        "toolchain": author["toolchain"],
        "confinement": author["confinement"],
        "owner_outcome": author["owner_outcome"],
    });
    let mut packet = serde_json::to_string_pretty(&packet_value).map_err(|error| Failure {
        stage: "packet",
        reason: format!("serialize qualified packet: {error}"),
        next_action: "inspect the closed owner outcome without mutating the target",
    })?;
    packet.push('\n');
    if packet.len() > profile.max_packet_bytes || packet.len() > MAX_WORKSPACE_EDIT_PAYLOAD_BYTES_V1
    {
        return Err(Failure {
            stage: "packet_limit",
            reason: format!(
                "qualified packet is {} bytes; profile allows {} and Vellum workspace-edit allows {}",
                packet.len(),
                profile.max_packet_bytes,
                MAX_WORKSPACE_EDIT_PAYLOAD_BYTES_V1
            ),
            next_action: "prepare a smaller semantically complete Rune unit or explicitly redesign and reapprove the publication contract; do not truncate or substitute the packet",
        });
    }
    let packet_sha256 = sha256(packet.as_bytes());
    let connector = connector(&profile.vellum_socket, &profile.vellum_credential)?;
    let transport =
        RemoteTransport::new(connector, TransportLimits::reference()).map_err(|error| Failure {
            stage: "vellum_transport",
            reason: format!("Vellum transport refused: {error:?}"),
            next_action: "verify the deployment-owned Vellum endpoint and limits",
        })?;
    let mut client = WorkspaceClientV1::connect(transport).map_err(|error| Failure {
        stage: "vellum_connect",
        reason: format!("Vellum connection refused: {error:?}"),
        next_action: "verify the deployment-owned endpoint and credential",
    })?;
    client.refresh_inventory().map_err(vellum_failure)?;
    let inventory = client.projection().inventory().ok_or(Failure {
        stage: "vellum_inventory",
        reason: "Vellum inventory is absent after refresh".into(),
        next_action: "stop publication and inspect the owner projection",
    })?;
    let workspace = inventory.workspace.get();
    let document = inventory
        .documents
        .iter()
        .copied()
        .find(|document| document.get() == profile.vellum_document)
        .ok_or(Failure {
            stage: "vellum_membership",
            reason: "deployment-pinned document is not a current workspace member".into(),
            next_action: "have the human deployment owner select an already-admitted packet document; this capability cannot change membership",
        })?;
    client.select_document(document).map_err(vellum_failure)?;
    client.refresh_snapshot().map_err(vellum_failure)?;
    let snapshot = client.projection().snapshot().ok_or(Failure {
        stage: "vellum_snapshot",
        reason: "Vellum snapshot is absent after refresh".into(),
        next_action: "stop publication and inspect the owner projection",
    })?;
    let target_revision = snapshot.revision;
    let target_bytes = snapshot.text.len();
    let target_sha256 = sha256(snapshot.text.as_bytes());
    let replacement_sha256 = packet_sha256.clone();
    let patch = crate::vellum_patch::minimal_patch(snapshot.text.as_ref(), &packet);
    if !patch.changed() {
        return Err(Failure {
            stage: "no_change",
            reason: "the exact qualified packet is already the current Vellum document".into(),
            next_action: "retain the current Vellum revision; do not manufacture an edit or durable save",
        });
    }
    if patch.inserted.len() > MAX_WORKSPACE_EDIT_PAYLOAD_BYTES_V1 {
        return Err(Failure {
            stage: "patch_limit",
            reason: format!(
                "minimal Vellum patch inserts {} bytes; owner workspace-edit allows {}",
                patch.inserted.len(),
                MAX_WORKSPACE_EDIT_PAYLOAD_BYTES_V1
            ),
            next_action: "split the reviewed artifact at a semantic boundary or qualify a larger owner contract; do not expand the patch span or truncate bytes",
        });
    }
    let inserted_sha256 = sha256(patch.inserted.as_bytes());
    let confirmation = confirmation(
        &packet_sha256,
        &ConfirmationBinding {
            workspace,
            document: document.get(),
            revision: target_revision,
            target_sha256: &target_sha256,
            replacement_sha256: &replacement_sha256,
            patch_start: patch.start,
            patch_end: patch.end,
            inserted_sha256: &inserted_sha256,
        },
        operation_text,
    );
    Ok(Prepared {
        profile_sha256,
        packet,
        packet_sha256,
        workspace,
        document,
        target_revision,
        target_bytes,
        target_sha256,
        replacement_sha256,
        patch,
        inserted_sha256,
        operation,
        operation_text: operation_text.to_owned(),
        confirmation,
        rune_owner_requests,
        client: Some(client),
    })
}

fn preview_json(
    prepared: &Prepared,
    advanced: PantheonRunePacketPublicationAdvanced,
    published: bool,
    duration_ms: u128,
) -> Value {
    json!({
        "version": 1,
        "capability": CAPABILITY,
        "status": "preview_returned",
        "complete": true,
        "published": published,
        "machine_state": state_name(advanced.state),
        "signal": signal_name(advanced.signal),
        "packet": prepared.packet,
        "packet_bytes": prepared.packet.len(),
        "packet_sha256": prepared.packet_sha256,
        "target": {
            "workspace": prepared.workspace,
            "document": prepared.document.get(),
            "revision": prepared.target_revision,
            "bytes": prepared.target_bytes,
            "sha256": prepared.target_sha256,
        },
        "replacement_sha256": prepared.replacement_sha256,
        "patch": {
            "start": prepared.patch.start,
            "end": prepared.patch.end,
            "inserted_bytes": prepared.patch.inserted.len(),
            "inserted_sha256": prepared.inserted_sha256,
        },
        "durable_operation": prepared.operation_text,
        "confirmation": prepared.confirmation,
        "approval_template": {
            "version": 1,
            "decision": "approve",
            "confirmation": prepared.confirmation,
            "packet_sha256": prepared.packet_sha256,
            "workspace": prepared.workspace,
            "document": prepared.document.get(),
            "target_revision": prepared.target_revision,
            "target_sha256": prepared.target_sha256,
            "replacement_sha256": prepared.replacement_sha256,
            "patch_start": prepared.patch.start,
            "patch_end": prepared.patch.end,
            "inserted_sha256": prepared.inserted_sha256,
            "durable_operation": prepared.operation_text,
        },
        "observation": {
            "schema": "rune.packet-publication.observation.v1",
            "classification": "preview",
            "requests": 1,
            "rune_owner_requests": prepared.rune_owner_requests,
            "duration_ms": duration_ms,
            "mutations": 0,
            "packet_bytes": prepared.packet.len(),
            "target_bytes": prepared.target_bytes,
            "inserted_bytes": prepared.patch.inserted.len(),
            "workspace": prepared.workspace,
            "document": prepared.document.get(),
            "revision": prepared.target_revision,
            "oculus_published": false,
        },
        "next_action": "an accountable human may write the exact approval_template to a private mode-0600 file, then invoke publish; preview evidence is never trusted without recomputation",
        "boundary": BOUNDARY,
    })
}

fn validate_profile(profile: &ProfileV1) -> Result<(), Failure> {
    if profile.version != 1
        || profile.vellum_document == 0
        || profile.max_packet_bytes == 0
        || profile.max_packet_bytes > MAX_WORKSPACE_EDIT_PAYLOAD_BYTES_V1
        || !profile.rune_profile.is_absolute()
        || !profile.vellum_socket.is_absolute()
        || !profile.vellum_credential.is_absolute()
    {
        return Err(Failure {
            stage: "profile",
            reason: "publication profile version, absolute owner paths, nonzero document, or packet bound is invalid".into(),
            next_action: "supply a closed version-one profile whose packet limit is within Vellum's owner limit",
        });
    }
    Ok(())
}

fn validate_approval(path: &Path, prepared: &Prepared) -> Result<(), Failure> {
    let bytes = crate::private_document::read(path, MAX_APPROVAL_BYTES).map_err(|error| Failure {
        stage: "approval",
        reason: format!("private approval refused: {error}"),
        next_action: "write the exact approval template to one stable, non-linked, owner-private mode-0600 regular file",
    })?;
    let approval: ApprovalV1 = serde_json::from_slice(&bytes).map_err(|error| Failure {
        stage: "approval",
        reason: format!("invalid closed approval: {error}"),
        next_action: "copy the exact current preview approval template without adding fields",
    })?;
    if approval.version != 1
        || approval.decision != "approve"
        || approval.confirmation != prepared.confirmation
        || approval.packet_sha256 != prepared.packet_sha256
        || approval.workspace != prepared.workspace
        || approval.document != prepared.document.get()
        || approval.target_revision != prepared.target_revision
        || approval.target_sha256 != prepared.target_sha256
        || approval.replacement_sha256 != prepared.replacement_sha256
        || usize::try_from(approval.patch_start).ok() != Some(prepared.patch.start)
        || usize::try_from(approval.patch_end).ok() != Some(prepared.patch.end)
        || approval.inserted_sha256 != prepared.inserted_sha256
        || approval.durable_operation != prepared.operation_text
    {
        return Err(Failure {
            stage: "approval",
            reason: "approval does not exactly bind the freshly recomputed packet, target and durable operation".into(),
            next_action: "review a fresh preview and approve that exact template; do not repair stale approval mechanically",
        });
    }
    Ok(())
}

fn connector(socket: &Path, credential: &Path) -> Result<Connector, Failure> {
    let mut store = vellum_adapter_local_authentication::LocalCredentialStoreV1::default();
    let record = store.load(credential).map_err(|error| Failure {
        stage: "credential",
        reason: format!("Vellum credential refused: {error:?}"),
        next_action: "verify the deployment-owned private credential",
    })?;
    let provider = vellum_adapter_local_authentication::LocalCredentialClaimProviderV1::new(record);
    Ok(Connector::Authenticated(
        AuthenticatedUnixConnector::with_provider(socket.to_path_buf(), provider),
    ))
}

fn parse_operation(value: &str) -> Result<DurableOperationRefV1, Failure> {
    let hex = value
        .strip_prefix("vdo1.")
        .filter(|hex| hex.len() == 32)
        .ok_or(Failure {
            stage: "operation",
            reason: "durable operation must be vdo1 followed by 32 lowercase hexadecimal digits"
                .into(),
            next_action: "persist one fresh nonzero operation identity before preview",
        })?;
    let mut bytes = [0_u8; 16];
    for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
        let pair = std::str::from_utf8(pair).map_err(|_| invalid_operation())?;
        if pair.bytes().any(|byte| byte.is_ascii_uppercase()) {
            return Err(invalid_operation());
        }
        bytes[index] = u8::from_str_radix(pair, 16).map_err(|_| invalid_operation())?;
    }
    DurableOperationRefV1::new(bytes).ok_or_else(invalid_operation)
}

fn invalid_operation() -> Failure {
    Failure {
        stage: "operation",
        reason: "durable operation identity is malformed or zero".into(),
        next_action: "persist one fresh nonzero vdo1 operation identity before preview",
    }
}

fn requalify_profile(path: &Path, expected: &str) -> Result<(), Failure> {
    let bytes = read_regular(path, MAX_PROFILE_BYTES, "profile")?;
    if sha256(&bytes) != expected {
        return Err(Failure {
            stage: "profile_changed",
            reason: "publication profile changed after target observation".into(),
            next_action: "prepare a new preview under one stable deployment profile",
        });
    }
    Ok(())
}

fn read_regular(path: &Path, max: u64, stage: &'static str) -> Result<Vec<u8>, Failure> {
    let metadata = fs::metadata(path).map_err(|error| Failure {
        stage,
        reason: format!("inspect {stage}: {error}"),
        next_action: "supply the exact bounded regular input",
    })?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > max {
        return Err(Failure {
            stage,
            reason: format!("{stage} must be a nonempty regular file within {max} bytes"),
            next_action: "supply the exact bounded regular input",
        });
    }
    fs::read(path).map_err(|error| Failure {
        stage,
        reason: format!("read {stage}: {error}"),
        next_action: "supply a stable readable input",
    })
}

fn vellum_failure(error: WorkspaceClientErrorV1) -> Failure {
    Failure {
        stage: "vellum_observation",
        reason: format!("Vellum owner observation refused: {error:?}"),
        next_action: "refresh deployment and workspace evidence without changing membership",
    }
}

fn compact_status(value: &Value) -> String {
    format!(
        "status={} state={} signal={}",
        value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("absent"),
        value
            .get("machine_state")
            .and_then(Value::as_str)
            .unwrap_or("absent"),
        value
            .get("signal")
            .and_then(Value::as_str)
            .unwrap_or("absent")
    )
}

fn confirmation(packet: &str, binding: &ConfirmationBinding<'_>, operation: &str) -> String {
    format!(
        "PUBLISH RUNE PACKET {packet} TO VELLUM WORKSPACE {} DOCUMENT {} REVISION {} CONTENT {} REPLACEMENT {} PATCH {} {} INSERTED {} DURABLE {operation}",
        binding.workspace,
        binding.document,
        binding.revision,
        binding.target_sha256,
        binding.replacement_sha256,
        binding.patch_start,
        binding.patch_end,
        binding.inserted_sha256,
    )
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn required(args: &mut impl Iterator<Item = OsString>) -> Result<OsString, &'static str> {
    args.next()
        .ok_or("rune-packet-publication requires MODE PROFILE REQUEST OPERATION [APPROVAL]")
}

fn text(value: OsString) -> Result<String, &'static str> {
    value
        .into_string()
        .map_err(|_| "Rune packet publication argument must be UTF-8")
}

fn failure_json(failure: Failure) -> Value {
    let refused = advance(
        PantheonRunePacketPublicationState::INITIAL,
        PantheonRunePacketPublicationSignal::PublicationRefused,
    );
    json!({
        "version": 1,
        "capability": CAPABILITY,
        "status": "refused",
        "complete": false,
        "machine_state": refused.as_ref().map(|value| state_name(value.state)).unwrap_or("refused"),
        "signal": "publication-refused",
        "stage": failure.stage,
        "reason": failure.reason,
        "next_action": failure.next_action,
        "boundary": BOUNDARY,
    })
}

struct Gate {
    signal: PantheonRunePacketPublicationSignal,
}

impl PantheonRunePacketPublicationEvaluator for Gate {
    fn evaluate(&mut self, obligation: &PantheonRunePacketPublicationObligation) -> bool {
        obligation.signal == pascal_signal(self.signal)
    }
}

impl PantheonRunePacketPublicationChooser for Gate {
    fn choose(
        &mut self,
        _state: PantheonRunePacketPublicationState,
        eligible: &[PantheonRunePacketPublicationCandidate],
    ) -> Option<usize> {
        (!eligible.is_empty()).then_some(0)
    }
}

impl PantheonRunePacketPublicationObserver for Gate {
    fn observe(&mut self, _observation: &PantheonRunePacketPublicationObservation) {}
}

fn advance(
    state: PantheonRunePacketPublicationState,
    signal: PantheonRunePacketPublicationSignal,
) -> Result<PantheonRunePacketPublicationAdvanced, Failure> {
    let mut evaluator = Gate { signal };
    let mut chooser = Gate { signal };
    let mut observer = Gate { signal };
    state
        .advance(&mut evaluator, &mut chooser, &mut observer)
        .map_err(|error| Failure {
            stage: "controller",
            reason: format!("forged publication controller refused {signal:?}: {error:?}"),
            next_action: "stop publication and repair the Rune/Scry/Smith contract without bypassing it",
        })
}

const fn pascal_signal(signal: PantheonRunePacketPublicationSignal) -> &'static str {
    match signal {
        PantheonRunePacketPublicationSignal::DurablePublished => "DurablePublished",
        PantheonRunePacketPublicationSignal::EditAccepted => "EditAccepted",
        PantheonRunePacketPublicationSignal::EditOutcomeUnknown => "EditOutcomeUnknown",
        PantheonRunePacketPublicationSignal::PersistencePending => "PersistencePending",
        PantheonRunePacketPublicationSignal::PreviewReturned => "PreviewReturned",
        PantheonRunePacketPublicationSignal::PublicationArmed => "PublicationArmed",
        PantheonRunePacketPublicationSignal::PublicationRefused => "PublicationRefused",
    }
}

const fn state_name(state: PantheonRunePacketPublicationState) -> &'static str {
    match state {
        PantheonRunePacketPublicationState::PublicationRequestReceived => {
            "publication_request_received"
        }
        PantheonRunePacketPublicationState::PreviewReturned => "preview_returned",
        PantheonRunePacketPublicationState::PublicationArmed => "publication_armed",
        PantheonRunePacketPublicationState::InMemoryPublished => "in_memory_published",
        PantheonRunePacketPublicationState::EditOutcomeUnknown => "edit_outcome_unknown",
        PantheonRunePacketPublicationState::PersistencePending => "persistence_pending",
        PantheonRunePacketPublicationState::DurablePublished => "durable_published",
        PantheonRunePacketPublicationState::Refused => "refused",
    }
}

const fn signal_name(signal: PantheonRunePacketPublicationSignal) -> &'static str {
    match signal {
        PantheonRunePacketPublicationSignal::DurablePublished => "durable-published",
        PantheonRunePacketPublicationSignal::EditAccepted => "edit-accepted",
        PantheonRunePacketPublicationSignal::EditOutcomeUnknown => "edit-outcome-unknown",
        PantheonRunePacketPublicationSignal::PersistencePending => "persistence-pending",
        PantheonRunePacketPublicationSignal::PreviewReturned => "preview-returned",
        PantheonRunePacketPublicationSignal::PublicationArmed => "publication-armed",
        PantheonRunePacketPublicationSignal::PublicationRefused => "publication-refused",
    }
}
