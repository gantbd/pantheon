use daimon_protocol::{
    AgentSessionIdV1, AgentTurnIdV1, ContentDigestV1, ExecutionDispositionV1, ExecutionIdV1,
    TaskAttemptRefV1,
    agent::{AgentEventPageV1, AgentEventV1, AgentResultBindingV1, AgentTurnStopReasonV1},
};
use sha2::{Digest, Sha256};
use std::{ffi::OsString, path::PathBuf, time::Duration};

pub const CAPABILITY: &str = "agents.result.observe.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualifiedAgentResult {
    pub session: AgentSessionIdV1,
    pub turn: AgentTurnIdV1,
    pub assistant_cursor: u64,
    pub finished_cursor: u64,
    pub output_digest: ContentDigestV1,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerBoundAgentResult {
    pub session: AgentSessionIdV1,
    pub turn: AgentTurnIdV1,
    pub execution: ExecutionIdV1,
    pub task_attempt: TaskAttemptRefV1,
    pub terminal_revision: u64,
    pub assistant_cursor: u64,
    pub finished_cursor: u64,
    pub output_digest: ContentDigestV1,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentResultRefusal {
    Truncated,
    MissingAssistant,
    AmbiguousAssistant,
    MissingFinished,
    AmbiguousFinished,
    UnexpectedContentEvent,
    EmptyAssistant,
    InvalidUtf8,
    InputLimit,
    InvalidOrder,
    TurnMismatch,
    IncompleteTurn,
    MissingDigest,
    DigestMismatch,
    OwnerBindingMismatch,
    IntervalMismatch,
}

/// Qualifies one bounded, complete Daimon event window as the final assistant
/// result for an exact turn. This is observation only: it grants no Agora,
/// Chronos, provider, tool, file, or approval authority.
pub fn qualify_agent_result(
    page: &AgentEventPageV1,
    expected_turn: AgentTurnIdV1,
    max_bytes: usize,
) -> Result<QualifiedAgentResult, AgentResultRefusal> {
    if page.next_after.is_some() {
        return Err(AgentResultRefusal::Truncated);
    }
    if page
        .entries
        .windows(2)
        .any(|pair| pair[0].cursor >= pair[1].cursor)
    {
        return Err(AgentResultRefusal::InvalidOrder);
    }

    let mut assistant = None;
    let mut finished = None;
    for record in &page.entries {
        match &record.event {
            AgentEventV1::AssistantMessage { bytes } => {
                if assistant.replace((record.cursor, bytes)).is_some() {
                    return Err(AgentResultRefusal::AmbiguousAssistant);
                }
            }
            AgentEventV1::TurnFinished {
                turn,
                disposition,
                stop_reason,
                output_digest,
            } => {
                if finished
                    .replace((
                        record.cursor,
                        *turn,
                        *disposition,
                        *stop_reason,
                        *output_digest,
                    ))
                    .is_some()
                {
                    return Err(AgentResultRefusal::AmbiguousFinished);
                }
            }
            AgentEventV1::Usage { .. } => {}
            AgentEventV1::Thought { .. }
            | AgentEventV1::Plan { .. }
            | AgentEventV1::ToolCall { .. }
            | AgentEventV1::Terminal { .. }
            | AgentEventV1::Permission(_)
            | AgentEventV1::FileChange { .. }
            | AgentEventV1::FileRead { .. } => {
                return Err(AgentResultRefusal::UnexpectedContentEvent);
            }
        }
    }

    let (assistant_cursor, bytes) = assistant.ok_or(AgentResultRefusal::MissingAssistant)?;
    let (finished_cursor, turn, disposition, stop_reason, output_digest) =
        finished.ok_or(AgentResultRefusal::MissingFinished)?;
    if assistant_cursor >= finished_cursor {
        return Err(AgentResultRefusal::InvalidOrder);
    }
    if bytes.is_empty() {
        return Err(AgentResultRefusal::EmptyAssistant);
    }
    if bytes.len() > max_bytes.min(4_096) {
        return Err(AgentResultRefusal::InputLimit);
    }
    if std::str::from_utf8(bytes).is_err() {
        return Err(AgentResultRefusal::InvalidUtf8);
    }
    if turn != expected_turn {
        return Err(AgentResultRefusal::TurnMismatch);
    }
    if disposition != ExecutionDispositionV1::Completed
        || stop_reason != AgentTurnStopReasonV1::EndTurn
    {
        return Err(AgentResultRefusal::IncompleteTurn);
    }
    let output_digest = output_digest.ok_or(AgentResultRefusal::MissingDigest)?;
    let observed_digest = ContentDigestV1::new(Sha256::digest(bytes).into());
    if output_digest != observed_digest {
        return Err(AgentResultRefusal::DigestMismatch);
    }

    Ok(QualifiedAgentResult {
        session: page.session,
        turn,
        assistant_cursor,
        finished_cursor,
        output_digest,
        bytes: bytes.clone(),
    })
}

/// Qualifies content only when Daimon's retained operation receipt binds the
/// exact turn, execution, task attempt, authority, context, and terminal event
/// interval. The receipt remains observational and carries no settlement or
/// downstream mutation authority.
pub fn qualify_bound_agent_result(
    page: &AgentEventPageV1,
    binding: &AgentResultBindingV1,
    max_bytes: usize,
) -> Result<OwnerBoundAgentResult, AgentResultRefusal> {
    if page.session != binding.session
        || binding.terminal_cursor < binding.event_start_cursor
        || page.entries.first().map(|record| record.cursor) != Some(binding.event_start_cursor)
        || page.entries.last().map(|record| record.cursor) != Some(binding.terminal_cursor)
    {
        return Err(AgentResultRefusal::IntervalMismatch);
    }
    let qualified = qualify_agent_result(page, binding.turn, max_bytes)?;
    if qualified.finished_cursor != binding.terminal_cursor
        || qualified.output_digest
            != binding
                .output_digest
                .ok_or(AgentResultRefusal::MissingDigest)?
        || binding.disposition != ExecutionDispositionV1::Completed
        || binding.stop_reason != AgentTurnStopReasonV1::EndTurn
    {
        return Err(AgentResultRefusal::OwnerBindingMismatch);
    }
    Ok(OwnerBoundAgentResult {
        session: binding.session,
        turn: binding.turn,
        execution: binding.execution,
        task_attempt: binding.task_attempt,
        terminal_revision: binding.terminal_revision,
        assistant_cursor: qualified.assistant_cursor,
        finished_cursor: qualified.finished_cursor,
        output_digest: qualified.output_digest,
        bytes: qualified.bytes,
    })
}

pub fn run(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    use daimon_client::AgentReadOnlyClient;
    use daimon_protocol::agent::{AgentOutcomeV1, AgentReplyV1};

    let usage = "J1 agent-result observation requires SOCKET SESSION_HEX TURN_HEX";
    let socket = PathBuf::from(args.next().ok_or(usage)?);
    let session_text = args
        .next()
        .ok_or(usage)?
        .into_string()
        .map_err(|_| "session identity must be UTF-8")?;
    let turn_text = args
        .next()
        .ok_or(usage)?
        .into_string()
        .map_err(|_| "turn identity must be UTF-8")?;
    if args.next().is_some() {
        return Err(usage.into());
    }
    let session = daimon_tui::parse_agent_session(&session_text)
        .ok_or("session identity must be exactly 32 hexadecimal digits")?;
    let turn = daimon_tui::parse_agent_turn(&turn_text)
        .ok_or("turn identity must be exactly 32 hexadecimal digits")?;
    let (service_session, port) = daimon_client::unix::connect_agent(
        socket,
        daimon_wire::LimitsV1::default(),
        daimon_wire::agent::AgentWireLimitsV1::default(),
        Duration::from_secs(5),
    )
    .map_err(|error| format!("Daimon result connection refused: {error:?}"))?;
    let mut client = AgentReadOnlyClient::new(service_session, port);
    let binding = match client
        .result(session, turn)
        .map_err(|error| format!("Daimon result binding request failed: {error:?}"))?
    {
        AgentReplyV1::Accepted(AgentOutcomeV1::Result(binding)) => binding,
        reply => return Err(format!("Daimon result binding refused: {reply:?}").into()),
    };
    let event_count = binding
        .terminal_cursor
        .checked_sub(binding.event_start_cursor)
        .and_then(|distance| distance.checked_add(1))
        .ok_or("Daimon result binding published an invalid event interval")?;
    let page = client
        .event_page(
            session,
            binding.event_start_cursor.checked_sub(1),
            event_count,
        )
        .ok_or("Daimon refused the exact result event interval")?;
    let result = qualify_bound_agent_result(&page, &binding, 4_096)
        .map_err(|refusal| format!("Daimon result qualification refused: {refusal:?}"))?;
    let content = String::from_utf8(result.bytes.clone())
        .map_err(|_| "qualified assistant result was not UTF-8")?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "peer": "daimon",
            "capability": CAPABILITY,
            "complete": true,
            "status": "qualified",
            "session": session_text,
            "turn": turn_text,
            "execution": hex(&result.execution.into_bytes()),
            "task": {
                "key": hex(&result.task_attempt.task_key),
                "incarnation": result.task_attempt.task_incarnation,
                "attempt": result.task_attempt.attempt_number,
            },
            "terminal_revision": result.terminal_revision,
            "assistant_cursor": result.assistant_cursor,
            "finished_cursor": result.finished_cursor,
            "output_digest": format!("sha256:{}", hex(&result.output_digest.into_bytes())),
            "content_bytes": result.bytes.len(),
            "content": content,
            "boundary": "one exact Daimon-owned turn-to-execution-and-task-attempt result binding; observation only, with no Agora publication, no Chronos mutation, no provider invocation, no tool authority, no file authority, no approval, no acceptance, and no rollout authority",
        }))?
    );
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use daimon_protocol::{
        AgentSessionIdV1, AgentTurnIdV1, ContentDigestV1, ExecutionDispositionV1,
        agent::{AgentEventPageV1, AgentEventRecordV1, AgentEventV1, AgentTurnStopReasonV1},
    };
    use sha2::{Digest, Sha256};

    #[allow(dead_code)]
    mod forged {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../design/generated/j1_agent_result_observation_tests.rs"
        ));
    }
    use forged::*;

    fn session() -> AgentSessionIdV1 {
        AgentSessionIdV1::new([0x15; 16])
    }
    fn turn() -> AgentTurnIdV1 {
        AgentTurnIdV1::new([0x16; 16])
    }
    fn digest(bytes: &[u8]) -> ContentDigestV1 {
        ContentDigestV1::new(Sha256::digest(bytes).into())
    }
    fn valid_page() -> AgentEventPageV1 {
        let bytes = b"The acceptance evidence is bounded and attributable.".to_vec();
        AgentEventPageV1 {
            session: session(),
            entries: vec![
                AgentEventRecordV1 {
                    cursor: 1,
                    event: AgentEventV1::AssistantMessage {
                        bytes: bytes.clone(),
                    },
                },
                AgentEventRecordV1 {
                    cursor: 2,
                    event: AgentEventV1::TurnFinished {
                        turn: turn(),
                        disposition: ExecutionDispositionV1::Completed,
                        stop_reason: AgentTurnStopReasonV1::EndTurn,
                        output_digest: Some(digest(&bytes)),
                    },
                },
            ],
            next_after: None,
        }
    }

    fn valid_binding() -> AgentResultBindingV1 {
        AgentResultBindingV1 {
            session: session(),
            turn: turn(),
            execution: ExecutionIdV1::new([0x18; 16]),
            task_attempt: TaskAttemptRefV1 {
                task_key: [0x14; 16],
                task_incarnation: 1,
                attempt_number: 2,
            },
            principal: daimon_protocol::PrincipalIdV1::new([0x10; 16]),
            profile: daimon_protocol::HarnessProfileIdV1::new([0x11; 16]),
            operation: daimon_protocol::IdempotencyKeyV1::new([0x20; 16]),
            authority_decision: [0x21; 16],
            authority_expires_at: 100,
            prompt_digest: ContentDigestV1::new([0x22; 32]),
            context_manifest_digest: ContentDigestV1::new([0x23; 32]),
            event_start_cursor: 1,
            terminal_cursor: 2,
            terminal_revision: 7,
            disposition: ExecutionDispositionV1::Completed,
            stop_reason: AgentTurnStopReasonV1::EndTurn,
            output_digest: Some(digest(
                b"The acceptance evidence is bounded and attributable.",
            )),
        }
    }

    #[test]
    fn owner_binding_is_required_to_attribute_content_to_an_execution_and_task_attempt() {
        let page = valid_page();
        let binding = valid_binding();
        let qualified = qualify_bound_agent_result(&page, &binding, 4_096).unwrap();
        assert_eq!(qualified.execution, binding.execution);
        assert_eq!(qualified.task_attempt, binding.task_attempt);
        assert_eq!(qualified.terminal_revision, 7);

        let mut wrong_interval = binding.clone();
        wrong_interval.event_start_cursor = 0;
        assert_eq!(
            qualify_bound_agent_result(&page, &wrong_interval, 4_096),
            Err(AgentResultRefusal::IntervalMismatch)
        );

        let mut false_completion = binding;
        false_completion.disposition = ExecutionDispositionV1::Failed;
        assert_eq!(
            qualify_bound_agent_result(&page, &false_completion, 4_096),
            Err(AgentResultRefusal::OwnerBindingMismatch)
        );
    }

    struct Adapter {
        baseline: bool,
        result: bool,
        evaluated: usize,
    }
    impl Adapter {
        fn prove_baseline(&mut self) -> bool {
            let empty = AgentEventPageV1 {
                session: session(),
                entries: Vec::new(),
                next_after: None,
            };
            self.baseline = matches!(
                qualify_agent_result(&empty, turn(), 4_096),
                Err(AgentResultRefusal::MissingAssistant)
            );
            self.baseline
        }

        fn prove_result(&mut self) -> bool {
            let valid = valid_page();
            let qualified = qualify_agent_result(&valid, turn(), 4_096);
            let mut truncated = valid_page();
            truncated.next_after = Some(2);
            let mut extra_content = valid_page();
            extra_content.entries.insert(
                1,
                AgentEventRecordV1 {
                    cursor: 2,
                    event: AgentEventV1::Thought {
                        bytes: b"private".to_vec(),
                    },
                },
            );
            extra_content.entries[2].cursor = 3;
            let mut invalid_utf8 = valid_page();
            invalid_utf8.entries[0].event = AgentEventV1::AssistantMessage { bytes: vec![0xff] };
            let mut oversized = valid_page();
            oversized.entries[0].event = AgentEventV1::AssistantMessage {
                bytes: vec![b'x'; 4_097],
            };
            let mut mismatch = valid_page();
            if let AgentEventV1::TurnFinished { output_digest, .. } = &mut mismatch.entries[1].event
            {
                *output_digest = Some(ContentDigestV1::new([0x44; 32]));
            }
            let mut wrong_turn = valid_page();
            if let AgentEventV1::TurnFinished { turn, .. } = &mut wrong_turn.entries[1].event {
                *turn = AgentTurnIdV1::new([0x99; 16]);
            }
            let mut failed = valid_page();
            if let AgentEventV1::TurnFinished { disposition, .. } = &mut failed.entries[1].event {
                *disposition = ExecutionDispositionV1::Failed;
            }
            let mut reversed = valid_page();
            reversed.entries[0].cursor = 3;
            let mut duplicate = valid_page();
            duplicate.entries.insert(
                1,
                AgentEventRecordV1 {
                    cursor: 2,
                    event: AgentEventV1::AssistantMessage {
                        bytes: b"second".to_vec(),
                    },
                },
            );
            duplicate.entries[2].cursor = 3;

            self.result = qualified.as_ref().is_ok_and(|result| {
                result.session == session()
                    && result.turn == turn()
                    && result.assistant_cursor == 1
                    && result.finished_cursor == 2
                    && result.bytes == b"The acceptance evidence is bounded and attributable."
                    && result.output_digest == digest(&result.bytes)
            }) && matches!(
                qualify_agent_result(&truncated, turn(), 4_096),
                Err(AgentResultRefusal::Truncated)
            ) && matches!(
                qualify_agent_result(&extra_content, turn(), 4_096),
                Err(AgentResultRefusal::UnexpectedContentEvent)
            ) && matches!(
                qualify_agent_result(&invalid_utf8, turn(), 4_096),
                Err(AgentResultRefusal::InvalidUtf8)
            ) && matches!(
                qualify_agent_result(&oversized, turn(), 4_096),
                Err(AgentResultRefusal::InputLimit)
            ) && matches!(
                qualify_agent_result(&mismatch, turn(), 4_096),
                Err(AgentResultRefusal::DigestMismatch)
            ) && matches!(
                qualify_agent_result(&wrong_turn, turn(), 4_096),
                Err(AgentResultRefusal::TurnMismatch)
            ) && matches!(
                qualify_agent_result(&failed, turn(), 4_096),
                Err(AgentResultRefusal::IncompleteTurn)
            ) && matches!(
                qualify_agent_result(&reversed, turn(), 4_096),
                Err(AgentResultRefusal::InvalidOrder)
            ) && matches!(
                qualify_agent_result(&duplicate, turn(), 4_096),
                Err(AgentResultRefusal::AmbiguousAssistant)
            );
            self.result
        }
    }

    impl J1AgentResultObservationTestsEvaluator for Adapter {
        fn evaluate(&mut self, obligation: &J1AgentResultObservationTestsObligation) -> bool {
            self.evaluated += 1;
            match obligation.digest {
                "sha256:889b2f432c4272c925b4ab19d1ec4c3ff4fc86491985b6c91e47121e72603bab" => {
                    self.prove_baseline()
                }
                "sha256:abae8d307a804016b673afb3218f4454203a4c6c5de740d5140077d06dabc3cf" => {
                    self.prove_result()
                }
                "sha256:f4102d3574bb620072bd467968e589b6b9e521862f6b8253a1e3ec3b42ceac9f" => {
                    self.result
                }
                "sha256:b8dd92727958dfa165a917b20067dfe0fd48acd10135003611e9a0292cd6d854" => {
                    self.evaluated == 4
                }
                _ => false,
            }
        }
    }

    struct NoChoice;
    impl J1AgentResultObservationTestsChooser for NoChoice {
        fn choose(
            &mut self,
            _: J1AgentResultObservationTestsState,
            _: &[J1AgentResultObservationTestsCandidate],
        ) -> Option<usize> {
            panic!("the forged result-observation machine is compelled")
        }
    }

    #[derive(Default)]
    struct Observer(Vec<J1AgentResultObservationTestsObservation>);
    impl J1AgentResultObservationTestsObserver for Observer {
        fn observe(&mut self, observation: &J1AgentResultObservationTestsObservation) {
            self.0.push(*observation);
        }
    }

    #[test]
    fn forged_j1_result_observation_requires_actual_owner_evidence() {
        let mut adapter = Adapter {
            baseline: false,
            result: false,
            evaluated: 0,
        };
        let mut chooser = NoChoice;
        let mut observer = Observer::default();
        let mut state = J1AgentResultObservationTestsState::INITIAL;
        while state != J1AgentResultObservationTestsState::Complete {
            state = state
                .advance(&mut adapter, &mut chooser, &mut observer)
                .expect("every forged result obligation must be proven")
                .state;
        }
        assert_eq!(adapter.evaluated, 4);
        assert_eq!(observer.0.len(), 2);
        assert!(observer.0.iter().all(|observation| {
            observation.lineage == &J1AGENTRESULTOBSERVATIONTESTS_LINEAGE
                && observation.outcome == J1AgentResultObservationTestsObservationOutcome::Advanced
                && !observation.resource_error
                && !observation.saturation
        }));
    }
}
