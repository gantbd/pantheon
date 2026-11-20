//! In-memory composition of the question-bound Rune blueprint adapter and the
//! existing human-gated Vellum packet publisher.

use serde_json::{Value, json};
use std::{ffi::OsString, path::PathBuf, time::Instant};

#[allow(dead_code)]
mod generated {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../design/generated/pantheon_rune_blueprint_publication.rs"
    ));
}

use generated::{
    PANTHEONRUNEBLUEPRINTPUBLICATION_LINEAGE, PantheonRuneBlueprintPublicationAdvanced,
    PantheonRuneBlueprintPublicationCandidate, PantheonRuneBlueprintPublicationChooser,
    PantheonRuneBlueprintPublicationEvaluator, PantheonRuneBlueprintPublicationInvocation,
    PantheonRuneBlueprintPublicationInvocationVerdict, PantheonRuneBlueprintPublicationObligation,
    PantheonRuneBlueprintPublicationObligationKind, PantheonRuneBlueprintPublicationObservation,
    PantheonRuneBlueprintPublicationObserver, PantheonRuneBlueprintPublicationSignal,
    PantheonRuneBlueprintPublicationState,
};

const CAPABILITY: &str = "rune.blueprint-publication.v1";
const BOUNDARY: &str = "in-memory composition of exact blueprint qualification and the existing packet-publication owner only; no inferred answer, copied publication semantics, arbitrary Vellum target, task effect, acceptance, go-no-go, or rollout authority";

pub fn run(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let mode = text(required(&mut args)?)?;
    if !matches!(mode.as_str(), "preview" | "publish") {
        return Err("Rune blueprint publication mode must be preview or publish".into());
    }
    let profile = PathBuf::from(required(&mut args)?);
    let blueprint = PathBuf::from(required(&mut args)?);
    let operation = text(required(&mut args)?)?;
    let approval = if mode == "publish" {
        Some(PathBuf::from(
            args.next()
                .ok_or("publish requires one private approval document")?,
        ))
    } else {
        None
    };
    let answers = match args.next() {
        None => None,
        Some(flag) if flag == "--answers" => Some(PathBuf::from(
            args.next()
                .ok_or("--answers requires a private answer-set path")?,
        )),
        Some(_) => return Err("unexpected Rune blueprint publication argument".into()),
    };
    if args.next().is_some() {
        return Err("unexpected Rune blueprint publication argument".into());
    }

    match execute(
        &mode,
        &profile,
        &blueprint,
        answers.as_deref(),
        &operation,
        approval.as_deref(),
    ) {
        Ok(value) => {
            println!("{}", serde_json::to_string_pretty(&value)?);
            Ok(())
        }
        Err(value) => {
            println!("{}", serde_json::to_string_pretty(&value)?);
            Err(std::io::Error::other("Rune blueprint publication refused").into())
        }
    }
}

fn execute(
    mode: &str,
    publication_profile: &std::path::Path,
    blueprint_path: &std::path::Path,
    answers_path: Option<&std::path::Path>,
    operation: &str,
    approval_path: Option<&std::path::Path>,
) -> Result<Value, Value> {
    let started = Instant::now();
    let profile = crate::rune_packet_publication::publication_profile_binding(publication_profile)
        .map_err(|failure| {
            refusal(
                "publication_profile",
                "the deployment-owned publication profile refused before blueprint invocation",
                &Value::Null,
                Some(failure),
                started,
            )
        })?;
    let prepared =
        crate::rune_blueprint::prepare(&profile.rune_profile, blueprint_path, answers_path)
            .map_err(|failure| {
                refusal(
                    "blueprint",
                    "the Rune blueprint owner refused preparation",
                    &failure,
                    None,
                    started,
                )
            })?;
    let blueprint = prepared.response;
    let status = blueprint
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("refused");

    if status == "questions_pending" {
        let parent = advance(
            PantheonRuneBlueprintPublicationState::INITIAL,
            PantheonRuneBlueprintPublicationSignal::ClarificationRequired,
            PantheonRuneBlueprintPublicationInvocationVerdict::Unknown,
        )
        .map_err(|detail| refusal("composition", detail, &blueprint, None, started))?;
        return Ok(response(
            "questions_pending",
            false,
            parent.state,
            parent.signal,
            blueprint,
            None,
            started,
            None,
        ));
    }

    if status != "projection_qualified"
        || blueprint.get("complete").and_then(Value::as_bool) != Some(true)
    {
        return Err(refusal(
            "blueprint",
            "Rune blueprint did not produce one complete qualified projection",
            &blueprint,
            None,
            started,
        ));
    }

    let projected = advance(
        PantheonRuneBlueprintPublicationState::INITIAL,
        PantheonRuneBlueprintPublicationSignal::ProjectionAdmitted,
        PantheonRuneBlueprintPublicationInvocationVerdict::Unknown,
    )
    .map_err(|detail| refusal("composition", detail, &blueprint, None, started))?;
    let eligible = blueprint
        .pointer("/risk_policy/publication")
        .and_then(Value::as_str)
        == Some("eligible_for_separate_human_approved_vellum_patch");
    if !eligible {
        let refused = advance(
            projected.state,
            PantheonRuneBlueprintPublicationSignal::PublicationRefused,
            PantheonRuneBlueprintPublicationInvocationVerdict::Unknown,
        )
        .map_err(|detail| refusal("composition", detail, &blueprint, None, started))?;
        return Err(response(
            "publication_refused",
            false,
            refused.state,
            refused.signal,
            blueprint,
            None,
            started,
            Some("risk_admission"),
        ));
    }
    let admitted = advance(
        projected.state,
        PantheonRuneBlueprintPublicationSignal::RiskAdmitted,
        PantheonRuneBlueprintPublicationInvocationVerdict::Unknown,
    )
    .map_err(|detail| refusal("composition", detail, &blueprint, None, started))?;

    let owner = blueprint.get("owner_outcome").ok_or_else(|| {
        refusal(
            "blueprint",
            "qualified blueprint omitted the exact Rune owner outcome",
            &blueprint,
            None,
            started,
        )
    })?;
    let child = match crate::rune_packet_publication::execute_qualified(
        mode,
        publication_profile,
        &prepared.revised_request,
        owner,
        &profile.sha256,
        operation,
        approval_path,
    ) {
        Ok(child) => child,
        Err(child) => {
            let _ = advance(
                admitted.state,
                PantheonRuneBlueprintPublicationSignal::BlueprintPublished,
                PantheonRuneBlueprintPublicationInvocationVerdict::Refused,
            );
            return Err(refusal(
                "packet_publication",
                "the existing packet-publication owner refused",
                &blueprint,
                Some(child),
                started,
            ));
        }
    };

    if mode == "preview" {
        return Ok(response(
            "preview_returned",
            true,
            admitted.state,
            admitted.signal,
            blueprint,
            Some(child),
            started,
            None,
        ));
    }

    let child_complete = child.get("status").and_then(Value::as_str) == Some("durable_published")
        && child.get("complete").and_then(Value::as_bool) == Some(true);
    let verdict = if child_complete {
        PantheonRuneBlueprintPublicationInvocationVerdict::Succeeded
    } else {
        PantheonRuneBlueprintPublicationInvocationVerdict::Unknown
    };
    let child_status = child
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("outcome_unknown")
        .to_owned();
    match advance(
        admitted.state,
        PantheonRuneBlueprintPublicationSignal::BlueprintPublished,
        verdict,
    ) {
        Ok(parent) if child_complete => Ok(response(
            "durable_published",
            true,
            parent.state,
            parent.signal,
            blueprint,
            Some(child),
            started,
            None,
        )),
        _ => Ok(response(
            &child_status,
            false,
            admitted.state,
            admitted.signal,
            blueprint,
            Some(child),
            started,
            None,
        )),
    }
}

struct Evaluator {
    signal: &'static str,
    verdict: PantheonRuneBlueprintPublicationInvocationVerdict,
}

impl PantheonRuneBlueprintPublicationEvaluator for Evaluator {
    fn evaluate(&mut self, obligation: &PantheonRuneBlueprintPublicationObligation) -> bool {
        obligation.kind == PantheonRuneBlueprintPublicationObligationKind::Postcondition
            || obligation.signal == self.signal
    }

    fn settle_invocation(
        &mut self,
        invocation: &PantheonRuneBlueprintPublicationInvocation,
    ) -> PantheonRuneBlueprintPublicationInvocationVerdict {
        if invocation.child == "pantheon-rune-packet-publication" {
            self.verdict
        } else {
            PantheonRuneBlueprintPublicationInvocationVerdict::Unknown
        }
    }
}

struct Chooser;
impl PantheonRuneBlueprintPublicationChooser for Chooser {
    fn choose(
        &mut self,
        _: PantheonRuneBlueprintPublicationState,
        _: &[PantheonRuneBlueprintPublicationCandidate],
    ) -> Option<usize> {
        None
    }
}

struct Observer;
impl PantheonRuneBlueprintPublicationObserver for Observer {
    fn observe(&mut self, _: &PantheonRuneBlueprintPublicationObservation) {}
}

fn advance(
    state: PantheonRuneBlueprintPublicationState,
    signal: PantheonRuneBlueprintPublicationSignal,
    verdict: PantheonRuneBlueprintPublicationInvocationVerdict,
) -> Result<PantheonRuneBlueprintPublicationAdvanced, String> {
    let signal_name = signal_name(signal);
    state
        .advance(
            &mut Evaluator {
                signal: signal_name,
                verdict,
            },
            &mut Chooser,
            &mut Observer,
        )
        .map_err(|refusal| format!("generated composition refused: {refusal:?}"))
}

fn response(
    status: &str,
    complete: bool,
    state: PantheonRuneBlueprintPublicationState,
    signal: PantheonRuneBlueprintPublicationSignal,
    blueprint: Value,
    child: Option<Value>,
    started: Instant,
    refusal_stage: Option<&str>,
) -> Value {
    let observation = observation(&blueprint, child.as_ref(), started, refusal_stage);
    json!({
        "version": 1,
        "capability": CAPABILITY,
        "status": status,
        "complete": complete,
        "machine_state": state_name(state),
        "signal": signal_name(signal),
        "blueprint": blueprint,
        "packet_publication": child,
        "lineage": {
            "source_sha256": PANTHEONRUNEBLUEPRINTPUBLICATION_LINEAGE.source_sha256,
            "typed_ir_sha256": PANTHEONRUNEBLUEPRINTPUBLICATION_LINEAGE.typed_ir_sha256,
            "smith_sha256": PANTHEONRUNEBLUEPRINTPUBLICATION_LINEAGE.smith_sha256,
        },
        "observation": observation,
        "boundary": BOUNDARY,
    })
}

fn refusal(
    stage: &str,
    detail: impl Into<String>,
    blueprint: &Value,
    child: Option<Value>,
    started: Instant,
) -> Value {
    let observation = observation(blueprint, child.as_ref(), started, Some(stage));
    json!({
        "version": 1,
        "capability": CAPABILITY,
        "status": "refused",
        "complete": false,
        "stage": stage,
        "detail": detail.into(),
        "blueprint": blueprint,
        "packet_publication": child,
        "observation": observation,
        "boundary": BOUNDARY,
    })
}

fn observation(
    blueprint: &Value,
    child: Option<&Value>,
    started: Instant,
    refusal_stage: Option<&str>,
) -> Value {
    let duration_ms = started.elapsed().as_millis();
    let blueprint_duration_ms = blueprint
        .pointer("/observation/duration_ms")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let owner_requests = blueprint
        .pointer("/observation/request_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let questions_before = blueprint
        .pointer("/observation/questions_before")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let questions_after = blueprint
        .pointer("/observation/questions_after")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let deterministic_revisions = blueprint
        .pointer("/observation/deterministic_revision_operations")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let request_sha256 = blueprint
        .pointer("/lineage/request_ast/current_sha256")
        .and_then(Value::as_str)
        .unwrap_or("not_observed");
    let packet_bytes = child
        .and_then(|value| value.get("packet_bytes"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let inserted_bytes = child
        .and_then(|value| value.pointer("/patch/inserted_bytes"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    json!({
        "schema": "rune.blueprint-publication.observation.v1",
        "source": {
            "component": "pantheon",
            "capability": CAPABILITY,
            "machine_source_sha256": PANTHEONRUNEBLUEPRINTPUBLICATION_LINEAGE.source_sha256,
            "machine_typed_ir_sha256": PANTHEONRUNEBLUEPRINTPUBLICATION_LINEAGE.typed_ir_sha256,
            "request_sha256": request_sha256,
        },
        "scope": "one_composed_invocation",
        "sampling": {
            "status": "complete",
            "population": "this_invocation",
        },
        "disclosure": "content_free",
        "duration_ms": duration_ms,
        "blueprint_duration_ms": blueprint_duration_ms,
        "downstream_duration_ms": duration_ms.saturating_sub(u128::from(blueprint_duration_ms)),
        "rune_owner_requests": owner_requests,
        "questions_before": questions_before,
        "questions_after": questions_after,
        "deterministic_revision_operations": deterministic_revisions,
        "publication_invocations": u8::from(child.is_some()),
        "refusals": u8::from(refusal_stage.is_some()),
        "refusal_stage": refusal_stage,
        "semantic_handoff_files_materialized": 0,
        "manual_handoff_files_avoided": u8::from(child.is_some()),
        "packet_bytes": packet_bytes,
        "inserted_bytes": inserted_bytes,
        "pre_code_findings": "not_observed",
        "rework_operations": "not_observed",
        "token_usage": "not_observed",
        "context_usage": "not_observed",
        "escaped_defects": "not_observed",
        "gaps": [
            "wall_clock_window",
            "pre_code_findings",
            "rework_operations",
            "token_usage",
            "context_usage",
            "escaped_defects",
            "oculus_owner_receipt",
        ],
        "oculus_published": false,
    })
}

fn state_name(state: PantheonRuneBlueprintPublicationState) -> &'static str {
    match state {
        PantheonRuneBlueprintPublicationState::BlueprintReceived => "blueprint_received",
        PantheonRuneBlueprintPublicationState::QuestionsReturned => "questions_returned",
        PantheonRuneBlueprintPublicationState::ProjectionQualified => "projection_qualified",
        PantheonRuneBlueprintPublicationState::PublicationEligible => "publication_eligible",
        PantheonRuneBlueprintPublicationState::DurablePublished => "durable_published",
        PantheonRuneBlueprintPublicationState::Refused => "refused",
    }
}

fn signal_name(signal: PantheonRuneBlueprintPublicationSignal) -> &'static str {
    match signal {
        PantheonRuneBlueprintPublicationSignal::AnswersBound => "AnswersBound",
        PantheonRuneBlueprintPublicationSignal::BlueprintPublished => "BlueprintPublished",
        PantheonRuneBlueprintPublicationSignal::ClarificationRequired => "ClarificationRequired",
        PantheonRuneBlueprintPublicationSignal::ProjectionAdmitted => "ProjectionAdmitted",
        PantheonRuneBlueprintPublicationSignal::PublicationRefused => "PublicationRefused",
        PantheonRuneBlueprintPublicationSignal::RiskAdmitted => "RiskAdmitted",
    }
}

fn required(args: &mut impl Iterator<Item = OsString>) -> Result<OsString, &'static str> {
    args.next().filter(|value| !value.is_empty()).ok_or(
        "rune-blueprint-publication requires MODE PUBLICATION_PROFILE BLUEPRINT OPERATION [APPROVAL] [--answers ANSWER_SET]",
    )
}

fn text(value: OsString) -> Result<String, &'static str> {
    value.into_string().map_err(|_| "argument must be UTF-8")
}
