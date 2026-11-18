//! Read-only composition of Hestia runtime evidence with a selected Daimon
//! provider. This module never infers a binding from ports or process state.

use hestia_local_client::LocalClient;
use hestia_protocol::{
    ControlPostureV1, EndpointScopeV1, ReadinessBasisV1, ReadinessDispositionV1, RuntimeIdV1,
    RuntimeLifecycleV1,
};
use serde::Serialize;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_identity_decode_is_closed() {
        assert_eq!(decode_fixed_hex::<2>("01aF"), Some([1, 175]));
        assert_eq!(decode_fixed_hex::<2>("01"), None);
        assert_eq!(decode_fixed_hex::<2>("01zz"), None);
    }
}
