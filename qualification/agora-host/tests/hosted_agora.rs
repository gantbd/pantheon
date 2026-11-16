#![cfg(target_os = "macos")]

use std::{
    fs,
    os::unix::{fs::PermissionsExt, net::UnixStream},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use aegis_client::ServicePort;
use aegis_protocol::{
    DecisionHandleV1, DecisionSummaryV1, EngineNamespaceV1, OutcomeV1, ReplyV1, SessionHandleV1,
};
use agora_aegis_verifier::{AegisContextV1, AegisVerifierV1};
use agora_client::scheduled::ScheduledPublicationWireClientV1;
use agora_client::v2::ClientV2;
use agora_engine::{
    AdmissionPolicy, CommandV2 as EngineCommandV2, Engine, LogicalTime, Participant, ParticipantId,
    ParticipantKind, RoomId,
};
use agora_host::{HostLimits, HostTimeV1, VerifiedSubjectV1, v2::HostV2};
use agora_local_client::ScheduledLocalClientV1;
use agora_local_daemon::{
    AuthenticatorV1, DaemonCleanupV1, DaemonLimitsV1, DaemonV1, HostClockV1, ServerIncarnationV1,
};
use agora_scheduled_publication_port::{
    ScheduledPublicationIdV1, ScheduledPublicationIntentV1, ScheduledPublicationObservationV1,
    ScheduledPublicationRequestV1,
};
use agora_service::v2::{ServiceLimitsV2, ServiceV2};
use agora_wire::WireLimits;
use ratatui::{Terminal, backend::TestBackend};
use workbench_tui::{Focus, Shell, Surface, draw_shell};
use workbench_tui_cli::{
    headless::{HeadlessRequest, observe as observe_headless},
    native_host::{ChronosHostOptions, NativeHost},
    profile::AgoraProfile,
    registry::{HostedPeerId, ShutdownDisposition},
};

struct AuthorityPort;

impl ServicePort for AuthorityPort {
    type Error = ();

    fn execute(
        &mut self,
        _: SessionHandleV1,
        command: aegis_protocol::CommandV1,
    ) -> Result<ReplyV1, Self::Error> {
        let aegis_protocol::CommandV1::Authorize { action, .. } = command else {
            return Err(());
        };
        Ok(ReplyV1::Accepted(OutcomeV1::Authorized(
            DecisionSummaryV1 {
                handle: DecisionHandleV1(1),
                epoch: 1,
                expires_at: u64::MAX,
                engine: EngineNamespaceV1::Agora,
                action,
            },
        )))
    }
}

struct Authentication;

impl AuthenticatorV1 for Authentication {
    type Error = ();

    fn authenticate(&mut self, _: &UnixStream) -> Result<AegisContextV1, Self::Error> {
        Ok(AegisContextV1::new(
            "operator",
            VerifiedSubjectV1::from_verifier([7; 16]),
            SessionHandleV1(1),
            aegis_protocol::GrantHandleV1(1),
        ))
    }
}

struct Clock(u64);

impl HostClockV1 for Clock {
    fn now(&mut self) -> HostTimeV1 {
        let value = self.0;
        self.0 += 1;
        HostTimeV1(value)
    }
}

fn scratch() -> PathBuf {
    PathBuf::from("/private/tmp").join(format!("workbench-all-owner-{}", std::process::id()))
}

fn wire_limits() -> WireLimits {
    WireLimits {
        max_frame_bytes: 8_192,
        max_text_bytes: 128,
        max_fields: 512,
        max_list_items: 32,
    }
}

#[test]
fn one_process_owns_all_three_peers_and_serves_real_agora_client() {
    let root = scratch();
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let socket = root.join("agora.sock");
    let chronos = root.join("chronos");
    let vellum = root.join("vellum.sock");
    let source = root.join("source.txt");
    fs::write(&source, "qualified three-peer owner").unwrap();

    let mut engine = Engine::new();
    engine
        .apply(agora_engine::CommandV2::RegisterParticipant(
            Participant::new("operator", "Operator", ParticipantKind::Human),
        ))
        .unwrap();
    engine
        .apply(EngineCommandV2::CreateRoom {
            room_id: RoomId::from("operator-room"),
            name: "Operator Room".into(),
            owner: ParticipantId::from("operator"),
            admission: AdmissionPolicy::InviteOnly,
        })
        .unwrap();
    engine
        .apply(EngineCommandV2::Speak {
            room_id: RoomId::from("operator-room"),
            author: ParticipantId::from("operator"),
            at: LogicalTime(1),
            body: "All three owners reached the Workbench surface".into(),
        })
        .unwrap();
    let host = HostV2::new(
        ServiceV2::new(engine, ServiceLimitsV2::default()),
        AegisVerifierV1::new(AuthorityPort, [5; 16]),
        HostLimits {
            max_sessions: 4,
            max_retired_sessions: 8,
            wire: wire_limits(),
        },
    );
    let daemon = DaemonV1::bind_new(
        &socket,
        host,
        Authentication,
        Clock(1),
        ServerIncarnationV1::new([9; 16]),
        DaemonLimitsV1 {
            max_frame_bytes: 8_192,
            max_frames_after_connect: 8,
            io_timeout: Duration::from_secs(2),
        },
    )
    .unwrap();
    let stopping = Arc::new(AtomicBool::new(false));
    let owner_stopping = Arc::clone(&stopping);
    let owner = thread::spawn(move || {
        let mut daemon = daemon;
        while !owner_stopping.load(Ordering::Acquire) {
            let _ = daemon.serve_next();
        }
        let _ = daemon.begin_drain();
        daemon.stop()
    });

    let mut native = NativeHost::new();
    native
        .start_chronos("Atlas", &chronos, ChronosHostOptions::default())
        .unwrap();
    native
        .start_vellum(&vellum, std::slice::from_ref(&source), None, false, false)
        .unwrap();
    let shutdown_socket = socket.clone();
    let mut owner = Some(owner);
    native
        .start_agora(|| {
            Ok::<_, String>(((), move || {
                stopping.store(true, Ordering::Release);
                let _ = UnixStream::connect(&shutdown_socket);
                match owner.take().map(thread::JoinHandle::join) {
                    Some(Ok(stopped)) if stopped.cleanup == DaemonCleanupV1::Clean => {
                        ShutdownDisposition::Clean
                    }
                    _ => ShutdownDisposition::OutcomeUnknown,
                }
            }))
        })
        .unwrap();
    native.seal().unwrap();
    assert!(chronos.join("chronos.sock").exists());
    assert!(vellum.exists());

    let observation = observe_headless(HeadlessRequest {
        workspace: "All peers".into(),
        agora: Some(AgoraProfile {
            deployment: workbench_tui_cli::profile::DeploymentMode::Attach,
            transport: workbench_tui_cli::profile::AgoraTransportProfile::Conversation,
            socket: socket.clone(),
            participant: "operator".into(),
            host: None,
            requires: vec!["rooms.read.v2".into()],
        }),
        moira_socket: None,
        moira_requires: Vec::new(),
        daimon_socket: None,
        daimon_requires: Vec::new(),
        hermes_socket: None,
        hermes_stream: None,
        hermes_consumer: None,
        hermes_requires: Vec::new(),
        aegis_socket: None,
        aegis_requires: Vec::new(),
        grounding_endpoint: None,
        grounding_token: None,
        grounding_requires: Vec::new(),
        chronos: vec![chronos_tui::Workstream {
            name: "Atlas".into(),
            directory: chronos.clone(),
        }],
        chronos_requires: vec!["tasks.observe.v1".into()],
        vellum_socket: Some(vellum.clone()),
        vellum_credential: None,
        vellum_requires: vec!["workspace.inventory.v1".into()],
    })
    .unwrap();
    assert!(observation.complete);
    assert_eq!(observation.workspace, "All peers");
    assert_eq!(
        observation
            .peers
            .iter()
            .map(|peer| (
                peer["peer"].as_str().unwrap(),
                peer["status"].as_str().unwrap()
            ))
            .collect::<Vec<_>>(),
        [
            ("agora", "current"),
            ("chronos", "current"),
            ("vellum", "current")
        ]
    );
    assert_eq!(observation.peers[0]["rooms"], 1);

    let (client_id, port) = agora_tui_local_client::LocalServicePort::connect(
        &socket,
        "operator",
        agora_tui_local_client::LocalClientLimits::default(),
    )
    .unwrap();
    let mut component = agora_tui::Component::new(ClientV2::new(client_id, port));
    let shell = Shell::new(vec![Surface::Agora]).unwrap();
    let render = |component: &agora_tui::Component| {
        let mut terminal = Terminal::new(TestBackend::new(120, 20)).unwrap();
        terminal
            .draw(|frame| {
                draw_shell(
                    frame,
                    &shell,
                    None,
                    None,
                    Some(component),
                    Focus::Chronos,
                    "three owners",
                )
            })
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    };
    for _ in 0..200 {
        component.poll();
        if render(&component).contains("Operator Room") {
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    assert!(render(&component).contains("Operator Room"));
    component.update(agora_tui::Input::Open, true);
    for _ in 0..200 {
        component.poll();
        if render(&component).contains("All three owners reached the Workbench surface") {
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    assert!(render(&component).contains("All three owners reached the Workbench surface"));
    drop(component);

    let receipt = native.shutdown().unwrap();
    assert!(receipt.clean());
    assert_eq!(receipt.receipts.len(), 3);
    assert_eq!(receipt.receipts[0].peer, HostedPeerId::Agora);
    assert_eq!(receipt.receipts[1].peer, HostedPeerId::Vellum);
    assert_eq!(
        receipt.receipts[2].peer,
        HostedPeerId::Chronos("Atlas".into())
    );
    assert!(!socket.exists());
    assert!(!vellum.exists());
    assert!(!vellum.with_file_name("vellum.sock.control").exists());
    assert!(!chronos.join("chronos.sock").exists());
    assert!(!chronos.join("chronos.control.sock").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn workbench_lifecycle_returns_the_same_public_scheduled_client() {
    let root = scratch().with_file_name(format!(
        "workbench-agora-composition-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let socket = root.join("agora-composition.sock");

    let mut engine = Engine::new();
    engine
        .apply(EngineCommandV2::RegisterParticipant(Participant::new(
            "operator",
            "Operator",
            ParticipantKind::Human,
        )))
        .unwrap();
    engine
        .apply(EngineCommandV2::CreateRoom {
            room_id: RoomId::from("scheduled-room"),
            name: "Scheduled Room".into(),
            owner: ParticipantId::from("operator"),
            admission: AdmissionPolicy::InviteOnly,
        })
        .unwrap();
    let host = HostV2::new(
        ServiceV2::new(engine, ServiceLimitsV2::default()),
        AegisVerifierV1::new(AuthorityPort, [5; 16]),
        HostLimits {
            max_sessions: 4,
            max_retired_sessions: 8,
            wire: wire_limits(),
        },
    );
    let daemon = DaemonV1::bind_new_composition(
        &socket,
        host,
        Authentication,
        Clock(2),
        ServerIncarnationV1::new([10; 16]),
        DaemonLimitsV1 {
            max_frame_bytes: 8_192,
            max_frames_after_connect: 4,
            io_timeout: Duration::from_secs(2),
        },
    )
    .unwrap();
    let stopping = Arc::new(AtomicBool::new(false));
    let owner_stopping = Arc::clone(&stopping);
    let owner = thread::spawn(move || {
        let mut daemon = daemon;
        while !owner_stopping.load(Ordering::Acquire) {
            let _ = daemon.serve_next();
        }
        let _ = daemon.begin_drain();
        daemon.stop()
    });

    let mut native = NativeHost::new();
    let client_socket = socket.clone();
    let shutdown_socket = socket.clone();
    let mut owner = Some(owner);
    let mut client = native
        .start_agora(|| {
            Ok::<_, String>((
                ScheduledLocalClientV1::new(
                    client_socket,
                    "operator",
                    wire_limits(),
                    Duration::from_secs(2),
                ),
                move || {
                    stopping.store(true, Ordering::Release);
                    let _ = UnixStream::connect(&shutdown_socket);
                    match owner.take().map(thread::JoinHandle::join) {
                        Some(Ok(stopped)) if stopped.cleanup == DaemonCleanupV1::Clean => {
                            ShutdownDisposition::Clean
                        }
                        _ => ShutdownDisposition::OutcomeUnknown,
                    }
                },
            ))
        })
        .unwrap();
    native.seal().unwrap();

    let request = ScheduledPublicationRequestV1 {
        id: ScheduledPublicationIdV1::new([8; 16]).unwrap(),
        intent: ScheduledPublicationIntentV1::new(
            "scheduled-room",
            2,
            "same public client under Workbench lifecycle",
        ),
    };
    let accepted = client.publish_scheduled(request.clone()).unwrap().unwrap();
    assert_eq!(
        client.observe_scheduled(request).unwrap().unwrap(),
        ScheduledPublicationObservationV1::Accepted(accepted)
    );

    let receipt = native.shutdown().unwrap();
    assert!(receipt.clean());
    assert_eq!(receipt.receipts.len(), 1);
    assert_eq!(receipt.receipts[0].peer, HostedPeerId::Agora);
    assert!(!socket.exists());
    fs::remove_dir_all(root).unwrap();
}
