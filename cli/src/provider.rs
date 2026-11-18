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
const PROVIDER_CATALOG_TIMEOUT: Duration = Duration::from_secs(3);
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
    pub(crate) contexts: Vec<ContextCatalogProjection>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct TaskBindingCatalogProjection {
    pub(crate) external_task_key: String,
    pub(crate) chronos_task_id: String,
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

pub(crate) fn invoke_provider_owner(
    executable: &Path,
    config: &Path,
    command: &str,
    request: Option<&[u8]>,
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
        let deadline = Instant::now() + PROVIDER_CATALOG_TIMEOUT;
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
        read_bounded(&stdout_path)
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
    if catalog
        .providers
        .iter()
        .any(|provider| !valid_provider_catalog_entry(provider))
    {
        return Err("owner returned malformed provider catalogue facts".into());
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
    Ok(())
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| "owner projection output unavailable".to_owned())?
        .take(MAX_PROVIDER_CATALOG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "owner projection output unreadable".to_owned())?;
    if bytes.len() as u64 > MAX_PROVIDER_CATALOG_BYTES {
        return Err("owner projection exceeded the 256 KiB bound".into());
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

fn valid_projection_text(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.chars().all(|character| !character.is_control())
}
