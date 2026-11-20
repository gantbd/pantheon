#![cfg(unix)]

use aegis_client::ServicePort as AegisServicePort;
use aegis_protocol::{
    CurrentPrincipalV1, DecisionHandleV1, DecisionSummaryV1, EngineNamespaceV1,
    OutcomeV1 as AegisOutcomeV1, ReplyV1 as AegisReplyV1, SessionHandleV1 as AegisSessionHandleV1,
};
use agora_aegis_verifier::{AegisContextV1, AegisVerifierV1};
use agora_chronos_adapter::task_conversation_local::{
    ConversationReferenceWireV1, LinkWireV1, LocalRequestV1, LocalServiceLimitsV1,
    TaskConversationLocalClientV1, TaskConversationLocalServiceV1, TaskReferenceWireV1,
};
use agora_engine::{
    AdmissionPolicy, CommandV2 as AgoraCommandV2, Engine as AgoraEngine, LogicalTime,
    OutcomeV2 as AgoraOutcomeV2, Participant, ParticipantId, ParticipantKind, RoomId,
};
use agora_host::{HostLimits, HostTimeV1, VerifiedSubjectV1, v2::HostV2};
use agora_local_daemon::{
    AuthenticatorV1 as AgoraAuthenticatorV1, DaemonLimitsV1, DaemonV1,
    HostClockV1 as AgoraHostClockV1, ServerIncarnationV1,
};
use agora_service::v2::{ServiceLimitsV2, ServiceV2};
use chronos_client::{ReviewWireClientV10, WireClientV1, WorkflowWireClientV9};
use chronos_engine::{Agent, CommandOutcomeV2, TaskState};
use chronos_host::worker::{WorkerClientV1, WorkerRequestV1 as Work};
use chronos_local::LocalClient;
use chronos_wire::{
    Command, Outcome, Request, RetrySafety, Revision, TaskSpec, review_v10 as review,
    workflow_v9 as workflow,
};
use daimon_host::AuthenticatedSessionV1;
use daimon_local_daemon::{
    AgentServicePort,
    unix::{AgentDaemon, AgentSessionPort, Authenticator as DaimonAuthenticator},
};
use daimon_protocol::{
    AgentSessionIdV1 as DaimonSessionIdV1, AgentTurnIdV1 as DaimonTurnIdV1, ContentDigestV1,
    ExecutionDispositionV1, ExecutionIdV1, HarnessProfileIdV1 as DaimonProfileIdV1, PrincipalIdV1,
    SessionHandleV1 as DaimonSessionHandleV1, TaskAttemptRefV1,
    agent::{
        AgentCommandV1, AgentEventPageV1, AgentEventRecordV1, AgentEventV1, AgentOutcomeV1,
        AgentRefusalV1, AgentReplyV1, AgentResultBindingV1, AgentTurnStopReasonV1,
    },
};
use moira_core::{
    ChronosAttemptReceiptV1, ChronosTaskRefV1, WorkflowId,
    agent_participation::{
        AgentSessionIdV1, AttemptV1, BindingV1, Engine as ParticipationEngine,
        LimitsV1 as ParticipationLimitsV1, ParticipationIdV1, ProfileIdV1 as DomainProfileIdV1,
    },
    deliberation::{
        DeliberationIdV1, EnsembleAdmissionV1, EnsembleMemberIdV1, GroundingRefV1, MethodRefV1,
        PersonaLensV1, SurfaceDepthV1,
    },
    task_agent_journey as domain,
};
use moira_host::{
    TaskAgentJourneyMutationV1, TaskAgentJourneyStore, TaskAgentParticipationPreparationV1,
};
use moira_local_daemon::unix::{AgentParticipationAuthenticator, AgentParticipationDaemon};
use moira_local_daemon::unix::{Authenticator, TaskAgentJourneyDaemon};
use moira_protocol::{
    TaskRefV1, agent_participation as participation, agent_participation::ProfileIdV1,
    deliberation as public_deliberation, task_agent_journey as public,
};
use moira_service::{
    AccessV1,
    agent_participation::AgentParticipationService,
    task_agent_journey::{TaskAgentJourneyService, TrustedTimeV1},
};
use moira_sqlite::agent_participation::{
    AgentParticipationSqliteStore, AgentParticipationStoreLimitsV1,
};
use moira_sqlite::task_agent_journey::{
    TaskAgentJourneySqliteStore, TaskAgentJourneyStoreLimitsV1,
};
use moira_wire::LimitsV1;
use serde_json::Value;
use std::{
    convert::Infallible,
    fs,
    io::Write,
    os::unix::net::{UnixStream, UnixStream as ShutdownStream},
    path::{Path, PathBuf},
    process::Command as Process,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, SystemTime},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    chronos: PathBuf,
    moira_database: PathBuf,
    moira_socket: PathBuf,
    participation_database: PathBuf,
    participation_socket: PathBuf,
    daimon_socket: PathBuf,
    agora_socket: PathBuf,
    relationship_state: PathBuf,
    relationship_socket: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(format!(
            "/private/tmp/pj{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        chronos_standalone::ownership::create_private_directory(&root).unwrap();
        let chronos = root.clone();
        Self {
            moira_database: root.join("moira.db"),
            moira_socket: root.join("moira.sock"),
            participation_database: root.join("participation.db"),
            participation_socket: root.join("participation.sock"),
            daimon_socket: root.join("daimon.sock"),
            agora_socket: root.join("agora.sock"),
            relationship_state: root.join("relationships"),
            relationship_socket: root.join("relationships.sock"),
            chronos,
            root,
        }
    }

    fn moira_store(&self) -> TaskAgentJourneySqliteStore {
        TaskAgentJourneySqliteStore::open(
            &self.moira_database,
            domain::OwnerIdV1([1; 16]),
            store_limits(),
            [template()],
        )
        .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.moira_socket);
        let _ = fs::remove_file(&self.participation_socket);
        let _ = fs::remove_file(&self.daimon_socket);
        let _ = fs::remove_file(&self.agora_socket);
        let _ = fs::remove_file(&self.relationship_socket);
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct Time(u64);

impl TrustedTimeV1 for Time {
    type Error = Infallible;

    fn now(&mut self) -> Result<u64, Self::Error> {
        let value = self.0;
        self.0 += 1;
        Ok(value)
    }
}

struct Authentication;

impl Authenticator for Authentication {
    type Error = Infallible;

    fn authenticate(&mut self, _: &UnixStream) -> Result<AccessV1, Self::Error> {
        Ok(AccessV1 {
            read: true,
            mutate: false,
        })
    }
}

#[derive(Clone, Copy)]
struct ParticipationAuthentication;

impl AgentParticipationAuthenticator for ParticipationAuthentication {
    type Error = Infallible;

    fn authenticate(&mut self, _: &UnixStream) -> Result<AccessV1, Self::Error> {
        Ok(AccessV1 {
            read: true,
            mutate: false,
        })
    }

    fn authorize_permission(
        &mut self,
        _: participation::PermissionSelectionV1,
    ) -> Result<Option<moira_service::agent_participation::PermissionAccessV1>, Self::Error> {
        Ok(None)
    }
}

struct AgoraAuthority;

impl AegisServicePort for AgoraAuthority {
    type Error = ();

    fn execute(
        &mut self,
        _: AegisSessionHandleV1,
        command: aegis_protocol::CommandV1,
    ) -> Result<AegisReplyV1, Self::Error> {
        if command == aegis_protocol::CommandV1::CurrentPrincipal {
            return Ok(AegisReplyV1::Accepted(AegisOutcomeV1::CurrentPrincipal(
                CurrentPrincipalV1 { principal: [7; 16] },
            )));
        }
        let aegis_protocol::CommandV1::Authorize { action, .. } = command else {
            return Err(());
        };
        Ok(AegisReplyV1::Accepted(AegisOutcomeV1::Authorized(
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

struct AgoraAuthentication;

impl AgoraAuthenticatorV1 for AgoraAuthentication {
    type Error = Infallible;

    fn authenticate(&mut self, _: &UnixStream) -> Result<AegisContextV1, Self::Error> {
        Ok(AegisContextV1::new(
            "exact-agent",
            VerifiedSubjectV1::from_verifier([7; 16]),
            AegisSessionHandleV1(1),
            aegis_protocol::GrantHandleV1(1),
        ))
    }
}

struct AgoraClock(u64);

impl AgoraHostClockV1 for AgoraClock {
    fn now(&mut self) -> HostTimeV1 {
        let value = self.0;
        self.0 += 1;
        HostTimeV1(value)
    }
}

struct DaimonAuthentication;

impl DaimonAuthenticator for DaimonAuthentication {
    type Error = Infallible;

    fn authenticate(&mut self, _: &UnixStream) -> Result<AuthenticatedSessionV1, Self::Error> {
        Ok(AuthenticatedSessionV1 {
            principal: [35; 16],
            authority_context: [44; 16],
        })
    }
}

struct DaimonPort;

impl AgentSessionPort for DaimonPort {
    fn open_agent_service_session(
        &mut self,
        _: AuthenticatedSessionV1,
    ) -> Result<DaimonSessionHandleV1, AgentRefusalV1> {
        Ok(DaimonSessionHandleV1(1))
    }

    fn close_agent_service_session(&mut self, session: DaimonSessionHandleV1) -> bool {
        session == DaimonSessionHandleV1(1)
    }
}

impl AgentServicePort for DaimonPort {
    type Error = Infallible;

    fn execute_agent(
        &mut self,
        session: DaimonSessionHandleV1,
        command: AgentCommandV1,
    ) -> Result<AgentReplyV1, Self::Error> {
        assert_eq!(session, DaimonSessionHandleV1(1));
        let result_session = DaimonSessionIdV1::new([33; 16]);
        let result_turn = DaimonTurnIdV1::new([42; 16]);
        let output_digest = ContentDigestV1::new(result_digest());
        Ok(match command {
            AgentCommandV1::Result { session, turn }
                if session == result_session && turn == result_turn =>
            {
                AgentReplyV1::Accepted(AgentOutcomeV1::Result(AgentResultBindingV1 {
                    session,
                    turn,
                    execution: ExecutionIdV1::new([41; 16]),
                    task_attempt: TaskAttemptRefV1 {
                        task_key: [9; 16],
                        task_incarnation: 0,
                        attempt_number: 1,
                    },
                    principal: PrincipalIdV1::new([35; 16]),
                    profile: DaimonProfileIdV1::new([4; 16]),
                    operation: daimon_protocol::IdempotencyKeyV1::new([40; 16]),
                    authority_decision: [34; 16],
                    authority_expires_at: 400,
                    prompt_digest: ContentDigestV1::new([45; 32]),
                    context_manifest_digest: ContentDigestV1::new([46; 32]),
                    event_start_cursor: 1,
                    terminal_cursor: 2,
                    terminal_revision: 4,
                    disposition: ExecutionDispositionV1::Completed,
                    stop_reason: AgentTurnStopReasonV1::EndTurn,
                    output_digest: Some(output_digest),
                }))
            }
            AgentCommandV1::Events {
                session,
                after: Some(0),
                limit: 2,
            } if session == result_session => {
                AgentReplyV1::Accepted(AgentOutcomeV1::Events(AgentEventPageV1 {
                    session,
                    entries: vec![
                        AgentEventRecordV1 {
                            cursor: 1,
                            event: AgentEventV1::AssistantMessage {
                                bytes: result_body().to_vec(),
                            },
                        },
                        AgentEventRecordV1 {
                            cursor: 2,
                            event: AgentEventV1::TurnFinished {
                                turn: result_turn,
                                disposition: ExecutionDispositionV1::Completed,
                                stop_reason: AgentTurnStopReasonV1::EndTurn,
                                output_digest: Some(output_digest),
                            },
                        },
                    ],
                    next_after: None,
                }))
            }
            other => panic!("unexpected strict J1 Daimon command: {other:?}"),
        })
    }
}

fn result_body() -> &'static [u8] {
    b"The acceptance evidence is bounded and attributable."
}

fn result_digest() -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(result_body()).into()
}

struct ChronosOwner {
    shutdown: ShutdownStream,
    thread: thread::JoinHandle<chronos_standalone::Server>,
}

impl ChronosOwner {
    fn stop(mut self) {
        self.shutdown.write_all(&[1]).unwrap();
        drop(self.thread.join().unwrap());
    }
}

fn start_chronos(
    directory: &Path,
    with_worker: bool,
) -> (
    ChronosOwner,
    Option<chronos_standalone::TrustedWorkerHandle>,
) {
    let mut server = chronos_standalone::Server::bind_with_options(
        directory,
        chronos_standalone::ServerOptions {
            allow_writes: true,
            atomic_state: true,
            ..Default::default()
        },
    )
    .unwrap();
    let worker = with_worker.then(|| server.trusted_worker().unwrap());
    let (read, shutdown) = ShutdownStream::pair().unwrap();
    let thread = thread::spawn(move || {
        server.run_with_shutdown(&read).unwrap();
        server
    });
    (ChronosOwner { shutdown, thread }, worker)
}

fn seed_chronos(directory: &Path, worker: &mut chronos_standalone::TrustedWorkerHandle) -> u64 {
    let mut client = LocalClient::new(directory);
    assert!(matches!(
        client
            .request(Request {
                version: 1,
                command: Command::SubmitGraph {
                    expected: Revision::Finite("0".into()),
                    tasks: vec![TaskSpec {
                        id: "j1-owned-task".into(),
                        priority: 0,
                        dependencies: vec![],
                        capabilities: vec![],
                        max_attempts: 2,
                        retry_safety: RetrySafety::FenceUnknown,
                    }],
                },
            })
            .unwrap()
            .outcome,
        Outcome::Applied {}
    ));
    let CommandOutcomeV2::AdvanceTime(Ok(time)) = worker
        .call_worker(Work::AdvanceTime {
            now: SystemTime::UNIX_EPOCH,
        })
        .unwrap()
        .unwrap()
    else {
        panic!("engine time expected")
    };
    let CommandOutcomeV2::IssueGrant(Ok(grant)) = worker
        .call_worker(Work::IssueGrant {
            agent: Agent::new("j1-review-link-fixture"),
            time: time.clone(),
            validity_duration: Duration::from_secs(60),
            claim_budget: Some(1),
        })
        .unwrap()
        .unwrap()
    else {
        panic!("grant expected")
    };
    let CommandOutcomeV2::ClaimNext(Ok(Some(task))) = worker
        .call_worker(Work::ClaimNext {
            grant,
            time: time.clone(),
            lease_duration: Duration::from_secs(30),
        })
        .unwrap()
        .unwrap()
    else {
        panic!("claim expected")
    };
    let TaskState::Running(claim) = task.state else {
        panic!("running claim expected")
    };
    assert!(matches!(
        worker
            .call_worker(Work::Complete {
                task_id: task.id,
                lease: claim.lease,
                time,
            })
            .unwrap()
            .unwrap(),
        CommandOutcomeV2::Complete(Ok(()))
    ));
    let response = client
        .request_workflow_v9(workflow::Request {
            version: 9,
            command: workflow::Command::Snapshot {
                task: "j1-owned-task".into(),
            },
        })
        .unwrap();
    let workflow::Outcome::Snapshot {
        value: Some(entry),
        revision: Some(Revision::Finite(revision)),
    } = response.outcome
    else {
        panic!("current workflow snapshot expected")
    };
    assert_eq!(entry.snapshot.status, workflow::Status::ReadyForReview);
    assert_eq!(entry.snapshot.attempt_number, Some(1));
    assert_eq!(entry.reference.incarnation, "0");
    revision.parse().unwrap()
}

fn require_revision_required(directory: &Path) {
    let mut client = LocalClient::new(directory);
    let snapshot = client
        .request_workflow_v9(workflow::Request {
            version: 9,
            command: workflow::Command::Snapshot {
                task: "j1-owned-task".into(),
            },
        })
        .unwrap();
    let workflow::Outcome::Snapshot {
        value: Some(entry),
        revision: Some(expected),
    } = snapshot.outcome
    else {
        panic!("reviewable workflow snapshot expected")
    };
    assert_eq!(entry.snapshot.status, workflow::Status::ReadyForReview);
    assert_eq!(entry.snapshot.attempt_number, Some(1));
    let digest = |byte: u8| format!("{byte:02x}").repeat(32);
    let response = client
        .request_review_v10(review::Request {
            version: 10,
            command: review::Command::ExecuteReview {
                invocation: review::Invocation {
                    serial: "1".into(),
                    key: "j1-revision-required".into(),
                },
                expected,
                task: entry.reference,
                attempt: 1,
                disposition: review::Disposition::RevisionRequired,
                evidence: review::Evidence {
                    decision_sha256: digest(1),
                    reviewer_sha256: digest(2),
                    authority_sha256: digest(3),
                    challenged_sha256: digest(4),
                    criteria_sha256: digest(5),
                    findings_sha256: digest(6),
                    feedback_sha256: Some(digest(7)),
                },
            },
        })
        .unwrap();
    assert!(matches!(
        response.outcome,
        review::Outcome::ReviewResult {
            result: review::ReviewResult::Applied {
                attempt: 1,
                disposition: review::Disposition::RevisionRequired,
                ..
            },
            ..
        }
    ));
}

fn complete_reopened_attempt(worker: &mut chronos_standalone::TrustedWorkerHandle) {
    let CommandOutcomeV2::AdvanceTime(Ok(time)) = worker
        .call_worker(Work::AdvanceTime {
            now: SystemTime::UNIX_EPOCH + Duration::from_secs(1),
        })
        .unwrap()
        .unwrap()
    else {
        panic!("reopened engine time expected")
    };
    let CommandOutcomeV2::IssueGrant(Ok(grant)) = worker
        .call_worker(Work::IssueGrant {
            agent: Agent::new("j1-reopened-fixture"),
            time: time.clone(),
            validity_duration: Duration::from_secs(60),
            claim_budget: Some(1),
        })
        .unwrap()
        .unwrap()
    else {
        panic!("reopened grant expected")
    };
    let CommandOutcomeV2::ClaimNext(Ok(Some(task))) = worker
        .call_worker(Work::ClaimNext {
            grant,
            time: time.clone(),
            lease_duration: Duration::from_secs(30),
        })
        .unwrap()
        .unwrap()
    else {
        panic!("reopened task expected")
    };
    let TaskState::Running(claim) = task.state else {
        panic!("reopened running claim expected")
    };
    assert!(matches!(
        worker
            .call_worker(Work::Complete {
                task_id: task.id,
                lease: claim.lease,
                time,
            })
            .unwrap()
            .unwrap(),
        CommandOutcomeV2::Complete(Ok(()))
    ));
}

fn assert_attempt(
    directory: &Path,
    expected_attempt: Option<u32>,
    expected_status: workflow::Status,
) {
    let mut client = LocalClient::new(directory);
    let response = client
        .request_workflow_v9(workflow::Request {
            version: 9,
            command: workflow::Command::Snapshot {
                task: "j1-owned-task".into(),
            },
        })
        .unwrap();
    let workflow::Outcome::Snapshot {
        value: Some(entry), ..
    } = response.outcome
    else {
        panic!("current workflow snapshot expected")
    };
    assert_eq!(entry.snapshot.status, expected_status);
    assert_eq!(entry.snapshot.attempt_number, expected_attempt);
}

fn template() -> domain::TemplateV1 {
    domain::TemplateV1 {
        id: domain::TemplateIdV1([2; 16]),
        version: 1,
        digest: [3; 32],
        work_mode: domain::WorkModeV1::Collaborator,
        profile: DomainProfileIdV1([4; 16]),
        room_policy: domain::RoomPolicyV1::SharedContext,
        budget: domain::BudgetV1 {
            max_context_bytes: 4_096,
            max_output_tokens: 512,
            max_elapsed_millis: 10_000,
        },
        grounding_policy_digest: [5; 32],
        authority_policy_digest: [6; 32],
        consequence_digest: [7; 32],
        retired: false,
    }
}

fn store_limits() -> TaskAgentJourneyStoreLimitsV1 {
    let domain = domain::LimitsV1 {
        max_templates: 2,
        max_previews: 4,
        max_journeys: 4,
        max_preview_ttl: 60,
    };
    TaskAgentJourneyStoreLimitsV1 {
        codec: domain::persistence::CodecLimitsV1 {
            max_bytes: 65_536,
            domain,
        },
        domain,
        max_database_pages: 64,
    }
}

fn preview_request() -> public::PreviewRequestV1 {
    public::PreviewRequestV1 {
        operation: public::PreviewOperationIdV1([8; 16]),
        template: public::TemplateRefV1 {
            id: public::TemplateIdV1([2; 16]),
            version: 1,
            digest: [3; 32],
        },
        task: public::TaskAdmissionV1 {
            task: TaskRefV1 {
                external_task_key: [9; 16],
                incarnation: 0,
            },
            chronos_revision: 1,
            evidence_digest: [10; 32],
        },
        profile: ProfileIdV1([4; 16]),
        room_policy: public::RoomPolicyV1::SharedContext,
        initiating_principal: public::PrincipalIdV1([12; 16]),
        accountable_human: public::PrincipalIdV1([13; 16]),
        objective_digest: [14; 32],
        acceptance_digest: [15; 32],
        current_evidence_digest: [16; 32],
        requested_ttl: 30,
    }
}

#[derive(Clone, Copy)]
struct AgoraPublication {
    room_revision: u64,
    publication_revision: u64,
    message_sequence: u64,
    message_id: [u8; 16],
}

fn seeded_agora() -> (AgoraEngine, AgoraPublication) {
    let mut engine = AgoraEngine::new();
    engine
        .apply(AgoraCommandV2::RegisterParticipant(Participant::new(
            "exact-agent",
            "Exact Agent",
            ParticipantKind::Agent,
        )))
        .unwrap();
    engine
        .apply(AgoraCommandV2::CreateRoom {
            room_id: RoomId::from("exact-room"),
            name: "Exact J1 Review".into(),
            owner: ParticipantId::from("exact-agent"),
            admission: AdmissionPolicy::InviteOnly,
        })
        .unwrap();
    let room_revision = engine.revision().0;
    let AgoraOutcomeV2::MessageAccepted { room_id, sequence } = engine
        .apply(AgoraCommandV2::Speak {
            room_id: RoomId::from("exact-room"),
            author: ParticipantId::from("exact-agent"),
            at: LogicalTime(1),
            body: std::str::from_utf8(result_body()).unwrap().into(),
        })
        .unwrap()
    else {
        panic!("Agora message acceptance expected")
    };
    let publication = AgoraPublication {
        room_revision,
        publication_revision: engine.revision().0,
        message_sequence: sequence,
        message_id: engine
            .message(&room_id, sequence)
            .unwrap()
            .id()
            .into_bytes(),
    };
    (engine, publication)
}

fn seed_moira(fixture: &Fixture, publication: AgoraPublication) -> public::JourneyIdV1 {
    let mut service = TaskAgentJourneyService::new(fixture.moira_store(), Time(100));
    let public::ReplyV1::Accepted(preview) = service
        .execute(public::CommandV1::Preview {
            expected_storage_revision: None,
            request: preview_request(),
        })
        .unwrap()
    else {
        panic!("preview expected")
    };
    let public::OutcomeV1::Previewed { receipt, .. } = *preview else {
        panic!("preview outcome expected")
    };
    let request = public::InstantiateRequestV1 {
        operation: public::InstantiateOperationIdV1([17; 16]),
        preview: receipt.preview.id,
        expected_owner_revision: receipt.preview.owner_revision,
        confirmation: public::HumanConfirmationV1 {
            preview: receipt.preview.id,
            accountable_human: receipt.preview.request.accountable_human,
            consequence_digest: receipt.preview.consequence_digest,
        },
    };
    let public::ReplyV1::Accepted(admitted) = service
        .execute(public::CommandV1::Instantiate {
            expected_storage_revision: Some(1),
            request,
        })
        .unwrap()
    else {
        panic!("journey expected")
    };
    let public::OutcomeV1::Admitted { receipt, .. } = *admitted else {
        panic!("journey outcome expected")
    };
    let journey = receipt.journey.id;
    let domain_journey = domain::JourneyIdV1 {
        owner: domain::OwnerIdV1(journey.owner.0),
        sequence: journey.sequence,
    };
    let (mut store, _) = service.into_parts();
    let attempt = ChronosAttemptReceiptV1::new(ChronosTaskRefV1::new([9; 16], 0), 1, 2);
    store
        .apply(
            Some(2),
            TaskAgentJourneyMutationV1::BindTaskAttemptNamespace {
                journey: domain_journey,
                receipt: attempt,
                chronos_task_id: "j1-owned-task".into(),
            },
        )
        .unwrap();
    let room = domain::AgoraRoomReceiptV1 {
        source: domain_journey.owner.0,
        room_sequence: domain_journey.sequence,
        agora_revision: publication.room_revision,
    };
    store
        .apply(
            Some(3),
            TaskAgentJourneyMutationV1::BindRoom {
                journey: domain_journey,
                receipt: room,
            },
        )
        .unwrap();
    let workflow = domain::ParticipationWorkflowReceiptV1 {
        source: domain_journey.owner.0,
        workflow_sequence: domain_journey.sequence,
        participation_workflow: WorkflowId::new([20; 16]),
        relationship_revision: 0,
    };
    let deliberation = domain::DeliberationMemberReceiptV1 {
        deliberation: DeliberationIdV1([21; 16]),
        member: EnsembleMemberIdV1([22; 16]),
        method: MethodRefV1 {
            machine_digest: [23; 32],
            state_digest: [24; 32],
        },
        grounding: GroundingRefV1 {
            authority_digest: [25; 32],
            surface_digest: [26; 32],
            depth: SurfaceDepthV1::Activated,
        },
        persona: PersonaLensV1::Researcher,
        ensemble_admission: EnsembleAdmissionV1::affirmed(
            [27; 32], [28; 32], [29; 32], [30; 32], [31; 16],
        ),
        owner_revision: 0,
    };
    let binding = BindingV1 {
        participation: ParticipationIdV1([32; 16]),
        workflow: workflow.participation_workflow,
        deliberation: deliberation.deliberation,
        member: deliberation.member,
        session: AgentSessionIdV1([33; 16]),
        profile: DomainProfileIdV1([4; 16]),
        attempt: AttemptV1 {
            task_key: [9; 16],
            incarnation: 0,
            attempt: 1,
        },
        method: deliberation.method,
        grounding: deliberation.grounding,
        persona: deliberation.persona,
    };
    store
        .apply(
            Some(4),
            TaskAgentJourneyMutationV1::PrepareParticipation(Box::new(
                TaskAgentParticipationPreparationV1 {
                    journey: domain_journey,
                    workflow,
                    deliberation,
                    binding,
                },
            )),
        )
        .unwrap();
    let authority = domain::ParticipationAdmissionAuthorityV1 {
        journey: domain_journey,
        participation: binding.participation,
        decision: [34; 16],
        principal: [35; 16],
        epoch: 6,
        expires_at: 400,
    };
    store
        .apply(
            Some(5),
            TaskAgentJourneyMutationV1::AttributeParticipation {
                journey: domain_journey,
                authority,
            },
        )
        .unwrap();
    let mut owner = ParticipationEngine::try_new(ParticipationLimitsV1::default()).unwrap();
    owner.admit(binding).unwrap();
    store
        .apply(
            Some(6),
            TaskAgentJourneyMutationV1::BindParticipation {
                journey: domain_journey,
                observed: Box::new(owner.participation(binding.participation).unwrap().clone()),
            },
        )
        .unwrap();
    store
        .apply(
            Some(7),
            TaskAgentJourneyMutationV1::BindActorParticipationRelationship {
                journey: domain_journey,
                publication: domain::ActorPublicationEvidenceV1 {
                    effect: [36; 16],
                    source: room.source,
                    room_sequence: room.room_sequence,
                    room_id: "exact-room".into(),
                    message_sequence: publication.message_sequence,
                    message_id: publication.message_id,
                    agora_revision: publication.publication_revision,
                    actor: "exact-agent".into(),
                    subject: authority.principal,
                },
            },
        )
        .unwrap();
    drop(store);
    journey
}

fn participation_binding() -> participation::BindingV1 {
    participation::BindingV1 {
        participation: participation::ParticipationIdV1([32; 16]),
        workflow: moira_protocol::WorkflowIdV1([20; 16]),
        deliberation: public_deliberation::DeliberationIdV1([21; 16]),
        member: public_deliberation::EnsembleMemberIdV1([22; 16]),
        session: participation::AgentSessionIdV1([33; 16]),
        profile: participation::ProfileIdV1([4; 16]),
        attempt: participation::AttemptV1 {
            task_key: [9; 16],
            incarnation: 0,
            attempt: 1,
        },
        method: public_deliberation::MethodRefV1 {
            machine_digest: [23; 32],
            state_digest: [24; 32],
        },
        grounding: public_deliberation::GroundingRefV1 {
            authority_digest: [25; 32],
            surface_digest: [26; 32],
            depth: public_deliberation::SurfaceDepthV1::Activated,
        },
        persona: public_deliberation::PersonaLensV1::Researcher,
    }
}

fn participation_limits() -> AgentParticipationStoreLimitsV1 {
    AgentParticipationStoreLimitsV1 {
        codec: moira_core::agent_participation::persistence::CodecLimitsV1 {
            max_bytes: 65_536,
            domain: ParticipationLimitsV1 {
                max_participations: 4,
            },
        },
        max_database_pages: 64,
    }
}

fn apply_participation(
    service: &mut AgentParticipationService<AgentParticipationSqliteStore>,
    storage_revision: Option<u64>,
    mutation: participation::MutationV1,
) {
    assert!(matches!(
        service
            .execute(participation::CommandV1::Apply {
                expected_storage_revision: storage_revision,
                mutation: Box::new(mutation),
            })
            .unwrap(),
        participation::ReplyV1::Accepted(_)
    ));
}

fn seed_terminal_participation(fixture: &Fixture) {
    let store = AgentParticipationSqliteStore::open(
        &fixture.participation_database,
        [1; 16],
        participation_limits(),
    )
    .unwrap();
    let mut service = AgentParticipationService::new(store);
    let binding = participation_binding();
    apply_participation(
        &mut service,
        None,
        participation::MutationV1::Admit(binding),
    );
    let open = participation::OperationV1::Open {
        key: participation::IdempotencyKeyV1([38; 16]),
        execution: participation::ExecutionIdV1([39; 16]),
    };
    apply_participation(
        &mut service,
        Some(0),
        participation::MutationV1::Prepare {
            participation: binding.participation,
            expected_revision: 0,
            operation: open,
        },
    );
    apply_participation(
        &mut service,
        Some(1),
        participation::MutationV1::Observe {
            participation: binding.participation,
            expected_revision: 1,
            key: participation::IdempotencyKeyV1([38; 16]),
            observation: participation::ObservationV1 {
                phase: participation::PhaseV1::Ready,
                session_revision: 1,
                active_turn: None,
                last_event_cursor: None,
                profile_document_digest: [30; 32],
                capabilities: participation::CapabilitiesV1::default(),
                permission: None,
                terminal: None,
            },
        },
    );
    let prompt = participation::OperationV1::Prompt {
        key: participation::IdempotencyKeyV1([40; 16]),
        turn: participation::AgentTurnIdV1([42; 16]),
        execution: participation::ExecutionIdV1([41; 16]),
        prompt_digest: [45; 32],
        attempt: binding.attempt,
    };
    apply_participation(
        &mut service,
        Some(2),
        participation::MutationV1::Prepare {
            participation: binding.participation,
            expected_revision: 2,
            operation: prompt,
        },
    );
    apply_participation(
        &mut service,
        Some(3),
        participation::MutationV1::Observe {
            participation: binding.participation,
            expected_revision: 3,
            key: participation::IdempotencyKeyV1([40; 16]),
            observation: participation::ObservationV1 {
                phase: participation::PhaseV1::Terminal,
                session_revision: 4,
                active_turn: None,
                last_event_cursor: Some(2),
                profile_document_digest: [30; 32],
                capabilities: participation::CapabilitiesV1::default(),
                permission: None,
                terminal: Some(participation::TerminalEvidenceV1 {
                    turn: participation::AgentTurnIdV1([42; 16]),
                    disposition: participation::DispositionV1::Completed,
                    stop_reason: participation::StopReasonV1::EndTurn,
                    output_digest: result_digest(),
                    provider_evidence_digest: [43; 32],
                }),
            },
        },
    );
}

fn start_moira(fixture: &Fixture) -> thread::JoinHandle<()> {
    start_moira_connections(fixture, 1)
}

fn start_moira_connections(fixture: &Fixture, connections: usize) -> thread::JoinHandle<()> {
    let mut daemon = TaskAgentJourneyDaemon::bind_new(
        &fixture.moira_socket,
        TaskAgentJourneyService::new(fixture.moira_store(), Time(200)),
        Authentication,
        LimitsV1::default(),
        Duration::from_secs(3),
        8,
    )
    .unwrap();
    thread::spawn(move || {
        for _ in 0..connections {
            daemon.serve_next().unwrap();
        }
    })
}

fn start_participation(fixture: &Fixture, connections: usize) -> thread::JoinHandle<()> {
    let store = AgentParticipationSqliteStore::open(
        &fixture.participation_database,
        [1; 16],
        participation_limits(),
    )
    .unwrap();
    let mut daemon = AgentParticipationDaemon::bind_new(
        &fixture.participation_socket,
        AgentParticipationService::new(store),
        ParticipationAuthentication,
        moira_wire::LimitsV1::default(),
        Duration::from_secs(3),
        8,
    )
    .unwrap();
    thread::spawn(move || {
        for _ in 0..connections {
            daemon.serve_next().unwrap();
        }
    })
}

fn start_daimon(fixture: &Fixture) -> thread::JoinHandle<()> {
    let mut daemon = AgentDaemon::bind_new(
        &fixture.daimon_socket,
        DaimonPort,
        DaimonAuthentication,
        daimon_wire::LimitsV1::default(),
        daimon_wire::agent::AgentWireLimitsV1::default(),
        Duration::from_secs(3),
        8,
    )
    .unwrap();
    thread::spawn(move || daemon.serve_next().unwrap())
}

fn start_agora(fixture: &Fixture, engine: AgoraEngine) -> thread::JoinHandle<()> {
    let host = HostV2::new(
        ServiceV2::new(engine, ServiceLimitsV2::default()),
        AegisVerifierV1::new(AgoraAuthority, [5; 16]),
        HostLimits {
            max_sessions: 4,
            max_retired_sessions: 8,
            wire: agora_wire::WireLimits {
                max_frame_bytes: 8_192,
                max_text_bytes: 16 * 1_024,
                max_fields: 100_000,
                max_list_items: 4_096,
            },
        },
    );
    let mut daemon = DaemonV1::bind_new(
        &fixture.agora_socket,
        host,
        AgoraAuthentication,
        AgoraClock(1),
        ServerIncarnationV1::new([9; 16]),
        DaemonLimitsV1 {
            max_frame_bytes: 8_192,
            max_frames_after_connect: 16,
            io_timeout: Duration::from_secs(3),
        },
    )
    .unwrap();
    thread::spawn(move || daemon.serve_next().unwrap())
}

fn start_relationships(fixture: &Fixture, reopen: bool) -> thread::JoinHandle<()> {
    let mut service = if reopen {
        TaskConversationLocalServiceV1::open_existing(
            &fixture.relationship_state,
            &fixture.relationship_socket,
            LocalServiceLimitsV1::default(),
        )
    } else {
        TaskConversationLocalServiceV1::create_new(
            &fixture.relationship_state,
            &fixture.relationship_socket,
            LocalServiceLimitsV1::default(),
        )
    }
    .unwrap();
    thread::spawn(move || service.serve().unwrap())
}

fn invoke_output(fixture: &Fixture, journey: public::JourneyIdV1) -> std::process::Output {
    Process::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "j1-review-link-observe",
            fixture.moira_socket.to_str().unwrap(),
            fixture.relationship_socket.to_str().unwrap(),
            fixture.chronos.to_str().unwrap(),
            &hex(&journey.owner.0),
            &journey.sequence.to_string(),
        ])
        .output()
        .unwrap()
}

fn invoke(fixture: &Fixture, journey: public::JourneyIdV1) -> Value {
    let output = invoke_output(fixture, journey);
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn invoke_refusal(fixture: &Fixture, journey: public::JourneyIdV1) -> Value {
    let output = invoke_output(fixture, journey);
    assert!(!output.status.success(), "refusal unexpectedly succeeded");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn invoke_strict_output(fixture: &Fixture, journey: public::JourneyIdV1) -> std::process::Output {
    Process::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "j1-strict-review-observe",
            fixture.moira_socket.to_str().unwrap(),
            fixture.participation_socket.to_str().unwrap(),
            fixture.relationship_socket.to_str().unwrap(),
            fixture.chronos.to_str().unwrap(),
            fixture.daimon_socket.to_str().unwrap(),
            fixture.agora_socket.to_str().unwrap(),
            "exact-agent",
            "conversation",
            &hex(&journey.owner.0),
            &journey.sequence.to_string(),
        ])
        .output()
        .unwrap()
}

fn invoke_strict(fixture: &Fixture, journey: public::JourneyIdV1) -> Value {
    let output = invoke_strict_output(fixture, journey);
    assert!(
        output.status.success(),
        "strict command failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn invoke_strict_refusal(fixture: &Fixture, journey: public::JourneyIdV1) -> Value {
    let output = invoke_strict_output(fixture, journey);
    assert!(
        !output.status.success(),
        "strict refusal unexpectedly succeeded"
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn hex<const N: usize>(value: &[u8; N]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn production_command_composes_real_owners_and_survives_restart() {
    let fixture = Fixture::new();
    let (chronos, mut worker) = start_chronos(&fixture.chronos, true);
    let chronos_revision = seed_chronos(&fixture.chronos, worker.as_mut().unwrap());
    drop(worker);
    assert!(chronos_revision >= 2);
    let (_, publication) = seeded_agora();
    let journey = seed_moira(&fixture, publication);
    seed_terminal_participation(&fixture);

    let relationships = start_relationships(&fixture, false);
    let link_client = TaskConversationLocalClientV1::new(&fixture.relationship_socket);
    let created = link_client
        .request(&LocalRequestV1::Bind {
            link: LinkWireV1 {
                link_id: "11111111111111111111111111111111".into(),
                task: TaskReferenceWireV1 {
                    scope: "product-delivery".into(),
                    task: "j1-owned-task".into(),
                    incarnation: "0".into(),
                    attempt: Some(1),
                },
                conversation: ConversationReferenceWireV1 {
                    room: "exact-room".into(),
                    thread_root: None,
                },
                operation_id: "22222222222222222222222222222222".into(),
                principal_id: "33333333333333333333333333333333".into(),
                created_at_unix_seconds: 100,
                visibility_principal_id: None,
                chronos_revision: "2".into(),
                agora_revision: publication.room_revision,
                observed_at_unix_seconds: 101,
            },
        })
        .unwrap();
    assert_eq!(created.status, "created");

    let moira = start_moira(&fixture);
    let first = invoke(&fixture, journey);
    moira.join().unwrap();
    assert_eq!(first["status"], "review_ready_with_result");
    assert_eq!(first["task"]["scope"], "product-delivery");
    assert_eq!(first["task"]["task_id"], "j1-owned-task");
    assert_eq!(first["task"]["workflow_status"], "ready_for_review");
    assert_eq!(first["publication"]["actor"], "exact-agent");
    assert!(
        first["boundary"]
            .as_str()
            .unwrap()
            .contains("no result bytes")
    );

    let moira = start_moira_connections(&fixture, 2);
    let participation = start_participation(&fixture, 2);
    let daimon = start_daimon(&fixture);
    let (agora_engine, restarted_publication) = seeded_agora();
    assert_eq!(
        restarted_publication.message_id, publication.message_id,
        "deterministic Agora restart fixture identity"
    );
    let agora = start_agora(&fixture, agora_engine);
    let strict_first = invoke_strict(&fixture, journey);
    moira.join().unwrap();
    participation.join().unwrap();
    daimon.join().unwrap();
    agora.join().unwrap();
    assert_eq!(strict_first["status"], "strict_review_evidence_issued");
    assert_eq!(strict_first["task"]["id"], "j1-owned-task");
    assert_eq!(strict_first["agent"]["participation"], hex(&[32; 16]));
    assert_eq!(strict_first["publication"]["actor"], "exact-agent");
    assert_eq!(
        strict_first["content"],
        std::str::from_utf8(result_body()).unwrap()
    );

    link_client.request(&LocalRequestV1::Drain).unwrap();
    relationships.join().unwrap();
    chronos.stop();

    let relationships = start_relationships(&fixture, true);
    let (chronos, _) = start_chronos(&fixture.chronos, false);
    let moira = start_moira(&fixture);
    let second = invoke(&fixture, journey);
    moira.join().unwrap();
    assert_eq!(second["task"], first["task"]);
    assert_eq!(second["publication"], first["publication"]);
    assert_eq!(second["relationship"], first["relationship"]);

    let moira = start_moira_connections(&fixture, 2);
    let participation = start_participation(&fixture, 2);
    let daimon = start_daimon(&fixture);
    let (agora_engine, restarted_publication) = seeded_agora();
    assert_eq!(restarted_publication.message_id, publication.message_id);
    let agora = start_agora(&fixture, agora_engine);
    let strict_second = invoke_strict(&fixture, journey);
    moira.join().unwrap();
    participation.join().unwrap();
    daimon.join().unwrap();
    agora.join().unwrap();
    assert_eq!(
        strict_second["receipt_digest"], strict_first["receipt_digest"],
        "same owner state after restart must identify the same strict evidence"
    );

    chronos.stop();
    let moira = start_moira(&fixture);
    let strict_unavailable = invoke_strict_refusal(&fixture, journey);
    moira.join().unwrap();
    assert_eq!(strict_unavailable["complete"], false);
    assert_eq!(strict_unavailable["status"], "evidence_refused");
    assert!(
        strict_unavailable["detail"]
            .as_str()
            .unwrap()
            .contains("initial J1 review-link observation refused")
    );
    assert!(
        strict_unavailable["boundary"]
            .as_str()
            .unwrap()
            .contains("Maybe Machine")
    );
    let moira = start_moira(&fixture);
    let unavailable = invoke_refusal(&fixture, journey);
    moira.join().unwrap();
    assert_eq!(unavailable["complete"], false);
    assert_eq!(unavailable["status"], "chronos_unavailable");
    assert!(
        unavailable["next_action"]
            .as_str()
            .unwrap()
            .contains("reattach")
    );

    let (chronos, _) = start_chronos(&fixture.chronos, false);
    let duplicate = TaskConversationLocalClientV1::new(&fixture.relationship_socket)
        .request(&LocalRequestV1::Bind {
            link: LinkWireV1 {
                link_id: "44444444444444444444444444444444".into(),
                task: TaskReferenceWireV1 {
                    scope: "another-scope".into(),
                    task: "j1-owned-task".into(),
                    incarnation: "0".into(),
                    attempt: Some(1),
                },
                conversation: ConversationReferenceWireV1 {
                    room: "exact-room".into(),
                    thread_root: None,
                },
                operation_id: "55555555555555555555555555555555".into(),
                principal_id: "66666666666666666666666666666666".into(),
                created_at_unix_seconds: 102,
                visibility_principal_id: None,
                chronos_revision: "2".into(),
                agora_revision: publication.room_revision,
                observed_at_unix_seconds: 103,
            },
        })
        .unwrap();
    assert_eq!(duplicate.status, "created");
    let moira = start_moira(&fixture);
    let ambiguous = invoke_refusal(&fixture, journey);
    moira.join().unwrap();
    assert_eq!(ambiguous["complete"], false);
    assert_eq!(ambiguous["status"], "evidence_refused");
    assert!(
        ambiguous["detail"]
            .as_str()
            .unwrap()
            .contains("AmbiguousTaskConversationObservation")
    );

    TaskConversationLocalClientV1::new(&fixture.relationship_socket)
        .request(&LocalRequestV1::Drain)
        .unwrap();
    relationships.join().unwrap();
    chronos.stop();
}

#[test]
fn strict_command_refuses_reopened_work_and_changed_attempt() {
    let fixture = Fixture::new();
    let (chronos, mut worker) = start_chronos(&fixture.chronos, true);
    seed_chronos(&fixture.chronos, worker.as_mut().unwrap());
    let (_, publication) = seeded_agora();
    let journey = seed_moira(&fixture, publication);
    seed_terminal_participation(&fixture);

    let relationships = start_relationships(&fixture, false);
    let link_client = TaskConversationLocalClientV1::new(&fixture.relationship_socket);
    assert_eq!(
        link_client
            .request(&LocalRequestV1::Bind {
                link: LinkWireV1 {
                    link_id: "77777777777777777777777777777777".into(),
                    task: TaskReferenceWireV1 {
                        scope: "product-delivery".into(),
                        task: "j1-owned-task".into(),
                        incarnation: "0".into(),
                        attempt: Some(1),
                    },
                    conversation: ConversationReferenceWireV1 {
                        room: "exact-room".into(),
                        thread_root: None,
                    },
                    operation_id: "88888888888888888888888888888888".into(),
                    principal_id: "99999999999999999999999999999999".into(),
                    created_at_unix_seconds: 100,
                    visibility_principal_id: None,
                    chronos_revision: "2".into(),
                    agora_revision: publication.room_revision,
                    observed_at_unix_seconds: 101,
                },
            })
            .unwrap()
            .status,
        "created"
    );

    let moira = start_moira_connections(&fixture, 2);
    let participation = start_participation(&fixture, 2);
    let daimon = start_daimon(&fixture);
    let (agora_engine, _) = seeded_agora();
    let agora = start_agora(&fixture, agora_engine);
    let baseline = invoke_strict(&fixture, journey);
    moira.join().unwrap();
    participation.join().unwrap();
    daimon.join().unwrap();
    agora.join().unwrap();
    assert_eq!(baseline["status"], "strict_review_evidence_issued");

    require_revision_required(&fixture.chronos);
    assert_attempt(&fixture.chronos, None, workflow::Status::Open);
    let moira = start_moira(&fixture);
    let reopened = invoke_strict_refusal(&fixture, journey);
    moira.join().unwrap();
    assert_eq!(reopened["status"], "evidence_refused");
    assert!(
        reopened["detail"]
            .as_str()
            .unwrap()
            .contains("initial J1 review-link observation refused")
    );
    assert!(
        reopened["boundary"]
            .as_str()
            .unwrap()
            .contains("Maybe Machine")
    );

    complete_reopened_attempt(worker.as_mut().unwrap());
    assert_attempt(&fixture.chronos, Some(2), workflow::Status::ReadyForReview);
    let moira = start_moira(&fixture);
    let changed_attempt = invoke_strict_refusal(&fixture, journey);
    moira.join().unwrap();
    assert_eq!(changed_attempt["status"], "evidence_refused");
    assert!(
        changed_attempt["detail"]
            .as_str()
            .unwrap()
            .contains("AttemptMismatch")
    );

    link_client.request(&LocalRequestV1::Drain).unwrap();
    relationships.join().unwrap();
    drop(worker);
    chronos.stop();
}
