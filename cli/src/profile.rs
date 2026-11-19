use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use pantheon::{Shell, Surface};

const MAX_PROFILE_BYTES: u64 = 64 * 1024;
const MAX_WORKSTREAMS: usize = 16;
const MAX_BINDINGS: usize = 16;
const MAX_REQUIRED_CAPABILITIES: usize = 16;
const MAX_CAPABILITY_BYTES: usize = 64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceProfile {
    pub version: u8,
    pub name: String,
    pub surfaces: Vec<Surface>,
    #[serde(default)]
    pub keybindings: KeybindingProfile,
    pub agora: Option<AgoraProfile>,
    pub task_conversation: Option<TaskConversationProfile>,
    pub moira: Option<MoiraProfile>,
    pub daimon: Option<DaimonProfile>,
    #[serde(default)]
    pub hestia: Option<HestiaProfile>,
    pub hermes: Option<HermesProfile>,
    pub aegis: Option<AegisProfile>,
    pub tellus: Option<TellusProfile>,
    pub chronos: Option<ChronosProfile>,
    pub vellum: Option<VellumProfile>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KeybindingProfile {
    #[default]
    Pantheon,
    Emacs,
}

impl KeybindingProfile {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pantheon => "pantheon",
            Self::Emacs => "emacs",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TaskConversationProfile {
    pub socket: PathBuf,
    /// Attributed relationship creator; this is not an authority grant or a
    /// substitute for the interactive human gate.
    pub principal: String,
    /// Exact Agora participant identity that may be deliberately promoted from
    /// an addressed room message into a reviewed Daimon prompt. Omission keeps
    /// the cross-component command unavailable.
    pub agent_addressee: Option<String>,
    /// Bounded participant-to-session routing for shared Agora context. Each
    /// route selects one exact configured Daimon session; it grants neither
    /// room membership nor prompt authority.
    #[serde(default)]
    pub agent_routes: Vec<TaskConversationAgentRouteProfile>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TaskConversationAgentRouteProfile {
    pub addressee: String,
    pub session: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgoraProfile {
    #[serde(default)]
    pub deployment: DeploymentMode,
    #[serde(default)]
    pub transport: AgoraTransportProfile,
    pub socket: PathBuf,
    pub participant: String,
    pub host: Option<AgoraHostProfile>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgoraTransportProfile {
    #[default]
    Conversation,
    Composition,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgoraHostProfile {
    pub executable: PathBuf,
    pub config: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MoiraProfile {
    #[serde(default)]
    pub deployment: DeploymentMode,
    pub socket: Option<PathBuf>,
    /// Optional independently attached owner endpoint for one governed agent
    /// participation. The saga host does not implicitly serve this protocol.
    pub participation_socket: Option<PathBuf>,
    pub participation: Option<String>,
    /// Optional independently attached read-only methodical-deliberation
    /// endpoint. The saga host does not implicitly serve this protocol.
    pub deliberation_socket: Option<PathBuf>,
    pub deliberation: Option<String>,
    /// Trusted composition from one opaque Moira material route to one exact
    /// Vellum document. This grants read access only; it is not human-gate
    /// response authority.
    pub decision_packet_reader: Option<MoiraDecisionPacketReaderProfile>,
    /// Separately selected rationale artifact used to construct one exact
    /// human response. Moira still authorizes the command through Aegis.
    pub human_gate_response: Option<MoiraHumanGateResponseProfile>,
    /// Optional one-step owner worker. Pantheon treats the configuration as
    /// opaque and accepts only an exact participation-bound process receipt.
    pub agent_worker: Option<MoiraAgentWorkerProfile>,
    pub host: Option<MoiraHostProfile>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MoiraDecisionPacketReaderProfile {
    pub owner_route: String,
    pub artifact_route: String,
    pub vellum_workspace: u64,
    pub vellum_document: u64,
    pub max_packet_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MoiraHumanGateResponseProfile {
    pub rationale_owner_route: String,
    pub rationale_artifact_route: String,
    pub vellum_workspace: u64,
    pub vellum_document: u64,
    pub max_rationale_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MoiraAgentWorkerProfile {
    pub executable: PathBuf,
    pub config: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MoiraHostProfile {
    pub executable: PathBuf,
    pub config: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DaimonProfile {
    #[serde(default)]
    pub deployment: DeploymentMode,
    pub socket: Option<PathBuf>,
    /// Dedicated terminal-model protocol endpoint. It is never interpreted as
    /// the legacy presence protocol or the interactive-agent protocol.
    pub model_socket: Option<PathBuf>,
    pub agent_socket: Option<PathBuf>,
    pub agent_session: Option<String>,
    /// Explicit local workset. This is not owner-side discovery and is capped
    /// so interactive fan-out remains inspectable.
    #[serde(default)]
    pub agent_sessions: Vec<String>,
    /// Policy for opening a fresh owner-backed interactive session. When this
    /// is present the initial session workset may be empty; Pantheon must
    /// obtain a human confirmation before asking Daimon to create a session.
    pub agent_open: Option<DaimonAgentOpenProfile>,
    /// Credential-free execution policy with one frozen task attempt. Each
    /// interactive prompt, permission, or reattach still receives fresh
    /// execution and idempotency identities.
    pub execution: Option<DaimonExecutionPolicyProfile>,
    pub host: Option<DaimonHostProfile>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DaimonExecutionPolicyProfile {
    pub principal: String,
    pub profile: String,
    pub task_attempt: DaimonTaskAttemptProfile,
    #[serde(default)]
    pub context: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    pub budget: DaimonExecutionBudgetProfile,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DaimonTaskAttemptProfile {
    pub task_key: String,
    pub task_incarnation: u64,
    pub attempt_number: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DaimonExecutionBudgetProfile {
    pub max_elapsed_millis: u64,
    pub max_input_tokens: u64,
    pub max_output_tokens: u64,
    pub max_tool_calls: u32,
    pub max_context_bytes: usize,
    pub max_output_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DaimonAgentOpenProfile {
    /// Exact workspace identity understood by the configured Daimon owner.
    pub workspace: String,
    pub budget: DaimonAgentSessionBudgetProfile,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DaimonAgentSessionBudgetProfile {
    pub max_elapsed_millis: u64,
    pub max_turns: u32,
    pub max_events: u64,
    pub max_event_bytes: usize,
    pub max_prompt_bytes: usize,
    pub max_permission_requests: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DaimonHostProfile {
    pub executable: PathBuf,
    pub config: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HestiaProfile {
    pub socket: PathBuf,
    #[serde(default)]
    pub runtime_bindings: Vec<HestiaRuntimeBindingProfile>,
    #[serde(default)]
    pub artifact_catalog: bool,
}

/// Explicit cross-owner composition only. Hestia does not learn Daimon
/// provider identity and Daimon does not gain Hestia lifecycle authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HestiaRuntimeBindingProfile {
    pub provider_id: String,
    pub runtime_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HermesProfile {
    #[serde(default)]
    pub deployment: DeploymentMode,
    pub socket: PathBuf,
    pub stream: Option<String>,
    pub consumer: Option<String>,
    pub host: Option<HermesHostProfile>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HermesHostProfile {
    pub executable: PathBuf,
    pub directory: PathBuf,
    pub mode: HermesOpenMode,
    pub effective_uid: u32,
    pub effective_gid: u32,
    pub subject: String,
    pub access: HermesAccessProfile,
    pub operator_stream: Option<String>,
    #[serde(default)]
    pub consumer_stream: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HermesOpenMode {
    Create,
    Open,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HermesAccessProfile {
    Observer,
    Operator,
    Consumer,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AegisProfile {
    #[serde(default)]
    pub deployment: DeploymentMode,
    pub socket: PathBuf,
    pub host: Option<AegisHostProfile>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AegisHostProfile {
    pub executable: PathBuf,
    pub directory: PathBuf,
    pub mode: AegisOpenMode,
    pub authority: String,
    pub effective_uid: u32,
    pub effective_gid: u32,
    pub principal: String,
    pub session_namespace: String,
    pub role: AegisRole,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AegisOpenMode {
    Create,
    Open,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AegisRole {
    Administrator,
    Principal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TellusProfile {
    pub endpoint: SocketAddr,
    pub token: Option<PathBuf>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChronosProfile {
    #[serde(default)]
    pub deployment: DeploymentMode,
    pub workstreams: Vec<WorkstreamProfile>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub moira: Vec<MoiraBinding>,
    #[serde(default)]
    pub daimon: Vec<DaimonBinding>,
    #[serde(default)]
    pub alignment: Vec<AlignmentBinding>,
    /// Optional private in-process worker retained with one Pantheon-hosted
    /// Chronos owner. It is configuration, not a claim or transferable grant.
    pub trusted_worker: Option<ChronosTrustedWorkerProfile>,
    /// Optional sovereign product-alignment process. Pantheon owns only its
    /// lifecycle; the opaque config retains all owner bindings and credentials.
    pub product_alignment: Option<ChronosProductAlignmentProfile>,
    pub planning: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChronosTrustedWorkerProfile {
    pub workstream: String,
    pub agent: String,
    pub grant_validity_seconds: u64,
    pub lease_duration_seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChronosProductAlignmentProfile {
    pub workstream: String,
    pub executable: PathBuf,
    pub config: PathBuf,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMode {
    #[default]
    Attach,
    Hosted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkstreamProfile {
    pub name: String,
    pub directory: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MoiraBinding {
    pub workstream: String,
    pub socket: PathBuf,
    pub scope: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DaimonBinding {
    pub workstream: String,
    pub socket: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AlignmentBinding {
    pub workstream: String,
    pub directory: PathBuf,
    pub scope: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VellumProfile {
    #[serde(default)]
    pub deployment: DeploymentMode,
    pub socket: PathBuf,
    pub credential: Option<PathBuf>,
    #[serde(default)]
    pub roots: Vec<PathBuf>,
    /// Explicit hosted action ceiling for runtime Ctrl-O file admission.
    #[serde(default)]
    pub allow_add_file: bool,
    #[serde(default)]
    pub requires: Vec<String>,
}

impl WorkspaceProfile {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_with_digest(path).map(|(profile, _)| profile)
    }

    pub fn load_with_digest(path: &Path) -> Result<(Self, String), Box<dyn std::error::Error>> {
        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_PROFILE_BYTES {
            return Err("workspace profile exceeds 64 KiB".into());
        }
        let bytes = fs::read(path)?;
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        let digest = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        Self::parse(&bytes, base).map(|profile| (profile, digest))
    }

    pub fn parse(bytes: &[u8], base: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() as u64 > MAX_PROFILE_BYTES {
            return Err("workspace profile exceeds 64 KiB".into());
        }
        let mut profile: Self = serde_json::from_slice(bytes)?;
        profile.validate()?;
        if let Some(agora) = &mut profile.agora {
            agora.socket = resolve(base, &agora.socket);
            if let Some(host) = &mut agora.host {
                host.executable = resolve(base, &host.executable);
                host.config = resolve(base, &host.config);
            }
        }
        if let Some(task_conversation) = &mut profile.task_conversation {
            task_conversation.socket = resolve(base, &task_conversation.socket);
        }
        if let Some(moira) = &mut profile.moira {
            if let Some(socket) = &mut moira.socket {
                *socket = resolve(base, socket);
            }
            if let Some(socket) = &mut moira.participation_socket {
                *socket = resolve(base, socket);
            }
            if let Some(socket) = &mut moira.deliberation_socket {
                *socket = resolve(base, socket);
            }
            if let Some(host) = &mut moira.host {
                host.executable = resolve(base, &host.executable);
                host.config = resolve(base, &host.config);
            }
            if let Some(worker) = &mut moira.agent_worker {
                worker.executable = resolve(base, &worker.executable);
                worker.config = resolve(base, &worker.config);
            }
        }
        if let Some(daimon) = &mut profile.daimon {
            if let Some(socket) = &mut daimon.socket {
                *socket = resolve(base, socket);
            }
            if let Some(socket) = &mut daimon.model_socket {
                *socket = resolve(base, socket);
            }
            if let Some(socket) = &mut daimon.agent_socket {
                *socket = resolve(base, socket);
            }
            if let Some(host) = &mut daimon.host {
                host.executable = resolve(base, &host.executable);
                host.config = resolve(base, &host.config);
            }
        }
        if let Some(hestia) = &mut profile.hestia {
            hestia.socket = resolve(base, &hestia.socket);
        }
        if let Some(hermes) = &mut profile.hermes {
            hermes.socket = resolve(base, &hermes.socket);
            if let Some(host) = &mut hermes.host {
                host.executable = resolve(base, &host.executable);
                host.directory = resolve(base, &host.directory);
            }
        }
        if let Some(aegis) = &mut profile.aegis {
            aegis.socket = resolve(base, &aegis.socket);
            if let Some(host) = &mut aegis.host {
                host.executable = resolve(base, &host.executable);
                host.directory = resolve(base, &host.directory);
            }
        }
        if let Some(tellus) = &mut profile.tellus
            && let Some(token) = &mut tellus.token
        {
            *token = resolve(base, token);
        }
        if let Some(chronos) = &mut profile.chronos {
            for workstream in &mut chronos.workstreams {
                workstream.directory = resolve(base, &workstream.directory);
            }
            for binding in &mut chronos.moira {
                binding.socket = resolve(base, &binding.socket);
            }
            for binding in &mut chronos.daimon {
                binding.socket = resolve(base, &binding.socket);
            }
            for binding in &mut chronos.alignment {
                binding.directory = resolve(base, &binding.directory);
            }
            if let Some(product_alignment) = &mut chronos.product_alignment {
                product_alignment.executable = resolve(base, &product_alignment.executable);
                product_alignment.config = resolve(base, &product_alignment.config);
            }
            if let Some(path) = &mut chronos.planning {
                *path = resolve(base, path);
            }
        }
        if let Some(vellum) = &mut profile.vellum {
            vellum.socket = resolve(base, &vellum.socket);
            if let Some(path) = &mut vellum.credential {
                *path = resolve(base, path);
            }
            for root in &mut vellum.roots {
                *root = resolve(base, root);
            }
        }
        Ok(profile)
    }

    pub fn shell(&self) -> Result<Shell, &'static str> {
        Shell::new(self.surfaces.clone())
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.version != 1 {
            return Err("unsupported workspace profile version");
        }
        validate_text(&self.name, "workspace name")?;
        self.shell()?;
        if let Some(agora) = &self.agora {
            if !self.surfaces.contains(&Surface::Agora) {
                return Err("Agora configuration requires the Agora surface");
            }
            validate_path(&agora.socket)?;
            validate_text(&agora.participant, "Agora participant")?;
            match (agora.deployment, &agora.host) {
                (DeploymentMode::Hosted, Some(host)) => {
                    if agora.transport != AgoraTransportProfile::Composition {
                        return Err("hosted Agora requires the composition transport");
                    }
                    validate_path(&host.executable)?;
                    validate_path(&host.config)?;
                }
                (DeploymentMode::Hosted, None) => {
                    return Err("hosted Agora requires an owner configuration");
                }
                (DeploymentMode::Attach, None) => {}
                (DeploymentMode::Attach, Some(_)) => {
                    return Err("attached Agora cannot carry owner configuration");
                }
            }
            validate_required_capabilities(&agora.requires)?;
        }
        if let Some(task_conversation) = &self.task_conversation {
            if self.agora.is_none() || self.chronos.is_none() {
                return Err("task-conversation configuration requires Agora and Chronos");
            }
            validate_path(&task_conversation.socket)?;
            validate_hex_identifier(&task_conversation.principal, "task-conversation principal")?;
            if let Some(addressee) = &task_conversation.agent_addressee
                && (addressee.is_empty()
                    || addressee.len() > 128
                    || addressee.chars().any(char::is_control))
            {
                return Err(
                    "task-conversation agent_addressee must be 1–128 bytes without control text",
                );
            }
            if task_conversation
                .agent_addressee
                .as_ref()
                .is_some_and(|addressee| {
                    self.agora
                        .as_ref()
                        .is_some_and(|agora| &agora.participant == addressee)
                })
            {
                return Err(
                    "task-conversation agent_addressee must be distinct from the Agora operator participant",
                );
            }
            if task_conversation.agent_addressee.is_some()
                && !task_conversation.agent_routes.is_empty()
            {
                return Err(
                    "task-conversation selects either legacy agent_addressee or bounded agent_routes, not both",
                );
            }
            if task_conversation.agent_routes.len() > 16 {
                return Err("task-conversation agent_routes exceeds the 16-route bound");
            }
            let configured_sessions = self
                .daimon
                .as_ref()
                .map(|daimon| {
                    daimon
                        .agent_session
                        .iter()
                        .chain(daimon.agent_sessions.iter())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            for (index, route) in task_conversation.agent_routes.iter().enumerate() {
                if route.addressee.is_empty()
                    || route.addressee.len() > 128
                    || route.addressee.chars().any(char::is_control)
                {
                    return Err(
                        "task-conversation agent route addressee must be 1–128 bytes without control text",
                    );
                }
                if self
                    .agora
                    .as_ref()
                    .is_some_and(|agora| agora.participant == route.addressee)
                {
                    return Err(
                        "task-conversation agent route must be distinct from the Agora operator participant",
                    );
                }
                validate_hex_identifier(&route.session, "task-conversation agent route session")?;
                if !configured_sessions.contains(&&route.session) {
                    return Err(
                        "task-conversation agent route must select an exact configured Daimon session",
                    );
                }
                if task_conversation.agent_routes[..index].iter().any(|prior| {
                    prior.addressee == route.addressee || prior.session == route.session
                }) {
                    return Err(
                        "task-conversation agent routes require unique addressees and sessions",
                    );
                }
            }
            validate_required_capabilities(&task_conversation.requires)?;
        }
        if let Some(moira) = &self.moira {
            if !self.surfaces.contains(&Surface::Moira) {
                return Err("Moira configuration requires the Moira surface");
            }
            if let Some(socket) = &moira.socket {
                validate_path(socket)?;
            }
            match (&moira.participation_socket, &moira.participation) {
                (Some(socket), Some(participation)) => {
                    validate_path(socket)?;
                    validate_hex_identifier(participation, "Moira participation")?;
                }
                (None, None) => {}
                _ => {
                    return Err(
                        "Moira participation socket and participation identifier must be configured together",
                    );
                }
            }
            match (&moira.deliberation_socket, &moira.deliberation) {
                (Some(socket), Some(deliberation)) => {
                    validate_path(socket)?;
                    validate_hex_identifier(deliberation, "Moira deliberation")?;
                }
                (None, None) => {}
                _ => {
                    return Err(
                        "Moira deliberation socket and deliberation identifier must be configured together",
                    );
                }
            }
            if let Some(reader) = &moira.decision_packet_reader {
                if moira.deliberation.is_none() {
                    return Err(
                        "Moira decision-packet reader requires an exact selected deliberation",
                    );
                }
                if self.vellum.is_none() {
                    return Err("Moira decision-packet reader requires Vellum configuration");
                }
                validate_lower_hex(&reader.owner_route, 32, "Moira decision-packet owner route")?;
                validate_lower_hex(
                    &reader.artifact_route,
                    32,
                    "Moira decision-packet artifact route",
                )?;
                if reader.vellum_workspace == 0 || reader.vellum_document == 0 {
                    return Err("Moira decision-packet Vellum identities must be non-zero");
                }
                if !(1..=65_536).contains(&reader.max_packet_bytes) {
                    return Err("Moira decision-packet byte limit must be between 1 and 65536");
                }
            }
            if let Some(response) = &moira.human_gate_response {
                if moira.deliberation_socket.is_none() || moira.deliberation.is_none() {
                    return Err(
                        "Moira human-gate response requires an exact selected deliberation",
                    );
                }
                if moira.decision_packet_reader.is_none() {
                    return Err(
                        "Moira human-gate response requires the selected decision-packet reader",
                    );
                }
                if self.vellum.is_none() {
                    return Err("Moira human-gate response requires Vellum configuration");
                }
                validate_lower_hex(
                    &response.rationale_owner_route,
                    32,
                    "Moira response rationale owner route",
                )?;
                validate_lower_hex(
                    &response.rationale_artifact_route,
                    32,
                    "Moira response rationale artifact route",
                )?;
                if response.vellum_workspace == 0 || response.vellum_document == 0 {
                    return Err("Moira response rationale Vellum identities must be non-zero");
                }
                if !(1..=65_536).contains(&response.max_rationale_bytes) {
                    return Err("Moira response rationale byte limit must be between 1 and 65536");
                }
            }
            if let Some(worker) = &moira.agent_worker {
                if moira.participation.is_none() {
                    return Err("Moira agent worker requires an exact selected participation");
                }
                validate_path(&worker.executable)?;
                validate_path(&worker.config)?;
            }
            match (moira.deployment, &moira.host) {
                (DeploymentMode::Hosted, Some(host)) => {
                    if moira.socket.is_none() {
                        return Err("hosted Moira requires a saga socket");
                    }
                    validate_path(&host.executable)?;
                    validate_path(&host.config)?;
                }
                (DeploymentMode::Hosted, None) => {
                    return Err("hosted Moira requires an owner configuration");
                }
                (DeploymentMode::Attach, Some(_)) => {
                    return Err("attached Moira cannot carry owner configuration");
                }
                (DeploymentMode::Attach, None) => {}
            }
            if moira.socket.is_none()
                && moira.participation_socket.is_none()
                && moira.deliberation_socket.is_none()
            {
                return Err("Moira requires at least one selected presentation endpoint");
            }
            validate_required_capabilities(&moira.requires)?;
        }
        if let Some(daimon) = &self.daimon {
            if !self.surfaces.contains(&Surface::Daimon) {
                return Err("Daimon configuration requires the Daimon surface");
            }
            if daimon.socket.is_none()
                && daimon.model_socket.is_none()
                && daimon.agent_socket.is_none()
            {
                return Err("Daimon configuration requires a presence, model, or agent socket");
            }
            if let Some(socket) = &daimon.socket {
                validate_path(socket)?;
            }
            if let Some(socket) = &daimon.model_socket {
                validate_path(socket)?;
            }
            if daimon.agent_session.is_some() && !daimon.agent_sessions.is_empty() {
                return Err("Daimon accepts agent_session or agent_sessions, not both");
            }
            let agent_sessions = daimon
                .agent_session
                .iter()
                .chain(daimon.agent_sessions.iter())
                .collect::<Vec<_>>();
            if agent_sessions.len() > daimon_tui::AgentSetComponent::MAX_SESSIONS {
                return Err("Daimon agent session set exceeds the 16-session bound");
            }
            match (
                &daimon.agent_socket,
                agent_sessions.is_empty(),
                daimon.agent_open.is_some(),
            ) {
                (Some(socket), false, _) | (Some(socket), true, true) => {
                    validate_path(socket)?;
                    for (index, session) in agent_sessions.iter().enumerate() {
                        if daimon_tui::parse_agent_session(session).is_none() {
                            return Err("Daimon agent sessions must be 32 hexadecimal digits");
                        }
                        if agent_sessions[..index].contains(session) {
                            return Err("Daimon agent sessions must be unique");
                        }
                    }
                }
                (None, true, false) => {}
                _ => {
                    return Err(
                        "Daimon agent socket requires an initial session set or session-open policy",
                    );
                }
            }
            if let Some(open) = &daimon.agent_open {
                validate_hex_identifier(&open.workspace, "Daimon session-open workspace")?;
                if daimon.execution.is_none() {
                    return Err("Daimon session-open policy requires an execution policy");
                }
                if open.budget.max_elapsed_millis == 0
                    || open.budget.max_turns == 0
                    || open.budget.max_events == 0
                    || open.budget.max_event_bytes == 0
                    || open.budget.max_prompt_bytes == 0
                {
                    return Err(
                        "Daimon session-open policy requires non-zero time, turn, event, and byte budgets",
                    );
                }
            }
            if let Some(execution) = &daimon.execution {
                if daimon.agent_socket.is_none()
                    || (agent_sessions.is_empty() && daimon.agent_open.is_none())
                {
                    return Err(
                        "Daimon execution policy requires an agent socket and session set or session-open policy",
                    );
                }
                validate_hex_identifier(&execution.principal, "Daimon execution principal")?;
                validate_hex_identifier(&execution.profile, "Daimon execution profile")?;
                validate_hex_identifier(
                    &execution.task_attempt.task_key,
                    "Daimon execution task key",
                )?;
                if execution.task_attempt.attempt_number == 0 {
                    return Err("Daimon execution attempt number must be non-zero");
                }
                if execution.context.len() > daimon_agent_command::MAX_CONTEXT_REFERENCES {
                    return Err("Daimon execution context exceeds the 64-reference bound");
                }
                if execution.tools.len() > daimon_agent_command::MAX_TOOLS {
                    return Err("Daimon execution tools exceed the 64-capability bound");
                }
                for value in &execution.context {
                    validate_hex_identifier(value, "Daimon execution context reference")?;
                }
                for value in &execution.tools {
                    validate_hex_identifier(value, "Daimon execution tool capability")?;
                }
                if execution.budget.max_elapsed_millis == 0
                    || execution.budget.max_input_tokens == 0
                    || execution.budget.max_output_tokens == 0
                    || execution.budget.max_context_bytes == 0
                    || execution.budget.max_output_bytes == 0
                {
                    return Err(
                        "Daimon execution policy requires non-zero time, token, and byte budgets",
                    );
                }
            }
            match (daimon.deployment, &daimon.host) {
                (DeploymentMode::Hosted, Some(host)) => {
                    let model_owner = daimon.socket.is_none()
                        && daimon.model_socket.is_some()
                        && daimon.agent_socket.is_none()
                        && agent_sessions.is_empty();
                    let agent_owner = daimon.socket.is_none()
                        && daimon.model_socket.is_none()
                        && daimon.agent_socket.is_some()
                        && (!agent_sessions.is_empty() || daimon.agent_open.is_some());
                    if !model_owner && !agent_owner {
                        return Err(
                            "hosted Daimon requires exactly one model endpoint or exact agent selection",
                        );
                    }
                    validate_path(&host.executable)?;
                    validate_path(&host.config)?;
                }
                (DeploymentMode::Hosted, None) => {
                    return Err("hosted Daimon requires an owner configuration");
                }
                (DeploymentMode::Attach, Some(_)) => {
                    return Err("attached Daimon cannot carry owner configuration");
                }
                (DeploymentMode::Attach, None) => {}
            }
            validate_required_capabilities(&daimon.requires)?;
        }
        if let Some(hestia) = &self.hestia {
            if !self.surfaces.contains(&Surface::Hestia) {
                return Err("Hestia configuration requires the Hestia surface");
            }
            validate_path(&hestia.socket)?;
            if hestia.runtime_bindings.is_empty() && !hestia.artifact_catalog {
                return Err(
                    "Hestia requires an artifact catalog or 1–16 explicit runtime bindings",
                );
            }
            if hestia.runtime_bindings.len() > 16 {
                return Err("Hestia supports at most 16 explicit runtime bindings");
            }
            if !hestia.runtime_bindings.is_empty()
                && self
                    .daimon
                    .as_ref()
                    .and_then(|profile| profile.model_socket.as_ref())
                    .is_none()
            {
                return Err("Hestia runtime correlation requires a selected Daimon model endpoint");
            }
            for (index, binding) in hestia.runtime_bindings.iter().enumerate() {
                validate_hex_identifier(&binding.provider_id, "Hestia provider binding")?;
                validate_hex_identifier(&binding.runtime_id, "Hestia runtime binding")?;
                if hestia.runtime_bindings[..index].iter().any(|prior| {
                    prior.provider_id == binding.provider_id
                        || prior.runtime_id == binding.runtime_id
                }) {
                    return Err("Hestia provider and runtime bindings must be one-to-one");
                }
            }
        }
        if let Some(hermes) = &self.hermes {
            if !self.surfaces.contains(&Surface::Hermes) {
                return Err("Hermes configuration requires the Hermes surface");
            }
            validate_path(&hermes.socket)?;
            match (hermes.deployment, &hermes.host) {
                (DeploymentMode::Hosted, Some(host)) => {
                    validate_path(&host.executable)?;
                    validate_path(&host.directory)?;
                    validate_lower_hex(&host.subject, 32, "hosted Hermes subject")?;
                    match (host.access, &host.operator_stream, &host.consumer_stream) {
                        (HermesAccessProfile::Observer, None, None) => {}
                        (HermesAccessProfile::Observer, _, _) => {
                            return Err("observer Hermes host cannot select a stream");
                        }
                        (HermesAccessProfile::Operator, Some(stream), None) => {
                            validate_lower_hex(stream, 32, "hosted Hermes operator stream")?;
                            if let Some(presentation_stream) = &hermes.stream
                                && presentation_stream != stream
                            {
                                return Err(
                                    "hosted Hermes operator and presentation streams must match",
                                );
                            }
                        }
                        (HermesAccessProfile::Operator, None, None) => {
                            return Err("operator Hermes host requires an exact stream");
                        }
                        (HermesAccessProfile::Consumer, None, Some(stream)) => {
                            validate_lower_hex(stream, 32, "hosted Hermes consumer stream")?;
                            if hermes.stream.as_deref() != Some(stream.as_str()) {
                                return Err(
                                    "hosted Hermes consumer requires its exact presentation stream",
                                );
                            }
                            if hermes.consumer.is_none() {
                                return Err(
                                    "hosted Hermes consumer requires a presentation consumer",
                                );
                            }
                        }
                        (HermesAccessProfile::Consumer, None, None) => {
                            return Err("consumer Hermes host requires an exact stream");
                        }
                        (HermesAccessProfile::Operator, _, _)
                        | (HermesAccessProfile::Consumer, _, _) => {
                            return Err("hosted Hermes access selects exactly one stream kind");
                        }
                    }
                }
                (DeploymentMode::Hosted, None) => {
                    return Err("hosted Hermes requires an explicit host configuration");
                }
                (DeploymentMode::Attach, Some(_)) => {
                    return Err("attached Hermes cannot declare a host configuration");
                }
                (DeploymentMode::Attach, None) => {}
            }
            if let Some(stream) = &hermes.stream
                && hermes_tui::parse_stream_id(stream).is_none()
            {
                return Err("Hermes stream must be 32 hexadecimal digits");
            }
            if let Some(consumer) = &hermes.consumer {
                if hermes.stream.is_none() {
                    return Err("Hermes consumer requires a stream");
                }
                if hermes_tui::parse_consumer_id(consumer).is_none() {
                    return Err("Hermes consumer must be 32 hexadecimal digits");
                }
            }
            validate_required_capabilities(&hermes.requires)?;
        }
        if let Some(aegis) = &self.aegis {
            if !self.surfaces.contains(&Surface::Aegis) {
                return Err("Aegis configuration requires the Aegis surface");
            }
            validate_path(&aegis.socket)?;
            match (aegis.deployment, &aegis.host) {
                (DeploymentMode::Hosted, Some(host)) => {
                    validate_path(&host.executable)?;
                    validate_path(&host.directory)?;
                    validate_lower_hex(&host.authority, 32, "hosted Aegis authority")?;
                    validate_lower_hex(&host.principal, 32, "hosted Aegis principal")?;
                    validate_lower_hex(
                        &host.session_namespace,
                        16,
                        "hosted Aegis session namespace",
                    )?;
                }
                (DeploymentMode::Hosted, None) => {
                    return Err("hosted Aegis requires an explicit host configuration");
                }
                (DeploymentMode::Attach, Some(_)) => {
                    return Err("attached Aegis cannot declare a host configuration");
                }
                (DeploymentMode::Attach, None) => {}
            }
            validate_required_capabilities(&aegis.requires)?;
        }
        if let Some(tellus) = &self.tellus {
            if !self.surfaces.contains(&Surface::Tellus) {
                return Err("Tellus configuration requires the Tellus surface");
            }
            if !tellus.endpoint.ip().is_loopback() {
                return Err("Tellus endpoint must be loopback");
            }
            if let Some(token) = &tellus.token {
                validate_path(token)?;
            }
            validate_required_capabilities(&tellus.requires)?;
        }
        if self.surfaces.contains(&Surface::Workspace)
            && self.chronos.is_none()
            && self.vellum.is_none()
        {
            return Err("workspace surface requires Chronos or Vellum configuration");
        }
        if self.surfaces.contains(&Surface::Chronos) && self.chronos.is_none() {
            return Err("Chronos surface requires Chronos configuration");
        }
        if self.surfaces.contains(&Surface::Vellum) && self.vellum.is_none() {
            return Err("Vellum surface requires Vellum configuration");
        }
        if let Some(chronos) = &self.chronos {
            validate_required_capabilities(&chronos.requires)?;
            if chronos.workstreams.is_empty() || chronos.workstreams.len() > MAX_WORKSTREAMS {
                return Err("Chronos workstream count is outside the supported range");
            }
            if chronos.moira.len() > MAX_BINDINGS
                || chronos.daimon.len() > MAX_BINDINGS
                || chronos.alignment.len() > MAX_BINDINGS
            {
                return Err("Chronos peer binding count is outside the supported range");
            }
            for (index, workstream) in chronos.workstreams.iter().enumerate() {
                validate_text(&workstream.name, "workstream name")?;
                validate_path(&workstream.directory)?;
                if chronos.workstreams[..index]
                    .iter()
                    .any(|other| other.name == workstream.name)
                {
                    return Err("Chronos workstream names must be unique");
                }
            }
            for (index, binding) in chronos.moira.iter().enumerate() {
                validate_text(&binding.workstream, "Moira workstream")?;
                validate_text(&binding.scope, "Moira scope")?;
                validate_path(&binding.socket)?;
                validate_binding(
                    &binding.workstream,
                    chronos
                        .workstreams
                        .iter()
                        .map(|workstream| &workstream.name),
                )?;
                if chronos.moira[..index]
                    .iter()
                    .any(|other| other.workstream == binding.workstream)
                {
                    return Err("Moira workstream bindings must be unique");
                }
            }
            for (index, binding) in chronos.daimon.iter().enumerate() {
                validate_text(&binding.workstream, "Daimon workstream")?;
                validate_path(&binding.socket)?;
                validate_binding(
                    &binding.workstream,
                    chronos
                        .workstreams
                        .iter()
                        .map(|workstream| &workstream.name),
                )?;
                if chronos.daimon[..index]
                    .iter()
                    .any(|other| other.workstream == binding.workstream)
                {
                    return Err("Daimon workstream bindings must be unique");
                }
            }
            for (index, binding) in chronos.alignment.iter().enumerate() {
                validate_text(&binding.workstream, "alignment workstream")?;
                validate_text(&binding.scope, "alignment scope")?;
                validate_path(&binding.directory)?;
                validate_binding(
                    &binding.workstream,
                    chronos
                        .workstreams
                        .iter()
                        .map(|workstream| &workstream.name),
                )?;
                if chronos.alignment[..index]
                    .iter()
                    .any(|other| other.workstream == binding.workstream)
                {
                    return Err("alignment workstream bindings must be unique");
                }
            }
            if let Some(worker) = &chronos.trusted_worker {
                if chronos.deployment != DeploymentMode::Hosted {
                    return Err("Chronos trusted worker requires hosted deployment");
                }
                if chronos
                    .workstreams
                    .iter()
                    .filter(|workstream| workstream.name == worker.workstream)
                    .count()
                    != 1
                {
                    return Err("Chronos trusted worker must select one exact hosted workstream");
                }
                if worker.agent.is_empty()
                    || worker.agent.len() > 128
                    || worker.agent.chars().any(char::is_control)
                {
                    return Err(
                        "Chronos trusted worker agent must be 1–128 bytes without control text",
                    );
                }
                if worker.grant_validity_seconds == 0 || worker.lease_duration_seconds == 0 {
                    return Err("Chronos trusted worker durations must be non-zero seconds");
                }
            }
            if let Some(product_alignment) = &chronos.product_alignment {
                validate_text(
                    &product_alignment.workstream,
                    "product-alignment workstream",
                )?;
                validate_path(&product_alignment.executable)?;
                validate_path(&product_alignment.config)?;
                if chronos
                    .workstreams
                    .iter()
                    .filter(|workstream| workstream.name == product_alignment.workstream)
                    .count()
                    != 1
                    || chronos
                        .alignment
                        .iter()
                        .filter(|binding| binding.workstream == product_alignment.workstream)
                        .count()
                        != 1
                {
                    return Err(
                        "product alignment must select one exact workstream with an alignment binding",
                    );
                }
            }
            if let Some(path) = &chronos.planning {
                validate_path(path)?;
            }
        }
        if let Some(vellum) = &self.vellum {
            validate_path(&vellum.socket)?;
            validate_required_capabilities(&vellum.requires)?;
            if let Some(path) = &vellum.credential {
                validate_path(path)?;
            }
            if vellum.roots.len() > 16 {
                return Err("Vellum root count exceeds the supported range");
            }
            for root in &vellum.roots {
                validate_path(root)?;
            }
            if vellum.deployment == DeploymentMode::Hosted && vellum.roots.is_empty() {
                return Err("hosted Vellum requires at least one workspace root");
            }
            if vellum.deployment == DeploymentMode::Attach && !vellum.roots.is_empty() {
                return Err("attached Vellum cannot declare hosted workspace roots");
            }
            if vellum.deployment == DeploymentMode::Attach && vellum.allow_add_file {
                return Err("attached Vellum cannot declare hosted file-admission authority");
            }
        }
        Ok(())
    }
}

fn validate_hex_identifier(value: &str, label: &'static str) -> Result<(), &'static str> {
    if value.len() != 32 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(match label {
            "Daimon execution principal" => {
                "Daimon execution principal must be 32 hexadecimal digits"
            }
            "Daimon execution profile" => "Daimon execution profile must be 32 hexadecimal digits",
            "Daimon execution task key" => {
                "Daimon execution task key must be 32 hexadecimal digits"
            }
            "Daimon execution context reference" => {
                "Daimon execution context references must be 32 hexadecimal digits"
            }
            _ => "Daimon execution tool capabilities must be 32 hexadecimal digits",
        });
    }
    Ok(())
}

fn validate_required_capabilities(values: &[String]) -> Result<(), &'static str> {
    if values.len() > MAX_REQUIRED_CAPABILITIES {
        return Err("required capability count exceeds the supported range");
    }
    for (index, value) in values.iter().enumerate() {
        if value.is_empty()
            || value.len() > MAX_CAPABILITY_BYTES
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'-'
            })
        {
            return Err("required capability identifier is invalid");
        }
        if values[..index].contains(value) {
            return Err("required capabilities must be unique");
        }
    }
    Ok(())
}

fn validate_lower_hex(value: &str, length: usize, field: &'static str) -> Result<(), &'static str> {
    if value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(match field {
            "hosted Aegis authority" => {
                "hosted Aegis authority must be 32 lowercase hexadecimal digits"
            }
            "hosted Aegis principal" => {
                "hosted Aegis principal must be 32 lowercase hexadecimal digits"
            }
            _ => "hosted Aegis session namespace must be 16 lowercase hexadecimal digits",
        })
    }
}

fn validate_binding<'a>(
    binding: &str,
    workstreams: impl Iterator<Item = &'a String>,
) -> Result<(), &'static str> {
    if workstreams.into_iter().any(|name| name == binding) {
        Ok(())
    } else {
        Err("peer binding references an unknown Chronos workstream")
    }
}

fn validate_path(path: &Path) -> Result<(), &'static str> {
    if path.as_os_str().is_empty() {
        Err("workspace profile contains an empty path")
    } else {
        Ok(())
    }
}

fn validate_text(value: &str, field: &'static str) -> Result<(), &'static str> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(match field {
            "workspace name" => "workspace name is invalid",
            "workstream name" => "workstream name is invalid",
            "Moira workstream" => "Moira workstream name is invalid",
            "Moira scope" => "Moira scope is invalid",
            "alignment workstream" => "alignment workstream name is invalid",
            "alignment scope" => "alignment scope is invalid",
            "product-alignment workstream" => "product-alignment workstream name is invalid",
            "Agora participant" => "Agora participant is invalid",
            _ => "Daimon workstream name is invalid",
        });
    }
    Ok(())
}

fn resolve(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_selects_a_closed_keybinding_contract() {
        let emacs = br#"{
          "version":1,"name":"Emacs workspace","surfaces":["overview"],
          "keybindings":"emacs"
        }"#;
        let profile = WorkspaceProfile::parse(emacs, Path::new(".")).unwrap();
        assert_eq!(profile.keybindings, KeybindingProfile::Emacs);

        let default = br#"{
          "version":1,"name":"Default workspace","surfaces":["overview"]
        }"#;
        let profile = WorkspaceProfile::parse(default, Path::new(".")).unwrap();
        assert_eq!(profile.keybindings, KeybindingProfile::Pantheon);

        let unknown = br#"{
          "version":1,"name":"Unknown workspace","surfaces":["overview"],
          "keybindings":"almost_emacs"
        }"#;
        assert!(WorkspaceProfile::parse(unknown, Path::new(".")).is_err());
    }

    #[test]
    fn alignment_binding_selects_one_exact_chronos_workstream_and_scope() {
        let valid = br#"{
          "version":1,"name":"Alignment","surfaces":["chronos"],
          "chronos":{"workstreams":[{"name":"Atlas","directory":"chronos"}],
            "alignment":[{"workstream":"Atlas","directory":"alignment","scope":"product-delivery"}]}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let binding = &profile.chronos.unwrap().alignment[0];
        assert_eq!(binding.directory, Path::new("/tmp/workbench/alignment"));
        assert_eq!(binding.scope, "product-delivery");

        let unmatched = br#"{
          "version":1,"name":"Alignment","surfaces":["chronos"],
          "chronos":{"workstreams":[{"name":"Atlas","directory":"chronos"}],
            "alignment":[{"workstream":"Other","directory":"alignment","scope":"product-delivery"}]}
        }"#;
        assert!(WorkspaceProfile::parse(unmatched, Path::new(".")).is_err());
    }

    #[test]
    fn hosted_product_alignment_requires_one_presented_workstream_binding() {
        let valid = br#"{
          "version":1,"name":"Alignment owner","surfaces":["chronos"],
          "chronos":{"workstreams":[{"name":"Product","directory":"chronos"}],
            "alignment":[{"workstream":"Product","directory":"alignment","scope":"product-delivery"}],
            "product_alignment":{"workstream":"Product","executable":"bin/chronos-product-alignment-local","config":"alignment.json"}}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let alignment = profile.chronos.unwrap().product_alignment.unwrap();
        assert_eq!(alignment.workstream, "Product");
        assert_eq!(
            alignment.executable,
            Path::new("/tmp/workbench/bin/chronos-product-alignment-local")
        );
        assert_eq!(alignment.config, Path::new("/tmp/workbench/alignment.json"));

        let missing_binding = br#"{
          "version":1,"name":"Alignment owner","surfaces":["chronos"],
          "chronos":{"workstreams":[{"name":"Product","directory":"chronos"}],
            "product_alignment":{"workstream":"Product","executable":"alignment-owner","config":"alignment.json"}}
        }"#;
        assert!(WorkspaceProfile::parse(missing_binding, Path::new(".")).is_err());
    }

    #[test]
    fn profile_is_versioned_bounded_and_resolves_relative_paths() {
        let bytes = br#"{
          "version": 1,
          "name": "Local federation",
          "surfaces": ["overview", "workspace", "agora"],
          "agora": {"socket":"agora.sock", "participant":"alice", "requires":["rooms.read.v2"]},
          "chronos": {
            "workstreams": [{"name":"Atlas", "directory":"chronos"}],
            "requires": ["tasks.observe.v1"],
            "moira": [{"workstream":"Atlas", "socket":"moira.sock", "scope":"atlas"}],
            "daimon": [],
            "planning": "plan.json"
          },
          "vellum": {"socket":"vellum.sock", "credential":null, "requires":["workspace.inventory.v1"]}
        }"#;
        let profile =
            WorkspaceProfile::parse(bytes, Path::new("/tmp/workbench")).expect("valid profile");
        assert_eq!(profile.surfaces[2], Surface::Agora);
        assert_eq!(profile.agora.as_ref().unwrap().requires, ["rooms.read.v2"]);
        assert_eq!(
            profile.agora.as_ref().unwrap().socket,
            Path::new("/tmp/workbench/agora.sock")
        );
        assert_eq!(
            profile.chronos.as_ref().unwrap().workstreams[0].directory,
            Path::new("/tmp/workbench/chronos")
        );
        assert_eq!(
            profile.vellum.as_ref().unwrap().socket,
            Path::new("/tmp/workbench/vellum.sock")
        );
    }

    #[test]
    fn profile_refuses_unknown_versions_duplicates_and_authority_flags() {
        let unknown = br#"{"version":2,"name":"x","surfaces":["overview"],"agora":null,"chronos":null,"vellum":null}"#;
        assert!(WorkspaceProfile::parse(unknown, Path::new(".")).is_err());
        let duplicate = br#"{"version":1,"name":"x","surfaces":["agora","agora"],"agora":null,"chronos":null,"vellum":null}"#;
        assert!(WorkspaceProfile::parse(duplicate, Path::new(".")).is_err());
        let authority = br#"{"version":1,"name":"x","surfaces":["overview"],"agora":null,"chronos":null,"vellum":null,"allow_writes":true}"#;
        assert!(WorkspaceProfile::parse(authority, Path::new(".")).is_err());
    }

    #[test]
    fn profile_accepts_exact_opaque_hermes_stream_and_consumer_selectors() {
        let valid = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"socket":"hermes.sock","stream":"05050505050505050505050505050505","consumer":"06060606060606060606060606060606"}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let hermes = profile.hermes.unwrap();
        assert_eq!(
            hermes.stream.as_deref(),
            Some("05050505050505050505050505050505")
        );
        assert_eq!(
            hermes.consumer.as_deref(),
            Some("06060606060606060606060606060606")
        );
        let invalid = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"socket":"hermes.sock","stream":"not-an-opaque-id"}
        }"#;
        assert!(WorkspaceProfile::parse(invalid, Path::new(".")).is_err());
        let consumer_without_stream = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"socket":"hermes.sock","stream":null,"consumer":"06060606060606060606060606060606"}
        }"#;
        assert!(WorkspaceProfile::parse(consumer_without_stream, Path::new(".")).is_err());
    }

    #[test]
    fn profile_accepts_only_a_complete_exact_daimon_agent_selection() {
        let valid = br#"{
          "version":1,"name":"Daimon","surfaces":["daimon"],
          "daimon":{"socket":"presence.sock","agent_socket":"agent.sock",
            "agent_session":"07070707070707070707070707070707",
            "requires":["agents.presence.v1","agents.session.inspect.v1"]}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let daimon = profile.daimon.unwrap();
        assert_eq!(
            daimon.agent_socket.unwrap(),
            Path::new("/tmp/workbench/agent.sock")
        );
        assert_eq!(
            daimon.agent_session.as_deref(),
            Some("07070707070707070707070707070707")
        );

        let incomplete = br#"{
          "version":1,"name":"Daimon","surfaces":["daimon"],
          "daimon":{"socket":"presence.sock","agent_socket":"agent.sock"}
        }"#;
        assert!(WorkspaceProfile::parse(incomplete, Path::new(".")).is_err());
        let invalid = br#"{
          "version":1,"name":"Daimon","surfaces":["daimon"],
          "daimon":{"socket":"presence.sock","agent_socket":"agent.sock",
            "agent_session":"not-an-opaque-id"}
        }"#;
        assert!(WorkspaceProfile::parse(invalid, Path::new(".")).is_err());

        let set = br#"{
          "version":1,"name":"Daimon set","surfaces":["daimon"],
          "daimon":{"agent_socket":"agent.sock","agent_sessions":[
            "07070707070707070707070707070707",
            "08080808080808080808080808080808"]}
        }"#;
        let profile = WorkspaceProfile::parse(set, Path::new("/tmp/workbench")).unwrap();
        assert_eq!(profile.daimon.unwrap().agent_sessions.len(), 2);

        let duplicate = br#"{
          "version":1,"name":"Daimon set","surfaces":["daimon"],
          "daimon":{"agent_socket":"agent.sock","agent_sessions":[
            "07070707070707070707070707070707",
            "07070707070707070707070707070707"]}
        }"#;
        assert!(WorkspaceProfile::parse(duplicate, Path::new(".")).is_err());

        let ambiguous = br#"{
          "version":1,"name":"Daimon set","surfaces":["daimon"],
          "daimon":{"agent_socket":"agent.sock",
            "agent_session":"07070707070707070707070707070707",
            "agent_sessions":["08080808080808080808080808080808"]}
        }"#;
        assert!(WorkspaceProfile::parse(ambiguous, Path::new(".")).is_err());
    }

    #[test]
    fn hosted_daimon_requires_one_owner_endpoint_and_its_exact_selection() {
        let valid = br#"{
          "version":1,"name":"Daimon host","surfaces":["daimon"],
          "daimon":{"deployment":"hosted","agent_socket":"agent.sock",
            "agent_session":"07070707070707070707070707070707",
            "host":{"executable":"bin/daimon-local-agent","config":"agent.json"}}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let daimon = profile.daimon.unwrap();
        assert_eq!(daimon.deployment, DeploymentMode::Hosted);
        assert!(daimon.socket.is_none());
        assert_eq!(
            daimon.host.unwrap().config,
            Path::new("/tmp/workbench/agent.json")
        );

        let model = br#"{
          "version":1,"name":"Daimon host","surfaces":["daimon"],
          "daimon":{"deployment":"hosted","model_socket":"model.sock",
            "host":{"executable":"daimon-local-model","config":"model.json"}}
        }"#;
        let profile = WorkspaceProfile::parse(model, Path::new("/tmp/workbench")).unwrap();
        let daimon = profile.daimon.unwrap();
        assert_eq!(daimon.deployment, DeploymentMode::Hosted);
        assert_eq!(
            daimon.model_socket.unwrap(),
            Path::new("/tmp/workbench/model.sock")
        );
        assert!(daimon.agent_socket.is_none());

        let no_selection = br#"{
          "version":1,"name":"Daimon host","surfaces":["daimon"],
          "daimon":{"deployment":"hosted","agent_socket":"agent.sock",
            "host":{"executable":"daimon-local-agent","config":"agent.json"}}
        }"#;
        assert!(WorkspaceProfile::parse(no_selection, Path::new(".")).is_err());
        let ambiguous_owner = br#"{
          "version":1,"name":"Daimon host","surfaces":["daimon"],
          "daimon":{"deployment":"hosted","model_socket":"model.sock",
            "agent_socket":"agent.sock",
            "agent_session":"07070707070707070707070707070707",
            "host":{"executable":"daimon-local-agent","config":"agent.json"}}
        }"#;
        assert!(WorkspaceProfile::parse(ambiguous_owner, Path::new(".")).is_err());
        let attached_owner = br#"{
          "version":1,"name":"Daimon attach","surfaces":["daimon"],
          "daimon":{"socket":"presence.sock",
            "host":{"executable":"daimon-local-agent","config":"agent.json"}}
        }"#;
        assert!(WorkspaceProfile::parse(attached_owner, Path::new(".")).is_err());
    }

    #[test]
    fn hestia_runtime_bindings_are_explicit_one_to_one_and_model_scoped() {
        let valid = br#"{
          "version":1,"name":"Hestia correlation","surfaces":["daimon","hestia"],
          "daimon":{"model_socket":"model.sock"},
          "hestia":{"socket":"hestia.sock","runtime_bindings":[{
            "provider_id":"04040404040404040404040404040404",
            "runtime_id":"08080808080808080808080808080808"}]}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let hestia = profile.hestia.unwrap();
        assert_eq!(hestia.socket, Path::new("/tmp/workbench/hestia.sock"));
        assert_eq!(hestia.runtime_bindings.len(), 1);

        let duplicate = br#"{
          "version":1,"name":"Hestia correlation","surfaces":["daimon","hestia"],
          "daimon":{"model_socket":"model.sock"},
          "hestia":{"socket":"hestia.sock","runtime_bindings":[
            {"provider_id":"04040404040404040404040404040404","runtime_id":"08080808080808080808080808080808"},
            {"provider_id":"04040404040404040404040404040404","runtime_id":"09090909090909090909090909090909"}]}
        }"#;
        assert!(WorkspaceProfile::parse(duplicate, Path::new(".")).is_err());

        let unbound = br#"{
          "version":1,"name":"Hestia correlation","surfaces":["daimon","hestia"],
          "hestia":{"socket":"hestia.sock","runtime_bindings":[{
            "provider_id":"04040404040404040404040404040404",
            "runtime_id":"08080808080808080808080808080808"}]}
        }"#;
        assert!(WorkspaceProfile::parse(unbound, Path::new(".")).is_err());

        let artifact_only = br#"{
          "version":1,"name":"Hestia artifacts","surfaces":["hestia"],
          "hestia":{"socket":"hestia.sock","artifact_catalog":true}
        }"#;
        let profile = WorkspaceProfile::parse(artifact_only, Path::new("/tmp/workbench")).unwrap();
        let hestia = profile.hestia.unwrap();
        assert!(hestia.artifact_catalog);
        assert!(hestia.runtime_bindings.is_empty());

        let empty = br#"{
          "version":1,"name":"Empty Hestia","surfaces":["hestia"],
          "hestia":{"socket":"hestia.sock"}
        }"#;
        assert!(WorkspaceProfile::parse(empty, Path::new(".")).is_err());
    }

    #[test]
    fn daimon_execution_policy_is_credential_free_bounded_and_complete() {
        let valid = br#"{
          "version":1,"name":"Daimon prompt","surfaces":["daimon"],
          "daimon":{"agent_socket":"agent.sock","agent_sessions":[
            "07070707070707070707070707070707",
            "08080808080808080808080808080808"],
            "execution":{
              "principal":"10101010101010101010101010101010",
              "profile":"11111111111111111111111111111111",
              "task_attempt":{"task_key":"14141414141414141414141414141414",
                "task_incarnation":1,"attempt_number":1},
              "context":["12121212121212121212121212121212"],
              "tools":["13131313131313131313131313131313"],
              "budget":{"max_elapsed_millis":60000,"max_input_tokens":4096,
                "max_output_tokens":8192,"max_tool_calls":32,
                "max_context_bytes":1048576,"max_output_bytes":1048576}
            }}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let execution = profile.daimon.unwrap().execution.unwrap();
        assert_eq!(execution.context.len(), 1);
        assert_eq!(execution.tools.len(), 1);
        assert_eq!(execution.budget.max_tool_calls, 32);
        assert_eq!(execution.task_attempt.task_incarnation, 1);

        let invalid_identity = String::from_utf8(valid.to_vec())
            .unwrap()
            .replace("10101010101010101010101010101010", "not-a-principal");
        assert!(WorkspaceProfile::parse(invalid_identity.as_bytes(), Path::new(".")).is_err());
        let zero_budget = String::from_utf8(valid.to_vec())
            .unwrap()
            .replace("\"max_output_tokens\":8192", "\"max_output_tokens\":0");
        assert!(WorkspaceProfile::parse(zero_budget.as_bytes(), Path::new(".")).is_err());
    }

    #[test]
    fn daimon_session_open_policy_allows_an_honestly_empty_initial_workset() {
        let valid = br#"{
          "version":1,"name":"Daimon open","surfaces":["daimon"],
          "daimon":{"agent_socket":"agent.sock",
            "agent_open":{"workspace":"15151515151515151515151515151515",
              "budget":{"max_elapsed_millis":60000,"max_turns":8,
                "max_events":128,"max_event_bytes":65536,
                "max_prompt_bytes":16384,"max_permission_requests":4}},
            "execution":{
              "principal":"10101010101010101010101010101010",
              "profile":"11111111111111111111111111111111",
              "task_attempt":{"task_key":"14141414141414141414141414141414",
                "task_incarnation":1,"attempt_number":1},
              "context":[],"tools":[],
              "budget":{"max_elapsed_millis":60000,"max_input_tokens":4096,
                "max_output_tokens":8192,"max_tool_calls":32,
                "max_context_bytes":1048576,"max_output_bytes":1048576}
            }}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let daimon = profile.daimon.unwrap();
        assert!(daimon.agent_sessions.is_empty());
        assert_eq!(daimon.agent_open.unwrap().budget.max_permission_requests, 4);

        let without_execution = String::from_utf8(valid.to_vec()).unwrap().replace(
            ",\n            \"execution\":{",
            ",\n            \"unused\":{",
        );
        assert!(WorkspaceProfile::parse(without_execution.as_bytes(), Path::new(".")).is_err());
        let zero_turns = String::from_utf8(valid.to_vec())
            .unwrap()
            .replace("\"max_turns\":8", "\"max_turns\":0");
        assert!(WorkspaceProfile::parse(zero_turns.as_bytes(), Path::new(".")).is_err());
    }

    #[test]
    fn hosted_moira_requires_only_opaque_owner_inputs() {
        let valid = br#"{
          "version":1,"name":"Moira host","surfaces":["moira"],
          "moira":{"deployment":"hosted","socket":"moira.sock",
            "host":{"executable":"bin/moira-saga-local","config":"owner.json"}}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let moira = profile.moira.unwrap();
        assert_eq!(moira.deployment, DeploymentMode::Hosted);
        let host = moira.host.unwrap();
        assert_eq!(
            host.executable,
            Path::new("/tmp/workbench/bin/moira-saga-local")
        );
        assert_eq!(host.config, Path::new("/tmp/workbench/owner.json"));

        let missing_host = br#"{
          "version":1,"name":"Moira host","surfaces":["moira"],
          "moira":{"deployment":"hosted","socket":"moira.sock"}
        }"#;
        assert!(WorkspaceProfile::parse(missing_host, Path::new(".")).is_err());
        let attached_host = br#"{
          "version":1,"name":"Moira attach","surfaces":["moira"],
          "moira":{"socket":"moira.sock",
            "host":{"executable":"moira-saga-local","config":"owner.json"}}
        }"#;
        assert!(WorkspaceProfile::parse(attached_host, Path::new(".")).is_err());
    }

    #[test]
    fn moira_participation_requires_an_exact_attached_pair() {
        let valid = br#"{
          "version":1,"name":"Moira participation","surfaces":["moira"],
          "moira":{"socket":"saga.sock",
            "participation_socket":"participation.sock",
            "participation":"0102030405060708090a0b0c0d0e0f10",
            "agent_worker":{"executable":"bin/moira-agent-local","config":"agent.json"},
            "requires":["agent.participation.inspect.v1"]}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let moira = profile.moira.unwrap();
        assert_eq!(
            moira.participation_socket.as_deref(),
            Some(Path::new("/tmp/workbench/participation.sock"))
        );
        assert_eq!(
            moira.participation.as_deref(),
            Some("0102030405060708090a0b0c0d0e0f10")
        );
        let worker = moira.agent_worker.unwrap();
        assert_eq!(
            worker.executable,
            Path::new("/tmp/workbench/bin/moira-agent-local")
        );
        assert_eq!(worker.config, Path::new("/tmp/workbench/agent.json"));

        let missing_id = br#"{
          "version":1,"name":"Moira participation","surfaces":["moira"],
          "moira":{"socket":"saga.sock","participation_socket":"participation.sock"}
        }"#;
        assert!(WorkspaceProfile::parse(missing_id, Path::new(".")).is_err());

        let malformed_id = br#"{
          "version":1,"name":"Moira participation","surfaces":["moira"],
          "moira":{"socket":"saga.sock","participation_socket":"participation.sock",
            "participation":"not-an-exact-id"}
        }"#;
        assert!(WorkspaceProfile::parse(malformed_id, Path::new(".")).is_err());

        let worker_without_participation = br#"{
          "version":1,"name":"Moira worker","surfaces":["moira"],
          "moira":{"socket":"saga.sock",
            "agent_worker":{"executable":"moira-agent-local","config":"agent.json"}}
        }"#;
        assert!(WorkspaceProfile::parse(worker_without_participation, Path::new(".")).is_err());
    }

    #[test]
    fn moira_deliberation_requires_an_exact_attached_pair() {
        let valid = br#"{
          "version":1,"name":"Moira deliberation","surfaces":["moira"],
          "moira":{"socket":"saga.sock",
            "deliberation_socket":"deliberation.sock",
            "deliberation":"0102030405060708090a0b0c0d0e0f10",
            "requires":["deliberation.inspect.v1"]}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let moira = profile.moira.unwrap();
        assert_eq!(
            moira.deliberation_socket.as_deref(),
            Some(Path::new("/tmp/workbench/deliberation.sock"))
        );
        assert_eq!(
            moira.deliberation.as_deref(),
            Some("0102030405060708090a0b0c0d0e0f10")
        );

        let missing_id = br#"{
          "version":1,"name":"Moira deliberation","surfaces":["moira"],
          "moira":{"socket":"saga.sock","deliberation_socket":"deliberation.sock"}
        }"#;
        assert!(WorkspaceProfile::parse(missing_id, Path::new(".")).is_err());

        let malformed_id = br#"{
          "version":1,"name":"Moira deliberation","surfaces":["moira"],
          "moira":{"socket":"saga.sock","deliberation_socket":"deliberation.sock",
            "deliberation":"not-an-exact-id"}
        }"#;
        assert!(WorkspaceProfile::parse(malformed_id, Path::new(".")).is_err());
    }

    #[test]
    fn moira_decision_packet_reader_requires_exact_bounded_vellum_route() {
        let valid = br#"{
          "version":1,"name":"Moira packet","surfaces":["moira","vellum"],
          "moira":{"deliberation_socket":"deliberation.sock",
            "deliberation":"0102030405060708090a0b0c0d0e0f10",
            "decision_packet_reader":{
              "owner_route":"11111111111111111111111111111111",
              "artifact_route":"22222222222222222222222222222222",
              "vellum_workspace":7,"vellum_document":9,"max_packet_bytes":4096}},
          "vellum":{"socket":"vellum.sock","credential":null}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let reader = profile
            .moira
            .unwrap()
            .decision_packet_reader
            .expect("selected packet reader");
        assert_eq!(reader.vellum_workspace, 7);
        assert_eq!(reader.vellum_document, 9);
        assert_eq!(reader.max_packet_bytes, 4096);

        let missing_vellum = br#"{
          "version":1,"name":"Moira packet","surfaces":["moira"],
          "moira":{"deliberation_socket":"deliberation.sock",
            "deliberation":"0102030405060708090a0b0c0d0e0f10",
            "decision_packet_reader":{
              "owner_route":"11111111111111111111111111111111",
              "artifact_route":"22222222222222222222222222222222",
              "vellum_workspace":7,"vellum_document":9,"max_packet_bytes":4096}}
        }"#;
        assert!(WorkspaceProfile::parse(missing_vellum, Path::new(".")).is_err());

        let oversized = valid
            .windows(b"4096".len())
            .position(|window| window == b"4096")
            .map(|offset| {
                let mut bytes = valid.to_vec();
                bytes.splice(offset..offset + 4, b"65537".iter().copied());
                bytes
            })
            .unwrap();
        assert!(WorkspaceProfile::parse(&oversized, Path::new(".")).is_err());
    }

    #[test]
    fn moira_human_gate_response_requires_packet_and_bounded_rationale_route() {
        let valid = br#"{
          "version":1,"name":"Moira response","surfaces":["moira","vellum"],
          "moira":{"deliberation_socket":"deliberation.sock",
            "deliberation":"0102030405060708090a0b0c0d0e0f10",
            "decision_packet_reader":{
              "owner_route":"11111111111111111111111111111111",
              "artifact_route":"22222222222222222222222222222222",
              "vellum_workspace":7,"vellum_document":9,"max_packet_bytes":4096},
            "human_gate_response":{
              "rationale_owner_route":"33333333333333333333333333333333",
              "rationale_artifact_route":"44444444444444444444444444444444",
              "vellum_workspace":7,"vellum_document":10,"max_rationale_bytes":4096}},
          "vellum":{"socket":"vellum.sock","credential":null}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let response = profile
            .moira
            .unwrap()
            .human_gate_response
            .expect("selected human response");
        assert_eq!(response.vellum_document, 10);
        assert_eq!(response.max_rationale_bytes, 4096);

        let missing_packet = br#"{
          "version":1,"name":"Moira response","surfaces":["moira","vellum"],
          "moira":{"deliberation_socket":"deliberation.sock",
            "deliberation":"0102030405060708090a0b0c0d0e0f10",
            "human_gate_response":{
              "rationale_owner_route":"33333333333333333333333333333333",
              "rationale_artifact_route":"44444444444444444444444444444444",
              "vellum_workspace":7,"vellum_document":10,"max_rationale_bytes":4096}},
          "vellum":{"socket":"vellum.sock","credential":null}
        }"#;
        assert!(WorkspaceProfile::parse(missing_packet, Path::new(".")).is_err());
    }

    #[test]
    fn hosted_hermes_requires_explicit_owner_inputs_and_canonical_subject() {
        let valid = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"deployment":"hosted","socket":"hermes.sock","stream":null,"consumer":null,
            "host":{"executable":"bin/hermes-daemon","directory":"ledger","mode":"create",
              "effective_uid":501,"effective_gid":20,"subject":"07070707070707070707070707070707",
              "access":"observer","operator_stream":null}}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let host = profile.hermes.unwrap().host.unwrap();
        assert_eq!(
            host.executable,
            Path::new("/tmp/workbench/bin/hermes-daemon")
        );
        assert_eq!(host.directory, Path::new("/tmp/workbench/ledger"));

        let missing_host = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"deployment":"hosted","socket":"hermes.sock","stream":null,"consumer":null}
        }"#;
        assert!(WorkspaceProfile::parse(missing_host, Path::new(".")).is_err());
        let attached_host = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"socket":"hermes.sock","stream":null,"consumer":null,
            "host":{"executable":"hermes-daemon","directory":"ledger","mode":"open",
              "effective_uid":501,"effective_gid":20,"subject":"07070707070707070707070707070707",
              "access":"observer","operator_stream":null}}
        }"#;
        assert!(WorkspaceProfile::parse(attached_host, Path::new(".")).is_err());

        let operator = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"deployment":"hosted","socket":"hermes.sock",
            "stream":"05050505050505050505050505050505","consumer":null,
            "host":{"executable":"hermes-daemon","directory":"ledger","mode":"open",
              "effective_uid":501,"effective_gid":20,"subject":"07070707070707070707070707070707",
              "access":"operator","operator_stream":"05050505050505050505050505050505"}}
        }"#;
        let profile = WorkspaceProfile::parse(operator, Path::new(".")).unwrap();
        assert_eq!(
            profile.hermes.unwrap().host.unwrap().access,
            HermesAccessProfile::Operator
        );
        let consumer = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"deployment":"hosted","socket":"hermes.sock",
            "stream":"05050505050505050505050505050505",
            "consumer":"06060606060606060606060606060606",
            "host":{"executable":"hermes-daemon","directory":"ledger","mode":"open",
              "effective_uid":501,"effective_gid":20,"subject":"07070707070707070707070707070707",
              "access":"consumer","operator_stream":null,
              "consumer_stream":"05050505050505050505050505050505"}}
        }"#;
        let profile = WorkspaceProfile::parse(consumer, Path::new(".")).unwrap();
        assert_eq!(
            profile.hermes.unwrap().host.unwrap().access,
            HermesAccessProfile::Consumer
        );
        let ambiguous_consumer = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"deployment":"hosted","socket":"hermes.sock",
            "stream":"05050505050505050505050505050505",
            "consumer":"06060606060606060606060606060606",
            "host":{"executable":"hermes-daemon","directory":"ledger","mode":"open",
              "effective_uid":501,"effective_gid":20,"subject":"07070707070707070707070707070707",
              "access":"consumer",
              "operator_stream":"05050505050505050505050505050505",
              "consumer_stream":"05050505050505050505050505050505"}}
        }"#;
        assert!(WorkspaceProfile::parse(ambiguous_consumer, Path::new(".")).is_err());
        let mismatched_operator = br#"{
          "version":1,"name":"Hermes","surfaces":["hermes"],
          "hermes":{"deployment":"hosted","socket":"hermes.sock",
            "stream":"06060606060606060606060606060606","consumer":null,
            "host":{"executable":"hermes-daemon","directory":"ledger","mode":"open",
              "effective_uid":501,"effective_gid":20,"subject":"07070707070707070707070707070707",
              "access":"operator","operator_stream":"05050505050505050505050505050505"}}
        }"#;
        assert!(WorkspaceProfile::parse(mismatched_operator, Path::new(".")).is_err());
    }

    #[test]
    fn hosted_agora_requires_only_opaque_owner_inputs() {
        let valid = br#"{
          "version":1,"name":"Agora","surfaces":["agora"],
          "agora":{"deployment":"hosted","transport":"composition","socket":"agora.sock","participant":"alice",
            "host":{"executable":"bin/agora-private-local","config":"private/agora.conf"}}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let agora = profile.agora.unwrap();
        let host = agora.host.unwrap();
        assert_eq!(agora.deployment, DeploymentMode::Hosted);
        assert_eq!(agora.transport, AgoraTransportProfile::Composition);
        assert_eq!(
            host.executable,
            Path::new("/tmp/workbench/bin/agora-private-local")
        );
        assert_eq!(host.config, Path::new("/tmp/workbench/private/agora.conf"));

        let missing_host = br#"{
          "version":1,"name":"Agora","surfaces":["agora"],
          "agora":{"deployment":"hosted","socket":"agora.sock","participant":"alice"}
        }"#;
        assert!(WorkspaceProfile::parse(missing_host, Path::new(".")).is_err());
        let attached_host = br#"{
          "version":1,"name":"Agora","surfaces":["agora"],
          "agora":{"socket":"agora.sock","participant":"alice",
            "host":{"executable":"agora-private-local","config":"agora.conf"}}
        }"#;
        assert!(WorkspaceProfile::parse(attached_host, Path::new(".")).is_err());
    }

    #[test]
    fn hosted_aegis_requires_explicit_owner_inputs_and_canonical_authority_facts() {
        let valid = br#"{
          "version":1,"name":"Aegis","surfaces":["aegis"],
          "aegis":{"deployment":"hosted","socket":"aegis.sock",
            "host":{"executable":"bin/aegis-daemon","directory":"authority","mode":"create",
              "authority":"01010101010101010101010101010101",
              "effective_uid":501,"effective_gid":20,
              "principal":"02020202020202020202020202020202",
              "session_namespace":"0303030303030303","role":"administrator"}}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let host = profile.aegis.unwrap().host.unwrap();
        assert_eq!(
            host.executable,
            Path::new("/tmp/workbench/bin/aegis-daemon")
        );
        assert_eq!(host.directory, Path::new("/tmp/workbench/authority"));
        assert_eq!(host.role, AegisRole::Administrator);

        let missing_host = br#"{
          "version":1,"name":"Aegis","surfaces":["aegis"],
          "aegis":{"deployment":"hosted","socket":"aegis.sock"}
        }"#;
        assert!(WorkspaceProfile::parse(missing_host, Path::new(".")).is_err());
        let invalid_authority = br#"{
          "version":1,"name":"Aegis","surfaces":["aegis"],
          "aegis":{"deployment":"hosted","socket":"aegis.sock",
            "host":{"executable":"aegis-daemon","directory":"authority","mode":"open",
              "authority":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
              "effective_uid":501,"effective_gid":20,
              "principal":"02020202020202020202020202020202",
              "session_namespace":"0303030303030303","role":"principal"}}
        }"#;
        assert!(WorkspaceProfile::parse(invalid_authority, Path::new(".")).is_err());
    }

    #[test]
    fn profile_refuses_ambiguous_or_unresolved_peer_bindings() {
        let duplicate_workstream = br#"{
          "version":1,"name":"x","surfaces":["workspace"],"agora":null,
          "chronos":{"workstreams":[{"name":"x","directory":"a"},{"name":"x","directory":"b"}],"moira":[],"daimon":[],"planning":null},
          "vellum":null
        }"#;
        assert!(WorkspaceProfile::parse(duplicate_workstream, Path::new(".")).is_err());
        let missing_workstream = br#"{
          "version":1,"name":"x","surfaces":["workspace"],"agora":null,
          "chronos":{"workstreams":[{"name":"x","directory":"a"}],"moira":[{"workstream":"y","socket":"m.sock","scope":"s"}],"daimon":[],"planning":null},
          "vellum":null
        }"#;
        assert!(WorkspaceProfile::parse(missing_workstream, Path::new(".")).is_err());
        let hidden_agora = br#"{
          "version":1,"name":"x","surfaces":["overview"],
          "agora":{"socket":"a.sock","participant":"alice"},
          "chronos":null,"vellum":null
        }"#;
        assert!(WorkspaceProfile::parse(hidden_agora, Path::new(".")).is_err());
    }

    #[test]
    fn trusted_chronos_worker_selects_one_hosted_owner_without_serializing_authority() {
        let valid = br#"{
          "version":1,"name":"Trusted worker","surfaces":["chronos"],
          "chronos":{"deployment":"hosted",
            "workstreams":[{"name":"Product","directory":"chronos"}],
            "moira":[],"daimon":[],
            "trusted_worker":{"workstream":"Product","agent":"product-manager",
              "grant_validity_seconds":60,"lease_duration_seconds":30},
            "planning":null}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/workbench")).unwrap();
        let worker = profile.chronos.unwrap().trusted_worker.unwrap();
        assert_eq!(worker.workstream, "Product");
        assert_eq!(worker.agent, "product-manager");

        let attached = String::from_utf8(valid.to_vec())
            .unwrap()
            .replace("\"deployment\":\"hosted\",", "");
        assert!(WorkspaceProfile::parse(attached.as_bytes(), Path::new(".")).is_err());
        let missing_owner = String::from_utf8(valid.to_vec())
            .unwrap()
            .replace("\"workstream\":\"Product\"", "\"workstream\":\"Other\"");
        assert!(WorkspaceProfile::parse(missing_owner.as_bytes(), Path::new(".")).is_err());
        let zero_lease = String::from_utf8(valid.to_vec()).unwrap().replace(
            "\"lease_duration_seconds\":30",
            "\"lease_duration_seconds\":0",
        );
        assert!(WorkspaceProfile::parse(zero_lease.as_bytes(), Path::new(".")).is_err());
    }

    #[test]
    fn task_conversation_agent_addressee_is_explicit_and_bounded() {
        let valid = br#"{
          "version":1,"name":"Addressed message","surfaces":["chronos","agora"],
          "agora":{"socket":"a.sock","participant":"operator"},
          "task_conversation":{"socket":"links.sock",
            "principal":"01010101010101010101010101010101",
            "agent_addressee":"local-agent"},
          "chronos":{"workstreams":[{"name":"work","directory":"work"}],
            "requires":[],"moira":[],"daimon":[],"planning":null}
        }"#;
        let profile = WorkspaceProfile::parse(valid, Path::new("/tmp/addressed")).unwrap();
        assert_eq!(
            profile
                .task_conversation
                .and_then(|profile| profile.agent_addressee),
            Some("local-agent".into())
        );

        let invalid = String::from_utf8(valid.to_vec())
            .unwrap()
            .replace("local-agent", "");
        assert!(WorkspaceProfile::parse(invalid.as_bytes(), Path::new(".")).is_err());

        let aliased = String::from_utf8(valid.to_vec())
            .unwrap()
            .replace("local-agent", "operator");
        assert!(WorkspaceProfile::parse(aliased.as_bytes(), Path::new(".")).is_err());

        let routed = br#"{
          "version":1,"name":"Shared agent context","surfaces":["chronos","agora","daimon"],
          "agora":{"socket":"a.sock","participant":"operator"},
          "task_conversation":{"socket":"links.sock",
            "principal":"01010101010101010101010101010101",
            "agent_routes":[
              {"addressee":"sre-agent","session":"07070707070707070707070707070707"},
              {"addressee":"project-agent","session":"08080808080808080808080808080808"}]},
          "chronos":{"workstreams":[{"name":"work","directory":"work"}],
            "requires":[],"moira":[],"daimon":[],"planning":null},
          "daimon":{"agent_socket":"agent.sock","agent_sessions":[
            "07070707070707070707070707070707",
            "08080808080808080808080808080808"]}
        }"#;
        let profile = WorkspaceProfile::parse(routed, Path::new("/tmp/shared")).unwrap();
        assert_eq!(profile.task_conversation.unwrap().agent_routes.len(), 2);

        let missing_session = String::from_utf8(routed.to_vec()).unwrap().replace(
            "08080808080808080808080808080808\"}]},",
            "09090909090909090909090909090909\"}]},",
        );
        assert!(WorkspaceProfile::parse(missing_session.as_bytes(), Path::new(".")).is_err());
    }

    #[test]
    fn profile_refuses_invalid_or_duplicate_capability_requirements() {
        let invalid = br#"{
          "version":1,"name":"x","surfaces":["agora"],
          "agora":{"socket":"a.sock","participant":"alice","requires":["Rooms Read"]},
          "chronos":null,"vellum":null
        }"#;
        assert!(WorkspaceProfile::parse(invalid, Path::new(".")).is_err());
        let duplicate = br#"{
          "version":1,"name":"x","surfaces":["agora"],
          "agora":{"socket":"a.sock","participant":"alice","requires":["rooms.read.v2","rooms.read.v2"]},
          "chronos":null,"vellum":null
        }"#;
        assert!(WorkspaceProfile::parse(duplicate, Path::new(".")).is_err());
    }

    #[test]
    fn profile_accepts_published_hyphenated_capability_identifiers() {
        let profile = WorkspaceProfile::parse(
            br#"{
              "version":1,"name":"Hyphenated capability","surfaces":["workspace"],
              "agora":null,"chronos":null,
              "vellum":{"socket":"vellum.sock","credential":null,"requires":["workspace.durable-save.execute.v1"]}
            }"#,
            Path::new("/profiles"),
        )
        .unwrap();
        assert_eq!(
            profile.vellum.unwrap().requires,
            ["workspace.durable-save.execute.v1"]
        );
    }

    #[test]
    fn profile_requires_hosted_vellum_roots_and_rejects_attached_roots() {
        let hosted_without_roots = br#"{
          "version":1,"name":"x","surfaces":["workspace"],"agora":null,"chronos":null,
          "vellum":{"deployment":"hosted","socket":"v.sock","credential":null,"roots":[]}
        }"#;
        assert!(WorkspaceProfile::parse(hosted_without_roots, Path::new(".")).is_err());
        let attached_with_roots = br#"{
          "version":1,"name":"x","surfaces":["workspace"],"agora":null,"chronos":null,
          "vellum":{"deployment":"attach","socket":"v.sock","credential":null,"roots":["a"]}
        }"#;
        assert!(WorkspaceProfile::parse(attached_with_roots, Path::new(".")).is_err());
        let attached_with_host_authority = br#"{
          "version":1,"name":"x","surfaces":["vellum"],"agora":null,"chronos":null,
          "vellum":{"deployment":"attach","socket":"v.sock","credential":null,"allow_add_file":true}
        }"#;
        assert!(WorkspaceProfile::parse(attached_with_host_authority, Path::new(".")).is_err());
    }
}
