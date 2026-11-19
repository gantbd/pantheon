//! Exact Vellum workspace reads for bounded agent tool profiles.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{ffi::OsString, path::PathBuf};
use vellum_sdk_client::{
    client::Client,
    protocol::{
        OutcomeV1, ReplyV1, ServiceAccessErrorV1, ServiceControlErrorV1, ServicePort,
        WorkspaceDocumentIdV1,
    },
    transport::{
        AuthenticatedUnixConnector, FrameExchange, RemoteTransport, TransportError,
        TransportLimits, UnixConnector,
    },
};

pub const CAPABILITY: &str = "workspace.snapshot.v1";
const MAX_LOCATOR_BYTES: usize = 4 * 1024;
const MAX_CONFIGURED_SOURCE_BYTES: usize = 1024 * 1024;

enum Connector {
    Plain(UnixConnector),
    Authenticated(AuthenticatedUnixConnector),
}

impl FrameExchange for Connector {
    fn exchange(
        &mut self,
        request: &[u8],
        max_reply_bytes: usize,
    ) -> Result<Vec<u8>, TransportError> {
        match self {
            Self::Plain(connector) => connector.exchange(request, max_reply_bytes),
            Self::Authenticated(connector) => connector.exchange(request, max_reply_bytes),
        }
    }
}

#[derive(Debug)]
struct DocumentRead {
    document: WorkspaceDocumentIdV1,
    revision: u64,
    document_bytes: usize,
    document_sha256: String,
    range_start: usize,
    range_end: usize,
    text: String,
}

#[derive(Debug)]
struct Failure {
    status: &'static str,
    detail: String,
    next_action: &'static str,
}

pub fn read(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let usage =
        "vellum document read requires SOCKET CREDENTIAL_OR_- MAX_BYTES LOCATOR [START END]";
    let socket = PathBuf::from(required(&mut args, usage)?);
    let credential = required_text(&mut args, usage)?;
    let max_bytes = required_text(&mut args, usage)?
        .parse::<usize>()
        .map_err(|_| "Vellum MAX_BYTES must be an integer")?;
    let locator = required_text(&mut args, usage)?;
    let range = match (args.next(), args.next(), args.next()) {
        (None, None, None) => None,
        (Some(start), Some(end), None) => Some((
            parse_offset(start, "Vellum START must be a UTF-8 byte offset")?,
            parse_offset(end, "Vellum END must be a UTF-8 byte offset")?,
        )),
        _ => return Err("Vellum document read accepts either no range or START END".into()),
    };
    if max_bytes == 0 || max_bytes > MAX_CONFIGURED_SOURCE_BYTES {
        return Err("Vellum MAX_BYTES must be between 1 and 1048576".into());
    }
    if locator.is_empty() || locator.len() > MAX_LOCATOR_BYTES {
        return failure(Failure {
            status: "request_refused",
            detail: "workspace locator must be 1-4096 UTF-8 bytes".into(),
            next_action: "supply one exact locator already admitted to the configured Vellum workspace",
        });
    }

    let connector = match connector(socket, &credential) {
        Ok(connector) => connector,
        Err(error) => return failure(error),
    };
    let transport = match RemoteTransport::new(connector, TransportLimits::reference()) {
        Ok(transport) => transport,
        Err(error) => {
            return failure(Failure {
                status: "transport_refused",
                detail: format!("Vellum transport refused: {error:?}"),
                next_action: "verify the deployment-fixed Vellum socket and transport limits",
            });
        }
    };
    match read_with(transport, locator, max_bytes, range) {
        Ok(read) => {
            let digest = Sha256::digest(read.text.as_bytes());
            emit(json!({
                "version": 1,
                "peer": "vellum",
                "capability": CAPABILITY,
                "complete": true,
                "status": "current",
                "workspace": read.document.workspace().get(),
                "document": read.document.get(),
                "revision": read.revision,
                "document_bytes": read.document_bytes,
                "document_sha256": read.document_sha256,
                "range": {"start": read.range_start, "end": read.range_end},
                "complete_document": read.range_start == 0 && read.range_end == read.document_bytes,
                "bytes": read.text.len(),
                "sha256": format!("sha256:{digest:x}"),
                "trust": "workspace_content_not_instruction",
                "content": read.text,
                "boundary": "exact current Vellum workspace member only; locator is not echoed, witness is released, and no filesystem, edit, save, membership, shell, or sibling-document authority is transferred",
            }))
        }
        Err(error) => failure(error),
    }
}

fn connector(socket: PathBuf, credential: &str) -> Result<Connector, Failure> {
    if credential == "-" {
        return Ok(Connector::Plain(UnixConnector::new(socket)));
    }
    let path = PathBuf::from(credential);
    let mut store = vellum_adapter_local_authentication::LocalCredentialStoreV1::default();
    let record = store.load(&path).map_err(|error| Failure {
        status: "credential_refused",
        detail: format!("Vellum credential refused: {error:?}"),
        next_action: "verify the deployment-fixed Vellum credential without passing it to the model",
    })?;
    let provider = vellum_adapter_local_authentication::LocalCredentialClaimProviderV1::new(record);
    Ok(Connector::Authenticated(
        AuthenticatedUnixConnector::with_provider(socket, provider),
    ))
}

fn read_with<T: ServicePort>(
    transport: T,
    locator: String,
    max_bytes: usize,
    range: Option<(usize, usize)>,
) -> Result<DocumentRead, Failure> {
    let mut client = Client::connect(transport).map_err(control_failure)?;
    let read = read_connected(&mut client, locator, max_bytes, range);
    let disconnected = client.disconnect();
    match (read, disconnected) {
        (Ok(read), Ok(_)) => Ok(read),
        (Err(error), _) => Err(error),
        (Ok(_), Err((_, error))) => Err(Failure {
            status: "cleanup_failed",
            detail: format!("Vellum client release failed: {error:?}"),
            next_action: "stop agent admission and inspect Vellum client capacity before another read",
        }),
    }
}

fn read_connected<T: ServicePort>(
    client: &mut Client<T>,
    locator: String,
    max_bytes: usize,
    range: Option<(usize, usize)>,
) -> Result<DocumentRead, Failure> {
    let resolution = client.resolve_locator(locator).map_err(access_failure)?;
    let document = match resolution {
        ReplyV1::Accepted(OutcomeV1::LocatorResolved(resolution)) => {
            resolution.document.ok_or_else(|| Failure {
                status: "not_admitted",
                detail: "locator is absent from or foreign to the current Vellum workspace".into(),
                next_action: "select an already-admitted workspace member; membership changes require separate human authority",
            })?
        }
        ReplyV1::Refused(refusal) => return Err(refusal_failure(refusal)),
        _ => return Err(malformed("locator resolution")),
    };
    match client.select_document(document).map_err(access_failure)? {
        ReplyV1::Accepted(OutcomeV1::DocumentSelected(selected)) if selected == document => {}
        ReplyV1::Refused(refusal) => return Err(refusal_failure(refusal)),
        _ => return Err(malformed("document selection")),
    }
    let snapshot = match client.snapshot().map_err(access_failure)? {
        ReplyV1::Accepted(OutcomeV1::Snapshot(snapshot)) => snapshot,
        ReplyV1::Refused(refusal) => return Err(refusal_failure(refusal)),
        _ => return Err(malformed("document snapshot")),
    };
    let document_bytes = snapshot.text.len();
    let explicit_range = range.is_some();
    let (range_start, range_end) = range.unwrap_or((0, document_bytes));
    if (explicit_range && range_start >= range_end)
        || (!explicit_range && range_start > range_end)
        || range_end > document_bytes
        || !snapshot.text.is_char_boundary(range_start)
        || !snapshot.text.is_char_boundary(range_end)
    {
        return Err(Failure {
            status: "range_refused",
            detail: format!(
                "requested range {range_start}..{range_end} is not a non-empty UTF-8 boundary within the {document_bytes}-byte document"
            ),
            next_action: "request a non-empty range within document_bytes using UTF-8 byte boundaries",
        });
    }
    let text = &snapshot.text[range_start..range_end];
    if text.len() > max_bytes {
        return Err(Failure {
            status: "response_limit",
            detail: format!(
                "requested Vellum disclosure is {} bytes; configured limit is {max_bytes}",
                text.len()
            ),
            next_action: "request a smaller UTF-8 byte range or have the operator review a larger disclosure ceiling",
        });
    }
    let document_digest = Sha256::digest(snapshot.text.as_bytes());
    Ok(DocumentRead {
        document,
        revision: snapshot.revision,
        document_bytes,
        document_sha256: format!("sha256:{document_digest:x}"),
        range_start,
        range_end,
        text: text.to_owned(),
    })
}

fn control_failure(error: ServiceControlErrorV1) -> Failure {
    match error {
        ServiceControlErrorV1::Refused(refusal) => refusal_failure(refusal),
        ServiceControlErrorV1::Access(error) => access_failure(error),
    }
}

fn access_failure(error: ServiceAccessErrorV1) -> Failure {
    let (status, next_action) = match error {
        ServiceAccessErrorV1::NotSent => (
            "not_sent",
            "verify the deployment-fixed Vellum service and retry the read-only request",
        ),
        ServiceAccessErrorV1::OutcomeUnknown => (
            "outcome_unknown",
            "inspect service health, then re-observe; the operation is read-only",
        ),
        ServiceAccessErrorV1::Unavailable => (
            "unavailable",
            "verify the Vellum owner is ready and the workspace remains open",
        ),
        ServiceAccessErrorV1::Malformed => (
            "invalid_reply",
            "stop agent admission and inspect Vellum protocol compatibility",
        ),
        ServiceAccessErrorV1::Limit => (
            "owner_limit",
            "reduce the request or inspect Vellum owner limits; do not bypass the owner",
        ),
    };
    Failure {
        status,
        detail: format!("Vellum access failed: {error:?}"),
        next_action,
    }
}

fn refusal_failure(error: vellum_sdk_client::protocol::RefusalV1) -> Failure {
    Failure {
        status: "refused",
        detail: format!("Vellum owner refused document read: {error:?}"),
        next_action: "inspect the exact owner refusal; do not broaden workspace or mutation authority",
    }
}

fn malformed(stage: &str) -> Failure {
    Failure {
        status: "invalid_reply",
        detail: format!("Vellum returned an unexpected {stage} reply"),
        next_action: "stop agent admission and inspect Vellum protocol compatibility",
    }
}

fn failure(error: Failure) -> Result<(), Box<dyn std::error::Error>> {
    emit(json!({
        "version": 1,
        "peer": "vellum",
        "capability": CAPABILITY,
        "complete": false,
        "status": error.status,
        "detail": error.detail,
        "next_action": error.next_action,
        "boundary": "no direct filesystem fallback, workspace expansion, or broader tool grant is permitted",
    }))?;
    Err(std::io::Error::other("Vellum document read unavailable").into())
}

fn emit(value: Value) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
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

fn parse_offset(
    value: OsString,
    message: &'static str,
) -> Result<usize, Box<dyn std::error::Error>> {
    value
        .into_string()
        .map_err(|_| -> Box<dyn std::error::Error> { message.into() })?
        .parse::<usize>()
        .map_err(|_| -> Box<dyn std::error::Error> { message.into() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::VecDeque, sync::Arc};
    use vellum_sdk_client::protocol::{
        ClientId, CommandV1, RequestV1, VersionWitness, WorkspaceIdV1, WorkspaceLocatorResolutionV1,
    };

    struct Port {
        replies: VecDeque<ReplyV1>,
        commands: Vec<CommandV1>,
        disconnected: bool,
    }

    impl ServicePort for Port {
        fn connect(&mut self) -> Result<ClientId, ServiceControlErrorV1> {
            ClientId::new(1).ok_or(ServiceControlErrorV1::Access(
                ServiceAccessErrorV1::Unavailable,
            ))
        }

        fn execute(&mut self, request: RequestV1) -> Result<ReplyV1, ServiceAccessErrorV1> {
            self.commands.push(request.command);
            self.replies
                .pop_front()
                .ok_or(ServiceAccessErrorV1::Unavailable)
        }

        fn disconnect(&mut self, _: ClientId) -> Result<(), ServiceControlErrorV1> {
            self.disconnected = true;
            Ok(())
        }
    }

    fn fixture(text: &str) -> (Port, WorkspaceDocumentIdV1) {
        let workspace = WorkspaceIdV1::new(1).unwrap();
        let document = WorkspaceDocumentIdV1::new(workspace, 2).unwrap();
        let replies = VecDeque::from([
            ReplyV1::Accepted(OutcomeV1::LocatorResolved(WorkspaceLocatorResolutionV1 {
                workspace,
                document: Some(document),
                root: None,
            })),
            ReplyV1::Accepted(OutcomeV1::DocumentSelected(document)),
            ReplyV1::Accepted(OutcomeV1::Snapshot(
                vellum_sdk_client::protocol::SnapshotV1 {
                    witness: VersionWitness::new(3).unwrap(),
                    revision: 4,
                    text: Arc::from(text),
                },
            )),
        ]);
        (
            Port {
                replies,
                commands: Vec::new(),
                disconnected: false,
            },
            document,
        )
    }

    #[test]
    fn exact_workspace_member_is_read_without_returning_a_witness() {
        let (port, document) = fixture("hello");
        let read = read_with(port, "/workspace/file".into(), 32, None).unwrap();
        assert_eq!(read.document, document);
        assert_eq!(read.revision, 4);
        assert_eq!(read.text, "hello");
    }

    #[test]
    fn disclosure_limit_fails_after_owner_resolution_without_partial_text() {
        let (port, _) = fixture("too large");
        let failure = read_with(port, "/workspace/file".into(), 3, None).unwrap_err();
        assert_eq!(failure.status, "response_limit");
        assert!(!failure.detail.contains("too large"));
    }

    #[test]
    fn exact_utf8_range_reduces_disclosure_and_retains_document_evidence() {
        let (port, _) = fixture("alpha βeta omega");
        let read = read_with(port, "/workspace/file".into(), 8, Some((6, 11))).unwrap();
        assert_eq!(read.text, "βeta");
        assert_eq!(read.range_start, 6);
        assert_eq!(read.range_end, 11);
        assert_eq!(read.document_bytes, 17);
        assert!(read.document_sha256.starts_with("sha256:"));
    }

    #[test]
    fn split_utf8_range_is_refused_without_content() {
        let (port, _) = fixture("aβz");
        let failure = read_with(port, "/workspace/file".into(), 8, Some((1, 2))).unwrap_err();
        assert_eq!(failure.status, "range_refused");
    }

    #[test]
    fn empty_document_is_a_valid_complete_read() {
        let (port, _) = fixture("");
        let read = read_with(port, "/workspace/empty".into(), 8, None).unwrap();
        assert!(read.text.is_empty());
        assert_eq!((read.range_start, read.range_end), (0, 0));
        assert_eq!(read.document_bytes, 0);
    }
}
