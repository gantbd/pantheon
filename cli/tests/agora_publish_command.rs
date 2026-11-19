#![cfg(unix)]

use std::{
    fs,
    io::{Read, Write},
    os::unix::{fs::PermissionsExt, net::UnixListener},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use agora_actor_publication_port::{ActorPublicationAcceptedV1, ActorPublicationObservationV1};
use agora_protocol::v2::ClientIdV2;
use agora_wire::{
    WireLimits,
    local_handshake::{
        COMPOSITION_CLIENT_HELLO_BYTES_V1, ServerIncarnationV1, decode_composition_client_hello_v1,
        encode_composition_server_hello_v1,
    },
    local_transient::{
        LocalReplayEpochV1, LocalRouteV1, decode_local_routed_frame_v1,
        encode_local_routed_frame_v1, encode_local_session_hello_v1,
    },
    scheduled::{
        ScheduledWireReplyV1, ScheduledWireRequestV1, decode_scheduled_request_v1,
        encode_scheduled_reply_v1,
    },
    v2::{WireReplyV2, WireRequestV2, decode_request_v2, encode_reply_v2},
};
use sha2::{Digest, Sha256};

static NEXT: AtomicU64 = AtomicU64::new(0);

#[test]
fn execute_uses_exact_actor_publication_and_emits_content_free_receipt() {
    let fixture = Fixture::new();
    let server = fixture.serve(ScheduledWireReplyV1::ActorPublished(Ok(
        ActorPublicationAcceptedV1 {
            revision: 12,
            message_id: [0x44; 16],
            message_sequence: 9,
        },
    )));
    let digest = hex(&Sha256::digest(&fixture.request_bytes));
    let confirmation = format!(
        "PUBLISH AGORA ACTOR MESSAGE OPERATION {} REQUEST SHA256 {digest}",
        "11".repeat(16)
    );
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "agora-message-publish",
            fixture.socket.to_str().unwrap(),
            "operator",
            fixture.request.to_str().unwrap(),
            &confirmation,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["capability"],
        "messages.actor-publication.execute.v1"
    );
    assert_eq!(report["status"], "accepted");
    assert_eq!(report["receipt"]["revision"], 12);
    assert_eq!(report["receipt"]["message_id"], "44".repeat(16));
    let raw = String::from_utf8(output.stdout).unwrap();
    assert!(!raw.contains("private body"));
    assert!(!raw.contains("operator"));
    server.join().unwrap();
}

#[test]
fn observation_projects_owner_retained_accepted_evidence() {
    let fixture = Fixture::new();
    let server = fixture.serve(ScheduledWireReplyV1::ActorObserved(Ok(
        ActorPublicationObservationV1::Accepted(ActorPublicationAcceptedV1 {
            revision: 12,
            message_id: [0x44; 16],
            message_sequence: 9,
        }),
    )));
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "agora-message-publication-observe",
            fixture.socket.to_str().unwrap(),
            "operator",
            fixture.request.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["capability"],
        "messages.actor-publication.observe.v1"
    );
    assert_eq!(report["status"], "accepted");
    assert_eq!(report["receipt"]["message_sequence"], 9);
    server.join().unwrap();
}

#[test]
fn observation_projects_authoritative_absence_without_mutation_authority() {
    let fixture = Fixture::new();
    let server = fixture.serve(ScheduledWireReplyV1::ActorObserved(Ok(
        ActorPublicationObservationV1::Absent,
    )));
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "agora-message-publication-observe",
            fixture.socket.to_str().unwrap(),
            "operator",
            fixture.request.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "absent");
    assert!(
        report["boundary"]
            .as_str()
            .unwrap()
            .contains("not publication authority")
    );
    assert!(
        report["next_action"]
            .as_str()
            .unwrap()
            .contains("separately admitted")
    );
    server.join().unwrap();
}

struct Fixture {
    root: std::path::PathBuf,
    socket: std::path::PathBuf,
    request: std::path::PathBuf,
    request_bytes: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "pantheon-agora-publish-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let request = root.join("request.json");
        let request_bytes = br#"{"version":1,"operation_id":"11111111111111111111111111111111","room_id":"live-room","expected_revision":3,"logical_time":41,"body":"private body","addressed_to":["operator"]}"#.to_vec();
        fs::write(&request, &request_bytes).unwrap();
        fs::set_permissions(&request, fs::Permissions::from_mode(0o600)).unwrap();
        let socket = root.join("agora.sock");
        Self {
            root,
            socket,
            request,
            request_bytes,
        }
    }

    fn serve(&self, reply: ScheduledWireReplyV1) -> thread::JoinHandle<()> {
        let listener = UnixListener::bind(&self.socket).unwrap();
        fs::set_permissions(&self.socket, fs::Permissions::from_mode(0o600)).unwrap();
        thread::spawn(move || serve(listener, reply))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.socket);
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn serve(listener: UnixListener, reply: ScheduledWireReplyV1) {
    let limits = WireLimits::reference();
    let (mut stream, _) = listener.accept().unwrap();
    let mut hello = [0; COMPOSITION_CLIENT_HELLO_BYTES_V1];
    stream.read_exact(&mut hello).unwrap();
    decode_composition_client_hello_v1(&hello).unwrap();
    stream
        .write_all(
            &encode_composition_server_hello_v1(
                ServerIncarnationV1::new([7; 16]),
                limits.max_frame_bytes as u32,
                4,
            )
            .unwrap(),
        )
        .unwrap();

    let routed = read_frame(&mut stream);
    let routed = decode_local_routed_frame_v1(&routed, limits.max_frame_bytes).unwrap();
    assert_eq!(routed.route(), LocalRouteV1::Conversation);
    let WireRequestV2::Connect { participant_id } =
        decode_request_v2(routed.payload(), limits).unwrap()
    else {
        panic!("connect expected");
    };
    assert_eq!(participant_id, "operator");
    write_routed(
        &mut stream,
        LocalRouteV1::Conversation,
        &encode_reply_v2(&WireReplyV2::Connected(ClientIdV2(4)), limits).unwrap(),
        limits,
    );
    let replay = LocalReplayEpochV1::new([8; 32]).unwrap();
    write_routed(
        &mut stream,
        LocalRouteV1::Control,
        &encode_local_session_hello_v1(replay),
        limits,
    );

    let routed = read_frame(&mut stream);
    let routed = decode_local_routed_frame_v1(&routed, limits.max_frame_bytes).unwrap();
    assert_eq!(routed.route(), LocalRouteV1::Scheduled);
    let request = decode_scheduled_request_v1(routed.payload(), limits).unwrap();
    let actor = match request {
        ScheduledWireRequestV1::PublishActor(request)
        | ScheduledWireRequestV1::ObserveActor(request) => request,
        _ => panic!("actor publication expected"),
    };
    assert_eq!(actor.id.into_bytes(), [0x11; 16]);
    assert_eq!(actor.intent.room_id(), "live-room");
    assert_eq!(actor.intent.expected_revision(), 3);
    assert_eq!(actor.intent.logical_time(), 41);
    assert_eq!(actor.intent.body(), "private body");
    assert_eq!(actor.intent.addressed_to(), ["operator"]);
    write_routed(
        &mut stream,
        LocalRouteV1::Scheduled,
        &encode_scheduled_reply_v1(&reply, limits).unwrap(),
        limits,
    );
}

fn read_frame(stream: &mut impl Read) -> Vec<u8> {
    let mut length = [0; 4];
    stream.read_exact(&mut length).unwrap();
    let mut frame = vec![0; u32::from_be_bytes(length) as usize];
    stream.read_exact(&mut frame).unwrap();
    frame
}

fn write_routed(stream: &mut impl Write, route: LocalRouteV1, payload: &[u8], limits: WireLimits) {
    let frame = encode_local_routed_frame_v1(route, payload, limits.max_frame_bytes).unwrap();
    stream
        .write_all(&(frame.len() as u32).to_be_bytes())
        .unwrap();
    stream.write_all(&frame).unwrap();
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
