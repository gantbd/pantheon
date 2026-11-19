//! Read-only composition of Hestia runtime evidence with a selected Daimon
//! provider. This module never infers a binding from ports or process state.

use hestia_local_client::LocalClient;
use hestia_protocol::{
    ArtifactFailureV1, ArtifactFormatV1, ArtifactLifecycleV1, ArtifactObservationV1,
    ControlPostureV1, EndpointScopeV1, ReadinessBasisV1, ReadinessDispositionV1, RuntimeIdV1,
    RuntimeLifecycleV1, RuntimeObservationV1,
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::profile::HestiaProfile;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RuntimeEvidenceProjection {
    pub owner: &'static str,
    pub provider_id: String,
    pub runtime_id: String,
    pub authority_id: String,
    pub revision: u64,
    pub incarnation: u64,
    pub lifecycle: &'static str,
    pub readiness: &'static str,
    pub readiness_basis: Option<&'static str>,
    pub observed_at_unix_ms: Option<u64>,
    pub endpoint_reference: Option<String>,
    pub endpoint_scope: Option<&'static str>,
    pub hosting_profile: String,
    pub model_identity: String,
    pub model_matches_provider: bool,
    pub context_tokens: u64,
    pub memory_bytes: u64,
    pub accelerator: String,
    pub threads: u16,
    pub concurrency: u16,
    pub sandbox: &'static str,
    pub egress: &'static str,
    pub fenced: bool,
    pub invocation_eligible: bool,
    pub boundary: &'static str,
}

pub fn observe_provider_runtime(
    profile: &HestiaProfile,
    provider_id: &str,
    expected_model: &str,
) -> Result<RuntimeEvidenceProjection, String> {
    let mut matches = profile
        .runtime_bindings
        .iter()
        .filter(|binding| binding.provider_id == provider_id);
    let binding = matches.next().ok_or_else(|| {
        "no explicit Hestia runtime binding exists for the selected provider".to_owned()
    })?;
    if matches.next().is_some() {
        return Err("the selected provider has ambiguous Hestia runtime bindings".into());
    }
    let runtime_bytes = decode_fixed_hex::<16>(&binding.runtime_id)
        .ok_or_else(|| "the selected Hestia runtime identity is invalid".to_owned())?;
    let observation = LocalClient::new(&profile.socket)
        .observe(RuntimeIdV1(runtime_bytes))
        .map_err(|error| format!("Hestia runtime observation failed: {error:?}"))?
        .ok_or_else(|| "Hestia does not own the explicitly bound runtime".to_owned())?;
    let model_matches_provider = observation.model.identity.as_str() == expected_model;
    let invocation_eligible = model_matches_provider
        && observation.lifecycle == RuntimeLifecycleV1::Ready
        && observation.readiness.disposition == ReadinessDispositionV1::Ready
        && !observation.fenced;
    Ok(RuntimeEvidenceProjection {
        owner: "hestia",
        provider_id: provider_id.to_owned(),
        runtime_id: hex(&observation.runtime.0),
        authority_id: hex(&observation.authority.0),
        revision: observation.revision,
        incarnation: observation.incarnation,
        lifecycle: lifecycle(observation.lifecycle),
        readiness: readiness(observation.readiness.disposition),
        readiness_basis: observation.readiness.basis.map(readiness_basis),
        observed_at_unix_ms: observation.readiness.observed_at_unix_ms,
        endpoint_reference: observation
            .readiness
            .endpoint
            .as_ref()
            .map(|endpoint| endpoint.reference.as_str().to_owned()),
        endpoint_scope: observation
            .readiness
            .endpoint
            .as_ref()
            .map(|endpoint| endpoint_scope(endpoint.scope)),
        hosting_profile: observation.hosting_profile.as_str().to_owned(),
        model_identity: observation.model.identity.as_str().to_owned(),
        model_matches_provider,
        context_tokens: observation.model.context_tokens,
        memory_bytes: observation.resources.memory_bytes,
        accelerator: observation.resources.accelerator.as_str().to_owned(),
        threads: observation.resources.threads,
        concurrency: observation.resources.concurrency,
        sandbox: posture(observation.sandbox),
        egress: posture(observation.egress),
        fenced: observation.fenced,
        invocation_eligible,
        boundary: "Hestia lifecycle/readiness evidence only; Daimon owns provider qualification and execution",
    })
}

pub fn runtime_observation_projection(observation: &RuntimeObservationV1) -> Value {
    json!({
        "authority": hex(&observation.authority.0),
        "runtime": hex(&observation.runtime.0),
        "revision": observation.revision,
        "incarnation": observation.incarnation,
        "lifecycle": lifecycle(observation.lifecycle),
        "hosting_profile": observation.hosting_profile.as_str(),
        "model": model_projection(&observation.model),
        "resources": {
            "memory_bytes": observation.resources.memory_bytes,
            "accelerator": observation.resources.accelerator.as_str(),
            "threads": observation.resources.threads,
            "concurrency": observation.resources.concurrency,
        },
        "sandbox": posture(observation.sandbox),
        "egress": posture(observation.egress),
        "readiness": {
            "disposition": readiness(observation.readiness.disposition),
            "observed_at_unix_ms": observation.readiness.observed_at_unix_ms,
            "basis": observation.readiness.basis.map(readiness_basis),
            "endpoint": observation.readiness.endpoint.as_ref().map(|endpoint| json!({
                "reference": endpoint.reference.as_str(),
                "scope": endpoint_scope(endpoint.scope),
            })),
        },
        "fenced": observation.fenced,
    })
}

pub fn artifact_observation_projection(observation: &ArtifactObservationV1) -> Value {
    json!({
        "authority": hex(&observation.authority.0),
        "artifact": observation.entry.artifact.as_str(),
        "revision": observation.revision,
        "lifecycle": artifact_lifecycle(observation.lifecycle),
        "operation": observation.operation.map(|operation| hex(&operation.0)),
        "observed_bytes": observation.observed_bytes,
        "failure": observation.failure.map(artifact_failure),
        "entry": {
            "model": model_projection(&observation.entry.model),
            "registry": observation.entry.registry.as_str(),
            "source_revision": observation.entry.source_revision.as_str(),
            "license": observation.entry.license.as_str(),
            "format": artifact_format(observation.entry.format),
            "expected_size_bytes": observation.entry.expected_size_bytes,
            "expected_digest": format!("sha256:{}", hex(&observation.entry.expected_digest.0)),
        },
    })
}

fn model_projection(model: &hestia_protocol::ModelArtifactV1) -> Value {
    json!({
        "identity": model.identity.as_str(),
        "digest": model.digest.map(|digest| format!("sha256:{}", hex(&digest.0))),
        "quantization": model.quantization.as_ref().map(|value| value.as_str()),
        "context_tokens": model.context_tokens,
    })
}

fn decode_fixed_hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    if value.len() != N * 2 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut result = [0; N];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        result[index] = u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
    }
    Some(result)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

const fn lifecycle(value: RuntimeLifecycleV1) -> &'static str {
    match value {
        RuntimeLifecycleV1::Starting => "starting",
        RuntimeLifecycleV1::Loading => "loading",
        RuntimeLifecycleV1::Ready => "ready",
        RuntimeLifecycleV1::Degraded => "degraded",
        RuntimeLifecycleV1::Draining => "draining",
        RuntimeLifecycleV1::Stopped => "stopped",
        RuntimeLifecycleV1::Crashed => "crashed",
        RuntimeLifecycleV1::Orphaned => "orphaned",
        RuntimeLifecycleV1::OutcomeUnknown => "outcome_unknown",
    }
}

const fn readiness(value: ReadinessDispositionV1) -> &'static str {
    match value {
        ReadinessDispositionV1::Unknown => "unknown",
        ReadinessDispositionV1::Ready => "ready",
        ReadinessDispositionV1::Degraded => "degraded",
        ReadinessDispositionV1::Unavailable => "unavailable",
    }
}

const fn readiness_basis(value: ReadinessBasisV1) -> &'static str {
    match value {
        ReadinessBasisV1::TcpAccept => "tcp_accept",
    }
}

const fn endpoint_scope(value: EndpointScopeV1) -> &'static str {
    match value {
        EndpointScopeV1::Loopback => "loopback",
        EndpointScopeV1::PrivateNetwork => "private_network",
        EndpointScopeV1::PublicNetwork => "public_network",
    }
}

const fn posture(value: ControlPostureV1) -> &'static str {
    match value {
        ControlPostureV1::Inherited => "inherited",
        ControlPostureV1::Enforced => "enforced",
    }
}

const fn artifact_lifecycle(value: ArtifactLifecycleV1) -> &'static str {
    match value {
        ArtifactLifecycleV1::Cataloged => "cataloged",
        ArtifactLifecycleV1::Acquiring => "acquiring",
        ArtifactLifecycleV1::Verifying => "verifying",
        ArtifactLifecycleV1::Admitted => "admitted",
        ArtifactLifecycleV1::Failed => "failed",
        ArtifactLifecycleV1::OutcomeUnknown => "outcome_unknown",
    }
}

const fn artifact_failure(value: ArtifactFailureV1) -> &'static str {
    match value {
        ArtifactFailureV1::SourceUnavailable => "source_unavailable",
        ArtifactFailureV1::SizeMismatch => "size_mismatch",
        ArtifactFailureV1::DigestMismatch => "digest_mismatch",
        ArtifactFailureV1::StorageRefused => "storage_refused",
        ArtifactFailureV1::AdapterFailed => "adapter_failed",
    }
}

const fn artifact_format(value: ArtifactFormatV1) -> &'static str {
    match value {
        ArtifactFormatV1::Gguf => "gguf",
        ArtifactFormatV1::Mlx => "mlx",
        ArtifactFormatV1::Safetensors => "safetensors",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hestia_protocol::{
        AcceleratorV1, ArtifactCatalogEntryV1, ArtifactFormatV1, ArtifactIdV1, ArtifactLifecycleV1,
        HostAuthorityIdV1, LicenseIdV1, ModelArtifactV1, ModelDigestV1, ModelIdentityV1,
        QuantizationV1, RegistryIdV1, SourceRevisionV1,
    };

    #[test]
    fn fixed_identity_decode_is_closed() {
        assert_eq!(decode_fixed_hex::<2>("01aF"), Some([1, 175]));
        assert_eq!(decode_fixed_hex::<2>("01"), None);
        assert_eq!(decode_fixed_hex::<2>("01zz"), None);
    }

    #[test]
    fn artifact_projection_preserves_provenance_without_private_host_material() {
        let observation = ArtifactObservationV1 {
            authority: HostAuthorityIdV1([1; 16]),
            entry: ArtifactCatalogEntryV1 {
                artifact: ArtifactIdV1::new("devstral-q4").unwrap(),
                model: ModelArtifactV1 {
                    identity: ModelIdentityV1::new("devstral").unwrap(),
                    digest: Some(ModelDigestV1([2; 32])),
                    quantization: Some(QuantizationV1::new("Q4_K_M").unwrap()),
                    context_tokens: 32_768,
                },
                registry: RegistryIdV1::new("reviewed-registry").unwrap(),
                source_revision: SourceRevisionV1::new("revision-1").unwrap(),
                license: LicenseIdV1::new("Apache-2.0").unwrap(),
                format: ArtifactFormatV1::Gguf,
                expected_size_bytes: 42,
                expected_digest: ModelDigestV1([2; 32]),
            },
            revision: 3,
            lifecycle: ArtifactLifecycleV1::Admitted,
            operation: None,
            observed_bytes: 42,
            failure: None,
        };
        let projection = artifact_observation_projection(&observation);
        let encoded = serde_json::to_string(&projection).unwrap();
        assert_eq!(projection["artifact"], "devstral-q4");
        assert_eq!(projection["lifecycle"], "admitted");
        assert_eq!(projection["entry"]["expected_size_bytes"], 42);
        assert!(!encoded.contains("/private/model.gguf"));
        assert!(!encoded.contains("executable"));
        assert!(!encoded.contains("arguments"));
        assert!(!encoded.contains("environment"));
    }

    #[test]
    fn runtime_projection_keeps_readiness_separate_from_provider_qualification() {
        let observation = RuntimeObservationV1 {
            authority: HostAuthorityIdV1([1; 16]),
            runtime: RuntimeIdV1([3; 16]),
            revision: 4,
            incarnation: 2,
            lifecycle: RuntimeLifecycleV1::Ready,
            hosting_profile: hestia_protocol::HostingProfileIdV1::new("llama-cpp").unwrap(),
            model: ModelArtifactV1 {
                identity: ModelIdentityV1::new("devstral").unwrap(),
                digest: Some(ModelDigestV1([2; 32])),
                quantization: None,
                context_tokens: 32_768,
            },
            resources: hestia_protocol::ResourceExpectationsV1 {
                memory_bytes: 1024,
                accelerator: AcceleratorV1::new("metal").unwrap(),
                threads: 4,
                concurrency: 1,
            },
            sandbox: ControlPostureV1::Enforced,
            egress: ControlPostureV1::Inherited,
            readiness: hestia_protocol::RuntimeReadinessV1::unknown(),
            fenced: false,
        };
        let projection = runtime_observation_projection(&observation);
        assert_eq!(projection["lifecycle"], "ready");
        assert_eq!(projection["readiness"]["disposition"], "unknown");
        assert!(projection.get("provider_qualified").is_none());
        assert!(projection.get("invocation_eligible").is_none());
    }
}
