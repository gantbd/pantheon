//! Owner-backed, redacted provider configuration projections shared by every
//! Pantheon presentation mode.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const MAX_PROVIDER_CATALOG_BYTES: u64 = 256 * 1024;
const MAX_TASK_INVOCATION_BYTES: u64 = 8 * 1024 * 1024;
const PROVIDER_CATALOG_TIMEOUT: Duration = Duration::from_secs(3);
const PROVIDER_QUALIFICATION_TIMEOUT: Duration = Duration::from_secs(300);
static NEXT_PROVIDER_CATALOG: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ProviderCatalogProjection {
    pub(crate) version: u8,
    pub(crate) owner: String,
    pub(crate) document_digest: String,
    pub(crate) providers: Vec<ProviderCatalogEntryProjection>,
    #[serde(default)]
    pub(crate) task_bindings: Vec<TaskBindingCatalogProjection>,
    #[serde(default)]
    pub(crate) task_execution: Option<ModelTaskExecutionProjection>,
    #[serde(default)]
    pub(crate) contexts: Vec<ContextCatalogProjection>,
    #[serde(default)]
    pub(crate) latest_task_invocation: Option<ModelTaskInvocationSummaryProjection>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct TaskBindingCatalogProjection {
    pub(crate) external_task_key: String,
    pub(crate) chronos_task_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub(crate) struct ModelTaskBudgetProjection {
    pub(crate) max_elapsed_millis: u64,
    pub(crate) max_input_tokens: u64,
    pub(crate) max_output_tokens: u64,
    pub(crate) max_tool_calls: u32,
    pub(crate) max_context_bytes: usize,
    pub(crate) max_output_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ModelTaskExecutionProjection {
    pub(crate) principal: String,
    pub(crate) harness_profile_id: String,
    pub(crate) task_incarnation: u64,
    pub(crate) attempt_number: u32,
    #[serde(default)]
    pub(crate) context_references: Vec<String>,
    #[serde(default)]
    pub(crate) tool_capabilities: Vec<String>,
    pub(crate) budget: ModelTaskBudgetProjection,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ModelTaskInvocationSummaryProjection {
    pub(crate) version: u8,
    pub(crate) operation_id: String,
    pub(crate) document_digest: String,
    pub(crate) qualification_operation_id: String,
    pub(crate) chronos_task_id: String,
    pub(crate) task_key: String,
    pub(crate) task_incarnation: u64,
    pub(crate) attempt_number: u32,
    pub(crate) principal: String,
    pub(crate) provider_id: String,
    pub(crate) harness_profile_id: String,
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) context_references: Vec<String>,
    #[serde(default)]
    pub(crate) tool_capabilities: Vec<String>,
    pub(crate) budget: ModelTaskBudgetProjection,
    pub(crate) request_now: u64,
    pub(crate) host_observed_at_unix_ms: u64,
    pub(crate) disposition: String,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) output_bytes: usize,
    #[serde(default)]
    pub(crate) output_digest: Option<String>,
    #[serde(default)]
    pub(crate) evidence_digest: Option<String>,
    pub(crate) proposal_count: usize,
    #[serde(default)]
    pub(crate) known_deviations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ModelTaskInvocationReceiptProjection {
    pub(crate) version: u8,
    pub(crate) owner: String,
    pub(crate) operation_id: String,
    pub(crate) document_digest: String,
    pub(crate) qualification_operation_id: String,
    pub(crate) profile_document_digest: String,
    pub(crate) chronos_task_id: String,
    pub(crate) task_key: String,
    pub(crate) task_incarnation: u64,
    pub(crate) attempt_number: u32,
    pub(crate) principal: String,
    pub(crate) provider_id: String,
    pub(crate) harness_profile_id: String,
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) context_references: Vec<String>,
    #[serde(default)]
    pub(crate) tool_capabilities: Vec<String>,
    pub(crate) budget: ModelTaskBudgetProjection,
    pub(crate) request_now: u64,
    pub(crate) host_observed_at_unix_ms: u64,
    pub(crate) disposition: String,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) output_bytes: usize,
    #[serde(default)]
    pub(crate) output_digest: Option<String>,
    #[serde(default)]
    pub(crate) evidence_digest: Option<String>,
    pub(crate) proposal_count: usize,
    pub(crate) output: String,
    #[serde(default)]
    pub(crate) known_deviations: Vec<String>,
    pub(crate) replayed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ContextCatalogProjection {
    pub(crate) reference: String,
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) resolution: Option<String>,
    #[serde(default)]
    pub(crate) source_digest: Option<String>,
    #[serde(default)]
    pub(crate) bytes: Option<usize>,
    #[serde(default)]
    pub(crate) workspace_id: Option<u64>,
    #[serde(default)]
    pub(crate) document_ordinal: Option<u64>,
    #[serde(default)]
    pub(crate) expected_revision: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ProviderCatalogEntryProjection {
    pub(crate) provider_id: String,
    pub(crate) harness_profile_id: String,
    pub(crate) profile_document_digest: String,
    #[serde(default)]
    pub(crate) policy_digest: Option<String>,
    #[serde(default)]
    pub(crate) transport: Option<String>,
    #[serde(default)]
    pub(crate) isolation: Option<String>,
    #[serde(default)]
    pub(crate) executable_available: Option<bool>,
    #[serde(default)]
    pub(crate) argument_count: Option<usize>,
    #[serde(default)]
    pub(crate) inherited_environment_name_count: Option<usize>,
    #[serde(default)]
    pub(crate) authentication_method_configured: Option<bool>,
    #[serde(default)]
    pub(crate) wire: Option<String>,
    #[serde(default)]
    pub(crate) model: Option<String>,
    #[serde(default)]
    pub(crate) endpoint_scope: Option<String>,
    #[serde(default)]
    pub(crate) endpoint_port: Option<u16>,
    #[serde(default)]
    pub(crate) structured_output: Option<bool>,
    #[serde(default)]
    pub(crate) structured_tools: Option<bool>,
    #[serde(default)]
    pub(crate) sampling_temperature_milli: Option<u16>,
    #[serde(default)]
    pub(crate) bearer_token_configured: Option<bool>,
    #[serde(default)]
    pub(crate) qualification: Option<ProviderQualificationProjection>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ProviderQualificationProjection {
    pub(crate) version: u8,
    pub(crate) owner: String,
    pub(crate) operation_id: String,
    pub(crate) document_digest: String,
    pub(crate) profile_document_digest: String,
    pub(crate) provider_id: String,
    pub(crate) harness_profile_id: String,
    pub(crate) model: String,
    pub(crate) host_observed_at_unix_ms: u64,
    pub(crate) disposition: String,
    pub(crate) completion: bool,
    pub(crate) tool_continuation: String,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) output_bytes: usize,
    #[serde(default)]
    pub(crate) evidence_digest: Option<String>,
    #[serde(default)]
    pub(crate) known_deviations: Vec<String>,
    pub(crate) replayed: bool,
}

pub(crate) fn provider_catalog(
    executable: &Path,
    config: &Path,
) -> Result<ProviderCatalogProjection, String> {
    let bytes = invoke_provider_owner(executable, config, "--describe-providers", None)?;
    let projection: ProviderCatalogProjection = serde_json::from_slice(&bytes)
        .map_err(|_| "owner returned an invalid provider projection".to_owned())?;
    validate_provider_catalog(&projection)?;
    Ok(projection)
}

pub(crate) fn qualify_provider(
    executable: &Path,
    config: &Path,
    request: &[u8],
) -> Result<ProviderQualificationProjection, String> {
    let bytes = invoke_provider_owner_with_timeout(
        executable,
        config,
        "--qualify-provider",
        Some(request),
        PROVIDER_QUALIFICATION_TIMEOUT,
        MAX_PROVIDER_CATALOG_BYTES,
    )?;
    let projection: ProviderQualificationProjection = serde_json::from_slice(&bytes)
        .map_err(|_| "owner returned an invalid provider qualification receipt".to_owned())?;
    validate_provider_qualification(&projection, None)?;
    Ok(projection)
}

pub(crate) fn invoke_model_task(
    executable: &Path,
    config: &Path,
    request: &[u8],
) -> Result<ModelTaskInvocationReceiptProjection, String> {
    let bytes = invoke_provider_owner_with_timeout(
        executable,
        config,
        "--invoke-task",
        Some(request),
        PROVIDER_QUALIFICATION_TIMEOUT,
        MAX_TASK_INVOCATION_BYTES,
    )?;
    let projection: ModelTaskInvocationReceiptProjection = serde_json::from_slice(&bytes)
        .map_err(|_| "owner returned an invalid model task-invocation receipt".to_owned())?;
    validate_model_task_invocation_receipt(&projection)?;
    Ok(projection)
}

pub(crate) fn invoke_provider_owner(
    executable: &Path,
    config: &Path,
    command: &str,
    request: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    invoke_provider_owner_with_timeout(
        executable,
        config,
        command,
        request,
        PROVIDER_CATALOG_TIMEOUT,
        MAX_PROVIDER_CATALOG_BYTES,
    )
}

fn invoke_provider_owner_with_timeout(
    executable: &Path,
    config: &Path,
    command: &str,
    request: Option<&[u8]>,
    timeout: Duration,
    max_output_bytes: u64,
) -> Result<Vec<u8>, String> {
    let root = std::env::temp_dir().join(format!(
        "pantheon-provider-catalog-{}-{}",
        std::process::id(),
        NEXT_PROVIDER_CATALOG.fetch_add(1, Ordering::Relaxed)
    ));
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .map_err(|_| "private result directory unavailable".to_owned())?;
    let stdout_path = root.join("stdout");
    let stderr_path = root.join("stderr");
    let result = (|| {
        let stdout = File::create(&stdout_path)
            .map_err(|_| "owner projection output unavailable".to_owned())?;
        let stderr = File::create(&stderr_path)
            .map_err(|_| "owner refusal output unavailable".to_owned())?;
        let mut child = Command::new(executable)
            .arg(command)
            .arg(config)
            .stdin(if request.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|_| "selected owner executable could not start".to_owned())?;
        if let Some(request) = request {
            child
                .stdin
                .take()
                .ok_or_else(|| "owner request input unavailable".to_owned())?
                .write_all(request)
                .map_err(|_| "owner request input could not be written".to_owned())?;
        }
        let deadline = Instant::now() + timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("owner projection timed out and was terminated".to_owned());
                }
                Err(_) => return Err("owner projection status unavailable".to_owned()),
            }
        };
        if !status.success() {
            let refusal = read_bounded_text(&stderr_path)
                .unwrap_or_else(|_| "owner returned a non-success status".into());
            return Err(single_line(&refusal, 240));
        }
        read_bounded_limit(&stdout_path, max_output_bytes)
    })();
    let _ = fs::remove_dir_all(root);
    result
}

pub(crate) fn valid_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn valid_provider_catalog_entry(provider: &ProviderCatalogEntryProjection) -> bool {
    valid_id(&provider.provider_id)
        && valid_id(&provider.harness_profile_id)
        && valid_digest(&provider.profile_document_digest)
        && provider.policy_digest.as_deref().is_none_or(valid_digest)
        && provider
            .model
            .as_deref()
            .is_none_or(|value| valid_projection_text(value, 1_024))
        && provider
            .transport
            .as_deref()
            .is_none_or(|value| valid_projection_text(value, 64))
        && provider
            .isolation
            .as_deref()
            .is_none_or(|value| valid_projection_text(value, 64))
        && provider
            .wire
            .as_deref()
            .is_none_or(|value| valid_projection_text(value, 64))
        && provider
            .endpoint_scope
            .as_deref()
            .is_none_or(|value| valid_projection_text(value, 64))
        && provider.qualification.as_ref().is_none_or(|qualification| {
            validate_provider_qualification(qualification, Some(provider)).is_ok()
        })
}

pub(crate) fn validate_provider_qualification(
    qualification: &ProviderQualificationProjection,
    provider: Option<&ProviderCatalogEntryProjection>,
) -> Result<(), String> {
    let disposition_valid = matches!(
        qualification.disposition.as_str(),
        "qualified_completion" | "nonconformant" | "outcome_unknown" | "running"
    );
    if qualification.version != 1
        || qualification.owner != "daimon-local-model"
        || !valid_id(&qualification.operation_id)
        || !valid_digest(&qualification.document_digest)
        || !valid_digest(&qualification.profile_document_digest)
        || !valid_id(&qualification.provider_id)
        || !valid_id(&qualification.harness_profile_id)
        || !valid_projection_text(&qualification.model, 1_024)
        || qualification.host_observed_at_unix_ms == 0
        || !disposition_valid
        || qualification.tool_continuation != "not_exercised"
        || qualification
            .evidence_digest
            .as_deref()
            .is_some_and(|value| !valid_digest(value))
        || qualification.known_deviations.len() > 16
        || qualification
            .known_deviations
            .iter()
            .any(|value| !valid_projection_text(value, 256))
        || (qualification.completion != (qualification.disposition == "qualified_completion"))
        || (qualification.completion
            && (qualification.input_tokens == 0
                || qualification.output_tokens == 0
                || qualification.output_bytes == 0
                || qualification.evidence_digest.is_none()))
    {
        return Err("owner returned malformed provider qualification facts".into());
    }
    if let Some(provider) = provider
        && (qualification.provider_id != provider.provider_id
            || qualification.harness_profile_id != provider.harness_profile_id
            || qualification.profile_document_digest != provider.profile_document_digest
            || provider.model.as_deref() != Some(qualification.model.as_str()))
    {
        return Err("owner qualification does not bind the projected provider".into());
    }
    Ok(())
}

fn valid_model_task_budget(budget: ModelTaskBudgetProjection) -> bool {
    budget.max_elapsed_millis > 0
        && budget.max_input_tokens > 0
        && budget.max_output_tokens > 0
        && budget.max_context_bytes > 0
        && budget.max_output_bytes > 0
}

fn valid_model_task_disposition(disposition: &str) -> bool {
    matches!(
        disposition,
        "running"
            | "completed"
            | "awaiting_proposal_review"
            | "not_sent"
            | "refused"
            | "failed"
            | "cancelled"
            | "timed_out"
            | "outcome_unknown"
            | "nonconformant"
    )
}

fn valid_model_task_invocation_summary(invocation: &ModelTaskInvocationSummaryProjection) -> bool {
    invocation.version == 1
        && valid_id(&invocation.operation_id)
        && valid_digest(&invocation.document_digest)
        && valid_id(&invocation.qualification_operation_id)
        && valid_projection_text(&invocation.chronos_task_id, 256)
        && valid_id(&invocation.task_key)
        && invocation.task_incarnation > 0
        && invocation.attempt_number > 0
        && valid_id(&invocation.principal)
        && valid_id(&invocation.provider_id)
        && valid_id(&invocation.harness_profile_id)
        && valid_projection_text(&invocation.model, 1_024)
        && invocation
            .context_references
            .iter()
            .all(|reference| valid_id(reference))
        && invocation
            .tool_capabilities
            .iter()
            .all(|capability| valid_id(capability))
        && valid_model_task_budget(invocation.budget)
        && invocation.request_now > 0
        && invocation.host_observed_at_unix_ms > 0
        && valid_model_task_disposition(&invocation.disposition)
        && invocation.output_digest.as_deref().is_none_or(valid_digest)
        && invocation
            .evidence_digest
            .as_deref()
            .is_none_or(valid_digest)
        && invocation.known_deviations.len() <= 8
        && invocation
            .known_deviations
            .iter()
            .all(|deviation| valid_projection_text(deviation, 256))
        && (invocation.output_bytes == 0 || invocation.output_digest.is_some())
        && (invocation.disposition != "completed"
            || (invocation.input_tokens > 0
                && invocation.output_tokens > 0
                && invocation.output_bytes > 0
                && invocation.output_digest.is_some()
                && invocation.evidence_digest.is_some()))
        && (invocation.disposition != "awaiting_proposal_review"
            || (invocation.input_tokens > 0
                && invocation.proposal_count > 0
                && invocation.evidence_digest.is_some()))
}

pub(crate) fn validate_model_task_invocation_receipt(
    receipt: &ModelTaskInvocationReceiptProjection,
) -> Result<(), String> {
    let summary = ModelTaskInvocationSummaryProjection {
        version: receipt.version,
        operation_id: receipt.operation_id.clone(),
        document_digest: receipt.document_digest.clone(),
        qualification_operation_id: receipt.qualification_operation_id.clone(),
        chronos_task_id: receipt.chronos_task_id.clone(),
        task_key: receipt.task_key.clone(),
        task_incarnation: receipt.task_incarnation,
        attempt_number: receipt.attempt_number,
        principal: receipt.principal.clone(),
        provider_id: receipt.provider_id.clone(),
        harness_profile_id: receipt.harness_profile_id.clone(),
        model: receipt.model.clone(),
        context_references: receipt.context_references.clone(),
        tool_capabilities: receipt.tool_capabilities.clone(),
        budget: receipt.budget,
        request_now: receipt.request_now,
        host_observed_at_unix_ms: receipt.host_observed_at_unix_ms,
        disposition: receipt.disposition.clone(),
        input_tokens: receipt.input_tokens,
        output_tokens: receipt.output_tokens,
        output_bytes: receipt.output_bytes,
        output_digest: receipt.output_digest.clone(),
        evidence_digest: receipt.evidence_digest.clone(),
        proposal_count: receipt.proposal_count,
        known_deviations: receipt.known_deviations.clone(),
    };
    if receipt.owner != "daimon-local-model"
        || !valid_digest(&receipt.profile_document_digest)
        || !valid_model_task_invocation_summary(&summary)
        || (!receipt.output.is_empty() && receipt.output.len() != receipt.output_bytes)
        || (receipt.disposition == "completed" && receipt.output.len() != receipt.output_bytes)
        || (!receipt.output.is_empty()
            && !receipt
                .output
                .chars()
                .all(|character| character == '\n' || character == '\t' || !character.is_control()))
    {
        return Err("owner returned malformed model task-invocation facts".into());
    }
    Ok(())
}

pub(crate) fn validate_provider_catalog(catalog: &ProviderCatalogProjection) -> Result<(), String> {
    if catalog.version != 1
        || !matches!(
            catalog.owner.as_str(),
            "daimon-local-agent" | "daimon-local-model"
        )
        || !valid_digest(&catalog.document_digest)
        || catalog.providers.is_empty()
        || catalog.providers.len() > 64
        || catalog.task_bindings.is_empty()
        || catalog.task_bindings.len() > 1_024
        || catalog.contexts.len() > 1_024
    {
        return Err("owner returned unsupported provider catalogue facts".into());
    }
    if catalog.providers.iter().any(|provider| {
        !valid_provider_catalog_entry(provider)
            || provider
                .qualification
                .as_ref()
                .is_some_and(|qualification| {
                    qualification.document_digest != catalog.document_digest
                })
    }) {
        return Err("owner returned malformed provider catalogue facts".into());
    }
    if let Some(execution) = &catalog.task_execution
        && (catalog.owner != "daimon-local-model"
            || !valid_id(&execution.principal)
            || !valid_id(&execution.harness_profile_id)
            || execution.task_incarnation == 0
            || execution.attempt_number == 0
            || !valid_model_task_budget(execution.budget)
            || execution
                .context_references
                .iter()
                .any(|reference| !valid_id(reference))
            || execution
                .tool_capabilities
                .iter()
                .any(|capability| !valid_id(capability))
            || !catalog
                .providers
                .iter()
                .any(|provider| provider.harness_profile_id == execution.harness_profile_id))
    {
        return Err("owner returned malformed model task-execution facts".into());
    }
    if let Some(invocation) = &catalog.latest_task_invocation {
        let Some(execution) = &catalog.task_execution else {
            return Err("owner returned a task invocation without an execution policy".into());
        };
        if !valid_model_task_invocation_summary(invocation)
            || invocation.document_digest != catalog.document_digest
            || invocation.principal != execution.principal
            || invocation.harness_profile_id != execution.harness_profile_id
            || invocation.task_incarnation != execution.task_incarnation
            || invocation.attempt_number != execution.attempt_number
            || invocation.context_references != execution.context_references
            || invocation.tool_capabilities != execution.tool_capabilities
            || invocation.budget.max_elapsed_millis != execution.budget.max_elapsed_millis
            || invocation.budget.max_input_tokens != execution.budget.max_input_tokens
            || invocation.budget.max_output_tokens != execution.budget.max_output_tokens
            || invocation.budget.max_tool_calls != execution.budget.max_tool_calls
            || invocation.budget.max_context_bytes != execution.budget.max_context_bytes
            || invocation.budget.max_output_bytes != execution.budget.max_output_bytes
            || !catalog.providers.iter().any(|provider| {
                provider.provider_id == invocation.provider_id
                    && provider.harness_profile_id == invocation.harness_profile_id
                    && provider.model.as_deref() == Some(invocation.model.as_str())
                    && provider
                        .qualification
                        .as_ref()
                        .is_some_and(|qualification| {
                            qualification.operation_id == invocation.qualification_operation_id
                                && qualification.completion
                        })
            })
        {
            return Err(
                "owner returned a task invocation outside the current provider facts".into(),
            );
        }
    }
    let mut task_keys = std::collections::BTreeSet::new();
    for binding in &catalog.task_bindings {
        if !valid_id(&binding.external_task_key)
            || !valid_projection_text(&binding.chronos_task_id, 256)
            || !task_keys.insert(&binding.external_task_key)
        {
            return Err("owner returned malformed task-binding catalogue facts".into());
        }
    }
    let mut context_references = std::collections::BTreeSet::new();
    for context in &catalog.contexts {
        let static_context = matches!(context.kind.as_str(), "agora_message" | "chronos_work");
        let runtime_vellum = context.kind == "vellum_document"
            && context.resolution.as_deref() == Some("runtime_owner");
        if !valid_id(&context.reference)
            || !matches!(
                context.kind.as_str(),
                "agora_message" | "chronos_work" | "vellum_document" | "dendrite_grounding"
            )
            || !context_references.insert(&context.reference)
            || (static_context
                && (context
                    .source_digest
                    .as_deref()
                    .is_none_or(|value| !valid_digest(value))
                    || context.bytes.is_none_or(|bytes| bytes == 0)
                    || context.workspace_id.is_some()
                    || context.document_ordinal.is_some()
                    || context.expected_revision.is_some()))
            || (runtime_vellum
                && (context.source_digest.is_some()
                    || context.bytes.is_some()
                    || context.workspace_id.is_none_or(|value| value == 0)
                    || context.document_ordinal.is_none_or(|value| value == 0)
                    || context.expected_revision.is_none()))
            || (!static_context
                && !runtime_vellum
                && (context.kind != "dendrite_grounding"
                    || context.source_digest.is_some()
                    || context.bytes.is_some()
                    || context.workspace_id.is_some()
                    || context.document_ordinal.is_some()
                    || context.expected_revision.is_some()))
        {
            return Err("owner returned malformed context catalogue facts".into());
        }
    }
    if let Some(execution) = &catalog.task_execution {
        let unique_contexts = execution
            .context_references
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        let unique_tools = execution
            .tool_capabilities
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        if unique_contexts.len() != execution.context_references.len()
            || unique_tools.len() != execution.tool_capabilities.len()
            || execution
                .context_references
                .iter()
                .any(|reference| !context_references.contains(reference))
        {
            return Err("owner task-execution references are not in its current catalogue".into());
        }
    }
    Ok(())
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    read_bounded_limit(path, MAX_PROVIDER_CATALOG_BYTES)
}

fn read_bounded_limit(path: &Path, maximum: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| "owner projection output unavailable".to_owned())?
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "owner projection output unreadable".to_owned())?;
    if bytes.len() as u64 > maximum {
        return Err(format!(
            "owner projection exceeded the {} byte bound",
            maximum
        ));
    }
    Ok(bytes)
}

fn read_bounded_text(path: &Path) -> Result<String, String> {
    String::from_utf8(read_bounded(path)?)
        .map_err(|_| "owner refusal was not valid UTF-8".to_owned())
}

fn single_line(value: &str, maximum: usize) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(maximum)
        .collect()
}

pub(crate) fn valid_projection_text(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.chars().all(|character| !character.is_control())
}
