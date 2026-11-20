//! Deterministic bridge from one exact task statement to Rune's owner-owned
//! authoring boundary.
//!
//! This component retains prose as provenance, never infers semantics from it,
//! and accepts revisions only as answers bound to exact Rune-owner questions.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

const MAX_BLUEPRINT_BYTES: u64 = 512 * 1024;
const MAX_ANSWERS_BYTES: u64 = 512 * 1024;
const MAX_TASK_PROSE_BYTES: usize = 64 * 1024;
const MAX_TASK_ID_BYTES: usize = 256;
const MAX_ANSWERS: usize = 256;
const CAPABILITY: &str = "rune.blueprint.v1";
const BOUNDARY: &str = "exact task retention and question-bound deterministic request revision only; no prose inference, semantic invention, human-review substitution, tool selection, workspace mutation, publication, task effect, go-no-go, or rollout authority";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BlueprintV1 {
    version: u8,
    task: TaskV1,
    #[serde(default)]
    request: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TaskV1 {
    id: String,
    prose: String,
    risk: RiskV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum RiskV1 {
    Low,
    Moderate,
    High,
    Gray,
}

impl RiskV1 {
    fn name(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Moderate => "moderate",
            Self::High => "high",
            Self::Gray => "gray",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AnswerSetV1 {
    version: u8,
    base_request_sha256: String,
    answers: Vec<AnswerV1>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AnswerV1 {
    path: String,
    category: String,
    question: String,
    value: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Question {
    path: String,
    category: String,
    question: String,
}

#[derive(Debug, Eq, PartialEq)]
enum Segment {
    Key(String),
    Index(usize),
}

pub fn run(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let profile = PathBuf::from(required(
        &mut args,
        "rune-blueprint requires RUNE_PROFILE_JSON BLUEPRINT_JSON [ANSWER_SET_JSON]",
    )?);
    let blueprint = PathBuf::from(required(
        &mut args,
        "rune-blueprint requires RUNE_PROFILE_JSON BLUEPRINT_JSON [ANSWER_SET_JSON]",
    )?);
    let answers = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return Err("unexpected rune-blueprint argument".into());
    }

    match execute(&profile, &blueprint, answers.as_deref()) {
        Ok(value) => {
            println!("{}", serde_json::to_string_pretty(&value)?);
            Ok(())
        }
        Err(value) => {
            println!("{}", serde_json::to_string_pretty(&value)?);
            Err(std::io::Error::other("Rune blueprint refused").into())
        }
    }
}

fn execute(
    profile: &Path,
    blueprint_path: &Path,
    answers_path: Option<&Path>,
) -> Result<Value, Value> {
    let started = Instant::now();
    let blueprint_bytes = read_bounded(blueprint_path, MAX_BLUEPRINT_BYTES, "blueprint")?;
    let mut blueprint: BlueprintV1 = serde_json::from_slice(&blueprint_bytes)
        .map_err(|error| refusal("blueprint", format!("invalid closed blueprint: {error}")))?;
    validate_blueprint(&blueprint)?;
    normalize_request(&mut blueprint.request)?;

    let base_request = serde_json::to_vec(&blueprint.request)
        .map_err(|error| refusal("request", format!("serialize Rune request: {error}")))?;
    let base_request_sha256 = sha256(&base_request);
    let initial = crate::rune_author::execute_private_bytes(profile, &base_request)?;
    let current_questions = questions(&initial)?;

    let (owner, revised_request_sha256, answer_count, revised_blueprint) = if let Some(
        answers_path,
    ) = answers_path
    {
        if current_questions.is_empty() {
            return Err(refusal(
                "answers",
                "Rune owner returned no questions; answers cannot revise an already terminal request",
            ));
        }
        let answer_bytes = read_bounded(answers_path, MAX_ANSWERS_BYTES, "answers")?;
        let answers: AnswerSetV1 = serde_json::from_slice(&answer_bytes)
            .map_err(|error| refusal("answers", format!("invalid closed answer set: {error}")))?;
        validate_answers(&answers, &base_request_sha256, &current_questions)?;
        for answer in &answers.answers {
            set_path(&mut blueprint.request, &answer.path, answer.value.clone())?;
            remove_resolved_hole(&mut blueprint.request, answer);
        }
        let revised_request = serde_json::to_vec(&blueprint.request).map_err(|error| {
            refusal(
                "request",
                format!("serialize revised Rune request: {error}"),
            )
        })?;
        let revised_request_sha256 = sha256(&revised_request);
        let owner = crate::rune_author::execute_private_bytes(profile, &revised_request)?;
        (
            owner,
            revised_request_sha256,
            answers.answers.len(),
            Some(serde_json::to_value(&blueprint).map_err(|error| {
                refusal("blueprint", format!("serialize revised blueprint: {error}"))
            })?),
        )
    } else {
        (initial, base_request_sha256.clone(), 0, None)
    };

    let after = questions(&owner)?;
    let status = owner
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| refusal("owner", "Rune adapter response omits status"))?;
    let complete = owner
        .get("complete")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let answer_template = (!after.is_empty()).then(|| {
        json!({
            "version": 1,
            "base_request_sha256": revised_request_sha256,
            "answers": after.iter().map(|question| json!({
                "path": question.path,
                "category": question.category,
                "question": question.question,
                "value": Value::Null,
            })).collect::<Vec<_>>(),
        })
    });
    Ok(json!({
        "version": 1,
        "peer": "rune",
        "capability": CAPABILITY,
        "status": status,
        "complete": complete,
        "task": {
            "id": blueprint.task.id,
            "prose_sha256": sha256(blueprint.task.prose.as_bytes()),
            "prose_bytes": blueprint.task.prose.len(),
            "risk": blueprint.task.risk.name(),
        },
        "lineage": {
            "prose_lst": {
                "sha256": sha256(blueprint.task.prose.as_bytes()),
                "bytes": blueprint.task.prose.len(),
                "semantic_status": "retained_not_inferred",
            },
            "blueprint_lst": {
                "sha256": sha256(&blueprint_bytes),
                "bytes": blueprint_bytes.len(),
            },
            "request_ast": {
                "base_sha256": base_request_sha256,
                "current_sha256": revised_request_sha256,
            },
        },
        "questions": after.iter().map(question_json).collect::<Vec<_>>(),
        "answer_template": answer_template,
        "revised_blueprint": revised_blueprint,
        "owner_outcome": owner,
        "observation": {
            "schema": "rune.blueprint.observation.v1",
            "request_count": if answers_path.is_some() { 2 } else { 1 },
            "questions_before": current_questions.len(),
            "questions_after": after.len(),
            "answers_applied": answer_count,
            "deterministic_revision_operations": answer_count,
            "duration_ms": started.elapsed().as_millis(),
            "token_usage": "not_observed",
            "escaped_defects": "not_observed",
            "oculus_published": false,
        },
        "next_action": if complete {
            "inspect the exact private projection packet, then separately authorize bounded Vellum publication"
        } else if status == "questions_pending" {
            "answer only exact returned questions; preserve dissent and submit a new answer-bound revision"
        } else {
            "follow the Rune-owned refusal without deleting or weakening an obligation"
        },
        "boundary": BOUNDARY,
    }))
}

fn validate_blueprint(value: &BlueprintV1) -> Result<(), Value> {
    if value.version != 1 {
        return Err(refusal("blueprint", "blueprint version must be 1"));
    }
    if value.task.id.is_empty()
        || value.task.id.len() > MAX_TASK_ID_BYTES
        || !value
            .task
            .id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(refusal(
            "blueprint",
            "task id must be one lowercase hyphenated atom within 256 bytes",
        ));
    }
    if value.task.prose.trim().is_empty() || value.task.prose.len() > MAX_TASK_PROSE_BYTES {
        return Err(refusal(
            "blueprint",
            "task prose must be nonempty UTF-8 within 64 KiB",
        ));
    }
    Ok(())
}

fn normalize_request(value: &mut Value) -> Result<(), Value> {
    if value.is_null() {
        *value = Value::Object(Map::new());
    }
    let object = value
        .as_object_mut()
        .ok_or_else(|| refusal("request", "request must be an object or null"))?;
    for key in object.keys() {
        if ![
            "version",
            "dialect",
            "review",
            "holes",
            "machine",
            "projection",
        ]
        .contains(&key.as_str())
        {
            return Err(refusal(
                "request",
                format!("request contains unknown field {key:?}"),
            ));
        }
    }
    fixed(object, "version", json!(1))?;
    fixed(object, "dialect", json!({"kind": "machine", "version": 1}))?;
    fixed(object, "projection", json!("rust"))?;
    Ok(())
}

fn fixed(object: &mut Map<String, Value>, key: &str, expected: Value) -> Result<(), Value> {
    match object.get(key) {
        Some(actual) if actual != &expected => Err(refusal(
            "request",
            format!("fixed field {key:?} conflicts with Rune machine v1 Rust framing"),
        )),
        Some(_) => Ok(()),
        None => {
            object.insert(key.to_owned(), expected);
            Ok(())
        }
    }
}

fn questions(owner: &Value) -> Result<Vec<Question>, Value> {
    if owner.get("status").and_then(Value::as_str) != Some("questions_pending") {
        return Ok(Vec::new());
    }
    let values = owner
        .pointer("/owner_outcome/questions")
        .and_then(Value::as_array)
        .ok_or_else(|| refusal("owner", "questions_pending omits owner questions"))?;
    values
        .iter()
        .map(|value| {
            let object = value
                .as_object()
                .ok_or_else(|| refusal("owner", "owner question must be an object"))?;
            Ok(Question {
                path: text(object, "path")?.to_owned(),
                category: text(object, "category")?.to_owned(),
                question: text(object, "question")?.to_owned(),
            })
        })
        .collect()
}

fn validate_answers(
    value: &AnswerSetV1,
    base_request_sha256: &str,
    questions: &[Question],
) -> Result<(), Value> {
    if value.version != 1 {
        return Err(refusal("answers", "answer-set version must be 1"));
    }
    if value.base_request_sha256 != base_request_sha256 {
        return Err(refusal(
            "answers",
            "answer set does not bind the exact current Rune request",
        ));
    }
    if value.answers.is_empty() || value.answers.len() > MAX_ANSWERS {
        return Err(refusal(
            "answers",
            "answer set must contain 1 through 256 exact answers",
        ));
    }
    let mut seen = BTreeSet::new();
    for answer in &value.answers {
        let exact = Question {
            path: answer.path.clone(),
            category: answer.category.clone(),
            question: answer.question.clone(),
        };
        if !questions.contains(&exact) {
            return Err(refusal(
                "answers",
                format!(
                    "answer does not match current owner question at {:?}",
                    answer.path
                ),
            ));
        }
        if !seen.insert(answer.path.as_str()) {
            return Err(refusal(
                "answers",
                format!("duplicate answer path {:?}", answer.path),
            ));
        }
    }
    Ok(())
}

fn set_path(root: &mut Value, path: &str, value: Value) -> Result<(), Value> {
    let segments = parse_path(path)?;
    let (last, parents) = segments
        .split_last()
        .ok_or_else(|| refusal("answers", "root replacement is not admitted"))?;
    let mut current = root;
    for segment in parents {
        current = match segment {
            Segment::Key(key) => current
                .as_object_mut()
                .and_then(|object| object.get_mut(key))
                .ok_or_else(|| refusal("answers", format!("missing parent for {path:?}")))?,
            Segment::Index(index) => current
                .as_array_mut()
                .and_then(|array| array.get_mut(*index))
                .ok_or_else(|| refusal("answers", format!("missing parent for {path:?}")))?,
        };
    }
    match last {
        Segment::Key(key) => {
            current
                .as_object_mut()
                .ok_or_else(|| refusal("answers", format!("non-object parent for {path:?}")))?
                .insert(key.clone(), value);
        }
        Segment::Index(index) => {
            let slot = current
                .as_array_mut()
                .and_then(|array| array.get_mut(*index))
                .ok_or_else(|| refusal("answers", format!("missing array slot for {path:?}")))?;
            *slot = value;
        }
    }
    Ok(())
}

fn parse_path(path: &str) -> Result<Vec<Segment>, Value> {
    let bytes = path.as_bytes();
    if bytes.first() != Some(&b'$') {
        return Err(refusal("answers", "question path must begin with $"));
    }
    let mut index = 1;
    let mut segments = Vec::new();
    while index < bytes.len() {
        match bytes[index] {
            b'.' => {
                index += 1;
                let start = index;
                while index < bytes.len() && bytes[index] != b'.' && bytes[index] != b'[' {
                    index += 1;
                }
                if start == index {
                    return Err(refusal("answers", "question path contains an empty key"));
                }
                segments.push(Segment::Key(path[start..index].to_owned()));
            }
            b'[' => {
                index += 1;
                let start = index;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
                if start == index || bytes.get(index) != Some(&b']') {
                    return Err(refusal(
                        "answers",
                        "question path contains an invalid index",
                    ));
                }
                let number = path[start..index]
                    .parse::<usize>()
                    .map_err(|_| refusal("answers", "question path index is too large"))?;
                index += 1;
                segments.push(Segment::Index(number));
            }
            _ => return Err(refusal("answers", "question path uses unsupported syntax")),
        }
    }
    Ok(segments)
}

fn remove_resolved_hole(request: &mut Value, answer: &AnswerV1) {
    let Some(holes) = request.get_mut("holes").and_then(Value::as_array_mut) else {
        return;
    };
    holes.retain(|hole| {
        hole.get("path").and_then(Value::as_str) != Some(answer.path.as_str())
            || hole.get("category").and_then(Value::as_str) != Some(answer.category.as_str())
            || hole.get("question").and_then(Value::as_str) != Some(answer.question.as_str())
    });
}

fn question_json(value: &Question) -> Value {
    json!({
        "path": value.path,
        "category": value.category,
        "question": value.question,
    })
}

fn text<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a str, Value> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| refusal("owner", format!("owner question omits {field}")))
}

fn read_bounded(path: &Path, maximum: u64, label: &str) -> Result<Vec<u8>, Value> {
    let metadata =
        fs::metadata(path).map_err(|error| refusal(label, format!("inspect {label}: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > maximum {
        return Err(refusal(
            label,
            format!("{label} must be one nonempty regular file within {maximum} bytes"),
        ));
    }
    fs::read(path).map_err(|error| refusal(label, format!("read {label}: {error}")))
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn refusal(stage: &str, detail: impl Into<String>) -> Value {
    json!({
        "version": 1,
        "peer": "pantheon",
        "capability": CAPABILITY,
        "status": "refused",
        "complete": false,
        "stage": stage,
        "detail": detail.into(),
        "boundary": BOUNDARY,
    })
}

fn required(
    args: &mut impl Iterator<Item = OsString>,
    message: &'static str,
) -> Result<OsString, Box<dyn std::error::Error>> {
    args.next().ok_or_else(|| message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_request_framing_is_injected_without_prose_inference() {
        let mut request = Value::Null;
        normalize_request(&mut request).unwrap();
        assert_eq!(request["version"], 1);
        assert_eq!(request["dialect"]["kind"], "machine");
        assert_eq!(request["projection"], "rust");
        assert!(request.get("machine").is_none());
    }

    #[test]
    fn exact_question_path_is_revised() {
        let mut request = json!({"machine": {"name": "one"}});
        set_path(&mut request, "$.machine.name", json!("two")).unwrap();
        assert_eq!(request["machine"]["name"], "two");
    }

    #[test]
    fn missing_parent_is_refused_instead_of_invented() {
        let mut request = json!({});
        let error = set_path(&mut request, "$.machine.name", json!("two")).unwrap_err();
        assert_eq!(error["status"], "refused");
    }

    #[test]
    fn resolved_explicit_hole_is_removed_only_on_exact_tuple_match() {
        let mut request = json!({"holes": [{
            "path": "$.machine.name",
            "category": "identity",
            "question": "What is the name?"
        }]});
        let answer = AnswerV1 {
            path: "$.machine.name".into(),
            category: "identity".into(),
            question: "What is the name?".into(),
            value: json!("one"),
        };
        remove_resolved_hole(&mut request, &answer);
        assert_eq!(request["holes"], json!([]));
    }

    #[test]
    fn parser_accepts_owner_array_paths() {
        assert_eq!(
            parse_path("$.machine.states[2].rosetta.intent").unwrap(),
            vec![
                Segment::Key("machine".into()),
                Segment::Key("states".into()),
                Segment::Index(2),
                Segment::Key("rosetta".into()),
                Segment::Key("intent".into()),
            ]
        );
    }
}
