#![cfg(unix)]

use daimon_host::AuthenticatedSessionV1;
use daimon_local_daemon::{
    AgentServicePort,
    unix::{AgentDaemon, AgentSessionPort, Authenticator as DaimonAuthenticator},
};
use daimon_protocol::{
    ContentDigestV1, SessionHandleV1,
    agent::{
        AgentCapabilitiesV1, AgentCommandV1, AgentEventPageV1, AgentOutcomeV1, AgentReplyV1,
        AgentSessionPhaseV1, AgentSessionViewV1,
    },
};
use daimon_wire::{LimitsV1 as DaimonLimitsV1, agent::AgentWireLimitsV1};
use moira_core::agent_participation::{LimitsV1 as DomainLimits, persistence::CodecLimitsV1};
use moira_local_daemon::unix::{AgentParticipationAuthenticator, AgentParticipationDaemon};
use moira_protocol::{WorkflowIdV1, agent_participation as p, deliberation as d};
use moira_service::{AccessV1, agent_participation::AgentParticipationService};
use moira_sqlite::agent_participation::{
    AgentParticipationSqliteStore, AgentParticipationStoreLimitsV1,
};
use moira_wire::LimitsV1 as MoiraLimitsV1;
use std::{
    convert::Infallible,
    env, fs,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

struct ParticipationAuthentication;

impl AgentParticipationAuthenticator for ParticipationAuthentication {
    type Error = Infallible;

    fn authenticate(&mut self, _: &UnixStream) -> Result<AccessV1, Self::Error> {
        Ok(AccessV1 {
            read: true,
            mutate: true,
        })
    }

    fn authorize_permission(
        &mut self,
        _: p::PermissionSelectionV1,
    ) -> Result<Option<moira_service::agent_participation::PermissionAccessV1>, Self::Error> {
        Ok(None)
    }
}

struct DaimonAuthentication;

impl DaimonAuthenticator for DaimonAuthentication {
    type Error = Infallible;

    fn authenticate(&mut self, _: &UnixStream) -> Result<AuthenticatedSessionV1, Self::Error> {
        Ok(AuthenticatedSessionV1 {
            principal: [20; 16],
            authority_context: [40; 16],
        })
    }
}

struct AgentPort;

impl AgentServicePort for AgentPort {
    type Error = Infallible;

    fn execute_agent(
        &mut self,
        service_session: SessionHandleV1,
        command: AgentCommandV1,
    ) -> Result<AgentReplyV1, Self::Error> {
        assert_eq!(service_session, SessionHandleV1(17));
        Ok(match command {
            AgentCommandV1::Open { request, now } => {
                assert_eq!(request.session.into_bytes(), [6; 16]);
                assert_ne!(request.execution.execution_id.into_bytes(), [0; 16]);
                assert_ne!(request.execution.idempotency_key.into_bytes(), [0; 16]);
                assert_eq!(request.execution.principal.into_bytes(), [20; 16]);
                assert_eq!(request.execution.profile.into_bytes(), [7; 16]);
                assert_eq!(request.workspace, [21; 16]);
                assert_eq!(now, 7);
                AgentReplyV1::Accepted(AgentOutcomeV1::Session(AgentSessionViewV1 {
                    session: request.session,
                    revision: 1,
                    phase: AgentSessionPhaseV1::Ready,
                    profile_document_digest: ContentDigestV1::new([30; 32]),
                    active_turn: None,
                    last_event_cursor: None,
                    last_context: None,
                    capabilities: AgentCapabilitiesV1::default(),
                }))
            }
            AgentCommandV1::Events {
                session,
                after,
                limit,
            } => {
                assert_eq!(session.into_bytes(), [6; 16]);
                assert_eq!(after, None);
                assert_eq!(limit, 32);
                AgentReplyV1::Accepted(AgentOutcomeV1::Events(AgentEventPageV1 {
                    session,
                    entries: Vec::new(),
                    next_after: None,
                }))
            }
            other => panic!("unexpected Daimon command {other:?}"),
        })
    }
}

impl AgentSessionPort for AgentPort {
    fn open_agent_service_session(
        &mut self,
        facts: AuthenticatedSessionV1,
    ) -> Result<SessionHandleV1, daimon_protocol::agent::AgentRefusalV1> {
        assert_eq!(facts.principal, [20; 16]);
        Ok(SessionHandleV1(17))
    }

    fn close_agent_service_session(&mut self, session: SessionHandleV1) -> bool {
        assert_eq!(session, SessionHandleV1(17));
        true
    }
}

fn main() {
    let mut args = env::args_os().skip(1);
    let mode = args.next().expect("mode: serve or verify");
    let root = PathBuf::from(args.next().expect("fixture root"));
    assert!(args.next().is_none(), "unexpected fixture argument");
    match mode.to_str() {
        Some("serve") => serve(&root),
        Some("verify") => verify(&root),
        _ => panic!("mode must be serve or verify"),
    }
}

fn serve(root: &Path) {
    fs::create_dir_all(root).unwrap();
    let database = root.join("participation.db");
    prime_admitted(&database);

    let participation_service = AgentParticipationService::new(
        AgentParticipationSqliteStore::open(&database, [1; 16], participation_limits()).unwrap(),
    );
    let participation_daemon = AgentParticipationDaemon::bind_new(
        root.join("participation.sock"),
        participation_service,
        ParticipationAuthentication,
        MoiraLimitsV1::default(),
        Duration::from_secs(5),
        64,
    )
    .unwrap();
    let daimon_daemon = AgentDaemon::bind_new(
        root.join("daimon.sock"),
        AgentPort,
        DaimonAuthentication,
        DaimonLimitsV1::default(),
        AgentWireLimitsV1::default(),
        Duration::from_secs(5),
        8,
    )
    .unwrap();
    fs::write(root.join("ready"), b"ready\n").unwrap();

    let daimon = thread::spawn(move || {
        let mut daemon = daimon_daemon;
        for _ in 0..128 {
            // The qualification spans separate headless and TUI processes.
            // An accept timeout between those processes is not a daemon failure.
            let _ = daemon.serve_next();
        }
    });
    let mut daemon = participation_daemon;
    for _ in 0..128 {
        // Keep the durable owner available across the intentionally separate
        // observations and control session; the shell verifies final state.
        let _ = daemon.serve_next();
    }
    daimon.join().unwrap();
}

fn verify(root: &Path) {
    let mut service = AgentParticipationService::new(
        AgentParticipationSqliteStore::open(
            root.join("participation.db"),
            [1; 16],
            participation_limits(),
        )
        .unwrap(),
    );
    let reply = service
        .execute(p::CommandV1::Participation {
            participation: p::ParticipationIdV1([2; 16]),
        })
        .unwrap();
    assert!(matches!(
        reply,
        p::ReplyV1::Accepted(value)
            if matches!(*value, p::OutcomeV1::Participation(Some(p::ParticipationViewV1 {
                revision: 3,
                phase: p::PhaseV1::Ready,
                session_revision: Some(1),
                pending: None,
                ..
            })))
    ));
    println!("verified participation 02020202020202020202020202020202 ready at revision 3");
}

fn prime_admitted(database: &Path) {
    let mut service = AgentParticipationService::new(
        AgentParticipationSqliteStore::open(database, [1; 16], participation_limits()).unwrap(),
    );
    let reply = service
        .execute(p::CommandV1::Apply {
            expected_storage_revision: None,
            mutation: Box::new(p::MutationV1::Admit(binding())),
        })
        .unwrap();
    assert!(matches!(reply, p::ReplyV1::Accepted(_)));
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
