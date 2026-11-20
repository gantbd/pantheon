#![cfg(unix)]

use agora_chronos_adapter::task_conversation_local::{
    ConversationReferenceWireV1, LinkWireV1, LocalRequestV1, LocalServiceLimitsV1,
    TaskConversationLocalClientV1, TaskConversationLocalServiceV1, TaskReferenceWireV1,
};
use chronos_client::{WireClientV1, WorkflowWireClientV9};
use chronos_engine::{Agent, CommandOutcomeV2, TaskState};
use chronos_host::worker::{WorkerClientV1, WorkerRequestV1 as Work};
use chronos_local::LocalClient;
use chronos_wire::{
    Command, Outcome, Request, RetrySafety, Revision, TaskSpec, workflow_v9 as workflow,
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
use moira_local_daemon::unix::{Authenticator, TaskAgentJourneyDaemon};
use moira_protocol::{TaskRefV1, agent_participation::ProfileIdV1, task_agent_journey as public};
use moira_service::{
    AccessV1,
    task_agent_journey::{TaskAgentJourneyService, TrustedTimeV1},
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
                        max_attempts: 1,
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

fn seed_moira(fixture: &Fixture) -> public::JourneyIdV1 {
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
        agora_revision: 8,
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
                    message_sequence: 3,
                    message_id: [37; 16],
                    agora_revision: 9,
                    actor: "exact-agent".into(),
                    subject: authority.principal,
                },
            },
        )
        .unwrap();
    drop(store);
    journey
}

fn start_moira(fixture: &Fixture) -> thread::JoinHandle<()> {
    let mut daemon = TaskAgentJourneyDaemon::bind_new(
        &fixture.moira_socket,
        TaskAgentJourneyService::new(fixture.moira_store(), Time(200)),
        Authentication,
        LimitsV1::default(),
        Duration::from_secs(3),
        8,
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
    let journey = seed_moira(&fixture);

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
                agora_revision: 8,
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

    chronos.stop();
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
                agora_revision: 8,
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
