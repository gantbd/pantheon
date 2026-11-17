#![cfg(unix)]

use std::{
    convert::Infallible,
    fs,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use moira_client::unix::{AgentParticipationUnixPort, ParticipationClientLimitsV1};
use moira_core::agent_participation::{LimitsV1 as DomainLimits, persistence::CodecLimitsV1};
use moira_local_daemon::unix::{AgentParticipationAuthenticator, AgentParticipationDaemon};
use moira_protocol::{WorkflowIdV1, agent_participation as p, deliberation as d};
use moira_service::{AccessV1, agent_participation::AgentParticipationService};
use moira_sqlite::agent_participation::{
    AgentParticipationSqliteStore, AgentParticipationStoreLimitsV1,
};
use pantheon::{Focus, PeerViews, Shell, Surface, draw_shell_with_moira};
use pantheon_cli::headless::{HeadlessRequest, observe};
use ratatui::{Terminal, backend::TestBackend};

#[derive(Clone, Copy)]
struct Authentication {
    read: bool,
}

impl AgentParticipationAuthenticator for Authentication {
    type Error = Infallible;

    fn authenticate(&mut self, _: &UnixStream) -> Result<AccessV1, Self::Error> {
        Ok(AccessV1 {
            read: self.read,
            mutate: false,
        })
    }

    fn authorize_permission(
        &mut self,
        _: p::PermissionSelectionV1,
    ) -> Result<Option<moira_service::agent_participation::PermissionAccessV1>, Self::Error> {
        Ok(None)
    }
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
    socket: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "pantheon-moira-participation-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir(&root).unwrap();
        Self {
            database: root.join("participation.db"),
            socket: root.join("participation.sock"),
            root,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn restarted_owner_preserves_current_not_found_refusal_and_transport_truth() {
    let fixture = Fixture::new();
    prime(&fixture.database);

    let first = serve(&fixture, true, 2);
    let current = observe(request(
        &fixture.socket,
        p::ParticipationIdV1([2; 16]),
        "current-before-restart",
    ))
    .unwrap();
    assert!(current.complete);
    let peer = participation_peer(&current.peers);
    assert_eq!(peer["status"], "current");
    assert_eq!(peer["revision"], 2);
    assert_eq!(peer["phase"], "Ready");

    let missing = observe(request(
        &fixture.socket,
        p::ParticipationIdV1([99; 16]),
        "not-found",
    ))
    .unwrap();
    assert!(!missing.complete);
    assert_eq!(participation_peer(&missing.peers)["status"], "not_found");
    first.join().unwrap();

    let unavailable = observe(request(
        &fixture.socket,
        p::ParticipationIdV1([2; 16]),
        "owner-stopped",
    ))
    .unwrap();
    assert!(!unavailable.complete);
    assert_eq!(
        participation_peer(&unavailable.peers)["status"],
        "connect_refused"
    );

    let restarted = serve(&fixture, true, 2);
    let after_restart = observe(request(
        &fixture.socket,
        p::ParticipationIdV1([2; 16]),
        "current-after-restart",
    ))
    .unwrap();
    assert!(after_restart.complete);
    assert_eq!(participation_peer(&after_restart.peers)["revision"], 2);

    let port = AgentParticipationUnixPort::connect(
        &fixture.socket,
        ParticipationClientLimitsV1::default(),
    )
    .unwrap();
    let mut component = moira_tui::ParticipationComponent::new(port, p::ParticipationIdV1([2; 16]));
    let deadline = Instant::now() + Duration::from_secs(2);
    let rendered = loop {
        component.poll();
        let mut terminal = Terminal::new(TestBackend::new(120, 34)).unwrap();
        let shell = Shell::new(vec![Surface::Moira]).unwrap();
        terminal
            .draw(|frame| {
                draw_shell_with_moira(
                    frame,
                    &shell,
                    peer_views(&component),
                    Focus::Chronos,
                    "P0 participation qualification",
                );
            })
            .unwrap();
        let text = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        if text.contains("Current governed revision 2") {
            break text;
        }
        assert!(
            Instant::now() < deadline,
            "current TUI projection not rendered"
        );
        thread::sleep(Duration::from_millis(5));
    };
    assert!(rendered.contains("persona: Researcher"));
    assert!(rendered.contains("grounding depth: Activated"));
    restarted.join().unwrap();

    let refused = serve(&fixture, false, 1);
    let observation = observe(request(
        &fixture.socket,
        p::ParticipationIdV1([2; 16]),
        "read-refused",
    ))
    .unwrap();
    assert!(!observation.complete);
    assert_eq!(participation_peer(&observation.peers)["status"], "refused");
    refused.join().unwrap();
}

fn serve(fixture: &Fixture, read: bool, connections: usize) -> thread::JoinHandle<()> {
    let store =
        AgentParticipationSqliteStore::open(&fixture.database, [1; 16], participation_limits())
            .unwrap();
    let daemon = AgentParticipationDaemon::bind_new(
        &fixture.socket,
        AgentParticipationService::new(store),
        Authentication { read },
        moira_wire::LimitsV1::default(),
        Duration::from_secs(2),
        1,
    )
    .unwrap();
    thread::spawn(move || {
        let mut daemon = daemon;
        for _ in 0..connections {
            daemon.serve_next().unwrap();
        }
    })
}

fn request(socket: &Path, participation: p::ParticipationIdV1, operation: &str) -> HeadlessRequest {
    HeadlessRequest {
        operation_id: operation.into(),
        workspace: "P0 participation".into(),
        agora: None,
        agora_hosted: false,
        moira_socket: None,
        moira_participation_socket: Some(socket.into()),
        moira_participation: Some(participation),
        moira_requires: vec!["agent.participation.inspect.v1".into()],
        moira_hosted: false,
        daimon_socket: None,
        daimon_agent_socket: None,
        daimon_agent_session: None,
        daimon_requires: Vec::new(),
        daimon_hosted: false,
        hermes_socket: None,
        hermes_stream: None,
        hermes_consumer: None,
        hermes_requires: Vec::new(),
        hermes_hosted: false,
        aegis_socket: None,
        aegis_requires: Vec::new(),
        aegis_hosted: false,
        grounding_endpoint: None,
        grounding_token: None,
        grounding_requires: Vec::new(),
        chronos: Vec::new(),
        chronos_requires: Vec::new(),
        chronos_hosted: false,
        vellum_socket: None,
        vellum_credential: None,
        vellum_requires: Vec::new(),
        vellum_hosted: false,
    }
}

fn participation_peer(peers: &[serde_json::Value]) -> &serde_json::Value {
    peers
        .iter()
        .find(|peer| peer["peer"] == "moira-participation")
        .expect("Moira participation observation")
}

fn peer_views(component: &moira_tui::ParticipationComponent) -> PeerViews<'_> {
    PeerViews {
        chronos: None,
        vellum: None,
        vellum_atomic_replace: None,
        vellum_durable_save: None,
        agora: None,
        agora_create: None,
        agora_message: None,
        moira: None,
        moira_participation: Some(component),
        moira_reconciliation: None,
        daimon: None,
        daimon_agent: None,
        daimon_permission: None,
        daimon_prompt: None,
        daimon_control: None,
        hermes: None,
        hermes_compaction: None,
        hermes_disclosure: None,
        hermes_acknowledge: None,
        aegis: None,
        aegis_managed: None,
        aegis_revoke: None,
        grounding: None,
        operations: None,
        health: None,
        configuration: None,
    }
}

fn prime(database: &Path) {
    let mut service = AgentParticipationService::new(
        AgentParticipationSqliteStore::open(database, [1; 16], participation_limits()).unwrap(),
    );
    apply(&mut service, None, p::MutationV1::Admit(binding()));
    apply(
        &mut service,
        Some(0),
        p::MutationV1::Prepare {
            participation: p::ParticipationIdV1([2; 16]),
            expected_revision: 0,
            operation: p::OperationV1::Open {
                key: p::IdempotencyKeyV1([13; 16]),
                execution: p::ExecutionIdV1([14; 16]),
            },
        },
    );
    apply(
        &mut service,
        Some(1),
        p::MutationV1::Observe {
            participation: p::ParticipationIdV1([2; 16]),
            expected_revision: 1,
            key: p::IdempotencyKeyV1([13; 16]),
            observation: p::ObservationV1 {
                phase: p::PhaseV1::Ready,
                session_revision: 1,
                active_turn: None,
                last_event_cursor: None,
                profile_document_digest: [30; 32],
                capabilities: p::CapabilitiesV1::default(),
                permission: None,
                terminal: None,
            },
        },
    );
}

fn apply(
    service: &mut AgentParticipationService<AgentParticipationSqliteStore>,
    expected_storage_revision: Option<u64>,
    mutation: p::MutationV1,
) {
    assert!(matches!(
        service
            .execute(p::CommandV1::Apply {
                expected_storage_revision,
                mutation: Box::new(mutation),
            })
            .unwrap(),
        p::ReplyV1::Accepted(_)
    ));
}

fn binding() -> p::BindingV1 {
    p::BindingV1 {
        participation: p::ParticipationIdV1([2; 16]),
        workflow: WorkflowIdV1([3; 16]),
        deliberation: d::DeliberationIdV1([4; 16]),
        member: d::EnsembleMemberIdV1([5; 16]),
        session: p::AgentSessionIdV1([6; 16]),
        profile: p::ProfileIdV1([7; 16]),
        attempt: p::AttemptV1 {
            task_key: [8; 16],
            incarnation: 1,
            attempt: 1,
        },
        method: d::MethodRefV1 {
            machine_digest: [9; 32],
            state_digest: [10; 32],
        },
        grounding: d::GroundingRefV1 {
            authority_digest: [11; 32],
            surface_digest: [12; 32],
            depth: d::SurfaceDepthV1::Activated,
        },
        persona: d::PersonaLensV1::Researcher,
    }
}

fn participation_limits() -> AgentParticipationStoreLimitsV1 {
    AgentParticipationStoreLimitsV1 {
        codec: CodecLimitsV1 {
            max_bytes: 65_536,
            domain: DomainLimits {
                max_participations: 4,
            },
        },
        max_database_pages: 64,
    }
}
