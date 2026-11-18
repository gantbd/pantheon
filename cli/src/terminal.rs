use chronos_tui::{Hub, Lifecycle, WorkbenchIntent};
use crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags, MouseButton,
    MouseEventKind, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use daimon_agent_command::{
    AdvanceRequest, CancelRequest, CloseRequest, PermissionRequest, PromptRequest, ReattachRequest,
};
use daimon_protocol::{
    AgentTurnIdV1, ContextReferenceIdV1, ExecuteRequestV1, ExecutionBudgetV1, ExecutionIdV1,
    HarnessProfileIdV1, IdempotencyKeyV1, PrincipalIdV1, TaskAttemptRefV1, ToolCapabilityIdV1,
    agent::{AgentEventV1, AgentSessionPhaseV1},
};
use moira_protocol::{OperatorDispositionV1, ReconciliationProposalV1};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, TryRecvError, sync_channel};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use vellum_sdk_client::{
    protocol::{DurableOperationRefV1, ServicePort},
    transport::{
        AuthenticatedUnixConnector, FrameExchange, RemoteTransport, TransportError,
        TransportLimits, UnixConnector,
    },
};
use vellum_tui::{
    AtomicReplaceComponent, AtomicReplaceDisposition, AtomicReplaceInput, AtomicReplaceLifecycle,
    DurableSaveBatchComponent, DurableSaveBatchDisposition, DurableSaveBatchInput,
    DurableSaveTargetV1, PromptInput, ServiceAction, ServiceEditor, ServiceInput, TextPrompt,
    translate_service_input,
};

use crate::capabilities::GUARDED_CONTROL_ROUTES;
use crate::native_host::NativeHost;
use crate::profile::{AgoraTransportProfile, DaimonExecutionPolicyProfile, WorkspaceProfile};
#[cfg(test)]
use crate::provider::{
    ContextCatalogProjection, TaskBindingCatalogProjection, validate_provider_catalog,
};
use crate::provider::{
    ProviderCatalogEntryProjection, ProviderCatalogProjection, ProviderQualificationProjection,
    invoke_provider_owner, provider_catalog, qualify_provider, valid_digest, valid_id,
    valid_provider_catalog_entry, validate_provider_qualification,
};
use crate::reconfiguration::{self, ReconfigurationPlan};
use crate::registry::{HostedPeerId, RegistryLifecycle, ShutdownDisposition, TopologyShutdown};
use pantheon::{
    ActivityLevel, ActivityLog, AgendaEntry, AgendaKind, AgendaUrgency, CommandKind,
    CommandPalette, CommandSpec, DaimonModelView, ExitDecision, Focus, Instrumentation,
    OperationsReceiptView, OperationsView, OperatorAgendaView, PaletteOutcome, PeerHealthState,
    PeerHealthView, PeerViews, RoutedInput, Shell, ShellInput, SuiteHealthView, Surface,
    decide_exit, draw_activity, draw_command_palette, draw_help, draw_inspector,
    draw_shell_with_moira, route_key, shell_input, surface_tab_at,
};

struct InputCapture {
    enhanced_keyboard: bool,
}

impl InputCapture {
    fn enable() -> std::io::Result<Self> {
        let enhanced_keyboard = crossterm::terminal::supports_keyboard_enhancement()?;
        if enhanced_keyboard {
            crossterm::execute!(
                std::io::stdout(),
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                )
            )?;
        }
        crossterm::execute!(std::io::stdout(), EnableBracketedPaste, EnableMouseCapture)?;
        Ok(Self { enhanced_keyboard })
    }
}

impl Drop for InputCapture {
    fn drop(&mut self) {
        let _ = crossterm::execute!(
            std::io::stdout(),
            DisableMouseCapture,
            DisableBracketedPaste
        );
        if self.enhanced_keyboard {
            let _ = crossterm::execute!(std::io::stdout(), PopKeyboardEnhancementFlags);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkbenchOverlay {
    None,
    Commands,
    Help,
    Activity,
    Configuration,
    LocalProviders,
    ProviderEdit,
    ProviderQualification,
    AcpSessions,
    AgoraRooms,
    Reconfiguration,
    Instrumentation,
}

/// One host-owned behavioral decision for a key event. Peer-specific key
/// translation happens only after this facade returns `SurfaceLocal`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KeyBehavior {
    SafeExit,
    CommandPaletteInput,
    ProviderEditInput,
    ProviderQualificationInput,
    ReconfigurationInput,
    DismissOverlay,
    Suppress,
    OpenCommands,
    ExecuteCommand(&'static str),
    CommandUnavailable(&'static str),
    Navigate(ShellInput),
    SurfaceLocal,
}

struct BehaviorFacade;

impl BehaviorFacade {
    fn key(
        key: KeyEvent,
        overlay: WorkbenchOverlay,
        surface: Surface,
        vellum_active: bool,
    ) -> KeyBehavior {
        if is_control_char(key, 'q') {
            return KeyBehavior::SafeExit;
        }
        match overlay {
            WorkbenchOverlay::Commands => return KeyBehavior::CommandPaletteInput,
            WorkbenchOverlay::Reconfiguration => return KeyBehavior::ReconfigurationInput,
            WorkbenchOverlay::ProviderEdit => {
                return if is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    || is_key(key, KeyCode::F(1), KeyModifiers::NONE)
                    || is_control_char(key, 'g')
                {
                    KeyBehavior::DismissOverlay
                } else {
                    KeyBehavior::ProviderEditInput
                };
            }
            WorkbenchOverlay::ProviderQualification => {
                return if is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    || is_key(key, KeyCode::F(1), KeyModifiers::NONE)
                    || is_control_char(key, 'g')
                {
                    KeyBehavior::DismissOverlay
                } else {
                    KeyBehavior::ProviderQualificationInput
                };
            }
            WorkbenchOverlay::LocalProviders
                if is_key(key, KeyCode::Char('r'), KeyModifiers::NONE) =>
            {
                return KeyBehavior::ExecuteCommand("configuration.providers.refresh");
            }
            WorkbenchOverlay::LocalProviders => {
                return if is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    || is_key(key, KeyCode::F(1), KeyModifiers::NONE)
                    || is_control_char(key, 'g')
                {
                    KeyBehavior::DismissOverlay
                } else {
                    KeyBehavior::Suppress
                };
            }
            WorkbenchOverlay::Help
            | WorkbenchOverlay::Activity
            | WorkbenchOverlay::Configuration
            | WorkbenchOverlay::AcpSessions
            | WorkbenchOverlay::AgoraRooms
            | WorkbenchOverlay::Instrumentation => {
                return if is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    || is_key(key, KeyCode::F(1), KeyModifiers::NONE)
                    || is_control_char(key, 'g')
                {
                    KeyBehavior::DismissOverlay
                } else {
                    KeyBehavior::Suppress
                };
            }
            WorkbenchOverlay::None => {}
        }
        if is_control_char(key, 'p') {
            KeyBehavior::OpenCommands
        } else if is_key(key, KeyCode::F(1), KeyModifiers::NONE) {
            KeyBehavior::ExecuteCommand("workbench.help")
        } else if is_control_char(key, 'g') {
            KeyBehavior::ExecuteCommand("workbench.activity")
        } else if is_key(key, KeyCode::Char('r'), KeyModifiers::NONE)
            && matches!(
                surface,
                Surface::Overview | Surface::Agenda | Surface::Operations | Surface::Configuration
            )
        {
            KeyBehavior::ExecuteCommand("view.refresh")
        } else if vellum_active && is_control_char(key, 'o') {
            KeyBehavior::ExecuteCommand("vellum.document.open")
        } else if vellum_active && is_control_char(key, 's') {
            KeyBehavior::ExecuteCommand("vellum.save")
        } else if surface == Surface::Chronos
            && is_key(key, KeyCode::Char('N'), KeyModifiers::SHIFT)
        {
            KeyBehavior::ExecuteCommand("workitem.agent.new")
        } else if let Some(input) = shell_input(key) {
            match input {
                ShellInput::ActivateSurface(surface) => {
                    KeyBehavior::ExecuteCommand(Self::surface_command(surface))
                }
                // Vellum is a text workspace. Once it owns focus, printable
                // bracket characters belong to the document instead of the
                // workbench rail. Direct Ctrl/Alt surface shortcuts, the
                // command palette, and mouse navigation remain available.
                ShellInput::NextSurface | ShellInput::PreviousSurface if vellum_active => {
                    KeyBehavior::SurfaceLocal
                }
                _ => KeyBehavior::Navigate(input),
            }
        } else {
            KeyBehavior::SurfaceLocal
        }
    }

    fn palette(outcome: PaletteOutcome) -> KeyBehavior {
        match outcome {
            PaletteOutcome::None => KeyBehavior::Suppress,
            PaletteOutcome::Dismiss => KeyBehavior::DismissOverlay,
            PaletteOutcome::Execute(action) => KeyBehavior::ExecuteCommand(action),
            PaletteOutcome::Unavailable(reason) => KeyBehavior::CommandUnavailable(reason),
        }
    }

    const fn surface_command(surface: Surface) -> &'static str {
        match surface {
            Surface::Overview => "navigate.overview",
            Surface::Agenda => "navigate.agenda",
            Surface::Chronos => "navigate.chronos",
            Surface::Vellum => "navigate.vellum",
            Surface::Workspace => "navigate.workspace",
            Surface::Agora => "navigate.agora",
            Surface::Moira => "navigate.moira",
            Surface::Daimon => "navigate.daimon",
            Surface::Tellus => "navigate.tellus",
            Surface::Aegis => "navigate.aegis",
            Surface::Hermes => "navigate.hermes",
            Surface::Operations => "navigate.operations",
            Surface::Configuration => "navigate.configuration",
        }
    }

    fn navigate(shell: &mut Shell, input: ShellInput) {
        match input {
            ShellInput::NextSurface => shell.next(),
            ShellInput::PreviousSurface => shell.previous(),
            ShellInput::Activate(index) => {
                let _ = shell.activate_index(index);
            }
            ShellInput::ActivateSurface(surface) => {
                let _ = shell.activate(surface);
            }
        }
    }
}

const MAX_RECONFIGURATION_PATH_BYTES: usize = 4096;

struct ReconfigurationWorkflow {
    current: WorkspaceProfile,
    path: String,
    next_path: Option<PathBuf>,
    next_digest: Option<String>,
    plan: Option<ReconfigurationPlan>,
    armed: bool,
    notice: String,
}

impl ReconfigurationWorkflow {
    fn new(current: WorkspaceProfile) -> Self {
        Self {
            current,
            path: String::new(),
            next_path: None,
            next_digest: None,
            plan: None,
            armed: false,
            notice: "Enter a local workspace-profile path, then press Enter to preflight".into(),
        }
    }

    fn insert(&mut self, text: &str) {
        if self.plan.is_some() {
            return;
        }
        for character in text.chars().filter(|character| !character.is_control()) {
            if self.path.len() + character.len_utf8() > MAX_RECONFIGURATION_PATH_BYTES {
                self.notice = "Profile path reached the 4096-byte bound".into();
                break;
            }
            self.path.push(character);
        }
    }

    fn backspace(&mut self) {
        if self.plan.is_none() {
            self.path.pop();
        }
    }

    fn edit(&mut self) {
        self.next_path = None;
        self.next_digest = None;
        self.plan = None;
        self.armed = false;
        self.notice = "Edit the profile path, then press Enter to preflight again".into();
    }

    fn preflight(&mut self) {
        if self.path.is_empty() {
            self.notice = "A workspace-profile path is required".into();
            return;
        }
        let path = PathBuf::from(&self.path);
        match WorkspaceProfile::load_with_digest(&path) {
            Ok((next, digest)) => {
                let operation_id = format!(
                    "tui-reconfigure-{}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                );
                let plan = reconfiguration::plan(operation_id, &self.current, &next);
                self.notice = match plan.minimum_action {
                    "none" => "Preflight settled: profiles are equivalent; no restart needed",
                    "shell_restart" => "Preflight settled: shell restart required; press c to arm",
                    _ => "Preflight settled: full local topology restart required; press c to arm",
                }
                .into();
                self.next_path = Some(path);
                self.next_digest = Some(digest);
                self.plan = Some(plan);
                self.armed = false;
            }
            Err(error) => {
                self.next_path = None;
                self.next_digest = None;
                self.plan = None;
                self.armed = false;
                self.notice = format!("Profile preflight refused: {error}");
            }
        }
    }

    fn arm(&mut self) {
        if self
            .plan
            .as_ref()
            .is_some_and(|plan| plan.minimum_action != "none")
        {
            self.armed = true;
            self.notice = "ARMED: Enter drains owned topology and restarts this process with the preflighted profile".into();
        } else {
            self.notice = "A changed, valid profile must be preflighted before restart".into();
        }
    }

    fn confirmed_profile(&self) -> Option<(PathBuf, String)> {
        if !self.armed {
            return None;
        }
        self.next_path.clone().zip(self.next_digest.clone())
    }

    fn lines(&self) -> Vec<String> {
        let mut lines = vec![
            "scope               single user · local machine".into(),
            format!("next profile        {}", self.path),
            format!(
                "preflight           {}",
                if self.plan.is_some() {
                    "settled"
                } else {
                    "not run"
                }
            ),
        ];
        if let Some(plan) = &self.plan {
            lines.push(format!("operation            {}", plan.operation_id));
            lines.push(format!("minimum action       {}", plan.minimum_action));
            lines.push("live apply           unsupported; process restart is explicit".into());
            lines.push(String::new());
            if plan.changes.is_empty() {
                lines.push("No configuration changes.".into());
            } else {
                lines.push(format!("changes              {}", plan.changes.len()));
                lines.extend(plan.changes.iter().map(|change| {
                    format!(
                        "{} · {} · {}",
                        change.target, change.change, change.required_action
                    )
                }));
            }
        }
        lines.push(String::new());
        lines.push(self.notice.clone());
        lines.push(if self.plan.is_some() {
            "e edit path · c arm · Enter confirm armed restart · Esc close".into()
        } else {
            "type/paste path · Enter preflight · Esc close".into()
        });
        lines
    }
}

enum ChronosOperation {
    Task(u64),
    Scheduling(u64),
}

#[derive(Default)]
enum WorkItemAgentJourney {
    #[default]
    Idle,
    EditingTask {
        draft: String,
        lens: Option<AgentCollaborationLens>,
    },
    AwaitingTaskReceipt {
        draft: String,
        lens: Option<AgentCollaborationLens>,
        task_id: String,
    },
    AwaitingTaskBinding,
    ReviewingPrompt,
    Invoking,
    ConversationReady,
    Failed,
}

impl WorkItemAgentJourney {
    fn capture_task_key(&mut self, key: KeyEvent) {
        let Self::EditingTask { draft, .. } = self else {
            return;
        };
        match key.code {
            KeyCode::Backspace if key.modifiers == KeyModifiers::NONE => {
                draft.pop();
            }
            KeyCode::Tab if key.modifiers == KeyModifiers::NONE => {
                if draft.len() < 8_192 {
                    draft.push('\n');
                }
            }
            _ => {
                if let Some(character) = bounded_text_character(key)
                    && draft.len() + character.len_utf8() <= 8_192
                {
                    draft.push(character);
                }
            }
        }
    }

    fn task_submitted(&mut self) {
        if let Self::EditingTask { draft, lens } = std::mem::take(self) {
            let task_id = single_work_item_task_id(&draft)
                .expect("the behavior facade admits exactly one task before dispatch")
                .to_owned();
            *self = Self::AwaitingTaskReceipt {
                draft,
                lens,
                task_id,
            };
        }
    }

    fn task_id_for_submission(&self) -> Result<&str, &'static str> {
        let Self::EditingTask { draft, .. } = self else {
            return Ok("");
        };
        single_work_item_task_id(draft)
    }
}

fn single_work_item_task_id(draft: &str) -> Result<&str, &'static str> {
    let mut ids = draft.split('\n');
    let Some(id) = ids.next().filter(|id| !id.is_empty()) else {
        return Err("enter one Chronos task ID before continuing");
    };
    if ids.next().is_some() {
        return Err(
            "this journey accepts one task; use ordinary Chronos batch creation for several tasks",
        );
    }
    if id.len() > 256 {
        return Err("the Chronos task ID exceeds the 256-byte owner bound");
    }
    Ok(id)
}

fn work_item_agent_prompt(draft: &str, lens: Option<AgentCollaborationLens>) -> String {
    let work = draft.trim();
    let work = if work.is_empty() {
        "Review the newly created Chronos work item and ask the operator for its missing details."
            .to_owned()
    } else {
        work.to_owned()
    };
    if let Some(lens) = lens {
        format!(
            "{}\n\nWork item:\n{work}",
            agent_collaboration_lens_prompt(lens)
        )
    } else {
        work
    }
}

#[derive(Clone, Copy)]
struct AgentCollaborationLens {
    action: &'static str,
    new_task_action: &'static str,
    name: &'static str,
    title: &'static str,
    new_task_title: &'static str,
    guidance: &'static str,
}

const AGENT_COLLABORATION_LENSES: [AgentCollaborationLens; 4] = [
    AgentCollaborationLens {
        action: "workitem.agent.lens.executive",
        new_task_action: "workitem.agent.new.executive",
        name: "Executive sponsor",
        title: "Shift local agent to executive sponsor lens",
        new_task_title: "Create task with executive sponsor agent lens",
        guidance: "Adopt an executive-sponsor collaboration lens for this turn: connect the work to outcomes, material risk, sponsorship, decision ownership, and explicit human go/no-go gates.",
    },
    AgentCollaborationLens {
        action: "workitem.agent.lens.product-owner",
        new_task_action: "workitem.agent.new.product-owner",
        name: "Product owner",
        title: "Shift local agent to product owner lens",
        new_task_title: "Create task with product owner agent lens",
        guidance: "Adopt a product-owner collaboration lens for this turn: advocate for affected users, clarify value and acceptance evidence, expose tradeoffs, and propose ordering without silently committing priority.",
    },
    AgentCollaborationLens {
        action: "workitem.agent.lens.project-manager",
        new_task_action: "workitem.agent.new.project-manager",
        name: "Project manager",
        title: "Shift local agent to project manager lens",
        new_task_title: "Create task with project manager agent lens",
        guidance: "Adopt a project-manager collaboration lens for this turn: make dependencies, owners, sequencing, delivery risk, status, and next coordination actions explicit.",
    },
    AgentCollaborationLens {
        action: "workitem.agent.lens.ic-collaborator",
        new_task_action: "workitem.agent.new.ic-collaborator",
        name: "IC collaborator",
        title: "Shift local agent to IC collaborator lens",
        new_task_title: "Create task with IC collaborator agent lens",
        guidance: "Adopt an individual-contributor collaboration lens for this turn: work concretely with the operator, inspect evidence, propose and test implementation details, and surface uncertainty early.",
    },
];

fn agent_collaboration_lens(action: &str) -> Option<AgentCollaborationLens> {
    AGENT_COLLABORATION_LENSES
        .iter()
        .copied()
        .find(|lens| lens.action == action)
}

fn agent_task_collaboration_lens(action: &str) -> Option<AgentCollaborationLens> {
    AGENT_COLLABORATION_LENSES
        .iter()
        .copied()
        .find(|lens| lens.new_task_action == action)
}

fn agent_collaboration_lens_prompt(lens: AgentCollaborationLens) -> String {
    format!(
        "{}\n\nBoundary: this is a reviewable collaboration lens for one agent turn. It is not a human identity, organizational title, work-mode activation, delegation, authority, approval, or sign-off. Preserve the actual principal and accountable human, and ask before crossing any human gate.",
        lens.guidance
    )
}

#[derive(Default)]
struct RefreshOperations {
    chronos: Option<u64>,
    agora: Option<u64>,
    moira: Option<u64>,
    daimon: Option<u64>,
    hermes: Option<u64>,
    aegis: Option<u64>,
    tellus: Option<u64>,
}

struct HealthChannel {
    peer: &'static str,
    channel: &'static str,
    state: PeerHealthState,
    settled_at: Option<Instant>,
}

struct SuiteHealthLedger {
    channels: Vec<HealthChannel>,
}

impl SuiteHealthLedger {
    fn new(channels: impl IntoIterator<Item = (&'static str, &'static str)>) -> Self {
        Self {
            channels: channels
                .into_iter()
                .map(|(peer, channel)| HealthChannel {
                    peer,
                    channel,
                    state: PeerHealthState::Unobserved,
                    settled_at: None,
                })
                .collect(),
        }
    }

    fn observing(&mut self, peer: &str, channel: &str, pending: bool) {
        if pending
            && let Some(entry) = self
                .channels
                .iter_mut()
                .find(|entry| entry.peer == peer && entry.channel == channel)
        {
            entry.state = PeerHealthState::Observing;
        }
    }

    fn settle(&mut self, peer: &str, channel: &str, settlement: Option<Result<(), ()>>) {
        let Some(settlement) = settlement else {
            return;
        };
        if let Some(entry) = self
            .channels
            .iter_mut()
            .find(|entry| entry.peer == peer && entry.channel == channel)
        {
            entry.state = if settlement.is_ok() {
                PeerHealthState::Current
            } else {
                PeerHealthState::Degraded
            };
            entry.settled_at = Some(Instant::now());
        }
    }

    fn current(&mut self, peer: &str, channel: &str) {
        self.settle(peer, channel, Some(Ok(())));
    }

    fn view(
        &self,
        busy: impl Fn(&str) -> bool,
        uncertain: impl Fn(&str) -> bool,
    ) -> SuiteHealthView {
        SuiteHealthView {
            peers: self
                .channels
                .iter()
                .map(|entry| PeerHealthView {
                    peer: entry.peer,
                    channel: entry.channel,
                    state: entry.state,
                    age_millis: entry.settled_at.map(|settled| {
                        u64::try_from(settled.elapsed().as_millis()).unwrap_or(u64::MAX)
                    }),
                    busy: busy(entry.peer),
                    uncertain: uncertain(entry.peer),
                })
                .collect(),
        }
    }
}

struct DrainWorkerReply {
    host: NativeHost,
    result: Result<TopologyShutdown, String>,
}

struct OperationsRuntime {
    host: Option<NativeHost>,
    reply: Option<Receiver<DrainWorkerReply>>,
    retained: Option<TopologyShutdown>,
    drain_unknown: bool,
    armed: bool,
    notice: String,
}

impl OperationsRuntime {
    fn new(host: Option<NativeHost>) -> Self {
        let notice = if host.is_some() {
            "Live local host attached"
        } else {
            "No Pantheon-owned topology; peer lifecycles are external"
        };
        Self {
            host,
            reply: None,
            retained: None,
            drain_unknown: false,
            armed: false,
            notice: notice.into(),
        }
    }

    fn arm(&mut self) {
        if self.reply.is_some() {
            self.notice = "Topology drain is already pending".into();
        } else if self.host.is_none() {
            self.notice = "No Pantheon-owned topology is available to drain".into();
        } else if self.retained.is_some() {
            self.notice = "Topology is already drained; receipts are retained below".into();
        } else {
            self.armed = true;
            self.notice = "ARMED: Enter drains every hosted peer in reverse start order".into();
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
        self.notice = "Topology drain disarmed".into();
    }

    fn confirm(&mut self) -> bool {
        if !self.armed || self.reply.is_some() || self.retained.is_some() {
            return false;
        }
        let Some(mut host) = self.host.take() else {
            self.armed = false;
            self.notice = "No Pantheon-owned topology is available to drain".into();
            return false;
        };
        let (tx, rx) = sync_channel(1);
        std::thread::spawn(move || {
            let result = host
                .shutdown()
                .map_err(|error| format!("host drain refused: {error:?}"));
            let _ = tx.send(DrainWorkerReply { host, result });
        });
        self.reply = Some(rx);
        self.drain_unknown = false;
        self.armed = false;
        self.notice = "Topology drain pending on lifecycle worker".into();
        true
    }

    fn poll(&mut self) -> Option<(ActivityLevel, Option<TopologyShutdown>)> {
        let Some(reply) = &self.reply else {
            return None;
        };
        match reply.try_recv() {
            Ok(reply) => {
                self.reply = None;
                self.host = Some(reply.host);
                match reply.result {
                    Ok(receipt) => {
                        let clean = receipt.clean();
                        self.notice = if clean {
                            "Hosted topology drained cleanly; receipts retained"
                        } else {
                            "Topology drain incomplete; refused or uncertain receipts retained"
                        }
                        .into();
                        self.retained = Some(receipt.clone());
                        Some((
                            if clean {
                                ActivityLevel::Success
                            } else {
                                ActivityLevel::Uncertain
                            },
                            Some(receipt),
                        ))
                    }
                    Err(error) => {
                        self.notice = error;
                        Some((ActivityLevel::Warning, None))
                    }
                }
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.reply = None;
                self.drain_unknown = true;
                self.notice = "Lifecycle worker disconnected; drain outcome unknown".into();
                Some((ActivityLevel::Uncertain, None))
            }
        }
    }

    fn hosted_peer_ids(&self) -> Vec<HostedPeerId> {
        self.host
            .as_ref()
            .map(NativeHost::status)
            .map(|status| status.hosted_peers)
            .unwrap_or_default()
    }

    fn view(&self) -> OperationsView {
        let status = self.host.as_ref().map(NativeHost::status);
        let lifecycle = if self.drain_unknown {
            "local_host_shutdown_unknown".to_owned()
        } else if self.reply.is_some() {
            "local_host_draining".to_owned()
        } else {
            status.as_ref().map_or_else(
                || "external".to_owned(),
                |status| registry_lifecycle_label(status.lifecycle).to_owned(),
            )
        };
        let hosted_peers = status
            .as_ref()
            .map(|status| status.hosted_peers.iter().map(hosted_peer_label).collect())
            .unwrap_or_default();
        let receipts = self
            .retained
            .as_ref()
            .map(|shutdown| {
                shutdown
                    .receipts
                    .iter()
                    .map(|receipt| OperationsReceiptView {
                        peer: hosted_peer_label(&receipt.peer),
                        disposition: shutdown_disposition_label(receipt.disposition).into(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        OperationsView {
            lifecycle,
            reconfiguration_allowed: status
                .as_ref()
                .is_some_and(|status| status.reconfiguration_allowed),
            hosted_peers,
            drain_pending: self.reply.is_some(),
            drain_armed: self.armed,
            receipts,
            notice: self.notice.clone(),
        }
    }

    fn finish(mut self) -> Result<Option<TopologyShutdown>, String> {
        if self.reply.is_some() {
            return Err("cannot leave while topology drain is pending".into());
        }
        if self.drain_unknown {
            return Err("cannot prove hosted topology shutdown after lifecycle worker loss".into());
        }
        if let Some(receipt) = self.retained.take() {
            return Ok(Some(receipt));
        }
        self.host
            .as_mut()
            .map(NativeHost::shutdown)
            .transpose()
            .map_err(|error| format!("host shutdown refused: {error:?}"))
    }
}

fn operator_agenda_view(
    chronos: Option<&Hub>,
    moira_participation: Option<&moira_tui::ParticipationComponent>,
) -> OperatorAgendaView {
    let mut agenda = OperatorAgendaView::default();
    if let Some(chronos) = chronos {
        for observation in chronos.operator_agenda_observations() {
            let source = format!("chronos/{}", observation.workstream);
            if !observation.current {
                agenda.unavailable_sources.push(source);
                continue;
            }
            if observation.truncated {
                agenda.truncated_sources.push(source);
            }
            for task in observation.tasks {
                agenda.push(AgendaEntry {
                    source_peer: "chronos".into(),
                    source_instance: Some(observation.workstream.clone()),
                    source_revision: observation.revision.clone(),
                    resource_identity: format!("{}#{}", task.task, task.incarnation),
                    kind: AgendaKind::OutcomeUnknown,
                    urgency: AgendaUrgency::Blocking,
                    title: format!("Uncertain Chronos task · {}", task.task),
                    why_operator: "the owner fenced progress after an uncertain outcome".into(),
                    authority: "agenda presence grants none; Chronos owns task state".into(),
                    requested_action: "inspect owner evidence and choose a permitted recovery"
                        .into(),
                    consequence_of_waiting: "the task remains fenced; no blind retry is safe"
                        .into(),
                    next_owner_action: "open Chronos and inspect the exact task revision".into(),
                    route: "navigate.chronos".into(),
                });
            }
        }
    }
    if let Some(observation) =
        moira_participation.and_then(moira_tui::ParticipationComponent::operator_agenda_observation)
        && let Some(options) = observation.permission_option_count
    {
        agenda.push(AgendaEntry {
            source_peer: "moira-participation".into(),
            source_instance: None,
            source_revision: observation.revision.to_string(),
            resource_identity: "selected-participation".into(),
            kind: AgendaKind::GovernedPermission,
            urgency: AgendaUrgency::Blocking,
            title: format!("Governed permission · {options} exact option(s)"),
            why_operator: "the governed participation is awaiting an attributable choice".into(),
            authority: "only the Moira/Aegis owner contract can settle this request".into(),
            requested_action: "inspect the exact challenge and choose, refuse, or defer".into(),
            consequence_of_waiting: "the participation remains paused at the permission gate"
                .into(),
            next_owner_action: "open Moira; refresh before acting if the revision changed".into(),
            route: "navigate.moira".into(),
        });
    }
    agenda.sort();
    agenda
}

const fn registry_lifecycle_label(lifecycle: RegistryLifecycle) -> &'static str {
    match lifecycle {
        RegistryLifecycle::Assembling => "local_host_assembling",
        RegistryLifecycle::Ready => "local_host_ready",
        RegistryLifecycle::Degraded => "local_host_degraded",
        RegistryLifecycle::Draining => "local_host_draining",
        RegistryLifecycle::Stopped => "local_host_stopped",
    }
}

fn hosted_peer_label(peer: &HostedPeerId) -> String {
    match peer {
        HostedPeerId::Agora => "agora".into(),
        HostedPeerId::Moira => "moira".into(),
        HostedPeerId::Daimon => "daimon".into(),
        HostedPeerId::Hermes => "hermes".into(),
        HostedPeerId::Aegis => "aegis".into(),
        HostedPeerId::Tellus => "tellus".into(),
        HostedPeerId::Chronos(instance) => format!("chronos/{instance}"),
        HostedPeerId::Vellum => "vellum".into(),
    }
}

const fn hosted_peer_metric_label(peer: &HostedPeerId) -> &'static str {
    match peer {
        HostedPeerId::Agora => "agora",
        HostedPeerId::Moira => "moira",
        HostedPeerId::Daimon => "daimon",
        HostedPeerId::Hermes => "hermes",
        HostedPeerId::Aegis => "aegis",
        HostedPeerId::Tellus => "tellus",
        HostedPeerId::Chronos(_) => "chronos",
        HostedPeerId::Vellum => "vellum",
    }
}

const fn shutdown_level(disposition: ShutdownDisposition) -> ActivityLevel {
    match disposition {
        ShutdownDisposition::Clean => ActivityLevel::Success,
        ShutdownDisposition::Refused => ActivityLevel::Warning,
        ShutdownDisposition::OutcomeUnknown => ActivityLevel::Uncertain,
    }
}

fn begin_drain_operations(
    peers: Vec<HostedPeerId>,
    instrumentation: &mut Instrumentation,
    started: Instant,
) -> Vec<(HostedPeerId, u64)> {
    let correlation = instrumentation.new_correlation();
    peers
        .into_iter()
        .filter_map(|peer| {
            instrumentation
                .begin_correlated_peer_operation_at(
                    "operations.drain",
                    hosted_peer_metric_label(&peer),
                    started,
                    correlation,
                )
                .map(|operation| (peer, operation))
        })
        .collect()
}

fn settle_drain_operations(
    operations: &mut Vec<(HostedPeerId, u64)>,
    shutdown: Option<&TopologyShutdown>,
    fallback: ActivityLevel,
    instrumentation: &mut Instrumentation,
) {
    for (peer, operation) in operations.drain(..) {
        let disposition = shutdown.and_then(|shutdown| {
            shutdown
                .receipts
                .iter()
                .find(|receipt| receipt.peer == peer)
                .map(|receipt| receipt.disposition)
        });
        let peer_level = disposition.map_or(fallback, shutdown_level);
        instrumentation.mark_peer_stage(
            operation,
            if disposition.is_some() {
                "peer_shutdown_receipt"
            } else {
                "peer_shutdown_unresolved"
            },
            peer_level,
        );
        instrumentation.settle_peer_operation(operation, peer_level);
    }
}

const fn shutdown_disposition_label(disposition: ShutdownDisposition) -> &'static str {
    match disposition {
        ShutdownDisposition::Clean => "clean",
        ShutdownDisposition::Refused => "refused",
        ShutdownDisposition::OutcomeUnknown => "outcome_unknown",
    }
}

fn settle_refresh(
    operation: &mut Option<u64>,
    settlement: Option<Result<(), ()>>,
    peer: &'static str,
    instrumentation: &mut Instrumentation,
    activity: &mut ActivityLog,
) {
    let Some(settlement) = settlement else {
        return;
    };
    let Some(operation) = operation.take() else {
        return;
    };
    let (level, message) = if settlement.is_ok() {
        (
            ActivityLevel::Success,
            "Refresh settled to a current projection",
        )
    } else {
        (
            ActivityLevel::Warning,
            "Refresh settled without a current projection",
        )
    };
    instrumentation.settle_peer_operation(operation, level);
    activity.push(level, peer, message);
}

fn begin_refresh(
    newly_pending: bool,
    action: &'static str,
    peer: &'static str,
    operation: &mut Option<u64>,
    instrumentation: &mut Instrumentation,
    started: Instant,
) -> bool {
    begin_refresh_correlated(
        newly_pending,
        action,
        peer,
        operation,
        instrumentation,
        started,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn begin_refresh_correlated(
    newly_pending: bool,
    action: &'static str,
    peer: &'static str,
    operation: &mut Option<u64>,
    instrumentation: &mut Instrumentation,
    started: Instant,
    correlation_id: Option<u64>,
) -> bool {
    if newly_pending && operation.is_none() {
        *operation = match correlation_id {
            Some(correlation_id) => instrumentation.begin_correlated_peer_operation_at(
                action,
                peer,
                started,
                correlation_id,
            ),
            None => instrumentation.begin_peer_operation_at(action, peer, started),
        };
        return operation.is_some();
    }
    false
}

fn vellum_action_id(action: &ServiceAction) -> &'static str {
    match action {
        ServiceAction::Insert(_) => "vellum.edit.insert",
        ServiceAction::Backspace => "vellum.edit.backspace",
        ServiceAction::Delete => "vellum.edit.delete",
        ServiceAction::Left
        | ServiceAction::Right
        | ServiceAction::Up
        | ServiceAction::Down
        | ServiceAction::Home
        | ServiceAction::End => "vellum.cursor.move",
        ServiceAction::NextDocument => "vellum.document.next",
        ServiceAction::ToggleDocumentSelection => "vellum.document.toggle-selection",
        ServiceAction::Save => "vellum.save",
        ServiceAction::Refresh => "view.refresh",
    }
}

/// Runs the composite terminal for host-supplied sessions. A Vellum-owned
/// integration host can bind handoff on the Hub before entering this loop.
pub fn run_terminal<T: ServicePort>(
    chronos: Hub,
    vellum: ServiceEditor<T>,
) -> Result<(), Box<dyn std::error::Error>> {
    run_multi_peer_terminal(Shell::workbench(), Some(chronos), Some(vellum), None)
}

/// Runs a profile-selected shell. Services are independently optional; a
/// missing presentation is rendered as unavailable and receives no input.
pub fn run_profiled_terminal<T: ServicePort>(
    shell: Shell,
    chronos: Option<Hub>,
    vellum: Option<ServiceEditor<T>>,
) -> Result<(), Box<dyn std::error::Error>> {
    run_multi_peer_terminal(shell, chronos, vellum, None)
}

/// Runs the selected shell with independently supplied presentation peers.
/// The host routes each event explicitly and acquires no peer semantics.
pub fn run_multi_peer_terminal<T: ServicePort>(
    shell: Shell,
    chronos: Option<Hub>,
    vellum: Option<ServiceEditor<T>>,
    agora: Option<agora_tui::Component>,
) -> Result<(), Box<dyn std::error::Error>> {
    run_terminal_with_moira(shell, chronos, vellum, agora, None)
}

/// Runs the selected shell with an independently supplied Moira peer.
pub fn run_terminal_with_moira<T: ServicePort>(
    shell: Shell,
    chronos: Option<Hub>,
    vellum: Option<ServiceEditor<T>>,
    agora: Option<agora_tui::Component>,
    moira: Option<moira_tui::Component>,
) -> Result<(), Box<dyn std::error::Error>> {
    run_terminal_with_operational_peers(shell, chronos, vellum, agora, moira, None)
}

pub fn run_terminal_with_operational_peers<T: ServicePort>(
    shell: Shell,
    chronos: Option<Hub>,
    vellum: Option<ServiceEditor<T>>,
    agora: Option<agora_tui::Component>,
    moira: Option<moira_tui::Component>,
    daimon: Option<daimon_tui::Component>,
) -> Result<(), Box<dyn std::error::Error>> {
    run_terminal_with_all_peers(
        shell,
        chronos,
        vellum,
        OperationalPeers {
            agora,
            agora_control: None,
            moira,
            moira_participation: None,
            moira_participation_control: None,
            moira_agent_worker: None,
            moira_control_socket: None,
            daimon,
            daimon_model: None,
            daimon_agent: None,
            daimon_agent_control_socket: None,
            daimon_execution_policy: None,
            hermes: None,
            hermes_operator_socket: None,
            hermes_disclosure_socket: None,
            hermes_control_socket: None,
            aegis: None,
            aegis_control_socket: None,
            tellus: None,
            vellum_control: None,
            vellum_allow_add_file: false,
        },
    )
}

pub struct OperationalPeers {
    pub agora: Option<agora_tui::Component>,
    pub agora_control: Option<AgoraControlConfig>,
    pub moira: Option<moira_tui::Component>,
    pub moira_participation: Option<moira_tui::ParticipationComponent>,
    pub moira_participation_control: Option<(
        PathBuf,
        moira_protocol::agent_participation::ParticipationIdV1,
    )>,
    pub moira_agent_worker: Option<MoiraAgentWorkerConfig>,
    pub moira_control_socket: Option<PathBuf>,
    pub daimon: Option<daimon_tui::Component>,
    pub daimon_model: Option<DaimonModelView>,
    pub daimon_agent: Option<daimon_tui::AgentSetComponent>,
    pub daimon_agent_control_socket: Option<PathBuf>,
    pub daimon_execution_policy: Option<DaimonExecutionPolicyProfile>,
    pub hermes: Option<hermes_tui::Component>,
    pub hermes_operator_socket: Option<PathBuf>,
    pub hermes_disclosure_socket: Option<PathBuf>,
    pub hermes_control_socket: Option<PathBuf>,
    pub aegis: Option<aegis_tui::Component>,
    pub aegis_control_socket: Option<PathBuf>,
    pub tellus: Option<tellus::Component>,
    pub vellum_control: Option<VellumControlConfig>,
    pub vellum_allow_add_file: bool,
}

#[derive(Clone, Debug)]
pub struct MoiraAgentWorkerConfig {
    pub executable: PathBuf,
    pub config: PathBuf,
    pub participation: moira_protocol::agent_participation::ParticipationIdV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum MoiraAgentWorkerOutcome {
    Idle,
    Observed,
    FencedUnknown,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MoiraAgentWorkerReceipt {
    outcome: MoiraAgentWorkerOutcome,
    participation: String,
    participation_revision: u64,
}

#[derive(Debug)]
enum MoiraAgentWorkerResult {
    Receipt(MoiraAgentWorkerReceipt),
    Refused(String),
    OutcomeUnknown(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MoiraAgentWorkerSettlement {
    Idle,
    Observed,
    FencedUnknown,
    Refused,
}

struct MoiraAgentWorkerControl {
    receiver: Option<Receiver<MoiraAgentWorkerResult>>,
    pending: bool,
    uncertain: bool,
    notice: String,
}

impl MoiraAgentWorkerControl {
    fn new() -> Self {
        Self {
            receiver: None,
            pending: false,
            uncertain: false,
            notice: "Owner worker has not run".into(),
        }
    }

    fn start_refusal(&self) -> Option<&'static str> {
        if self.pending {
            return Some("Moira owner worker is already pending");
        }
        if self.uncertain {
            return Some(
                "Moira owner worker is fenced; inspect the participation before any retry",
            );
        }
        None
    }

    fn start(&mut self, config: MoiraAgentWorkerConfig) -> Result<(), &'static str> {
        if let Some(reason) = self.start_refusal() {
            return Err(reason);
        }
        let (sender, receiver) = sync_channel(1);
        std::thread::spawn(move || {
            let child = Command::new(&config.executable)
                .arg(&config.config)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();
            let result = match child {
                Err(error) => {
                    let _ = sender.send(MoiraAgentWorkerResult::Refused(format!(
                        "owner worker could not start: {error}"
                    )));
                    return;
                }
                Ok(child) => child
                    .wait_with_output()
                    .map_err(|error| format!("owner worker outcome unknown: {error}"))
                    .and_then(|output| {
                        if output.stdout.len() > 4_096 || output.stderr.len() > 4_096 {
                            return Err("owner worker output exceeded 4096 bytes".into());
                        }
                        if !output.status.success() {
                            let detail = String::from_utf8_lossy(&output.stderr);
                            return Err(format!(
                                "owner worker exited {}: {}",
                                output.status,
                                detail.trim()
                            ));
                        }
                        let receipt: MoiraAgentWorkerReceipt =
                            serde_json::from_slice(&output.stdout).map_err(|_| {
                                "owner worker returned an invalid receipt".to_string()
                            })?;
                        let expected = config
                            .participation
                            .0
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>();
                        if receipt.participation != expected {
                            return Err(
                                "owner worker receipt named a different participation".into()
                            );
                        }
                        Ok(receipt)
                    }),
            };
            let _ = sender.send(match result {
                Ok(receipt) => MoiraAgentWorkerResult::Receipt(receipt),
                Err(error) => MoiraAgentWorkerResult::OutcomeUnknown(error),
            });
        });
        self.receiver = Some(receiver);
        self.pending = true;
        self.notice = "Moira owner worker is dispatching one durable step".into();
        Ok(())
    }

    fn poll(&mut self) -> Option<MoiraAgentWorkerSettlement> {
        let result = match self.receiver.as_ref().map(Receiver::try_recv) {
            Some(Ok(result)) => result,
            Some(Err(TryRecvError::Empty)) | None => return None,
            Some(Err(TryRecvError::Disconnected)) => {
                self.pending = false;
                self.uncertain = true;
                self.notice = "Owner worker disconnected; outcome unknown".into();
                self.receiver = None;
                return Some(MoiraAgentWorkerSettlement::FencedUnknown);
            }
        };
        self.pending = false;
        self.receiver = None;
        match result {
            MoiraAgentWorkerResult::Receipt(receipt) => {
                self.uncertain = receipt.outcome == MoiraAgentWorkerOutcome::FencedUnknown;
                self.notice = format!(
                    "Moira owner worker {:?} participation revision {}",
                    receipt.outcome, receipt.participation_revision
                );
                Some(match receipt.outcome {
                    MoiraAgentWorkerOutcome::Idle => MoiraAgentWorkerSettlement::Idle,
                    MoiraAgentWorkerOutcome::Observed => MoiraAgentWorkerSettlement::Observed,
                    MoiraAgentWorkerOutcome::FencedUnknown => {
                        MoiraAgentWorkerSettlement::FencedUnknown
                    }
                })
            }
            MoiraAgentWorkerResult::Refused(error) => {
                self.notice = format!("Moira owner worker refused: {error}");
                Some(MoiraAgentWorkerSettlement::Refused)
            }
            MoiraAgentWorkerResult::OutcomeUnknown(error) => {
                self.uncertain = true;
                self.notice = format!(
                    "Moira owner worker outcome unknown: {error}; inspect participation before any retry"
                );
                Some(MoiraAgentWorkerSettlement::FencedUnknown)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct VellumControlConfig {
    pub socket: PathBuf,
    pub credential: Option<PathBuf>,
}

enum VellumBatchConnector {
    Plain(UnixConnector),
    Authenticated(AuthenticatedUnixConnector),
}

impl FrameExchange for VellumBatchConnector {
    fn exchange(
        &mut self,
        request: &[u8],
        max_reply_bytes: usize,
    ) -> Result<Vec<u8>, TransportError> {
        match self {
            Self::Plain(connector) => connector.exchange(request, max_reply_bytes),
            Self::Authenticated(connector) => connector.exchange(request, max_reply_bytes),
        }
    }
}

fn vellum_batch_transport(
    config: &VellumControlConfig,
) -> Result<RemoteTransport<VellumBatchConnector>, String> {
    let connector = if let Some(path) = &config.credential {
        let mut store = vellum_adapter_local_authentication::LocalCredentialStoreV1::default();
        let record = store
            .load(path)
            .map_err(|error| format!("Vellum credential refused: {error:?}"))?;
        let provider =
            vellum_adapter_local_authentication::LocalCredentialClaimProviderV1::new(record);
        VellumBatchConnector::Authenticated(AuthenticatedUnixConnector::with_provider(
            config.socket.clone(),
            provider,
        ))
    } else {
        VellumBatchConnector::Plain(UnixConnector::new(config.socket.clone()))
    };
    RemoteTransport::new(connector, TransportLimits::reference())
        .map_err(|error| format!("Vellum durable transport refused: {error:?}"))
}

#[derive(Clone, Debug)]
pub struct AgoraControlConfig {
    pub socket: PathBuf,
    pub participant: String,
    pub transport: AgoraTransportProfile,
}

fn is_key(key: crossterm::event::KeyEvent, code: KeyCode, modifiers: KeyModifiers) -> bool {
    key.kind == KeyEventKind::Press && key.code == code && key.modifiers == modifiers
}

fn is_control_char(key: KeyEvent, expected: char) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }
    let forbidden =
        KeyModifiers::ALT | KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META;
    if key.modifiers.intersects(forbidden) {
        return false;
    }
    (match key.code {
        KeyCode::Char(actual) => {
            let normalized = actual.to_ascii_lowercase();
            let legacy_control = char::from_u32(u32::from(expected as u8 & 0x1f));
            normalized == expected || legacy_control == Some(actual)
        }
        _ => false,
    }) && (key.modifiers.contains(KeyModifiers::CONTROL)
        || matches!(key.code, KeyCode::Char(actual) if actual.is_control()))
}

fn bounded_text_character(key: crossterm::event::KeyEvent) -> Option<char> {
    (key.kind == KeyEventKind::Press
        && (key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT))
        .then_some(key.code)
        .and_then(|code| match code {
            KeyCode::Char(character) if !character.is_control() => Some(character),
            _ => None,
        })
}

fn reconciliation_input(
    key: crossterm::event::KeyEvent,
    editing_logical_time: bool,
) -> Option<moira_tui::ReconciliationBatchInput> {
    if key.kind != KeyEventKind::Press || key.modifiers != KeyModifiers::NONE {
        return None;
    }
    match key.code {
        KeyCode::Char(value) if editing_logical_time && value.is_ascii_digit() => {
            Some(moira_tui::ReconciliationBatchInput::Digit(value))
        }
        KeyCode::Backspace if editing_logical_time => {
            Some(moira_tui::ReconciliationBatchInput::Backspace)
        }
        KeyCode::Enter if editing_logical_time => {
            Some(moira_tui::ReconciliationBatchInput::Prepare)
        }
        KeyCode::Char('r') => Some(moira_tui::ReconciliationBatchInput::Refresh),
        KeyCode::Char('c') => Some(moira_tui::ReconciliationBatchInput::Arm),
        KeyCode::Enter => Some(moira_tui::ReconciliationBatchInput::Confirm),
        _ => None,
    }
}

fn daimon_control_input(key: crossterm::event::KeyEvent) -> Option<daimon_tui::AgentControlInput> {
    if key.kind != KeyEventKind::Press || key.modifiers != KeyModifiers::NONE {
        return None;
    }
    match key.code {
        KeyCode::Char('c') => Some(daimon_tui::AgentControlInput::Arm),
        KeyCode::Enter => Some(daimon_tui::AgentControlInput::Confirm),
        _ => None,
    }
}

fn daimon_prompt_input(
    key: crossterm::event::KeyEvent,
    reviewing: bool,
) -> Option<daimon_tui::AgentPromptInput> {
    if key.kind != KeyEventKind::Press || key.modifiers != KeyModifiers::NONE {
        return None;
    }
    match key.code {
        KeyCode::Tab => Some(daimon_tui::AgentPromptInput::NextField),
        KeyCode::Backspace => Some(daimon_tui::AgentPromptInput::Backspace),
        KeyCode::Enter if reviewing => Some(daimon_tui::AgentPromptInput::Confirm),
        KeyCode::Enter => Some(daimon_tui::AgentPromptInput::Newline),
        KeyCode::Char('c') if reviewing => Some(daimon_tui::AgentPromptInput::Arm),
        KeyCode::Char(character) => Some(daimon_tui::AgentPromptInput::Character(character)),
        _ => None,
    }
}

fn daimon_permission_input(
    key: crossterm::event::KeyEvent,
) -> Option<daimon_tui::AgentPermissionInput> {
    if key.kind != KeyEventKind::Press || key.modifiers != KeyModifiers::NONE {
        return None;
    }
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(daimon_tui::AgentPermissionInput::PreviousSession),
        KeyCode::Down | KeyCode::Char('j') => Some(daimon_tui::AgentPermissionInput::NextSession),
        KeyCode::Left | KeyCode::Char('h') => {
            Some(daimon_tui::AgentPermissionInput::PreviousOption)
        }
        KeyCode::Right | KeyCode::Char('l') => Some(daimon_tui::AgentPermissionInput::NextOption),
        KeyCode::Char('c') => Some(daimon_tui::AgentPermissionInput::Arm),
        KeyCode::Enter => Some(daimon_tui::AgentPermissionInput::Confirm),
        _ => None,
    }
}

fn daimon_agent_set_input(key: crossterm::event::KeyEvent) -> Option<daimon_tui::AgentSetInput> {
    if key.kind != KeyEventKind::Press || key.modifiers != KeyModifiers::NONE {
        return None;
    }
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => Some(daimon_tui::AgentSetInput::PreviousSession),
        KeyCode::Right | KeyCode::Char('l') => Some(daimon_tui::AgentSetInput::NextSession),
        KeyCode::Char(' ') => Some(daimon_tui::AgentSetInput::ToggleSelection),
        KeyCode::Up | KeyCode::Char('k') => Some(daimon_tui::AgentSetInput::EventUp),
        KeyCode::Down | KeyCode::Char('j') => Some(daimon_tui::AgentSetInput::EventDown),
        KeyCode::Char('f') => Some(daimon_tui::AgentSetInput::CycleEventFilter),
        KeyCode::Char('r') => Some(daimon_tui::AgentSetInput::RefreshSelected),
        KeyCode::Char('R') => Some(daimon_tui::AgentSetInput::RefreshAll),
        KeyCode::Char('q') => Some(daimon_tui::AgentSetInput::Close),
        _ => None,
    }
}

#[cfg(test)]
fn reconciliation_level(notice: &str) -> ActivityLevel {
    let notice = notice.to_ascii_lowercase();
    if notice.contains("outcome unknown") {
        ActivityLevel::Uncertain
    } else if notice.contains("refused")
        || notice.contains("unavailable")
        || notice.contains("invalid")
    {
        ActivityLevel::Warning
    } else if notice.contains("durably accepted")
        || notice.contains("recovered from durable")
        || notice.contains("replayed")
    {
        ActivityLevel::Success
    } else {
        ActivityLevel::Info
    }
}

fn reconciliation_batch_level(
    disposition: moira_tui::ReconciliationBatchDisposition,
) -> ActivityLevel {
    match disposition {
        moira_tui::ReconciliationBatchDisposition::Applied => ActivityLevel::Success,
        moira_tui::ReconciliationBatchDisposition::Partial
        | moira_tui::ReconciliationBatchDisposition::ReceiptPending => ActivityLevel::Warning,
        moira_tui::ReconciliationBatchDisposition::OutcomeUnknown => ActivityLevel::Uncertain,
        moira_tui::ReconciliationBatchDisposition::Inspecting
        | moira_tui::ReconciliationBatchDisposition::Ready
        | moira_tui::ReconciliationBatchDisposition::Armed
        | moira_tui::ReconciliationBatchDisposition::Pending => ActivityLevel::Info,
    }
}

fn fresh_operation_facts() -> Result<(IdempotencyKeyV1, u64), String> {
    let bytes = fresh_identity()?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock precedes the Unix epoch".to_owned())?
        .as_secs();
    Ok((IdempotencyKeyV1::new(bytes), now))
}

fn fresh_identity() -> Result<[u8; 16], String> {
    let mut bytes = [0_u8; 16];
    File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut bytes))
        .map_err(|error| format!("local operation identity unavailable: {error}"))?;
    Ok(bytes)
}

fn parse_hex_identity(value: &str) -> Result<[u8; 16], String> {
    if value.len() != 32 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("validated Daimon prompt identity became invalid".into());
    }
    let mut bytes = [0_u8; 16];
    for (index, slot) in bytes.iter_mut().enumerate() {
        let offset = index * 2;
        *slot = u8::from_str_radix(&value[offset..offset + 2], 16)
            .map_err(|_| "validated Daimon prompt identity became invalid")?;
    }
    Ok(bytes)
}

fn daimon_workset_error(
    observations: &[&daimon_tui::AgentObservation],
    requirement: &str,
    eligible: impl Fn(&daimon_protocol::agent::AgentSessionViewV1) -> bool,
) -> Option<String> {
    let blocked = observations
        .iter()
        .enumerate()
        .filter(|(_, observation)| !eligible(&observation.view))
        .map(|(index, observation)| {
            format!(
                "#{} {:?} r{}{}",
                index + 1,
                observation.view.phase,
                observation.view.revision,
                if observation.view.active_turn.is_some() {
                    " active-turn"
                } else {
                    " no-active-turn"
                }
            )
        })
        .collect::<Vec<_>>();
    (!blocked.is_empty()).then(|| {
        format!(
            "Daimon workset blocked by {} of {} selected session(s): {}. Required: {requirement}",
            blocked.len(),
            observations.len(),
            blocked.join(", ")
        )
    })
}

fn daimon_prompt_requests(
    observations: &[&daimon_tui::AgentObservation],
    policy: &DaimonExecutionPolicyProfile,
    prompt: Vec<u8>,
) -> Result<Vec<daimon_tui::AgentControlRequest>, String> {
    if let Some(error) = daimon_workset_error(observations, "Ready with no active turn", |view| {
        view.phase == AgentSessionPhaseV1::Ready && view.active_turn.is_none()
    }) {
        return Err(error);
    }
    let now = current_unix_time()?;
    observations
        .iter()
        .map(|observation| {
            Ok(daimon_tui::AgentControlRequest::Prompt(PromptRequest {
                session: observation.view.session,
                turn: AgentTurnIdV1::new(fresh_identity()?),
                expected_revision: observation.view.revision,
                execution: daimon_execution_request(policy)?,
                prompt: prompt.clone(),
                now,
            }))
        })
        .collect()
}

fn daimon_reattach_requests(
    observations: &[&daimon_tui::AgentObservation],
    policy: &DaimonExecutionPolicyProfile,
) -> Result<Vec<daimon_tui::AgentControlRequest>, String> {
    if let Some(error) = daimon_workset_error(
        observations,
        "ReattachRequired with no active turn and resume support",
        |view| {
            view.phase == AgentSessionPhaseV1::ReattachRequired
                && view.active_turn.is_none()
                && view.capabilities.resume_session
        },
    ) {
        return Err(error);
    }
    let now = current_unix_time()?;
    observations
        .iter()
        .map(|observation| {
            Ok(daimon_tui::AgentControlRequest::Reattach(ReattachRequest {
                session: observation.view.session,
                expected_revision: observation.view.revision,
                execution: daimon_execution_request(policy)?,
                now,
            }))
        })
        .collect()
}

fn daimon_execution_request(
    policy: &DaimonExecutionPolicyProfile,
) -> Result<ExecuteRequestV1, String> {
    let principal = PrincipalIdV1::new(parse_hex_identity(&policy.principal)?);
    let profile = HarnessProfileIdV1::new(parse_hex_identity(&policy.profile)?);
    let context = policy
        .context
        .iter()
        .map(|value| parse_hex_identity(value).map(ContextReferenceIdV1::new))
        .collect::<Result<Vec<_>, _>>()?;
    let tools = policy
        .tools
        .iter()
        .map(|value| parse_hex_identity(value).map(ToolCapabilityIdV1::new))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ExecuteRequestV1 {
        execution_id: ExecutionIdV1::new(fresh_identity()?),
        task_attempt: TaskAttemptRefV1 {
            task_key: parse_hex_identity(&policy.task_attempt.task_key)?,
            task_incarnation: policy.task_attempt.task_incarnation,
            attempt_number: policy.task_attempt.attempt_number,
        },
        principal,
        profile,
        idempotency_key: IdempotencyKeyV1::new(fresh_identity()?),
        context,
        tools,
        budget: ExecutionBudgetV1 {
            max_elapsed_millis: policy.budget.max_elapsed_millis,
            max_input_tokens: policy.budget.max_input_tokens,
            max_output_tokens: policy.budget.max_output_tokens,
            max_tool_calls: policy.budget.max_tool_calls,
            max_context_bytes: policy.budget.max_context_bytes,
            max_output_bytes: policy.budget.max_output_bytes,
        },
    })
}

fn current_unix_time() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock precedes the Unix epoch".to_owned())
        .map(|duration| duration.as_secs())
}

fn daimon_permission_challenges(
    observations: &[&daimon_tui::AgentObservation],
) -> Result<Vec<daimon_tui::AgentPermissionChallenge>, String> {
    observations
        .iter()
        .enumerate()
        .map(|(index, observation)| {
            let failure = |reason: &str| {
                format!(
                    "Daimon permission workset blocked at selected #{} {:?} r{}: {reason}",
                    index + 1,
                    observation.view.phase,
                    observation.view.revision
                )
            };
            let Some(turn) = observation.view.active_turn else {
                return Err(failure("no active permission turn"));
            };
            if observation.view.phase != AgentSessionPhaseV1::AwaitingPermission {
                return Err(failure("session is not AwaitingPermission"));
            }
            let Some(last_cursor) = observation.view.last_event_cursor else {
                return Err(failure("session has no challenge cursor"));
            };
            let Some(record) = observation
                .events
                .entries
                .iter()
                .find(|record| record.cursor == last_cursor)
            else {
                return Err(failure(
                    "current challenge is outside the bounded event page",
                ));
            };
            let AgentEventV1::Permission(challenge) = &record.event else {
                return Err(failure("latest event is not a permission challenge"));
            };
            if challenge.turn != turn || challenge.options.is_empty() {
                return Err(failure(
                    "challenge does not match the active turn or has no options",
                ));
            }
            Ok(daimon_tui::AgentPermissionChallenge {
                session: observation.view.session,
                expected_revision: observation.view.revision,
                event_cursor: record.cursor,
                challenge: challenge.clone(),
            })
        })
        .collect()
}

fn daimon_permission_requests(
    selections: Vec<daimon_tui::AgentPermissionSelection>,
    policy: &DaimonExecutionPolicyProfile,
) -> Result<Vec<daimon_tui::AgentControlRequest>, String> {
    let now = current_unix_time()?;
    selections
        .into_iter()
        .map(|selection| {
            let challenge = selection.challenge;
            Ok(daimon_tui::AgentControlRequest::Permission(
                daimon_tui::AgentPermissionControlRequest {
                    request: PermissionRequest {
                        session: challenge.session,
                        expected_revision: challenge.expected_revision,
                        event_cursor: challenge.event_cursor,
                        permission_request: challenge.challenge.request,
                        option: selection.option,
                        execution: daimon_execution_request(policy)?,
                        now,
                    },
                    challenge: challenge.challenge,
                },
            ))
        })
        .collect()
}

fn fresh_durable_operation() -> Result<DurableOperationRefV1, String> {
    let mut bytes = [0_u8; 16];
    File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut bytes))
        .map_err(|error| format!("local durable operation identity unavailable: {error}"))?;
    DurableOperationRefV1::new(bytes)
        .ok_or_else(|| "local durable operation identity was invalid".to_owned())
}

fn daimon_control_request(
    action: &str,
    view: &daimon_protocol::agent::AgentSessionViewV1,
    idempotency_key: IdempotencyKeyV1,
    now: u64,
) -> Result<daimon_tui::AgentControlRequest, &'static str> {
    match action {
        "daimon.session.advance" => {
            let Some(turn) = view.active_turn else {
                return Err("Selected Daimon session has no active turn to advance");
            };
            if view.phase != AgentSessionPhaseV1::TurnActive {
                return Err("Selected Daimon session is not in TurnActive phase");
            }
            Ok(daimon_tui::AgentControlRequest::Advance(AdvanceRequest {
                session: view.session,
                turn,
                expected_revision: view.revision,
                idempotency_key,
                now,
            }))
        }
        "daimon.session.cancel" => {
            let Some(turn) = view.active_turn else {
                return Err("Selected Daimon session has no active turn to cancel");
            };
            if !matches!(
                view.phase,
                AgentSessionPhaseV1::TurnActive | AgentSessionPhaseV1::AwaitingPermission
            ) {
                return Err("Selected Daimon session is not cancellable in its current phase");
            }
            Ok(daimon_tui::AgentControlRequest::Cancel(CancelRequest {
                session: view.session,
                turn,
                expected_revision: view.revision,
                idempotency_key,
                now,
            }))
        }
        "daimon.session.close" => {
            if view.phase != AgentSessionPhaseV1::Ready || view.active_turn.is_some() {
                return Err("Selected Daimon session must be Ready with no active turn to close");
            }
            Ok(daimon_tui::AgentControlRequest::Close(CloseRequest {
                session: view.session,
                expected_revision: view.revision,
                idempotency_key,
                now,
            }))
        }
        _ => Err("Unknown Daimon control action"),
    }
}

#[allow(clippy::too_many_arguments)]
fn workbench_commands(
    shell: &Shell,
    chronos: bool,
    chronos_writes: bool,
    chronos_scheduling: bool,
    vellum: bool,
    vellum_durable: bool,
    agora: bool,
    agora_control: bool,
    moira: bool,
    moira_participation: bool,
    moira_agent_worker: bool,
    moira_control: bool,
    daimon: bool,
    daimon_model: bool,
    daimon_control: bool,
    daimon_execution: bool,
    hermes: bool,
    hermes_operator: bool,
    hermes_disclosure: bool,
    hermes_control: bool,
    aegis: bool,
    aegis_control: bool,
    tellus: bool,
    operations_hosted: bool,
    profile_configured: bool,
) -> Vec<CommandSpec> {
    let mut commands = shell
        .surfaces()
        .iter()
        .map(|surface| match surface {
            Surface::Overview => CommandSpec::available(
                "navigate.overview",
                "Go to overview",
                "See availability and status across the complete configured suite.",
                "Pantheon",
                Some("Ctrl-0"),
                CommandKind::Navigate,
            ),
            Surface::Agenda => CommandSpec::available(
                "navigate.agenda",
                "Go to operator agenda",
                "Inspect owner-native attention items and operating-review readiness without granting authority.",
                "Pantheon",
                Some("Alt-R"),
                CommandKind::Navigate,
            ),
            Surface::Chronos => CommandSpec::available(
                "navigate.chronos",
                "Go to Chronos",
                "Open the full Chronos planning and workstream surface.",
                "Chronos",
                Some("Ctrl-1"),
                CommandKind::Navigate,
            ),
            Surface::Vellum => CommandSpec::available(
                "navigate.vellum",
                "Go to Vellum",
                "Open the full Vellum document and editing surface.",
                "Vellum",
                Some("Ctrl-2"),
                CommandKind::Navigate,
            ),
            Surface::Workspace => CommandSpec::available(
                "navigate.workspace",
                "Go to workspace",
                "Work with Chronos and Vellum side by side.",
                "Pantheon",
                None,
                CommandKind::Navigate,
            ),
            Surface::Agora => CommandSpec::available(
                "navigate.agora",
                "Go to Agora",
                "Inspect rooms and selected message history.",
                "Agora",
                Some("Ctrl-3"),
                CommandKind::Navigate,
            ),
            Surface::Moira => CommandSpec::available(
                "navigate.moira",
                "Go to Moira",
                "Inspect sagas, evidence, reconciliation, and receipts.",
                "Moira",
                Some("Ctrl-4"),
                CommandKind::Navigate,
            ),
            Surface::Daimon => CommandSpec::available(
                "navigate.daimon",
                "Go to Daimon",
                "Inspect agent presence, sessions, and event evidence.",
                "Daimon",
                Some("Ctrl-5"),
                CommandKind::Navigate,
            ),
            Surface::Tellus => CommandSpec::available(
                "navigate.tellus",
                "Go to Tellus",
                "Inspect provenance and freshness of grounding loads.",
                "Tellus",
                Some("Ctrl-6"),
                CommandKind::Navigate,
            ),
            Surface::Aegis => CommandSpec::available(
                "navigate.aegis",
                "Go to Aegis",
                "Inspect authority, grants, and bounded decision evidence.",
                "Aegis",
                Some("Ctrl-7"),
                CommandKind::Navigate,
            ),
            Surface::Hermes => CommandSpec::available(
                "navigate.hermes",
                "Go to Hermes",
                "Inspect ledger, stream, cursor, and retention state.",
                "Hermes",
                Some("Ctrl-8"),
                CommandKind::Navigate,
            ),
            Surface::Operations => CommandSpec::available(
                "navigate.operations",
                "Go to operations",
                "Inspect local topology lifecycle, hosted peers, and retained receipts.",
                "Operations",
                Some("Ctrl-9"),
                CommandKind::Navigate,
            ),
            Surface::Configuration => CommandSpec::available(
                "navigate.configuration",
                "Go to configuration",
                "Inspect redacted live session facts and discover profile restart.",
                "Configuration",
                Some("Ctrl-,"),
                CommandKind::Navigate,
            ),
        })
        .collect::<Vec<_>>();
    let any_peer = chronos || vellum || agora || moira || daimon || hermes || aegis || tellus;
    let refresh = CommandSpec::available(
        "view.refresh",
        "Refresh current view",
        "Request a fresh peer-owned observation for the current surface.",
        "Current view",
        Some("r"),
        CommandKind::Observe,
    );
    commands.push(if any_peer {
        refresh
    } else {
        refresh.unavailable("No peer is configured")
    });
    let refresh_all = CommandSpec::available(
        "suite.refresh",
        "Refresh the complete suite",
        "Queue a fresh peer-owned observation for every configured service.",
        "Operations",
        None,
        CommandKind::Observe,
    );
    commands.push(if any_peer {
        refresh_all
    } else {
        refresh_all.unavailable("No peer is configured")
    });
    let focus = CommandSpec::available(
        "workspace.focus.toggle",
        "Switch workspace focus",
        "Move input between the Chronos and Vellum panes.",
        "Workspace",
        Some("Tab"),
        CommandKind::Navigate,
    );
    commands.push(
        if chronos && vellum && shell.surfaces().contains(&Surface::Workspace) {
            focus
        } else {
            focus.unavailable("Both Chronos and Vellum are required")
        },
    );
    let open_file = CommandSpec::available(
        "vellum.document.open",
        "Open a file in Vellum",
        "Enter one bounded locator and submit it through Vellum's owner-controlled runtime admission path.",
        "Vellum",
        Some("Ctrl-O"),
        CommandKind::Guarded,
    );
    commands.push(if vellum {
        open_file
    } else {
        open_file.unavailable("Vellum is not configured")
    });
    let save = CommandSpec::available(
        "vellum.save",
        "Save current Vellum document",
        "Submit the selected document through Vellum's guarded service save path.",
        "Vellum",
        Some("Ctrl-S"),
        CommandKind::Guarded,
    );
    commands.push(if vellum {
        save
    } else {
        save.unavailable("Vellum is not configured")
    });
    let durable_save = CommandSpec::available(
        "vellum.save-selected",
        "Durably save selected Vellum documents",
        "Prepare exact revisions for the selected 1–16 document workset, then save sequentially with independent restart-stable receipts.",
        "Vellum",
        None,
        CommandKind::Guarded,
    );
    commands.push(if vellum_durable {
        durable_save
    } else {
        durable_save.unavailable(if vellum {
            "A stable authenticated Vellum identity is required for durable receipts"
        } else {
            "Vellum is not configured"
        })
    });
    let atomic_replace = CommandSpec::available(
        "vellum.atomic-replace-selected",
        "Replace text atomically across selected Vellum documents",
        "Preview literal matches and publish one revision-bound in-memory transaction across the selected 1–16 document workset; persistence remains separate.",
        "Vellum",
        None,
        CommandKind::Guarded,
    );
    commands.push(if vellum {
        atomic_replace
    } else {
        atomic_replace.unavailable("Vellum is not configured")
    });
    let select_document = CommandSpec::available(
        "vellum.document.toggle-selection",
        "Toggle current Vellum document selection",
        "Add or remove the current document from the bounded multi-document workset; an empty workset falls back to the current document.",
        "Vellum",
        None,
        CommandKind::Work,
    );
    commands.push(if vellum {
        select_document
    } else {
        select_document.unavailable("Vellum is not configured")
    });
    let new_task = CommandSpec::available(
        "chronos.task.new",
        "Create a Chronos task",
        "Open Chronos's bounded new-task form on the active workstream.",
        "Chronos",
        Some("n"),
        CommandKind::Work,
    );
    commands.push(if chronos_writes {
        new_task
    } else {
        new_task.unavailable(if chronos {
            "Chronos writes are disabled for this session"
        } else {
            "Chronos is not configured"
        })
    });
    let agent_work_item = CommandSpec::available(
        "workitem.agent.new",
        "Create task with local agent",
        "Create a Chronos work item, carry its entered text into a guarded prompt for the selected local Daimon session, and open the resulting conversation.",
        "Work item journey",
        Some("Shift-N"),
        CommandKind::Guarded,
    );
    commands.push(if chronos_writes && daimon_execution {
        agent_work_item
    } else {
        agent_work_item.unavailable(if !chronos_writes {
            if chronos {
                "Chronos writes are disabled for this session"
            } else {
                "Chronos is not configured"
            }
        } else if daimon_control {
            "A credential-free Daimon execution policy is required"
        } else {
            "A selected local Daimon agent session is not configured"
        })
    });
    for lens in AGENT_COLLABORATION_LENSES {
        let new_task_command = CommandSpec::available(
            lens.new_task_action,
            lens.new_task_title,
            "Create exact Chronos work, then carry its reviewed text and the selected non-authority collaboration lens into one guarded first Daimon prompt.",
            "Work item journey",
            None,
            CommandKind::Guarded,
        );
        commands.push(if chronos_writes && daimon_execution {
            new_task_command
        } else {
            new_task_command.unavailable(if !chronos_writes {
                if chronos {
                    "Chronos writes are disabled for this session"
                } else {
                    "Chronos is not configured"
                }
            } else if daimon_control {
                "A credential-free Daimon execution policy is required"
            } else {
                "A selected local Daimon agent session is not configured"
            })
        });
        let command = CommandSpec::available(
            lens.action,
            lens.title,
            "Open a guarded, reviewable one-turn lens change for the selected local Daimon session. A lens never grants role, delegation, authority, approval, or human identity.",
            "Work item journey",
            None,
            CommandKind::Guarded,
        );
        commands.push(if daimon_execution {
            command
        } else {
            command.unavailable(if daimon_control {
                "A credential-free Daimon execution policy is required"
            } else {
                "A selected local Daimon agent session is not configured"
            })
        });
    }
    let return_to_board = CommandSpec::available(
        "workitem.agent.return",
        "Return to the Kanban board",
        "Return to Chronos and request a fresh owner projection so status changes are visible.",
        "Work item journey",
        None,
        CommandKind::Observe,
    );
    commands.push(if chronos {
        return_to_board
    } else {
        return_to_board.unavailable("Chronos is not configured")
    });
    let open_agent_chat = CommandSpec::available(
        "workitem.agent.chat",
        "Open the local-agent conversation",
        "Open Daimon's selected-session event timeline for the current local-agent discussion.",
        "Work item journey",
        None,
        CommandKind::Navigate,
    );
    commands.push(if daimon {
        open_agent_chat
    } else {
        open_agent_chat.unavailable("A Daimon session presentation is not configured")
    });
    let cancel_task = CommandSpec::available(
        "chronos.task.cancel",
        "Cancel marked Chronos tasks",
        "Cancel the marked task set (or current task) through independently receipted, revision-bound owner mutations; the run stops on uncertainty.",
        "Chronos",
        Some("c"),
        CommandKind::Guarded,
    );
    commands.push(if chronos_writes {
        cancel_task
    } else {
        cancel_task.unavailable(if chronos {
            "Chronos writes are disabled for this session"
        } else {
            "Chronos is not configured"
        })
    });
    let schedule = CommandSpec::available(
        "chronos.schedule.open",
        "Open Chronos scheduling",
        "Inspect eligibility, capacity, pause, and not-before controls.",
        "Chronos",
        Some("4"),
        CommandKind::Observe,
    );
    commands.push(if chronos_scheduling {
        schedule
    } else {
        schedule.unavailable(if chronos {
            "Chronos scheduling was not selected for this session"
        } else {
            "Chronos is not configured"
        })
    });
    let prompt = CommandSpec::available(
        "daimon.session.prompt",
        "Prompt selected Daimon sessions",
        "Compose one multiline prompt for the selected 1–16 ready sessions; every child receives a fresh exact execution envelope and independent receipt.",
        "Daimon",
        None,
        CommandKind::Guarded,
    );
    commands.push(if daimon_execution {
        prompt
    } else {
        prompt.unavailable(if daimon_control {
            "Daimon prompting requires a credential-free execution policy in the workspace profile"
        } else if daimon {
            "Daimon agent control connection was not selected for this session"
        } else {
            "A selected Daimon agent session is not configured"
        })
    });
    let qualify_model = CommandSpec::available(
        "models.provider.qualify.v1",
        "Qualify selected local model provider",
        "Review the exact redacted profile, then arm one owner-retained, non-retried completion-conformance request; tool continuation remains a separate qualification.",
        "Daimon",
        None,
        CommandKind::Guarded,
    );
    commands.push(if daimon_model {
        qualify_model
    } else {
        qualify_model.unavailable(if daimon {
            "The selected Daimon route is not a hosted local-model owner"
        } else {
            "Daimon local-model is not configured"
        })
    });
    let permission = CommandSpec::available(
        "daimon.session.permission",
        "Decide selected Daimon permissions",
        "Review each selected waiting session's exact owner-published challenge and choose one independent option per session.",
        "Daimon",
        None,
        CommandKind::Guarded,
    );
    commands.push(if daimon_execution {
        permission
    } else {
        permission.unavailable(if daimon_control {
            "Daimon permission decisions require the credential-free execution policy"
        } else if daimon {
            "Daimon agent control connection was not selected for this session"
        } else {
            "A selected Daimon agent session is not configured"
        })
    });
    let reattach = CommandSpec::available(
        "daimon.session.reattach",
        "Reattach selected Daimon sessions",
        "Resume the selected 1–16 detached sessions with fresh independent execution envelopes, exact revision guards, and child receipts.",
        "Daimon",
        None,
        CommandKind::Guarded,
    );
    commands.push(if daimon_execution {
        reattach
    } else {
        reattach.unavailable(if daimon_control {
            "Daimon reattachment requires the credential-free execution policy"
        } else if daimon {
            "Daimon agent control connection was not selected for this session"
        } else {
            "A selected Daimon agent session is not configured"
        })
    });
    for command in [
        CommandSpec::available(
            "chronos.schedule.task-pause",
            "Set marked Chronos task pause",
            "Set pause consistently across the marked task workset (or current task), preserving each task's not-before value and stopping on uncertainty.",
            "Chronos",
            Some("t"),
            CommandKind::Guarded,
        ),
        CommandSpec::available(
            "chronos.schedule.task-not-before",
            "Set marked Chronos not-before",
            "Open a bounded timestamp form for the marked task workset (or current task); each child preserves its pause state and settles independently.",
            "Chronos",
            Some("d"),
            CommandKind::Guarded,
        ),
    ] {
        commands.push(if chronos_writes && chronos_scheduling {
            command
        } else {
            command.unavailable(if chronos {
                "Chronos scheduling writes require both write and scheduling selections"
            } else {
                "Chronos is not configured"
            })
        });
    }
    let create_room = CommandSpec::available(
        "agora.room.create",
        "Create an Agora room",
        "Open a bounded conversation-native form bound to the current room revision.",
        "Agora",
        None,
        CommandKind::Guarded,
    );
    commands.push(if agora_control {
        create_room
    } else {
        create_room.unavailable(if agora {
            "Agora room control connection was not selected for this session"
        } else {
            "Agora is not configured"
        })
    });
    let send_message = CommandSpec::available(
        "agora.message.compose",
        "Compose an Agora message",
        "Compose a multiline message with an optional canonical recipient set, bound to the observed conversation revision.",
        "Agora",
        None,
        CommandKind::Guarded,
    );
    commands.push(if agora_control {
        send_message
    } else {
        send_message.unavailable(if agora {
            "Agora message control connection was not selected for this session"
        } else {
            "Agora is not configured"
        })
    });
    let prepare_agent_open = CommandSpec::available(
        "moira.participation.open.prepare",
        "Prepare selected Moira agent session",
        "Review the exact admitted participation, then ask Moira to derive and durably prepare its session-open effect; a separate owner worker dispatches it.",
        "Moira",
        None,
        CommandKind::Guarded,
    );
    commands.push(if moira_participation {
        prepare_agent_open
    } else {
        prepare_agent_open
            .unavailable("A selected Moira participation control endpoint is required")
    });
    let run_agent_worker = CommandSpec::available(
        "moira.participation.worker.run",
        "Dispatch prepared Moira agent session",
        "Run the configured Moira one-step owner worker only after an exact Open operation is durably prepared; verify its participation-bound receipt.",
        "Moira",
        None,
        CommandKind::Guarded,
    );
    commands.push(if moira_agent_worker {
        run_agent_worker
    } else {
        run_agent_worker.unavailable("A selected Moira owner-worker profile is required")
    });
    for (command, configured) in [
        (
            CommandSpec::available(
                "moira.saga.open",
                "Inspect selected Moira saga",
                "Open the selected saga's reconciliation state, evidence, and receipts.",
                "Moira",
                Some("Enter"),
                CommandKind::Observe,
            ),
            moira,
        ),
        (
            CommandSpec::available(
                "moira.saga.back",
                "Return to Moira saga list",
                "Leave saga detail and return to the bounded saga census.",
                "Moira",
                Some("Esc"),
                CommandKind::Navigate,
            ),
            moira,
        ),
        (
            CommandSpec::available(
                "agora.room.open",
                "Open selected Agora room",
                "Inspect the selected room's bounded message history.",
                "Agora",
                Some("Enter"),
                CommandKind::Observe,
            ),
            agora,
        ),
        (
            CommandSpec::available(
                "agora.room.back",
                "Return to Agora rooms",
                "Leave message history and return to the bounded room list.",
                "Agora",
                Some("Esc"),
                CommandKind::Navigate,
            ),
            agora,
        ),
        (
            CommandSpec::available(
                "aegis.page.next",
                "Show next Aegis authority page",
                "Request the next bounded page of authority observations.",
                "Aegis",
                Some("n"),
                CommandKind::Observe,
            ),
            aegis,
        ),
        (
            CommandSpec::available(
                "aegis.page.previous",
                "Show previous Aegis authority page",
                "Return to the preceding bounded authority page.",
                "Aegis",
                Some("p"),
                CommandKind::Observe,
            ),
            aegis,
        ),
    ] {
        commands.push(if configured {
            command
        } else {
            command.unavailable("Peer is not configured")
        });
    }
    for reconcile in [
        CommandSpec::available(
            "moira.reconciliation.keep-uncertain",
            "Keep selected Moira sagas uncertain",
            "Reconcile the selected 1–16 Moira sagas through independent server challenges and durable receipts; c arms and Enter confirms.",
            "Moira",
            None,
            CommandKind::Guarded,
        ),
        CommandSpec::available(
            "moira.reconciliation.fail-terminal",
            "Fail selected Moira sagas terminally",
            "Terminally fail the selected 1–16 uncertain sagas through independent server challenges and durable receipts; c arms and Enter confirms.",
            "Moira",
            None,
            CommandKind::Guarded,
        ),
        CommandSpec::available(
            "moira.reconciliation.cancel",
            "Cancel selected Moira sagas",
            "Cancel the selected 1–16 sagas at one visible wall-clock witness through independent server challenges and durable receipts.",
            "Moira",
            None,
            CommandKind::Guarded,
        ),
        CommandSpec::available(
            "moira.reconciliation.compensate",
            "Compensate selected Moira sagas",
            "Begin compensation for the selected 1–16 sagas at one visible wall-clock witness through independent server challenges and durable receipts.",
            "Moira",
            None,
            CommandKind::Guarded,
        ),
        CommandSpec::available(
            "moira.reconciliation.admit-failed",
            "Admit failed evidence for selected Moira sagas",
            "Admit failed evidence for each selected saga's exact current effect at one visible observation time.",
            "Moira",
            None,
            CommandKind::Guarded,
        ),
        CommandSpec::available(
            "moira.reconciliation.admit-succeeded",
            "Admit successful evidence for selected Moira sagas",
            "Admit successful evidence for each selected saga's exact current effect at one visible observation time.",
            "Moira",
            None,
            CommandKind::Guarded,
        ),
        CommandSpec::available(
            "moira.reconciliation.retry",
            "Retry failed effects for selected Moira sagas",
            "Retry each selected saga's exact current failed effect at one visible wall-clock witness.",
            "Moira",
            None,
            CommandKind::Guarded,
        ),
    ] {
        commands.push(if moira_control {
            reconcile
        } else {
            reconcile.unavailable(if moira {
                "Moira control connection was not selected for this session"
            } else {
                "Moira is not configured"
            })
        });
    }
    for command in [
        CommandSpec::available(
            "daimon.session.advance",
            "Advance selected Daimon turns",
            "Open a bounded run of exact revision-bound one-slice proposals for the selected sessions.",
            "Daimon",
            None,
            CommandKind::Guarded,
        ),
        CommandSpec::available(
            "daimon.session.cancel",
            "Cancel selected Daimon turns",
            "Open a bounded run of exact revision- and turn-bound cancellation proposals.",
            "Daimon",
            None,
            CommandKind::Guarded,
        ),
        CommandSpec::available(
            "daimon.session.close",
            "Close selected Daimon sessions",
            "Open a bounded run of exact proposals for ready sessions with no active turn.",
            "Daimon",
            None,
            CommandKind::Guarded,
        ),
    ] {
        commands.push(if daimon_control {
            command
        } else {
            command.unavailable(if daimon {
                "Daimon agent control connection was not selected for this session"
            } else {
                "A selected Daimon agent session is not configured"
            })
        });
    }
    let revoke_epoch = CommandSpec::available(
        "aegis.epoch.revoke",
        "Revoke current Aegis epoch",
        "Open Aegis's exact current-epoch global revocation sheet; c arms and Enter confirms once.",
        "Aegis",
        None,
        CommandKind::Guarded,
    );
    commands.push(if aegis_control {
        revoke_epoch
    } else {
        revoke_epoch.unavailable(if aegis {
            "Aegis administrative connection was not selected for this session"
        } else {
            "Aegis is not configured"
        })
    });
    let manage_authority = CommandSpec::available(
        "aegis.authority.manage",
        "Manage retained Aegis authority",
        "Open one bounded private session for 1–16-scope issuance, attenuation, authorization, redacted review, and explicit close.",
        "Aegis",
        None,
        CommandKind::Guarded,
    );
    commands.push(if aegis_control {
        manage_authority
    } else {
        manage_authority.unavailable(if aegis {
            "Aegis administrative connection was not selected for this session"
        } else {
            "Aegis is not configured"
        })
    });
    let acknowledge = CommandSpec::available(
        "ledger.acknowledge.v1",
        "Acknowledge a Hermes cursor range",
        "Select an exact range through the published edge; c arms and Enter confirms once.",
        "Hermes",
        None,
        CommandKind::Guarded,
    );
    commands.push(if hermes_control {
        acknowledge
    } else {
        acknowledge.unavailable(if hermes {
            "Hermes consumer control was not selected for this session"
        } else {
            "Hermes is not configured"
        })
    });
    let disclose = CommandSpec::available(
        "ledger.events.disclose.v1",
        "Disclose a Hermes event batch",
        "Select 1–16 events with bounded previews; c arms and Enter confirms one read.",
        "Hermes",
        None,
        CommandKind::Guarded,
    );
    commands.push(if hermes_disclosure {
        disclose
    } else {
        disclose.unavailable(if hermes {
            "Hermes stream disclosure was not selected for this session"
        } else {
            "Hermes is not configured"
        })
    });
    let compact = CommandSpec::available(
        "ledger.compact.v1",
        "Compact a Hermes stream",
        "Select an exact destructive retention boundary, then arm and confirm one attempt.",
        "Hermes",
        None,
        CommandKind::Guarded,
    );
    commands.push(if hermes_operator {
        compact
    } else {
        compact.unavailable(if hermes {
            "Hermes stream operator control was not selected for this session"
        } else {
            "Hermes is not configured"
        })
    });
    let drain = CommandSpec::available(
        "operations.drain",
        "Drain hosted topology",
        "Open Operations and arm a reverse-order drain of every Pantheon-hosted peer.",
        "Operations",
        None,
        CommandKind::Lifecycle,
    );
    commands.push(if operations_hosted {
        drain
    } else {
        drain.unavailable("No Pantheon-owned topology is attached to this terminal session")
    });
    commands.extend([
        CommandSpec::available(
            "configuration.providers.local",
            "Inspect local provider configuration",
            "Ask the selected Daimon owner to validate and return its redacted local-model or ACP provider catalogue.",
            "Configuration",
            None,
            CommandKind::Lifecycle,
        ),
        CommandSpec::available(
            "configuration.providers.agent.upsert",
            "Add or replace a local agent provider",
            "Complete a bounded form, review Daimon's redacted exact-change plan, then explicitly arm one atomic owner-document replacement.",
            "Configuration",
            None,
            CommandKind::Guarded,
        ),
        CommandSpec::available(
            "configuration.acp.sessions",
            "Configure ACP agent sessions",
            "Review the exact agent-session selection, execution-policy requirements, and clean profile restart path.",
            "Configuration",
            None,
            CommandKind::Lifecycle,
        ),
        CommandSpec::available(
            "configuration.agora.rooms",
            "Configure Agora room control",
            "Review the participant, transport, control capability, and profile restart required before creating rooms.",
            "Configuration",
            None,
            CommandKind::Lifecycle,
        ),
        CommandSpec::available(
            "workbench.activity",
            "Show activity and receipts",
            "Review recent actions, refusals, and uncertain outcomes.",
            "Pantheon",
            Some("Ctrl-G"),
            CommandKind::Observe,
        ),
        CommandSpec::available(
            "workbench.help",
            "Show contextual help",
            "Review global keys and every currently available action.",
            "Pantheon",
            Some("F1"),
            CommandKind::Observe,
        ),
        CommandSpec::available(
            "workbench.configuration",
            "Inspect session configuration",
            "Review selected surfaces, configured peers, and launch-time authority without exposing credentials.",
            "Configuration",
            None,
            CommandKind::Observe,
        ),
        if profile_configured {
            CommandSpec::available(
                "workbench.reconfigure",
                "Preflight and restart workspace profile",
                "Validate and diff a local profile, review the exact restart boundary, then explicitly arm a clean process replacement.",
                "Configuration",
                None,
                CommandKind::Lifecycle,
            )
        } else {
            CommandSpec::available(
                "workbench.reconfigure",
                "Preflight and restart workspace profile",
                "Validate and diff a local profile, review the exact restart boundary, then explicitly arm a clean process replacement.",
                "Configuration",
                None,
                CommandKind::Lifecycle,
            )
            .unavailable("This session was not launched from a workspace profile")
        },
        CommandSpec::available(
            "workbench.instrumentation",
            "Inspect TUI instrumentation",
            "Review bounded per-action dispatch, outcome, and latency metrics.",
            "Observability",
            None,
            CommandKind::Observe,
        ),
    ]);
    commands.extend(
        GUARDED_CONTROL_ROUTES
            .iter()
            .filter(|route| {
                !matches!(
                    route.capability,
                    "messages.speak.v2"
                        | "agents.session.prompt.v1"
                        | "agents.session.permission.v1"
                        | "agents.session.reattach.v1"
                        | "models.provider.qualify.v1"
                        | "ledger.acknowledge.v1"
                        | "ledger.events.disclose.v1"
                        | "ledger.compact.v1"
                )
            })
            .map(|route| {
                CommandSpec::available(
                    route.capability,
                    route.capability,
                    route.command,
                    route.peer,
                    None,
                    CommandKind::Guarded,
                )
                .unavailable("Qualified guard exists; interactive workflow is not composed yet")
            }),
    );
    commands
}

#[allow(clippy::too_many_arguments)]
fn execute_workbench_action<T: ServicePort>(
    action: &str,
    shell: &mut Shell,
    focus: &mut Focus,
    chronos: &mut Option<Hub>,
    vellum: &mut Option<ServiceEditor<T>>,
    vellum_control_config: Option<&VellumControlConfig>,
    vellum_durable_save: &mut Option<DurableSaveBatchComponent>,
    vellum_atomic_replace: &mut Option<AtomicReplaceComponent>,
    agora: &mut Option<agora_tui::Component>,
    agora_control_config: Option<&AgoraControlConfig>,
    agora_create: &mut Option<agora_tui::CreateRoomComponent>,
    agora_message: &mut Option<agora_tui::ComposeMessageComponent>,
    moira: &mut Option<moira_tui::Component>,
    moira_participation: &mut Option<moira_tui::ParticipationComponent>,
    moira_participation_control: Option<&(
        PathBuf,
        moira_protocol::agent_participation::ParticipationIdV1,
    )>,
    moira_open: &mut Option<moira_tui::OpenControlComponent>,
    moira_agent_worker_config: Option<&MoiraAgentWorkerConfig>,
    moira_agent_worker: &mut MoiraAgentWorkerControl,
    moira_control_socket: Option<&Path>,
    moira_reconciliation: &mut Option<moira_tui::ReconciliationBatchComponent>,
    daimon: &mut Option<daimon_tui::Component>,
    daimon_agent: &mut Option<daimon_tui::AgentSetComponent>,
    daimon_control_socket: Option<&Path>,
    daimon_execution_policy: Option<&DaimonExecutionPolicyProfile>,
    daimon_prompt_composer: &mut Option<daimon_tui::AgentPromptComposer>,
    daimon_permission_composer: &mut Option<daimon_tui::AgentPermissionComposer>,
    daimon_control: &mut Option<daimon_tui::AgentControlBatchComponent>,
    hermes: &mut Option<hermes_tui::Component>,
    hermes_operator_socket: Option<&Path>,
    hermes_compaction: &mut Option<hermes_tui::CompactionComponent>,
    hermes_disclosure_socket: Option<&Path>,
    hermes_disclosure: &mut Option<hermes_tui::DisclosureComponent>,
    hermes_control_socket: Option<&Path>,
    hermes_acknowledge: &mut Option<hermes_tui::AcknowledgeComponent>,
    aegis: &mut Option<aegis_tui::Component>,
    aegis_control_socket: Option<&Path>,
    aegis_revoke: &mut Option<aegis_tui::RevokeEpochComponent>,
    aegis_managed: &mut Option<aegis_tui::ManagedAuthorityComponent>,
    tellus: &mut Option<tellus::Component>,
    operations: &mut OperationsRuntime,
) -> (ActivityLevel, &'static str, String) {
    let destination = match action {
        "navigate.overview" => Some(Surface::Overview),
        "navigate.agenda" => Some(Surface::Agenda),
        "navigate.chronos" => Some(Surface::Chronos),
        "navigate.vellum" => Some(Surface::Vellum),
        "navigate.workspace" => Some(Surface::Workspace),
        "navigate.agora" => Some(Surface::Agora),
        "navigate.moira" => Some(Surface::Moira),
        "navigate.daimon" => Some(Surface::Daimon),
        "navigate.tellus" => Some(Surface::Tellus),
        "navigate.aegis" => Some(Surface::Aegis),
        "navigate.hermes" => Some(Surface::Hermes),
        "navigate.operations" => Some(Surface::Operations),
        "navigate.configuration" => Some(Surface::Configuration),
        _ => None,
    };
    if let Some(destination) = destination {
        let _ = shell.activate(destination);
        return (
            ActivityLevel::Info,
            "pantheon",
            format!("Opened {}", destination.label()),
        );
    }
    if agent_task_collaboration_lens(action).is_some() {
        return invoke_chronos_workbench(shell, focus, chronos, WorkbenchIntent::NewTask);
    }
    if let Some(lens) = agent_collaboration_lens(action) {
        let _ = shell.activate(Surface::Daimon);
        let Some(agent) = daimon_agent.as_ref() else {
            return (
                ActivityLevel::Warning,
                "daimon",
                "No current Daimon agent observation; refresh the selected session first".into(),
            );
        };
        let observations = agent.selected_observations();
        if observations.is_empty() {
            return (
                ActivityLevel::Warning,
                "daimon",
                "Selected Daimon sessions do not yet have current observations; refresh first"
                    .into(),
            );
        }
        if let Some(error) =
            daimon_workset_error(&observations, "Ready with no active turn", |view| {
                view.phase == AgentSessionPhaseV1::Ready && view.active_turn.is_none()
            })
        {
            return (ActivityLevel::Warning, "daimon", error);
        }
        let _ = daimon_execution_policy
            .expect("palette enables collaboration lenses only with an execution policy");
        let mut composer = match daimon_tui::AgentPromptComposer::new(observations.len()) {
            Ok(composer) => composer,
            Err(error) => return (ActivityLevel::Warning, "daimon", error.into()),
        };
        composer.insert_text(&agent_collaboration_lens_prompt(lens));
        *daimon_prompt_composer = Some(composer);
        *daimon_permission_composer = None;
        *daimon_control = None;
        return (
            ActivityLevel::Info,
            "work-item",
            format!(
                "{} collaboration lens prepared for {} selected local-agent session(s); review before sending, and retain actual authority boundaries",
                lens.name,
                observations.len()
            ),
        );
    }
    match action {
        "workspace.focus.toggle" => {
            let _ = shell.activate(Surface::Workspace);
            *focus = focus.other();
            (
                ActivityLevel::Info,
                "workspace",
                format!("Focus moved to {focus:?}"),
            )
        }
        "vellum.save" => {
            activate_peer_surface(shell, Surface::Vellum);
            *focus = Focus::Vellum;
            let Some(vellum) = vellum.as_mut() else {
                return (
                    ActivityLevel::Warning,
                    "vellum",
                    "Vellum service is not configured".into(),
                );
            };
            match vellum.apply(ServiceAction::Save) {
                Ok(()) => (
                    ActivityLevel::Info,
                    "vellum",
                    "Save submitted through Vellum's guarded service path; awaiting peer state"
                        .into(),
                ),
                Err(error) => (ActivityLevel::Warning, "vellum", error.to_string()),
            }
        }
        "vellum.document.toggle-selection" => {
            activate_peer_surface(shell, Surface::Vellum);
            *focus = Focus::Vellum;
            let editor = vellum
                .as_mut()
                .expect("palette enables selection only with Vellum");
            match editor.apply(ServiceAction::ToggleDocumentSelection) {
                Ok(()) => (
                    ActivityLevel::Info,
                    "vellum",
                    format!(
                        "{} Vellum document(s) selected for bounded work",
                        editor.selected_document_count()
                    ),
                ),
                Err(error) => (ActivityLevel::Warning, "vellum", error.to_string()),
            }
        }
        "vellum.save-selected" => {
            activate_peer_surface(shell, Surface::Vellum);
            *focus = Focus::Vellum;
            if vellum_atomic_replace.is_some() {
                return (
                    ActivityLevel::Warning,
                    "vellum",
                    "Close the atomic-replace composer before opening durable save".into(),
                );
            }
            let Some(config) = vellum_control_config.cloned() else {
                return (
                    ActivityLevel::Warning,
                    "vellum",
                    "Vellum durable-save connection was not configured".into(),
                );
            };
            let editor = vellum
                .as_mut()
                .expect("palette enables durable save only with Vellum");
            let exact = match editor.prepare_selected_save_targets() {
                Ok(exact) => exact,
                Err(error) => return (ActivityLevel::Warning, "vellum", error.to_string()),
            };
            let mut targets = Vec::with_capacity(exact.len());
            for (document_ordinal, expected_revision) in exact {
                let operation = match fresh_durable_operation() {
                    Ok(operation) => operation,
                    Err(error) => return (ActivityLevel::Warning, "vellum", error),
                };
                targets.push(DurableSaveTargetV1 {
                    operation,
                    document_ordinal,
                    expected_revision,
                });
            }
            *vellum_durable_save = match DurableSaveBatchComponent::new_with_connector(
                move || vellum_batch_transport(&config),
                targets,
            ) {
                Ok(component) => Some(component),
                Err(error) => return (ActivityLevel::Warning, "vellum", error.into()),
            };
            (
                ActivityLevel::Info,
                "vellum",
                "Exact Vellum durable-save workset opened; c arms, Enter starts once".into(),
            )
        }
        "vellum.atomic-replace-selected" => {
            activate_peer_surface(shell, Surface::Vellum);
            *focus = Focus::Vellum;
            if vellum_durable_save
                .as_ref()
                .is_some_and(|control| control.request_pending() || control.recovery_required())
            {
                return (
                    ActivityLevel::Warning,
                    "vellum",
                    "Settle the durable-save workset before opening atomic replace".into(),
                );
            }
            *vellum_atomic_replace = Some(AtomicReplaceComponent::new());
            (
                ActivityLevel::Info,
                "vellum",
                "Atomic replace opened for the marked 1–16 document workset".into(),
            )
        }
        "chronos.task.new" | "workitem.agent.new" => {
            invoke_chronos_workbench(shell, focus, chronos, WorkbenchIntent::NewTask)
        }
        "workitem.agent.return" => {
            activate_peer_surface(shell, Surface::Chronos);
            *focus = Focus::Chronos;
            let chronos = chronos
                .as_mut()
                .expect("palette enables board return only with Chronos");
            chronos.update(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
            (
                ActivityLevel::Info,
                "work-item",
                "Returned to the Chronos board and queued a fresh owner projection".into(),
            )
        }
        "workitem.agent.chat" => {
            let _ = shell.activate(Surface::Daimon);
            (
                ActivityLevel::Info,
                "work-item",
                "Opened the selected Daimon session conversation".into(),
            )
        }
        "chronos.task.cancel" => {
            invoke_chronos_workbench(shell, focus, chronos, WorkbenchIntent::CancelSelectedTask)
        }
        "chronos.schedule.open" => {
            invoke_chronos_workbench(shell, focus, chronos, WorkbenchIntent::OpenSchedule)
        }
        "chronos.schedule.task-pause" => {
            invoke_chronos_workbench(shell, focus, chronos, WorkbenchIntent::SetTaskPause)
        }
        "chronos.schedule.task-not-before" => {
            invoke_chronos_workbench(shell, focus, chronos, WorkbenchIntent::SetTaskNotBefore)
        }
        "agora.room.create" => {
            let _ = shell.activate(Surface::Agora);
            if agora_message.is_some() {
                return (
                    ActivityLevel::Warning,
                    "agora",
                    "Close the message composer before creating a room".into(),
                );
            }
            let Some(revision) = agora.as_ref().and_then(agora_tui::Component::room_revision)
            else {
                return (
                    ActivityLevel::Warning,
                    "agora",
                    "No stable Agora room revision; refresh the room list first".into(),
                );
            };
            let _ = agora_control_config
                .expect("palette enables room creation only with Agora control configuration");
            let handle = agora
                .as_ref()
                .expect("stable revision came from Agora")
                .control_handle();
            *agora_create = Some(agora_tui::CreateRoomComponent::new_with_handle(
                handle, revision,
            ));
            (
                ActivityLevel::Info,
                "agora",
                "Agora room form opened at the current room revision".into(),
            )
        }
        "agora.message.compose" => {
            let _ = shell.activate(Surface::Agora);
            if agora_create.is_some() {
                return (
                    ActivityLevel::Warning,
                    "agora",
                    "Close the room form before composing a message".into(),
                );
            }
            let Some(target) = agora
                .as_ref()
                .and_then(agora_tui::Component::message_target)
            else {
                return (
                    ActivityLevel::Warning,
                    "agora",
                    "Open and refresh an Agora room before composing a message".into(),
                );
            };
            let _ = agora_control_config.expect(
                "palette enables message composition only with Agora control configuration",
            );
            let handle = agora
                .as_ref()
                .expect("stable target came from Agora")
                .control_handle();
            *agora_message = Some(agora_tui::ComposeMessageComponent::new_with_handle(
                handle, target,
            ));
            (
                ActivityLevel::Info,
                "agora",
                "Agora message composer opened at the observed conversation revision".into(),
            )
        }
        "moira.saga.open" => {
            let _ = shell.activate(Surface::Moira);
            moira
                .as_mut()
                .expect("palette enables Moira action only with a component")
                .update(moira_tui::Input::Open, true);
            (
                ActivityLevel::Info,
                "moira",
                "Selected saga inspection queued".into(),
            )
        }
        "moira.saga.back" => {
            *moira_open = None;
            *moira_reconciliation = None;
            let _ = shell.activate(Surface::Moira);
            moira
                .as_mut()
                .expect("palette enables Moira action only with a component")
                .update(moira_tui::Input::Back, true);
            (
                ActivityLevel::Info,
                "moira",
                "Returned to the saga list".into(),
            )
        }
        "moira.participation.open.prepare" => {
            let _ = shell.activate(Surface::Moira);
            let (socket, participation) = moira_participation_control
                .expect("palette enables open preparation only with a selected control endpoint");
            let port = match moira_client::unix::AgentParticipationUnixPort::connect(
                socket,
                moira_client::unix::ParticipationClientLimitsV1::default(),
            ) {
                Ok(port) => port,
                Err(error) => {
                    return (
                        ActivityLevel::Warning,
                        "moira",
                        format!("Moira participation control connection refused: {error:?}"),
                    );
                }
            };
            *moira_reconciliation = None;
            *moira_open = Some(moira_tui::OpenControlComponent::new(port, *participation));
            (
                ActivityLevel::Info,
                "moira",
                "Exact admitted participation queued for session-open review; c arms and Enter asks Moira to prepare it"
                    .into(),
            )
        }
        "moira.participation.worker.run" => {
            let Some(config) = moira_agent_worker_config.cloned() else {
                return (
                    ActivityLevel::Warning,
                    "moira",
                    "Moira owner-worker profile is not configured".into(),
                );
            };
            if !moira_open
                .as_ref()
                .is_some_and(moira_tui::OpenControlComponent::dispatch_ready)
            {
                return (
                    ActivityLevel::Warning,
                    "moira",
                    "Dispatch refused: review and durably prepare this participation Open first"
                        .into(),
                );
            }
            if let Some(reason) = moira_agent_worker.start_refusal() {
                return (ActivityLevel::Warning, "moira", reason.into());
            }
            let released = moira_open
                .take()
                .is_some_and(moira_tui::OpenControlComponent::release_for_dispatch);
            if !released {
                return (
                    ActivityLevel::Warning,
                    "moira",
                    "Dispatch refused: prepared Moira review could not release its owner session"
                        .into(),
                );
            }
            match moira_agent_worker.start(config) {
                Ok(()) => (
                    ActivityLevel::Info,
                    "moira",
                    "Moira one-step owner worker started; the UI remains responsive while its exact receipt is pending"
                        .into(),
                ),
                Err(reason) => (ActivityLevel::Warning, "moira", reason.into()),
            }
        }
        action @ ("moira.reconciliation.keep-uncertain"
        | "moira.reconciliation.fail-terminal"
        | "moira.reconciliation.cancel"
        | "moira.reconciliation.compensate"
        | "moira.reconciliation.admit-failed"
        | "moira.reconciliation.admit-succeeded"
        | "moira.reconciliation.retry") => {
            let _ = shell.activate(Surface::Moira);
            *moira_open = None;
            let sagas = moira
                .as_ref()
                .map(moira_tui::Component::selected_saga_views)
                .unwrap_or_default();
            if sagas.is_empty() {
                return (
                    ActivityLevel::Warning,
                    "moira",
                    "No current saga is selected; refresh the Moira view first".into(),
                );
            }
            let socket = moira_control_socket
                .expect("palette enables reconciliation only with a control socket")
                .to_owned();
            let count = sagas.len();
            let disposition_label = match action {
                "moira.reconciliation.fail-terminal" => "fail-terminal",
                "moira.reconciliation.cancel" => "cancel",
                "moira.reconciliation.compensate" => "compensate",
                "moira.reconciliation.admit-failed" => "admit-failed-evidence",
                "moira.reconciliation.admit-succeeded" => "admit-succeeded-evidence",
                "moira.reconciliation.retry" => "retry-failed-effect",
                _ => "keep-uncertain",
            };
            let timed_kind = match action {
                "moira.reconciliation.cancel" => Some(moira_tui::TimedReconciliationKind::Cancel),
                "moira.reconciliation.compensate" => {
                    Some(moira_tui::TimedReconciliationKind::Compensate)
                }
                "moira.reconciliation.admit-failed" => {
                    Some(moira_tui::TimedReconciliationKind::AdmitFailed)
                }
                "moira.reconciliation.admit-succeeded" => {
                    Some(moira_tui::TimedReconciliationKind::AdmitSucceeded)
                }
                "moira.reconciliation.retry" => Some(moira_tui::TimedReconciliationKind::Retry),
                _ => None,
            };
            let component = if let Some(kind) = timed_kind {
                moira_tui::ReconciliationBatchComponent::new_timed_with_connector(
                    move || {
                        moira_client::unix::SagaUnixPort::connect(
                            &socket,
                            moira_client::unix::SagaClientLimitsV1::default(),
                        )
                    },
                    sagas,
                    kind,
                )
            } else {
                moira_tui::ReconciliationBatchComponent::new_with_connector(
                    move || {
                        moira_client::unix::SagaUnixPort::connect(
                            &socket,
                            moira_client::unix::SagaClientLimitsV1::default(),
                        )
                    },
                    sagas
                        .into_iter()
                        .map(|saga| ReconciliationProposalV1 {
                            saga: saga.saga,
                            disposition: if action == "moira.reconciliation.fail-terminal" {
                                OperatorDispositionV1::FailTerminal
                            } else {
                                OperatorDispositionV1::KeepUncertain
                            },
                        })
                        .collect(),
                )
            };
            *moira_reconciliation = match component {
                Ok(component) => Some(component),
                Err(error) => {
                    return (ActivityLevel::Warning, "moira", error.into());
                }
            };
            (
                ActivityLevel::Info,
                "moira",
                format!(
                    "{count} independently receipted {disposition_label} reconciliation(s) queued; c arms, Enter confirms"
                ),
            )
        }
        "agora.room.open" => {
            let _ = shell.activate(Surface::Agora);
            agora
                .as_mut()
                .expect("palette enables Agora action only with a component")
                .update(agora_tui::Input::Open, true);
            (
                ActivityLevel::Info,
                "agora",
                "Selected room history queued".into(),
            )
        }
        "agora.room.back" => {
            let _ = shell.activate(Surface::Agora);
            agora
                .as_mut()
                .expect("palette enables Agora action only with a component")
                .update(agora_tui::Input::Back, true);
            (
                ActivityLevel::Info,
                "agora",
                "Returned to the room list".into(),
            )
        }
        "aegis.page.next" | "aegis.page.previous" => {
            let _ = shell.activate(Surface::Aegis);
            let input = if action == "aegis.page.next" {
                aegis_tui::Input::NextPage
            } else {
                aegis_tui::Input::PreviousPage
            };
            aegis
                .as_mut()
                .expect("palette enables Aegis action only with a component")
                .update(input, true);
            (
                ActivityLevel::Info,
                "aegis",
                "Authority page request queued".into(),
            )
        }
        "aegis.epoch.revoke" => {
            let _ = shell.activate(Surface::Aegis);
            if aegis_managed.is_some() {
                return (
                    ActivityLevel::Warning,
                    "aegis",
                    "Close the managed authority session before opening global revocation".into(),
                );
            }
            let Some(epoch) = aegis
                .as_ref()
                .and_then(aegis_tui::Component::authority_epoch)
            else {
                return (
                    ActivityLevel::Warning,
                    "aegis",
                    "No current Aegis epoch; refresh authority state first".into(),
                );
            };
            let socket = aegis_control_socket
                .expect("palette enables Aegis revocation only with a control socket")
                .to_owned();
            *aegis_revoke = Some(aegis_tui::RevokeEpochComponent::new_with_connector(
                move || {
                    aegis_local_client::connect(
                        &socket,
                        aegis_wire::LimitsV1::default(),
                        Duration::from_secs(5),
                    )
                },
                epoch,
            ));
            (
                ActivityLevel::Info,
                "aegis",
                "Exact current-epoch revocation opened; c arms, Enter submits once".into(),
            )
        }
        "aegis.authority.manage" => {
            let _ = shell.activate(Surface::Aegis);
            if aegis_revoke.is_some() {
                return (
                    ActivityLevel::Warning,
                    "aegis",
                    "Close or settle global revocation before opening managed authority".into(),
                );
            }
            let socket = aegis_control_socket
                .expect("palette enables Aegis management only with a control socket")
                .to_owned();
            *aegis_managed = Some(aegis_tui::ManagedAuthorityComponent::new_with_connector(
                move || {
                    aegis_local_client::connect(
                        socket,
                        aegis_wire::LimitsV1::default(),
                        Duration::from_secs(5),
                    )
                },
            ));
            (
                ActivityLevel::Info,
                "aegis",
                "Aegis managed administration session opening; grant handles remain private".into(),
            )
        }
        "ledger.acknowledge.v1" => {
            let _ = shell.activate(Surface::Hermes);
            let Some(selection) = hermes
                .as_ref()
                .and_then(hermes_tui::Component::next_acknowledgement)
            else {
                return (
                    ActivityLevel::Warning,
                    "hermes",
                    "No acknowledgeable next event; refresh the selected Hermes cursor first"
                        .into(),
                );
            };
            let socket = hermes_control_socket
                .expect("palette enables acknowledgement only with a control socket")
                .to_owned();
            *hermes_acknowledge = Some(hermes_tui::AcknowledgeComponent::new_with_connector(
                move || {
                    hermes_local_client::LocalPort::connect(
                        &socket,
                        hermes_local_client::ClientLimits::default(),
                    )
                    .map(|port| (hermes_local_client::CONNECTION_SESSION, port))
                },
                selection,
            ));
            *hermes_compaction = None;
            *hermes_disclosure = None;
            (
                ActivityLevel::Info,
                "hermes",
                "Exact Hermes acknowledgement range opened; select N, then c arms and Enter submits once"
                    .into(),
            )
        }
        "ledger.events.disclose.v1" => {
            let _ = shell.activate(Surface::Hermes);
            let Some(selection) = hermes
                .as_ref()
                .and_then(hermes_tui::Component::next_disclosure)
            else {
                return (
                    ActivityLevel::Warning,
                    "hermes",
                    "No event is available at the selected Hermes position; refresh first".into(),
                );
            };
            let socket = hermes_disclosure_socket
                .expect("palette enables disclosure only with a selected stream")
                .to_owned();
            *hermes_disclosure = Some(hermes_tui::DisclosureComponent::new_with_connector(
                move || {
                    hermes_local_client::LocalPort::connect(
                        &socket,
                        hermes_local_client::ClientLimits::default(),
                    )
                    .map(|port| (hermes_local_client::CONNECTION_SESSION, port))
                },
                selection,
            ));
            *hermes_compaction = None;
            *hermes_acknowledge = None;
            (
                ActivityLevel::Info,
                "hermes",
                "Bounded Hermes event disclosure opened; sensitive content remains hidden until confirmed"
                    .into(),
            )
        }
        "ledger.compact.v1" => {
            let _ = shell.activate(Surface::Hermes);
            let Some(selection) = hermes
                .as_ref()
                .and_then(hermes_tui::Component::next_compaction)
            else {
                return (
                    ActivityLevel::Warning,
                    "hermes",
                    "No compactable retained range; refresh the selected Hermes stream first"
                        .into(),
                );
            };
            let socket = hermes_operator_socket
                .expect("palette enables compaction only with an operator socket")
                .to_owned();
            *hermes_compaction = Some(hermes_tui::CompactionComponent::new_with_connector(
                move || {
                    hermes_local_client::LocalPort::connect(
                        &socket,
                        hermes_local_client::ClientLimits::default(),
                    )
                    .map(|port| (hermes_local_client::CONNECTION_SESSION, port))
                },
                selection,
            ));
            *hermes_disclosure = None;
            *hermes_acknowledge = None;
            (
                ActivityLevel::Info,
                "hermes",
                "Hermes compaction boundary sheet opened; deletion remains unarmed".into(),
            )
        }
        "operations.drain" => {
            let _ = shell.activate(Surface::Operations);
            operations.arm();
            (
                if operations.armed {
                    ActivityLevel::Warning
                } else {
                    ActivityLevel::Info
                },
                "operations",
                operations.notice.clone(),
            )
        }
        "daimon.session.permission" => {
            let _ = shell.activate(Surface::Daimon);
            let Some(agent) = daimon_agent.as_ref() else {
                return (
                    ActivityLevel::Warning,
                    "daimon",
                    "No current Daimon agent observation; refresh the selected session first"
                        .into(),
                );
            };
            let observations = agent.selected_observations();
            if observations.is_empty() {
                return (
                    ActivityLevel::Warning,
                    "daimon",
                    "Selected Daimon sessions do not yet have current observations; refresh first"
                        .into(),
                );
            }
            let challenges = match daimon_permission_challenges(&observations) {
                Ok(challenges) => challenges,
                Err(error) => return (ActivityLevel::Warning, "daimon", error),
            };
            let _ = daimon_execution_policy
                .expect("palette enables permissions only with an execution policy");
            *daimon_permission_composer = match daimon_tui::AgentPermissionComposer::new(challenges)
            {
                Ok(composer) => Some(composer),
                Err(error) => return (ActivityLevel::Warning, "daimon", error.into()),
            };
            *daimon_prompt_composer = None;
            *daimon_control = None;
            (
                ActivityLevel::Info,
                "daimon",
                format!(
                    "Daimon permission review opened for {} selected session(s)",
                    observations.len()
                ),
            )
        }
        "daimon.session.prompt" => {
            let _ = shell.activate(Surface::Daimon);
            let Some(agent) = daimon_agent.as_ref() else {
                return (
                    ActivityLevel::Warning,
                    "daimon",
                    "No current Daimon agent observation; refresh the selected session first"
                        .into(),
                );
            };
            let observations = agent.selected_observations();
            if observations.is_empty() {
                return (
                    ActivityLevel::Warning,
                    "daimon",
                    "Selected Daimon sessions do not yet have current observations; refresh first"
                        .into(),
                );
            }
            if let Some(error) =
                daimon_workset_error(&observations, "Ready with no active turn", |view| {
                    view.phase == AgentSessionPhaseV1::Ready && view.active_turn.is_none()
                })
            {
                return (ActivityLevel::Warning, "daimon", error);
            }
            let _ = daimon_execution_policy
                .expect("palette enables prompting only with an execution policy");
            *daimon_prompt_composer = match daimon_tui::AgentPromptComposer::new(observations.len())
            {
                Ok(composer) => Some(composer),
                Err(error) => return (ActivityLevel::Warning, "daimon", error.into()),
            };
            *daimon_permission_composer = None;
            *daimon_control = None;
            (
                ActivityLevel::Info,
                "daimon",
                format!(
                    "Daimon prompt composer opened for {} selected session(s)",
                    observations.len()
                ),
            )
        }
        "daimon.session.reattach" => {
            let _ = shell.activate(Surface::Daimon);
            let Some(agent) = daimon_agent.as_ref() else {
                return (
                    ActivityLevel::Warning,
                    "daimon",
                    "No current Daimon agent observation; refresh the selected session first"
                        .into(),
                );
            };
            let observations = agent.selected_observations();
            if observations.is_empty() {
                return (
                    ActivityLevel::Warning,
                    "daimon",
                    "Selected Daimon sessions do not yet have current observations; refresh first"
                        .into(),
                );
            }
            let Some(policy) = daimon_execution_policy else {
                return (
                    ActivityLevel::Warning,
                    "daimon",
                    "Daimon execution policy became unavailable".into(),
                );
            };
            let requests = match daimon_reattach_requests(&observations, policy) {
                Ok(requests) => requests,
                Err(error) => return (ActivityLevel::Warning, "daimon", error),
            };
            let socket = daimon_control_socket
                .expect("palette enables Daimon reattachment only with an agent socket")
                .to_owned();
            *daimon_control = match daimon_tui::AgentControlBatchComponent::new_with_connector(
                move || {
                    daimon_client::unix::connect_agent(
                        &socket,
                        daimon_wire::LimitsV1::default(),
                        daimon_wire::agent::AgentWireLimitsV1::default(),
                        Duration::from_secs(5),
                    )
                },
                requests,
            ) {
                Ok(control) => Some(control),
                Err(error) => return (ActivityLevel::Warning, "daimon", error.into()),
            };
            *daimon_prompt_composer = None;
            *daimon_permission_composer = None;
            (
                ActivityLevel::Info,
                "daimon",
                "Exact Daimon reattach-set proposal opened; c arms, Enter starts the bounded run"
                    .into(),
            )
        }
        "daimon.session.advance" | "daimon.session.cancel" | "daimon.session.close" => {
            let _ = shell.activate(Surface::Daimon);
            let Some(agent) = daimon_agent.as_ref() else {
                return (
                    ActivityLevel::Warning,
                    "daimon",
                    "No current Daimon agent observation; refresh the selected session first"
                        .into(),
                );
            };
            let observations = agent.selected_observations();
            if observations.is_empty() {
                return (
                    ActivityLevel::Warning,
                    "daimon",
                    "Selected Daimon sessions do not yet have current observations; refresh first"
                        .into(),
                );
            }
            let requirement = match action {
                "daimon.session.advance" => "TurnActive with an active turn",
                "daimon.session.cancel" => "TurnActive or AwaitingPermission with an active turn",
                "daimon.session.close" => "Ready with no active turn",
                _ => unreachable!("closed Daimon control action"),
            };
            if let Some(error) =
                daimon_workset_error(&observations, requirement, |view| match action {
                    "daimon.session.advance" => {
                        view.phase == AgentSessionPhaseV1::TurnActive && view.active_turn.is_some()
                    }
                    "daimon.session.cancel" => {
                        matches!(
                            view.phase,
                            AgentSessionPhaseV1::TurnActive
                                | AgentSessionPhaseV1::AwaitingPermission
                        ) && view.active_turn.is_some()
                    }
                    "daimon.session.close" => {
                        view.phase == AgentSessionPhaseV1::Ready && view.active_turn.is_none()
                    }
                    _ => false,
                })
            {
                return (ActivityLevel::Warning, "daimon", error);
            }
            let mut requests = Vec::with_capacity(observations.len());
            for observation in observations {
                let (key, now) = match fresh_operation_facts() {
                    Ok(facts) => facts,
                    Err(error) => return (ActivityLevel::Warning, "daimon", error),
                };
                match daimon_control_request(action, &observation.view, key, now) {
                    Ok(request) => requests.push(request),
                    Err(reason) => return (ActivityLevel::Warning, "daimon", reason.into()),
                }
            }
            let socket = daimon_control_socket
                .expect("palette enables Daimon control only with an agent socket")
                .to_owned();
            *daimon_control = match daimon_tui::AgentControlBatchComponent::new_with_connector(
                move || {
                    daimon_client::unix::connect_agent(
                        &socket,
                        daimon_wire::LimitsV1::default(),
                        daimon_wire::agent::AgentWireLimitsV1::default(),
                        Duration::from_secs(5),
                    )
                },
                requests,
            ) {
                Ok(control) => Some(control),
                Err(error) => return (ActivityLevel::Warning, "daimon", error.into()),
            };
            (
                ActivityLevel::Info,
                "daimon",
                "Exact Daimon session-set proposal opened; c arms, Enter starts the bounded run"
                    .into(),
            )
        }
        "suite.refresh" => refresh_current(
            Surface::Overview,
            *focus,
            chronos,
            vellum,
            agora,
            moira,
            moira_participation,
            daimon,
            daimon_agent,
            hermes,
            aegis,
            tellus,
        ),
        "view.refresh" => refresh_current(
            shell.active(),
            *focus,
            chronos,
            vellum,
            agora,
            moira,
            moira_participation,
            daimon,
            daimon_agent,
            hermes,
            aegis,
            tellus,
        ),
        _ => (
            ActivityLevel::Warning,
            "pantheon",
            format!("Unknown workbench action: {action}"),
        ),
    }
}

fn invoke_chronos_workbench(
    shell: &mut Shell,
    focus: &mut Focus,
    chronos: &mut Option<Hub>,
    intent: WorkbenchIntent,
) -> (ActivityLevel, &'static str, String) {
    activate_peer_surface(shell, Surface::Chronos);
    *focus = Focus::Chronos;
    match chronos
        .as_mut()
        .expect("palette enables Chronos action only with a hub")
        .invoke_workbench(intent)
    {
        Ok(message) => (ActivityLevel::Info, "chronos", message.into()),
        Err(message) => (ActivityLevel::Warning, "chronos", message.into()),
    }
}

fn activate_peer_surface(shell: &mut Shell, preferred: Surface) {
    if !shell.activate(preferred) {
        let _ = shell.activate(Surface::Workspace);
    }
}

fn command_feedback(action: &str, level: ActivityLevel, message: &str) -> String {
    let (state, next) = match (level, action) {
        (ActivityLevel::Success, _) => ("DONE", "Next: continue on the active surface."),
        (ActivityLevel::Uncertain, _) => (
            "OUTCOME UNKNOWN",
            "Next: inspect Activity and recover the exact retained operation; do not retry blindly.",
        ),
        (ActivityLevel::Warning, "chronos.task.new") => (
            "REFUSED",
            "Next: enable Chronos writes, refresh current task state, then run the command again.",
        ),
        (ActivityLevel::Warning, "vellum.document.open") => (
            "REFUSED",
            "Next: configure Vellum with runtime file admission, then retry Ctrl-O.",
        ),
        (ActivityLevel::Warning, "agora.room.create") => (
            "REFUSED",
            "Next: configure Agora control, refresh the room revision, then retry.",
        ),
        (ActivityLevel::Warning, _) => (
            "REFUSED",
            "Next: inspect Configuration (Ctrl-,) and Activity (Ctrl-G), then retry only after the requirement is satisfied.",
        ),
        (ActivityLevel::Info, "chronos.task.new") => (
            "READY",
            "Next: enter one or more task IDs; Tab adds a row, Enter reviews owner fields, then Enter submits once.",
        ),
        (ActivityLevel::Info, "vellum.document.open") => (
            "READY",
            "Next: enter a file under an admitted root; Enter opens and Esc cancels.",
        ),
        (ActivityLevel::Info, "agora.room.create") => (
            "READY",
            "Next: complete the revision-bound room form, arm it, and confirm once.",
        ),
        (ActivityLevel::Info, action) if action.starts_with("navigate.") => (
            "OPENED",
            "Next: use M-x for available work or the surface footer for local controls.",
        ),
        (ActivityLevel::Info, "view.refresh" | "suite.refresh") => (
            "QUEUED",
            "Next: watch the status evidence settle; Ctrl-G retains the receipt.",
        ),
        (ActivityLevel::Info, _) => (
            "READY",
            "Next: follow the active workflow footer; Esc cancels before submission.",
        ),
    };
    format!("{state} · {message} · {next}")
}

const fn chronos_surface_active(surface: Surface, focus: Focus) -> bool {
    matches!(surface, Surface::Chronos)
        || matches!(surface, Surface::Workspace) && matches!(focus, Focus::Chronos)
}

const fn vellum_surface_active(surface: Surface, focus: Focus) -> bool {
    matches!(surface, Surface::Vellum)
        || matches!(surface, Surface::Workspace) && matches!(focus, Focus::Vellum)
}

fn route_key_for_surface(key: KeyEvent, surface: Surface, focus: Focus) -> RoutedInput {
    let effective_focus = match surface {
        Surface::Chronos => Focus::Chronos,
        Surface::Vellum => Focus::Vellum,
        _ => focus,
    };
    if key.kind == KeyEventKind::Press
        && key.modifiers == KeyModifiers::NONE
        && key.code == KeyCode::Tab
    {
        return match surface {
            Surface::Chronos => RoutedInput::Chronos(key),
            Surface::Vellum => RoutedInput::Vellum(vellum_tui::translate_key(key)),
            _ => route_key(key, effective_focus),
        };
    }
    route_key(key, effective_focus)
}

#[allow(clippy::too_many_arguments)]
fn refresh_current<T: ServicePort>(
    surface: Surface,
    focus: Focus,
    chronos: &mut Option<Hub>,
    vellum: &mut Option<ServiceEditor<T>>,
    agora: &mut Option<agora_tui::Component>,
    moira: &mut Option<moira_tui::Component>,
    moira_participation: &mut Option<moira_tui::ParticipationComponent>,
    daimon: &mut Option<daimon_tui::Component>,
    daimon_agent: &mut Option<daimon_tui::AgentSetComponent>,
    hermes: &mut Option<hermes_tui::Component>,
    aegis: &mut Option<aegis_tui::Component>,
    tellus: &mut Option<tellus::Component>,
) -> (ActivityLevel, &'static str, String) {
    let refresh_key = crossterm::event::KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
    match surface {
        Surface::Overview => {
            if let Some(component) = chronos {
                component.update(refresh_key);
            }
            if let Some(component) = vellum
                && let Err(error) = component.apply(ServiceAction::Refresh)
            {
                return (ActivityLevel::Warning, "vellum", error.to_string());
            }
            if let Some(component) = agora {
                component.update(agora_tui::Input::Refresh, true);
            }
            if let Some(component) = moira {
                component.update(moira_tui::Input::Refresh, true);
            }
            if let Some(component) = moira_participation {
                component.update(moira_tui::Input::Refresh, true);
            }
            if let Some(component) = daimon {
                component.update(daimon_tui::Input::Refresh, true);
            }
            if let Some(component) = daimon_agent {
                component.update(daimon_tui::AgentSetInput::RefreshAll, true);
            }
            if let Some(component) = hermes {
                component.update(hermes_tui::Input::Refresh, true);
            }
            if let Some(component) = aegis {
                component.update(aegis_tui::Input::Refresh, true);
            }
            if let Some(component) = tellus {
                component.update(tellus::Input::Refresh, true);
            }
            (
                ActivityLevel::Info,
                "pantheon",
                "Fresh observations queued across configured peers".into(),
            )
        }
        Surface::Agenda => refresh_current(
            Surface::Overview,
            focus,
            chronos,
            vellum,
            agora,
            moira,
            moira_participation,
            daimon,
            daimon_agent,
            hermes,
            aegis,
            tellus,
        ),
        Surface::Chronos => {
            if let Some(component) = chronos {
                component.update(refresh_key);
                (ActivityLevel::Info, "chronos", "Refresh queued".into())
            } else {
                (ActivityLevel::Warning, "chronos", "Not configured".into())
            }
        }
        Surface::Vellum => match vellum {
            Some(component) => match component.apply(ServiceAction::Refresh) {
                Ok(()) => (ActivityLevel::Info, "vellum", "Refresh queued".into()),
                Err(error) => (ActivityLevel::Warning, "vellum", error.to_string()),
            },
            None => (ActivityLevel::Warning, "vellum", "Not configured".into()),
        },
        Surface::Workspace if focus == Focus::Chronos => {
            if let Some(component) = chronos {
                component.update(refresh_key);
                (ActivityLevel::Info, "chronos", "Refresh queued".into())
            } else {
                (ActivityLevel::Warning, "chronos", "Not configured".into())
            }
        }
        Surface::Workspace => match vellum {
            Some(component) => match component.apply(ServiceAction::Refresh) {
                Ok(()) => (ActivityLevel::Info, "vellum", "Refresh queued".into()),
                Err(error) => (ActivityLevel::Warning, "vellum", error.to_string()),
            },
            None => (ActivityLevel::Warning, "vellum", "Not configured".into()),
        },
        Surface::Agora => refresh_peer(agora.as_mut(), agora_tui::Input::Refresh, "agora"),
        Surface::Moira => {
            if let Some(component) = moira_participation {
                component.update(moira_tui::Input::Refresh, true);
            }
            if moira.is_some() {
                refresh_peer(moira.as_mut(), moira_tui::Input::Refresh, "moira")
            } else if moira_participation.is_some() {
                (
                    ActivityLevel::Info,
                    "moira",
                    "Participation refresh queued".into(),
                )
            } else {
                (ActivityLevel::Warning, "moira", "Not configured".into())
            }
        }
        Surface::Daimon => {
            if let Some(component) = daimon_agent {
                component.update(daimon_tui::AgentSetInput::RefreshAll, true);
                (
                    ActivityLevel::Info,
                    "daimon",
                    "Session-set refresh queued".into(),
                )
            } else {
                refresh_peer(daimon.as_mut(), daimon_tui::Input::Refresh, "daimon")
            }
        }
        Surface::Hermes => refresh_peer(hermes.as_mut(), hermes_tui::Input::Refresh, "hermes"),
        Surface::Aegis => refresh_peer(aegis.as_mut(), aegis_tui::Input::Refresh, "aegis"),
        Surface::Tellus => refresh_peer(tellus.as_mut(), tellus::Input::Refresh, "tellus"),
        Surface::Operations => (
            ActivityLevel::Info,
            "operations",
            "Lifecycle status is maintained from the live local host".into(),
        ),
        Surface::Configuration => (
            ActivityLevel::Info,
            "configuration",
            "Session configuration is derived from immutable launch facts".into(),
        ),
    }
}

trait Refreshable<I> {
    fn refresh(&mut self, input: I);
}

impl Refreshable<agora_tui::Input> for agora_tui::Component {
    fn refresh(&mut self, input: agora_tui::Input) {
        self.update(input, true);
    }
}

impl Refreshable<moira_tui::Input> for moira_tui::Component {
    fn refresh(&mut self, input: moira_tui::Input) {
        self.update(input, true);
    }
}

impl Refreshable<daimon_tui::Input> for daimon_tui::Component {
    fn refresh(&mut self, input: daimon_tui::Input) {
        self.update(input, true);
    }
}

impl Refreshable<hermes_tui::Input> for hermes_tui::Component {
    fn refresh(&mut self, input: hermes_tui::Input) {
        self.update(input, true);
    }
}

impl Refreshable<aegis_tui::Input> for aegis_tui::Component {
    fn refresh(&mut self, input: aegis_tui::Input) {
        self.update(input, true);
    }
}

impl Refreshable<tellus::Input> for tellus::Component {
    fn refresh(&mut self, input: tellus::Input) {
        self.update(input, true);
    }
}

fn refresh_peer<C, I>(
    component: Option<&mut C>,
    input: I,
    peer: &'static str,
) -> (ActivityLevel, &'static str, String)
where
    C: Refreshable<I>,
{
    if let Some(component) = component {
        component.refresh(input);
        (ActivityLevel::Info, peer, "Refresh queued".into())
    } else {
        (ActivityLevel::Warning, peer, "Not configured".into())
    }
}

#[allow(clippy::too_many_arguments)]
fn session_configuration(
    shell: &Shell,
    chronos: Option<&Hub>,
    vellum: bool,
    vellum_allow_add_file: bool,
    agora: bool,
    agora_control: bool,
    moira: bool,
    moira_control: bool,
    daimon: bool,
    daimon_agent: bool,
    daimon_control: bool,
    daimon_execution: bool,
    hermes: bool,
    hermes_operator: bool,
    hermes_disclosure: bool,
    hermes_control: bool,
    aegis: bool,
    aegis_control: bool,
    tellus: bool,
) -> Vec<String> {
    let surfaces = shell
        .surfaces()
        .iter()
        .map(|surface| surface.label())
        .collect::<Vec<_>>()
        .join(" · ");
    let disposition = |configured| {
        if configured {
            "configured"
        } else {
            "not configured"
        }
    };
    vec![
        "scope               single user · local machine".into(),
        format!("surfaces            {surfaces}"),
        format!(
            "Chronos            {} · writes {} · scheduling {}",
            disposition(chronos.is_some()),
            if chronos.is_some_and(Hub::writes_enabled) {
                "enabled"
            } else {
                "disabled"
            },
            if chronos.is_some_and(Hub::scheduling_enabled) {
                "enabled"
            } else {
                "disabled"
            }
        ),
        format!(
            "Vellum             {} · Ctrl-O file admission {}",
            disposition(vellum),
            if vellum_allow_add_file {
                "selected"
            } else {
                "owner-controlled"
            }
        ),
        format!(
            "Agora              {} · room creation {}",
            disposition(agora),
            if agora_control { "selected" } else { "absent" }
        ),
        format!(
            "Moira              {} · reconciliation control {}",
            disposition(moira),
            if moira_control { "selected" } else { "absent" }
        ),
        format!(
            "Daimon             {} · selected agent {} · control {} · execution policy {}",
            disposition(daimon || daimon_agent),
            if daimon_agent { "configured" } else { "absent" },
            if daimon_control { "selected" } else { "absent" },
            if daimon_execution {
                "selected"
            } else {
                "absent"
            }
        ),
        format!(
            "Hermes             {} · disclosure {} · compaction {} · acknowledgement control {}",
            disposition(hermes),
            if hermes_disclosure {
                "selected"
            } else {
                "absent"
            },
            if hermes_operator {
                "selected"
            } else {
                "absent"
            },
            if hermes_control { "selected" } else { "absent" }
        ),
        format!(
            "Aegis              {} · epoch administration {}",
            disposition(aegis),
            if aegis_control { "selected" } else { "absent" }
        ),
        format!("Tellus             {}", disposition(tellus)),
        String::new(),
        "Credentials, grant handles, and provider locators are intentionally omitted.".into(),
    ]
}

struct ProviderInspection {
    updates: Receiver<Vec<String>>,
    refresh: bool,
    started: Instant,
}

#[derive(Clone)]
struct ProviderQualificationPlan {
    owner: String,
    document_digest: String,
    provider: ProviderCatalogEntryProjection,
}

enum ProviderQualificationWorkerResult {
    Planned(ProviderQualificationPlan),
    Qualified(ProviderQualificationProjection),
}

struct ProviderQualificationWorker {
    updates: Receiver<Result<ProviderQualificationWorkerResult, String>>,
    started: Instant,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ProviderQualificationPhase {
    Planning,
    Reviewing,
    Qualifying,
    Complete,
}

struct ProviderQualificationWorkflow {
    phase: ProviderQualificationPhase,
    plan: Option<ProviderQualificationPlan>,
    receipt: Option<ProviderQualificationProjection>,
    armed: bool,
    notice: String,
    worker: Option<ProviderQualificationWorker>,
}

impl ProviderQualificationWorkflow {
    fn new(profile: Option<WorkspaceProfile>) -> Self {
        let mut workflow = Self {
            phase: ProviderQualificationPhase::Planning,
            plan: None,
            receipt: None,
            armed: false,
            notice: "PLANNING · Asking Daimon for the exact redacted local-model profile; no provider request has occurred."
                .into(),
            worker: None,
        };
        workflow.begin_plan(profile);
        workflow
    }

    fn begin_plan(&mut self, profile: Option<WorkspaceProfile>) {
        let Some(host) = profile
            .and_then(|profile| profile.daimon)
            .filter(|daimon| daimon.model_socket.is_some())
            .and_then(|daimon| daimon.host)
        else {
            self.phase = ProviderQualificationPhase::Complete;
            self.notice = "REFUSED · No hosted Daimon local-model owner is selected · Next: configure and preflight one local-model owner, restart once, then retry."
                .into();
            return;
        };
        let (sender, updates) = sync_channel(1);
        std::thread::spawn(move || {
            let result = (|| {
                let catalog = provider_catalog(&host.executable, &host.config)?;
                if catalog.owner != "daimon-local-model" || catalog.providers.len() != 1 {
                    return Err(
                        "selected owner does not publish one local-model qualification target"
                            .into(),
                    );
                }
                let provider = catalog.providers[0].clone();
                if provider.model.is_none()
                    || provider.wire.as_deref() != Some("openai_chat_completions")
                {
                    return Err("selected owner returned an unsupported model profile".into());
                }
                Ok(ProviderQualificationWorkerResult::Planned(
                    ProviderQualificationPlan {
                        owner: catalog.owner,
                        document_digest: catalog.document_digest,
                        provider,
                    },
                ))
            })();
            let _ = sender.send(result);
        });
        self.phase = ProviderQualificationPhase::Planning;
        self.worker = Some(ProviderQualificationWorker {
            updates,
            started: Instant::now(),
        });
    }

    fn key(&mut self, key: KeyEvent, profile: Option<WorkspaceProfile>) {
        match self.phase {
            ProviderQualificationPhase::Reviewing => {
                if is_key(key, KeyCode::Char('c'), KeyModifiers::NONE) {
                    self.armed = true;
                    self.notice = "ARMED · Enter sends exactly one bounded completion-conformance request through the selected Daimon owner; transport uncertainty will not be retried."
                        .into();
                } else if is_key(key, KeyCode::Enter, KeyModifiers::NONE) && self.armed {
                    self.begin_qualification(profile);
                } else if is_key(key, KeyCode::Char('r'), KeyModifiers::NONE) {
                    self.plan = None;
                    self.receipt = None;
                    self.armed = false;
                    self.notice =
                        "PLANNING · Revalidating the owner document before qualification.".into();
                    self.begin_plan(profile);
                }
            }
            ProviderQualificationPhase::Complete => {
                if is_key(key, KeyCode::Char('r'), KeyModifiers::NONE) {
                    self.plan = None;
                    self.receipt = None;
                    self.armed = false;
                    self.notice = "PLANNING · Revalidating the owner document; an earlier operation identity will never be redispatched."
                        .into();
                    self.begin_plan(profile);
                }
            }
            ProviderQualificationPhase::Planning | ProviderQualificationPhase::Qualifying => {}
        }
    }

    fn begin_qualification(&mut self, profile: Option<WorkspaceProfile>) {
        let Some(host) = profile
            .and_then(|profile| profile.daimon)
            .filter(|daimon| daimon.model_socket.is_some())
            .and_then(|daimon| daimon.host)
        else {
            self.notice = "REFUSED · The reviewed local-model owner is no longer selected; no provider request was attempted."
                .into();
            self.armed = false;
            return;
        };
        let Some(plan) = self.plan.clone() else {
            self.notice = "REFUSED · The reviewed owner plan is unavailable · Next: press r to revalidate before qualifying."
                .into();
            self.armed = false;
            return;
        };
        let operation_id = match fresh_identity() {
            Ok(bytes) => bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            Err(reason) => {
                self.notice = format!("REFUSED · {reason}");
                self.armed = false;
                return;
            }
        };
        let request = serde_json::json!({
            "version": 1,
            "operation_id": operation_id,
            "expected_document_digest": plan.document_digest,
        });
        let request = match serde_json::to_vec(&request) {
            Ok(request) => request,
            Err(_) => {
                self.notice =
                    "REFUSED · Qualification request encoding failed before owner contact.".into();
                self.armed = false;
                return;
            }
        };
        let (sender, updates) = sync_channel(1);
        std::thread::spawn(move || {
            let result = (|| {
                let receipt = qualify_provider(&host.executable, &host.config, &request)?;
                validate_provider_qualification(&receipt, Some(&plan.provider))?;
                if receipt.operation_id != operation_id
                    || receipt.document_digest != plan.document_digest
                {
                    return Err(
                        "owner qualification receipt does not bind the reviewed plan".into(),
                    );
                }
                Ok(ProviderQualificationWorkerResult::Qualified(receipt))
            })();
            let _ = sender.send(result);
        });
        self.phase = ProviderQualificationPhase::Qualifying;
        self.armed = false;
        self.notice = "QUALIFYING · Daimon retained the operation marker before one non-retried provider request; the interface remains responsive."
            .into();
        self.worker = Some(ProviderQualificationWorker {
            updates,
            started: Instant::now(),
        });
    }

    fn settle(&mut self) -> Option<(ActivityLevel, Duration)> {
        let worker = self.worker.as_ref()?;
        let result = match worker.updates.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err("qualification worker ended without an owner result".into())
            }
        };
        let elapsed = worker.started.elapsed();
        self.worker = None;
        match result {
            Ok(ProviderQualificationWorkerResult::Planned(plan)) => {
                let retained = plan.provider.qualification.clone();
                self.plan = Some(plan);
                self.receipt = retained;
                self.phase = ProviderQualificationPhase::Reviewing;
                self.notice = "READY · Review the exact redacted provider and bounded completion-only check; press c to arm."
                    .into();
                Some((ActivityLevel::Success, elapsed))
            }
            Ok(ProviderQualificationWorkerResult::Qualified(receipt)) => {
                let level = if receipt.disposition == "qualified_completion" {
                    ActivityLevel::Success
                } else {
                    ActivityLevel::Warning
                };
                self.notice = match receipt.disposition.as_str() {
                    "qualified_completion" => "QUALIFIED · Daimon retained attributed completion, usage and evidence · Tool continuation remains not exercised · Next: inspect the receipt or press r for a fresh reviewed operation.",
                    "outcome_unknown" => "OUTCOME UNKNOWN · Daimon will not retry this operation identity · Next: inspect the retained receipt and provider before choosing a fresh qualification.",
                    _ => "NONCONFORMANT · Daimon retained the failed completion check · Next: inspect deviations, correct the provider/profile, then press r for a fresh reviewed operation.",
                }
                .into();
                self.receipt = Some(receipt);
                self.phase = ProviderQualificationPhase::Complete;
                Some((level, elapsed))
            }
            Err(reason) => {
                self.notice = format!(
                    "REFUSED · {reason} · Next: inspect the selected owner/profile and retry only after correction"
                );
                self.phase = ProviderQualificationPhase::Complete;
                Some((ActivityLevel::Warning, elapsed))
            }
        }
    }

    fn lines(&self) -> Vec<String> {
        let mut lines = vec![
            "owner               Daimon local-model".into(),
            "effect              one bounded, non-retried completion-conformance request".into(),
            "retention           owner marker before dispatch; replay never redispatches".into(),
            "human gate          review → arm → confirm".into(),
            String::new(),
        ];
        if let Some(plan) = &self.plan {
            let provider = &plan.provider;
            lines.extend([
                format!("owner identity      {}", plan.owner),
                format!("document digest     {}", plan.document_digest),
                format!("provider             {}", provider.provider_id),
                format!("profile              {}", provider.harness_profile_id),
                format!(
                    "model                {}",
                    provider.model.as_deref().unwrap_or("not published")
                ),
                format!(
                    "wire                 {} · {}:{}",
                    provider.wire.as_deref().unwrap_or("not published"),
                    provider.endpoint_scope.as_deref().unwrap_or("not published"),
                    provider.endpoint_port.unwrap_or_default()
                ),
                format!(
                    "policy               structured output {} · structured tools {} · temperature milli {}",
                    provider.structured_output.unwrap_or(false),
                    provider.structured_tools.unwrap_or(false),
                    provider.sampling_temperature_milli.unwrap_or_default()
                ),
                format!("profile digest       {}", provider.profile_document_digest),
            ]);
        }
        if let Some(receipt) = &self.receipt {
            lines.extend([
                String::new(),
                "Latest owner-retained qualification".into(),
                format!("operation            {}", receipt.operation_id),
                format!("disposition          {}", receipt.disposition),
                format!("host-observed unix ms {}", receipt.host_observed_at_unix_ms),
                format!(
                    "usage                 {} input · {} output tokens · {} output bytes",
                    receipt.input_tokens, receipt.output_tokens, receipt.output_bytes
                ),
                format!(
                    "evidence              {}",
                    receipt
                        .evidence_digest
                        .as_deref()
                        .unwrap_or("not available")
                ),
                format!("completion           {}", receipt.completion),
                format!("tool continuation    {}", receipt.tool_continuation),
                format!("replayed             {}", receipt.replayed),
            ]);
            for deviation in &receipt.known_deviations {
                lines.push(format!("deviation            {deviation}"));
            }
        }
        lines.extend([
            String::new(),
            "Boundary: completion qualification is not task invocation, delegation, tool-continuation conformance, workflow completion, or authority.".into(),
            self.notice.clone(),
            match self.phase {
                ProviderQualificationPhase::Reviewing => {
                    "c arm · Enter confirm when armed · r revalidate · Esc close"
                }
                ProviderQualificationPhase::Complete => "r start a fresh reviewed operation · Esc close",
                ProviderQualificationPhase::Planning | ProviderQualificationPhase::Qualifying => {
                    "Owner operation in progress · Esc hides but does not cancel"
                }
            }
            .into(),
        ]);
        lines
    }
}

struct WorkItemBindingInspection {
    result: Receiver<Result<WorkItemBindingEvidence, String>>,
    started: Instant,
    draft: String,
    lens: Option<AgentCollaborationLens>,
}

#[derive(Debug)]
struct WorkItemBindingEvidence {
    task_id: String,
    external_task_key: String,
    owner: String,
    document_digest: String,
}

const PROVIDER_EDIT_FIELDS: [&str; 11] = [
    "provider ID",
    "harness profile ID",
    "policy digest",
    "program (absolute path)",
    "arguments (one per line; Ctrl-J adds a line)",
    "inherited environment names (one per line; Ctrl-J adds a line)",
    "ACP authentication method (optional)",
    "isolation (direct or macos_seatbelt)",
    "seatbelt policy path (when selected)",
    "seatbelt policy digest (when selected)",
    "transport (acp or claude_stream_json)",
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderEditPlanProjection {
    version: u8,
    owner: String,
    operation_id: String,
    action: String,
    expected_document_digest: String,
    current_document_digest: String,
    candidate_document_digest: String,
    plan_digest: String,
    provider: ProviderCatalogEntryProjection,
    changed_fields: Vec<String>,
    restart_required: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderEditReceiptProjection {
    version: u8,
    owner: String,
    operation_id: String,
    disposition: String,
    applied: bool,
    replayed: bool,
    document_digest: String,
    provider_id: String,
    restart_required: bool,
}

enum ProviderEditWorkerResult {
    Planned {
        plan: Box<ProviderEditPlanProjection>,
        provider: serde_json::Value,
    },
    Applied(ProviderEditReceiptProjection),
}

struct ProviderEditWorker {
    updates: Receiver<Result<ProviderEditWorkerResult, String>>,
    started: Instant,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ProviderEditPhase {
    Editing,
    Planning,
    Reviewing,
    Applying,
    Complete,
}

struct ProviderEditWorkflow {
    values: [String; 11],
    selected: usize,
    phase: ProviderEditPhase,
    plan: Option<ProviderEditPlanProjection>,
    provider: Option<serde_json::Value>,
    armed: bool,
    notice: String,
    worker: Option<ProviderEditWorker>,
}

impl ProviderEditWorkflow {
    fn new() -> Self {
        let identity = fresh_identity()
            .map(|bytes| bytes.iter().map(|byte| format!("{byte:02x}")).collect())
            .unwrap_or_default();
        let harness = fresh_identity()
            .map(|bytes| bytes.iter().map(|byte| format!("{byte:02x}")).collect())
            .unwrap_or_default();
        let mut values: [String; 11] = std::array::from_fn(|_| String::new());
        values[0] = identity;
        values[1] = harness;
        values[7] = "direct".into();
        values[10] = "acp".into();
        Self {
            values,
            selected: 0,
            phase: ProviderEditPhase::Editing,
            plan: None,
            provider: None,
            armed: false,
            notice: "EDITING · Complete the owner fields; no configuration effect has occurred."
                .into(),
            worker: None,
        }
    }

    fn key(&mut self, key: KeyEvent, profile: Option<WorkspaceProfile>) {
        match self.phase {
            ProviderEditPhase::Editing => match key.code {
                KeyCode::Tab if key.modifiers == KeyModifiers::NONE => {
                    self.selected = (self.selected + 1) % self.values.len();
                }
                KeyCode::BackTab => {
                    self.selected = self
                        .selected
                        .checked_sub(1)
                        .unwrap_or(self.values.len() - 1);
                }
                KeyCode::Up if key.modifiers == KeyModifiers::NONE => {
                    self.selected = self.selected.saturating_sub(1);
                }
                KeyCode::Down if key.modifiers == KeyModifiers::NONE => {
                    self.selected = (self.selected + 1).min(self.values.len() - 1);
                }
                KeyCode::Backspace if key.modifiers == KeyModifiers::NONE => {
                    self.values[self.selected].pop();
                }
                _ if is_control_char(key, 'u') => {
                    self.values[self.selected].clear();
                }
                KeyCode::Enter if key.modifiers == KeyModifiers::NONE => {
                    if self.selected + 1 < self.values.len() {
                        self.selected += 1;
                    } else {
                        self.begin_plan(profile);
                    }
                }
                _ if is_control_char(key, 'j') && matches!(self.selected, 4 | 5) => {
                    if self.values[self.selected].len() < 16_384 {
                        self.values[self.selected].push('\n');
                    }
                }
                _ => {
                    if let Some(character) = bounded_text_character(key)
                        && self.values[self.selected].len() + character.len_utf8() <= 16_384
                    {
                        self.values[self.selected].push(character);
                    }
                }
            },
            ProviderEditPhase::Reviewing => match key.code {
                KeyCode::Char('c') if key.modifiers == KeyModifiers::NONE => {
                    self.armed = !self.armed;
                    self.notice = if self.armed {
                        "ARMED · Enter applies only the reviewed candidate digest; c disarms."
                            .into()
                    } else {
                        "REVIEWING · Plan disarmed; inspect the exact owner facts before applying."
                            .into()
                    };
                }
                KeyCode::Enter if key.modifiers == KeyModifiers::NONE && self.armed => {
                    self.begin_apply(profile);
                }
                KeyCode::Char('e') if key.modifiers == KeyModifiers::NONE => {
                    self.phase = ProviderEditPhase::Editing;
                    self.plan = None;
                    self.provider = None;
                    self.armed = false;
                    self.notice =
                        "EDITING · Prior plan discarded; submit a fresh owner plan.".into();
                }
                _ => {}
            },
            ProviderEditPhase::Complete => {
                if is_key(key, KeyCode::Char('n'), KeyModifiers::NONE) {
                    *self = Self::new();
                }
            }
            ProviderEditPhase::Planning | ProviderEditPhase::Applying => {}
        }
    }

    fn paste(&mut self, text: &str) {
        if self.phase != ProviderEditPhase::Editing {
            return;
        }
        let available = 16_384usize.saturating_sub(self.values[self.selected].len());
        self.values[self.selected].extend(
            text.chars()
                .filter(|character| !character.is_control() || *character == '\n')
                .take(available),
        );
    }

    fn begin_plan(&mut self, profile: Option<WorkspaceProfile>) {
        let Some(host) = profile
            .and_then(|profile| profile.daimon)
            .and_then(|daimon| daimon.host)
        else {
            self.notice = "REFUSED · No hosted Daimon owner is selected · Next: select and preflight a hosted interactive-agent owner, restart once, then retry."
                .into();
            return;
        };
        let provider = match self.provider_value() {
            Ok(provider) => provider,
            Err(reason) => {
                self.notice = format!("REFUSED · {reason}");
                return;
            }
        };
        let operation_id = match fresh_identity() {
            Ok(bytes) => bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            Err(reason) => {
                self.notice = format!("REFUSED · {reason}");
                return;
            }
        };
        let (sender, receiver) = sync_channel(1);
        std::thread::spawn(move || {
            let result = (|| {
                let catalog = provider_catalog(&host.executable, &host.config)?;
                if catalog.owner != "daimon-local-agent" {
                    return Err(
                        "selected owner does not publish interactive-agent provider edits".into(),
                    );
                }
                let request = serde_json::json!({
                    "version": 1,
                    "operation_id": operation_id,
                    "expected_document_digest": catalog.document_digest,
                    "provider": provider
                });
                let bytes = invoke_provider_owner(
                    &host.executable,
                    &host.config,
                    "--plan-provider-edit",
                    Some(
                        &serde_json::to_vec(&request)
                            .map_err(|_| "provider plan request could not be encoded")?,
                    ),
                )?;
                let plan: ProviderEditPlanProjection = serde_json::from_slice(&bytes)
                    .map_err(|_| "owner returned an invalid provider edit plan")?;
                validate_provider_edit_plan(
                    &plan,
                    &operation_id,
                    &catalog.document_digest,
                    &provider,
                )?;
                Ok(ProviderEditWorkerResult::Planned {
                    plan: Box::new(plan),
                    provider,
                })
            })();
            let _ = sender.send(result);
        });
        self.phase = ProviderEditPhase::Planning;
        self.notice = "PLANNING · Daimon is validating the complete current and candidate owner documents; the interface remains responsive."
            .into();
        self.worker = Some(ProviderEditWorker {
            updates: receiver,
            started: Instant::now(),
        });
    }

    fn begin_apply(&mut self, profile: Option<WorkspaceProfile>) {
        let Some(host) = profile
            .and_then(|profile| profile.daimon)
            .and_then(|daimon| daimon.host)
        else {
            self.notice =
                "REFUSED · Hosted Daimon owner is no longer selected; no apply was attempted."
                    .into();
            self.armed = false;
            return;
        };
        let (Some(plan), Some(provider)) = (self.plan.clone(), self.provider.clone()) else {
            self.notice =
                "REFUSED · Reviewed plan is unavailable; return to editing and plan again.".into();
            self.armed = false;
            return;
        };
        let (sender, receiver) = sync_channel(1);
        std::thread::spawn(move || {
            let result = (|| {
                let request = serde_json::json!({
                    "version": 1,
                    "operation_id": plan.operation_id,
                    "expected_document_digest": plan.expected_document_digest,
                    "expected_plan_digest": plan.plan_digest,
                    "expected_candidate_document_digest": plan.candidate_document_digest,
                    "provider": provider
                });
                let bytes = invoke_provider_owner(
                    &host.executable,
                    &host.config,
                    "--apply-provider-edit",
                    Some(
                        &serde_json::to_vec(&request)
                            .map_err(|_| "provider apply request could not be encoded")?,
                    ),
                )?;
                let receipt: ProviderEditReceiptProjection = serde_json::from_slice(&bytes)
                    .map_err(|_| "owner returned an invalid provider edit receipt")?;
                validate_provider_edit_receipt(&receipt, &plan)?;
                Ok(ProviderEditWorkerResult::Applied(receipt))
            })();
            let _ = sender.send(result);
        });
        self.phase = ProviderEditPhase::Applying;
        self.armed = false;
        self.notice = "APPLYING · Daimon is checking the reviewed plan and replacing the owner document atomically; do not infer delegation or agent invocation."
            .into();
        self.worker = Some(ProviderEditWorker {
            updates: receiver,
            started: Instant::now(),
        });
    }

    fn provider_value(&self) -> Result<serde_json::Value, String> {
        if !valid_id(self.values[0].trim()) {
            return Err("provider ID must be exactly 32 hexadecimal characters · Next: correct the selected field".into());
        }
        if !valid_id(self.values[1].trim()) {
            return Err("harness profile ID must be exactly 32 hexadecimal characters · Next: correct the selected field".into());
        }
        if !valid_digest(self.values[2].trim()) {
            return Err("policy digest must be sha256: followed by 64 hexadecimal characters · Next: correct the selected field".into());
        }
        if !Path::new(self.values[3].trim()).is_absolute() {
            return Err("program must be an absolute path · Next: select the owner executable path, not a shell command".into());
        }
        let isolation = match self.values[7].trim() {
            "direct" => serde_json::json!({"kind": "direct"}),
            "macos_seatbelt" => {
                if !Path::new(self.values[8].trim()).is_absolute()
                    || !valid_digest(self.values[9].trim())
                {
                    return Err("macos_seatbelt requires an absolute policy path and exact sha256 digest · Next: complete both seatbelt fields".into());
                }
                serde_json::json!({
                    "kind": "macos_seatbelt",
                    "policy_path": self.values[8].trim(),
                    "policy_digest": self.values[9].trim()
                })
            }
            _ => {
                return Err(
                    "isolation must be direct or macos_seatbelt · Next: correct the selected field"
                        .into(),
                );
            }
        };
        let transport = self.values[10].trim();
        if !matches!(transport, "acp" | "claude_stream_json") {
            return Err(
                "transport must be acp or claude_stream_json · Next: correct the selected field"
                    .into(),
            );
        }
        let lines = |value: &str| {
            value
                .lines()
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        Ok(serde_json::json!({
            "provider_id": self.values[0].trim(),
            "harness_profile_id": self.values[1].trim(),
            "policy_digest": self.values[2].trim(),
            "program": self.values[3].trim(),
            "arguments": lines(&self.values[4]),
            "inherited_environment_names": lines(&self.values[5]),
            "acp_agent_authentication_method": (!self.values[6].trim().is_empty()).then(|| self.values[6].trim()),
            "isolation": isolation,
            "transport": transport
        }))
    }

    fn settle(&mut self) -> Option<(ActivityLevel, Duration)> {
        let settlement =
            self.worker
                .as_ref()
                .and_then(|worker| match worker.updates.try_recv() {
                    Ok(value) => Some(value),
                    Err(TryRecvError::Disconnected) => Some(Err(
                        "provider edit worker ended without an owner result".into(),
                    )),
                    Err(TryRecvError::Empty) => None,
                })?;
        let elapsed = self
            .worker
            .take()
            .expect("settled worker exists")
            .started
            .elapsed();
        match settlement {
            Ok(ProviderEditWorkerResult::Planned { plan, provider }) => {
                self.plan = Some(*plan);
                self.provider = Some(provider);
                self.phase = ProviderEditPhase::Reviewing;
                self.notice = "REVIEWING · Daimon validated a redacted exact-change plan; c arms, Enter applies only while armed."
                    .into();
                Some((ActivityLevel::Info, elapsed))
            }
            Ok(ProviderEditWorkerResult::Applied(receipt)) => {
                self.phase = ProviderEditPhase::Complete;
                self.notice = format!(
                    "APPLIED · Daimon {} provider {} · owner document {} · restart required · Next: restart/qualify the provider, then explicitly bind it to work",
                    if receipt.replayed {
                        "verified an existing apply for"
                    } else {
                        "configured"
                    },
                    receipt.provider_id,
                    receipt.document_digest
                );
                Some((ActivityLevel::Success, elapsed))
            }
            Err(reason) => {
                self.phase = if self.plan.is_some() {
                    ProviderEditPhase::Reviewing
                } else {
                    ProviderEditPhase::Editing
                };
                self.armed = false;
                self.notice = format!(
                    "REFUSED · {reason} · Next: correct the named condition, then plan again; no delegation or invocation occurred"
                );
                Some((ActivityLevel::Warning, elapsed))
            }
        }
    }

    fn lines(&self) -> Vec<String> {
        let mut lines = vec![
            "scope               one Daimon interactive-agent owner document".into(),
            "boundary            configuration only · no qualification, task binding, delegation, persona activation, or invocation".into(),
            String::new(),
        ];
        if matches!(
            self.phase,
            ProviderEditPhase::Editing | ProviderEditPhase::Planning
        ) {
            for (index, label) in PROVIDER_EDIT_FIELDS.iter().enumerate() {
                let value = self.values[index].replace('\n', " ↵ ");
                lines.push(format!(
                    "{} {:<47} {}",
                    if index == self.selected { "▶" } else { " " },
                    label,
                    if value.is_empty() { "—" } else { &value }
                ));
            }
        } else if let Some(plan) = &self.plan {
            lines.extend([
                format!("action              {}", plan.action),
                format!("operation           {}", plan.operation_id),
                format!("current document    {}", plan.current_document_digest),
                format!("candidate document  {}", plan.candidate_document_digest),
                format!("confirmation plan   {}", plan.plan_digest),
                format!("changed fields      {}", plan.changed_fields.join(", ")),
                format!("restart required    {}", plan.restart_required),
                format!("provider            {}", plan.provider.provider_id),
                format!("harness profile     {}", plan.provider.harness_profile_id),
                format!(
                    "profile document    {}",
                    plan.provider.profile_document_digest
                ),
                format!(
                    "launch shape        {} · {} · {} argument(s) · {} environment name(s)",
                    plan.provider.transport.as_deref().unwrap_or("unknown"),
                    plan.provider.isolation.as_deref().unwrap_or("unknown"),
                    plan.provider.argument_count.unwrap_or_default(),
                    plan.provider
                        .inherited_environment_name_count
                        .unwrap_or_default()
                ),
            ]);
        }
        lines.extend([String::new(), self.notice.clone(), match self.phase {
            ProviderEditPhase::Editing => "Tab/Enter next field · Shift-Tab previous · Ctrl-U clears field · Ctrl-J newline in list fields · Enter on transport plans · Esc closes",
            ProviderEditPhase::Planning | ProviderEditPhase::Applying => "Owner operation in progress · Ctrl-Q requests safe exit after the worker settles",
            ProviderEditPhase::Reviewing => "e edits/discards plan · c arms/disarms · Enter confirms only while armed · Esc closes",
            ProviderEditPhase::Complete => "n starts another provider form · Esc closes",
        }.into()]);
        lines
    }
}

fn spawn_provider_inspection(
    profile: Option<WorkspaceProfile>,
    refresh: bool,
) -> ProviderInspection {
    let (updates, receiver) = sync_channel(1);
    std::thread::spawn(move || {
        let lines = local_provider_configuration(profile.as_ref());
        let _ = updates.send(lines);
    });
    ProviderInspection {
        updates: receiver,
        refresh,
        started: Instant::now(),
    }
}

fn spawn_work_item_binding_inspection(
    profile: Option<WorkspaceProfile>,
    task_id: String,
    draft: String,
    lens: Option<AgentCollaborationLens>,
) -> WorkItemBindingInspection {
    let (sender, result) = sync_channel(1);
    std::thread::spawn(move || {
        let inspected = inspect_work_item_binding(profile.as_ref(), &task_id);
        let _ = sender.send(inspected);
    });
    WorkItemBindingInspection {
        result,
        started: Instant::now(),
        draft,
        lens,
    }
}

fn inspect_work_item_binding(
    profile: Option<&WorkspaceProfile>,
    task_id: &str,
) -> Result<WorkItemBindingEvidence, String> {
    let daimon = profile
        .and_then(|profile| profile.daimon.as_ref())
        .ok_or_else(|| "no Daimon profile is selected".to_owned())?;
    let host = daimon.host.as_ref().ok_or_else(|| {
        "the selected Daimon profile does not publish its owner executable/config".to_owned()
    })?;
    let execution = daimon.execution.as_ref().ok_or_else(|| {
        "the selected Daimon profile does not publish a frozen execution task key".to_owned()
    })?;
    let catalog = provider_catalog(&host.executable, &host.config)?;
    validate_work_item_binding(&catalog, task_id, &execution.task_attempt.task_key)
}

fn validate_work_item_binding(
    catalog: &ProviderCatalogProjection,
    task_id: &str,
    configured_task_key: &str,
) -> Result<WorkItemBindingEvidence, String> {
    if catalog.owner != "daimon-local-agent" {
        return Err("the selected owner catalogue is not a Daimon local-agent catalogue".into());
    }
    let mut matching = catalog
        .task_bindings
        .iter()
        .filter(|binding| binding.chronos_task_id == task_id);
    let binding = matching.next().ok_or_else(|| {
        "the owner catalogue has no exact binding for the created Chronos task".to_owned()
    })?;
    if matching.next().is_some() {
        return Err(
            "the owner catalogue returned ambiguous bindings for the created Chronos task".into(),
        );
    }
    if !binding
        .external_task_key
        .eq_ignore_ascii_case(configured_task_key)
    {
        return Err(
            "the owner-bound task key does not match the selected Daimon execution profile".into(),
        );
    }
    Ok(WorkItemBindingEvidence {
        task_id: task_id.to_owned(),
        external_task_key: binding.external_task_key.clone(),
        owner: catalog.owner.clone(),
        document_digest: catalog.document_digest.clone(),
    })
}

fn local_provider_configuration(profile: Option<&WorkspaceProfile>) -> Vec<String> {
    let Some(host) = profile
        .and_then(|profile| profile.daimon.as_ref())
        .and_then(|daimon| daimon.host.as_ref())
    else {
        return vec![
            "REFUSED · No hosted Daimon owner configuration is selected.".into(),
            "Next: select a hosted Daimon executable/config in the workspace profile, preflight the exact profile, then restart once.".into(),
            "Pantheon did not inspect a fallback file or infer provider state from a socket.".into(),
            String::new(),
            "Esc closes".into(),
        ];
    };
    match provider_catalog(&host.executable, &host.config) {
        Ok(catalog) => provider_catalog_lines(catalog),
        Err(reason) => vec![
            format!("REFUSED · Daimon provider inspection failed: {reason}"),
            "No raw owner configuration, command, argument, credential name, or secret was displayed.".into(),
            "Next: run the selected Daimon executable with --check against its selected config, correct the owner refusal, then press r here.".into(),
            String::new(),
            "r retries exact owner inspection · Esc closes".into(),
        ],
    }
}

fn validate_provider_edit_plan(
    plan: &ProviderEditPlanProjection,
    operation_id: &str,
    document_digest: &str,
    expected_provider: &serde_json::Value,
) -> Result<(), String> {
    let expected_arguments = expected_provider["arguments"]
        .as_array()
        .ok_or_else(|| "provider plan request lost its argument facts".to_owned())?;
    let expected_environment = expected_provider["inherited_environment_names"]
        .as_array()
        .ok_or_else(|| "provider plan request lost its environment facts".to_owned())?;
    let expected_isolation = expected_provider["isolation"]["kind"]
        .as_str()
        .ok_or_else(|| "provider plan request lost its isolation fact".to_owned())?;
    if plan.version != 1
        || plan.owner != "daimon-local-agent"
        || plan.operation_id != operation_id
        || !matches!(plan.action.as_str(), "add" | "replace")
        || plan.expected_document_digest != document_digest
        || plan.current_document_digest != document_digest
        || !valid_digest(&plan.candidate_document_digest)
        || !valid_digest(&plan.plan_digest)
        || plan.changed_fields.is_empty()
        || plan.changed_fields.len() > 8
        || !plan.restart_required
        || !valid_provider_catalog_entry(&plan.provider)
        || plan.provider.provider_id != expected_provider["provider_id"]
        || plan.provider.harness_profile_id != expected_provider["harness_profile_id"]
        || plan.provider.policy_digest.as_deref() != expected_provider["policy_digest"].as_str()
        || plan.provider.transport.as_deref() != expected_provider["transport"].as_str()
        || plan.provider.isolation.as_deref() != Some(expected_isolation)
        || plan.provider.argument_count != Some(expected_arguments.len())
        || plan.provider.inherited_environment_name_count != Some(expected_environment.len())
        || plan.provider.authentication_method_configured
            != Some(!expected_provider["acp_agent_authentication_method"].is_null())
        || plan.provider.executable_available.is_none()
    {
        return Err("owner returned unsupported or mismatched provider plan facts".into());
    }
    if plan.changed_fields.iter().any(|field| {
        !matches!(
            field.as_str(),
            "provider_added"
                | "harness_profile_id"
                | "policy_digest"
                | "program"
                | "arguments"
                | "inherited_environment_names"
                | "authentication_method"
                | "isolation"
                | "transport"
        )
    }) {
        return Err("owner returned an unknown provider change field".into());
    }
    Ok(())
}

fn validate_provider_edit_receipt(
    receipt: &ProviderEditReceiptProjection,
    plan: &ProviderEditPlanProjection,
) -> Result<(), String> {
    if receipt.version != 1
        || receipt.owner != "daimon-local-agent"
        || receipt.operation_id != plan.operation_id
        || !matches!(receipt.disposition.as_str(), "applied" | "already_applied")
        || !receipt.applied
        || receipt.replayed != (receipt.disposition == "already_applied")
        || receipt.document_digest != plan.candidate_document_digest
        || receipt.provider_id != plan.provider.provider_id
        || !receipt.restart_required
    {
        return Err("owner returned unsupported or mismatched provider receipt facts".into());
    }
    Ok(())
}

fn provider_catalog_lines(catalog: ProviderCatalogProjection) -> Vec<String> {
    let mut lines = vec![
        "CURRENT · Daimon validated the selected owner document.".into(),
        format!("owner                {}", catalog.owner),
        format!("owner document       {}", catalog.document_digest),
        format!("providers            {} exact", catalog.providers.len()),
        format!("task bindings        {} exact", catalog.task_bindings.len()),
        format!("context references   {} exact", catalog.contexts.len()),
        "boundary             commands, arguments, paths, credential names, and secrets remain redacted by Daimon".into(),
    ];
    for (index, provider) in catalog.providers.into_iter().enumerate() {
        lines.push(String::new());
        lines.push(format!(
            "{}. provider           {}",
            index + 1,
            provider.provider_id
        ));
        lines.push(format!(
            "   harness profile     {}",
            provider.harness_profile_id
        ));
        lines.push(format!(
            "   profile document    {}",
            provider.profile_document_digest
        ));
        if let (Some(transport), Some(isolation), Some(executable)) = (
            provider.transport,
            provider.isolation,
            provider.executable_available,
        ) {
            lines.push(format!(
                "   agent transport     {transport} · isolation {isolation} · executable {}",
                if executable {
                    "available"
                } else {
                    "unavailable"
                }
            ));
            lines.push(format!(
                "   launch shape        {} argument(s) · {} inherited environment name(s) · authentication method {}",
                provider.argument_count.unwrap_or_default(),
                provider.inherited_environment_name_count.unwrap_or_default(),
                if provider.authentication_method_configured.unwrap_or(false) {
                    "configured"
                } else {
                    "not configured"
                }
            ));
            if let Some(policy) = provider.policy_digest {
                lines.push(format!("   policy               {policy}"));
            }
        }
        if let (Some(wire), Some(model), Some(scope), Some(port)) = (
            provider.wire,
            provider.model,
            provider.endpoint_scope,
            provider.endpoint_port,
        ) {
            lines.push(format!("   model                {model}"));
            lines.push(format!("   wire                 {wire} · {scope}:{port}"));
            lines.push(format!(
                "   conformance policy   structured output {} · structured tools {} · temperature milli {} · bearer token {}",
                provider.structured_output.unwrap_or(false),
                provider.structured_tools.unwrap_or(false),
                provider.sampling_temperature_milli.unwrap_or_default(),
                if provider.bearer_token_configured.unwrap_or(false) {
                    "configured"
                } else {
                    "not configured"
                }
            ));
            if let Some(qualification) = provider.qualification {
                lines.push(format!(
                    "   qualification       {} · completion {} · tool continuation {}",
                    qualification.disposition,
                    qualification.completion,
                    qualification.tool_continuation
                ));
                lines.push(format!(
                    "   qualification use   {} input · {} output tokens · {} output bytes · observed {}",
                    qualification.input_tokens,
                    qualification.output_tokens,
                    qualification.output_bytes,
                    qualification.host_observed_at_unix_ms
                ));
                if let Some(evidence) = qualification.evidence_digest {
                    lines.push(format!("   qualification proof {evidence}"));
                }
                for deviation in qualification.known_deviations {
                    lines.push(format!("   known deviation     {deviation}"));
                }
            } else {
                lines.push("   qualification       not yet run · next: M-x Qualify selected local model provider".into());
            }
        }
    }
    lines.push(String::new());
    lines.push("Chronos task bindings".into());
    for binding in catalog.task_bindings {
        lines.push(format!(
            "  {} → {}",
            binding.external_task_key, binding.chronos_task_id
        ));
    }
    if !catalog.contexts.is_empty() {
        lines.push(String::new());
        lines.push("Execution context manifest".into());
        for context in catalog.contexts {
            let owner_binding = match (
                context.workspace_id,
                context.document_ordinal,
                context.expected_revision,
            ) {
                (Some(workspace), Some(document), Some(revision)) => {
                    format!(" · workspace {workspace}/document {document} @ revision {revision}")
                }
                _ => String::new(),
            };
            lines.push(format!(
                "  {} · {} · {} · {} bytes{}",
                context.reference,
                context.kind,
                context
                    .source_digest
                    .as_deref()
                    .unwrap_or("runtime-resolved"),
                context
                    .bytes
                    .map_or_else(|| "runtime-resolved".into(), |value| value.to_string()),
                owner_binding,
            ));
        }
    }
    lines.extend([
        String::new(),
        "r revalidates the exact owner document · Esc closes".into(),
        "Task mappings and context digests are selection evidence, not a claim that work was invoked; Pantheon will not fall back to raw JSON mutation.".into(),
    ]);
    lines
}

fn acp_session_configuration(profile: Option<&WorkspaceProfile>) -> Vec<String> {
    let (sessions, execution) =
        profile
            .and_then(|profile| profile.daimon.as_ref())
            .map_or((0, false), |daimon| {
                (
                    daimon.agent_sessions.len() + usize::from(daimon.agent_session.is_some()),
                    daimon.execution.is_some(),
                )
            });
    let mut lines = vec![
        format!("current              {sessions} exact ACP/Daimon session selector(s)"),
        format!(
            "execution policy     {}",
            if execution { "selected" } else { "not selected" }
        ),
        "owner                Daimon creates, resumes, authenticates, and durably records ACP sessions".into(),
    ];
    if let Some(execution) = profile
        .and_then(|profile| profile.daimon.as_ref())
        .and_then(|daimon| daimon.execution.as_ref())
    {
        lines.extend([
            format!("task key            {}", execution.task_attempt.task_key),
            format!(
                "task attempt        incarnation {} · attempt {}",
                execution.task_attempt.task_incarnation,
                execution.task_attempt.attempt_number
            ),
            format!("harness profile      {}", execution.profile),
            format!(
                "execution bounds    {} context · {} tools · {} ms · input {} · output {} · tool calls {}",
                execution.context.len(),
                execution.tools.len(),
                execution.budget.max_elapsed_millis,
                execution.budget.max_input_tokens,
                execution.budget.max_output_tokens,
                execution.budget.max_tool_calls
            ),
            "boundary             every prompt/permission/reattach uses this frozen task attempt; operation and idempotency identities remain fresh".into(),
        ]);
    }
    lines.extend([
        String::new(),
        "1  Start or recover the session through the selected Daimon provider owner.".into(),
        "2  Put its exact 32-hex session ID in daimon.agent_sessions (1–16) and select agent_socket.".into(),
        "3  Add a credential-free execution policy only for prompt, permission, and reattach controls.".into(),
        "4  Preflight and restart the workspace profile from M-x; no live selector mutation is implied.".into(),
        "5  Open Daimon, refresh the set, and use Space to choose the bounded workset.".into(),
        String::new(),
        "Failure orientation: unavailable sessions stay explicit; provider liveness never fabricates workflow completion.".into(),
    ]);
    lines
}

fn agora_room_configuration(profile: Option<&WorkspaceProfile>) -> Vec<String> {
    let configured = profile.and_then(|profile| profile.agora.as_ref()).is_some();
    vec![
        format!(
            "current              Agora room control {}",
            if configured { "selected" } else { "not selected" }
        ),
        "owner                Agora daemon and actor-bound public client".into(),
        String::new(),
        "1  Configure or host Agora's owner daemon and public socket.".into(),
        "2  Add agora.socket, participant, and conversation/composition transport to the workspace profile.".into(),
        "3  Require rooms.read.v2 and the intended guarded room/message capabilities.".into(),
        "4  Preflight and restart the profile; refresh Agora to obtain a stable room revision.".into(),
        "5  Run M-x create an Agora room; complete the bounded form, arm, and confirm exactly once.".into(),
        String::new(),
        "The participant is an authenticated identity claim, not a credential or ambient room authority.".into(),
    ]
}

pub fn run_terminal_with_all_peers<T: ServicePort>(
    shell: Shell,
    chronos: Option<Hub>,
    vellum: Option<ServiceEditor<T>>,
    peers: OperationalPeers,
) -> Result<(), Box<dyn std::error::Error>> {
    run_terminal_with_all_peers_managed(shell, chronos, vellum, peers, None).map(|_| ())
}

pub fn run_terminal_with_all_peers_managed<T: ServicePort>(
    shell: Shell,
    chronos: Option<Hub>,
    vellum: Option<ServiceEditor<T>>,
    peers: OperationalPeers,
    native_host: Option<NativeHost>,
) -> Result<Option<TopologyShutdown>, Box<dyn std::error::Error>> {
    run_terminal_with_all_peers_managed_configuration(
        shell,
        chronos,
        vellum,
        peers,
        native_host,
        None,
    )
    .map(|outcome| outcome.shutdown)
}

pub struct ManagedTerminalOutcome {
    pub shutdown: Option<TopologyShutdown>,
    pub restart_profile: Option<RestartProfileRequest>,
}

pub struct RestartProfileRequest {
    pub path: PathBuf,
    pub sha256: String,
}

pub fn run_terminal_with_all_peers_managed_configuration<T: ServicePort>(
    mut shell: Shell,
    mut chronos: Option<Hub>,
    mut vellum: Option<ServiceEditor<T>>,
    peers: OperationalPeers,
    native_host: Option<NativeHost>,
    current_profile: Option<WorkspaceProfile>,
) -> Result<ManagedTerminalOutcome, Box<dyn std::error::Error>> {
    let operations_hosted = native_host.is_some();
    let profile_configured = current_profile.is_some();
    let OperationalPeers {
        mut agora,
        agora_control,
        mut moira,
        mut moira_participation,
        moira_participation_control,
        moira_agent_worker: moira_agent_worker_config,
        moira_control_socket,
        mut daimon,
        daimon_model,
        mut daimon_agent,
        daimon_agent_control_socket,
        daimon_execution_policy,
        mut hermes,
        hermes_operator_socket,
        hermes_disclosure_socket,
        hermes_control_socket,
        mut aegis,
        aegis_control_socket,
        mut tellus,
        vellum_control,
        vellum_allow_add_file,
    } = peers;
    let mut agora_create: Option<agora_tui::CreateRoomComponent> = None;
    let mut agora_message: Option<agora_tui::ComposeMessageComponent> = None;
    let mut chronos_operation = None;
    let mut last_agora_create_notice = String::new();
    let mut last_agora_message_notice = String::new();
    let mut agora_operation = None;
    let mut agora_message_operation = None;
    let mut moira_reconciliation: Option<moira_tui::ReconciliationBatchComponent> = None;
    let mut moira_open: Option<moira_tui::OpenControlComponent> = None;
    let mut moira_agent_worker = MoiraAgentWorkerControl::new();
    let mut last_moira_open_notice = String::new();
    let mut last_moira_reconciliation_notice = String::new();
    let mut active_moira_action: Option<&'static str> = None;
    let mut moira_operation = None;
    let mut daimon_control: Option<daimon_tui::AgentControlBatchComponent> = None;
    let mut daimon_prompt_composer: Option<daimon_tui::AgentPromptComposer> = None;
    let mut daimon_permission_composer: Option<daimon_tui::AgentPermissionComposer> = None;
    let mut last_daimon_control_notice = String::new();
    let mut active_daimon_action: Option<&'static str> = None;
    let mut daimon_operation = None;
    let mut daimon_post_receipt_refresh = false;
    let mut daimon_recovery_refresh = false;
    let mut daimon_unknown_evidence = false;
    let mut work_item_agent = WorkItemAgentJourney::default();
    let mut hermes_acknowledge: Option<hermes_tui::AcknowledgeComponent> = None;
    let mut hermes_compaction: Option<hermes_tui::CompactionComponent> = None;
    let mut last_hermes_compaction_notice = String::new();
    let mut hermes_compaction_operation = None;
    let mut hermes_compaction_post_receipt_refresh = false;
    let mut hermes_disclosure: Option<hermes_tui::DisclosureComponent> = None;
    let mut last_hermes_disclosure_notice = String::new();
    let mut hermes_disclosure_operation = None;
    let mut last_hermes_acknowledge_notice = String::new();
    let mut hermes_operation = None;
    let mut hermes_post_receipt_refresh = false;
    let mut aegis_revoke: Option<aegis_tui::RevokeEpochComponent> = None;
    let mut aegis_managed: Option<aegis_tui::ManagedAuthorityComponent> = None;
    let mut last_aegis_revoke_notice = String::new();
    let mut last_aegis_managed_notice = String::new();
    let mut aegis_operation = None;
    let mut focus = Focus::Chronos;
    let mut notice = String::from("workspace profile loaded");
    let mut confirming_discard = false;
    let mut last_vellum_poll = Instant::now();
    let mut vellum_durable_save: Option<DurableSaveBatchComponent> = None;
    let mut last_vellum_durable_notice = String::new();
    let mut vellum_durable_operation = None;
    let mut vellum_atomic_replace: Option<AtomicReplaceComponent> = None;
    let mut vellum_atomic_operation = None;
    let mut vellum_open_prompt: Option<TextPrompt> = None;
    let mut vellum_find_prompt: Option<TextPrompt> = None;
    let commands = workbench_commands(
        &shell,
        chronos.is_some(),
        chronos.as_ref().is_some_and(Hub::writes_enabled),
        chronos.as_ref().is_some_and(Hub::scheduling_enabled),
        vellum.is_some(),
        vellum_control.is_some(),
        agora.is_some(),
        agora_control.is_some(),
        moira.is_some(),
        moira_participation_control.is_some(),
        moira_agent_worker_config.is_some(),
        moira_control_socket.is_some(),
        daimon.is_some() || daimon_model.is_some() || daimon_agent.is_some(),
        daimon_model.is_some(),
        daimon_agent.is_some() && daimon_agent_control_socket.is_some(),
        daimon_agent.is_some()
            && daimon_agent_control_socket.is_some()
            && daimon_execution_policy.is_some(),
        hermes.is_some(),
        hermes_operator_socket.is_some(),
        hermes_disclosure_socket.is_some(),
        hermes_control_socket.is_some(),
        aegis.is_some(),
        aegis_control_socket.is_some(),
        tellus.is_some(),
        operations_hosted,
        profile_configured,
    );
    let mut palette = CommandPalette::new(commands);
    let mut overlay = WorkbenchOverlay::None;
    let mut activity = ActivityLog::default();
    let mut instrumentation = Instrumentation::default();
    let mut pending_ui_feedback: Option<(u64, ActivityLevel)> = None;
    let mut refresh_operations = RefreshOperations::default();
    let mut operations = OperationsRuntime::new(native_host);
    let mut restart_profile = None;
    let mut drain_operations: Vec<(HostedPeerId, u64)> = Vec::new();
    activity.push(
        ActivityLevel::Success,
        "pantheon",
        "Workspace profile loaded; peer refreshes queued",
    );
    let configuration = session_configuration(
        &shell,
        chronos.as_ref(),
        vellum.is_some(),
        vellum_allow_add_file,
        agora.is_some(),
        agora_control.is_some(),
        moira.is_some(),
        moira_control_socket.is_some(),
        daimon.is_some(),
        daimon_agent.is_some(),
        daimon_agent_control_socket.is_some(),
        daimon_execution_policy.is_some(),
        hermes.is_some(),
        hermes_operator_socket.is_some(),
        hermes_disclosure_socket.is_some(),
        hermes_control_socket.is_some(),
        aegis.is_some(),
        aegis_control_socket.is_some(),
        tellus.is_some(),
    );
    let mut provider_configuration_lines = vec![
        "Provider configuration has not been inspected in this session.".into(),
        "Run M-x Inspect local provider configuration to request a fresh redacted owner projection.".into(),
    ];
    let mut provider_inspection: Option<ProviderInspection> = None;
    let mut work_item_binding_inspection: Option<WorkItemBindingInspection> = None;
    let mut provider_edit: Option<ProviderEditWorkflow> = None;
    let mut provider_qualification: Option<ProviderQualificationWorkflow> = None;
    let acp_configuration_lines = acp_session_configuration(current_profile.as_ref());
    let agora_configuration_lines = agora_room_configuration(current_profile.as_ref());
    let mut reconfiguration = current_profile.clone().map(ReconfigurationWorkflow::new);
    let health_channels = [
        ("chronos", "tasks", chronos.is_some()),
        ("vellum", "workspace", vellum.is_some()),
        ("agora", "rooms/messages", agora.is_some()),
        ("moira", "sagas", moira.is_some()),
        ("daimon", "presence", daimon.is_some()),
        ("daimon", "sessions/events", daimon_agent.is_some()),
        ("hermes", "ledger/cursors", hermes.is_some()),
        ("aegis", "authority", aegis.is_some()),
        ("tellus", "provenance", tellus.is_some()),
    ]
    .into_iter()
    .filter_map(|(peer, channel, configured)| configured.then_some((peer, channel)));
    let mut health = SuiteHealthLedger::new(health_channels);
    if chronos.is_some() {
        health.observing("chronos", "tasks", true);
    }
    if vellum.is_some() {
        health.observing("vellum", "workspace", true);
    }

    let result = ratatui::run(|terminal| -> Result<(), Box<dyn std::error::Error>> {
        let _input_capture = InputCapture::enable()?;
        loop {
            if let Some((level, elapsed)) = provider_qualification
                .as_mut()
                .and_then(ProviderQualificationWorkflow::settle)
            {
                let workflow = provider_qualification
                    .as_ref()
                    .expect("settled provider qualification exists");
                notice = workflow.notice.clone();
                instrumentation.record("models.provider.qualify.result", "daimon", level, elapsed);
                activity.push(level, "daimon", notice.clone());
            }
            if let Some((level, elapsed)) = provider_edit
                .as_mut()
                .and_then(ProviderEditWorkflow::settle)
            {
                let edit = provider_edit
                    .as_ref()
                    .expect("settled provider edit exists");
                notice = edit.notice.clone();
                instrumentation.record(
                    "configuration.providers.edit.result",
                    "daimon",
                    level,
                    elapsed,
                );
                activity.push(level, "configuration", notice.clone());
            }
            let provider_settlement = provider_inspection.as_ref().and_then(|inspection| {
                match inspection.updates.try_recv() {
                    Ok(lines) => Some(Ok(lines)),
                    Err(TryRecvError::Disconnected) => Some(Err(())),
                    Err(TryRecvError::Empty) => None,
                }
            });
            if let Some(settlement) = provider_settlement {
                let inspection = provider_inspection
                    .take()
                    .expect("provider settlement requires an active inspection");
                match settlement {
                    Ok(lines) => {
                        let current = lines
                            .first()
                            .is_some_and(|line| line.starts_with("CURRENT ·"));
                        provider_configuration_lines = lines;
                        let level = if current {
                            ActivityLevel::Success
                        } else {
                            ActivityLevel::Warning
                        };
                        notice = if current && inspection.refresh {
                            "REFRESHED · Local provider configuration · Exact owner projection revalidated · Next: review changed digest/facts or Esc to close"
                                .into()
                        } else if current {
                            "CURRENT · Local provider configuration · Owner projection inspected · Next: review exact redacted facts, press r to revalidate, or Esc to close"
                                .into()
                        } else {
                            "REFUSED · Local provider configuration is not current · Next: follow the displayed owner-oriented recovery action"
                                .into()
                        };
                        instrumentation.record(
                            "configuration.providers.result",
                            "daimon",
                            level,
                            inspection.started.elapsed(),
                        );
                        activity.push(level, "configuration", notice.clone());
                    }
                    Err(()) => {
                        provider_configuration_lines = vec![
                            "REFUSED · Provider inspection worker ended without a projection."
                                .into(),
                            "Next: inspect Activity, verify the selected owner executable, then retry."
                                .into(),
                        ];
                        notice = "REFUSED · Provider inspection ended without a projection · Next: inspect Activity and retry only after correcting the owner executable"
                            .into();
                        instrumentation.record(
                            "configuration.providers.result",
                            "daimon",
                            ActivityLevel::Warning,
                            inspection.started.elapsed(),
                        );
                        activity.push(ActivityLevel::Warning, "configuration", notice.clone());
                    }
                }
            }
            let work_item_binding_settlement =
                work_item_binding_inspection
                    .as_ref()
                    .and_then(|inspection| match inspection.result.try_recv() {
                        Ok(result) => Some(result),
                        Err(TryRecvError::Disconnected) => Some(Err(
                            "the Daimon owner-inspection worker ended without evidence".into(),
                        )),
                        Err(TryRecvError::Empty) => None,
                    });
            if let Some(settlement) = work_item_binding_settlement {
                let inspection = work_item_binding_inspection
                    .take()
                    .expect("binding settlement requires an active inspection");
                match settlement {
                    Ok(evidence) => {
                        instrumentation.record(
                            "workitem.agent.binding.result",
                            "daimon",
                            ActivityLevel::Success,
                            inspection.started.elapsed(),
                        );
                        activity.push(
                            ActivityLevel::Success,
                            "work-item",
                            format!(
                                "Daimon owner {} mapped Chronos task {} to configured task key {} under catalogue {}",
                                evidence.owner,
                                evidence.task_id,
                                evidence.external_task_key,
                                evidence.document_digest
                            ),
                        );
                        let prompt = work_item_agent_prompt(&inspection.draft, inspection.lens);
                        let observations = daimon_agent
                            .as_ref()
                            .map(daimon_tui::AgentSetComponent::selected_observations)
                            .unwrap_or_default();
                        let readiness = if observations.is_empty() {
                            Some(
                                "No current selected Daimon session; refresh the configured local agent first"
                                    .to_owned(),
                            )
                        } else {
                            daimon_workset_error(
                                &observations,
                                "Ready with no active turn",
                                |view| {
                                    view.phase == AgentSessionPhaseV1::Ready
                                        && view.active_turn.is_none()
                                },
                            )
                        };
                        if let Some(reason) = readiness {
                            work_item_agent = WorkItemAgentJourney::Failed;
                            notice = command_feedback(
                                "workitem.agent.new",
                                ActivityLevel::Warning,
                                &format!(
                                    "Chronos created the task and its Daimon mapping was proven, but the local agent prompt could not open: {reason}"
                                ),
                            );
                            activity.push(ActivityLevel::Warning, "work-item", notice.clone());
                        } else {
                            let mut composer = daimon_tui::AgentPromptComposer::new(
                                observations.len(),
                            )
                            .expect("a bounded nonempty selected session set was validated");
                            composer.insert_text(&prompt);
                            daimon_prompt_composer = Some(composer);
                            daimon_permission_composer = None;
                            daimon_control = None;
                            let _ = shell.activate(Surface::Daimon);
                            work_item_agent = WorkItemAgentJourney::ReviewingPrompt;
                            notice = "READY · Chronos task created, exact Daimon owner binding proven, and local-agent prompt prepared · Next: review the carried work text, Tab to review, c to arm, then Enter once"
                                .into();
                            activity.push(ActivityLevel::Success, "work-item", notice.clone());
                        }
                    }
                    Err(reason) => {
                        instrumentation.record(
                            "workitem.agent.binding.result",
                            "daimon",
                            ActivityLevel::Warning,
                            inspection.started.elapsed(),
                        );
                        work_item_agent = WorkItemAgentJourney::Failed;
                        notice = command_feedback(
                            "workitem.agent.new",
                            ActivityLevel::Warning,
                            &format!(
                                "Chronos created the task, but Daimon owner mapping was not proven: {reason}; the local agent was not invoked"
                            ),
                        );
                        activity.push(ActivityLevel::Warning, "work-item", notice.clone());
                    }
                }
            }
            if let Some((level, shutdown)) = operations.poll() {
                settle_drain_operations(
                    &mut drain_operations,
                    shutdown.as_ref(),
                    level,
                    &mut instrumentation,
                );
                notice = operations.notice.clone();
                activity.push(level, "operations", notice.clone());
            }
            if let Some(chronos) = &mut chronos
                && chronos.poll()
            {
                health.current("chronos", "tasks");
                if let Some(operation) = chronos_operation.take() {
                    let (operation, level, message) = match operation {
                        ChronosOperation::Task(operation) => {
                            let (level, message) = match chronos.task_mutation_disposition() {
                                chronos_tui::TaskMutationDisposition::Applied => (
                                    ActivityLevel::Success,
                                    "Chronos task mutation settled with owner receipts and a current projection",
                                ),
                                chronos_tui::TaskMutationDisposition::Refused => (
                                    ActivityLevel::Warning,
                                    "Chronos task mutation was refused; no optimistic child success recorded",
                                ),
                                chronos_tui::TaskMutationDisposition::OutcomeUnknown => (
                                    ActivityLevel::Uncertain,
                                    "Chronos task mutation outcome unknown; explicit inspection required",
                                ),
                            };
                            (operation, level, message)
                        }
                        ChronosOperation::Scheduling(operation) => {
                            let (level, message) = match chronos.scheduling_mutation_disposition() {
                                chronos_tui::SchedulingMutationDisposition::Applied => (
                                    ActivityLevel::Success,
                                    "Chronos scheduling mutation settled with independent child dispositions",
                                ),
                                chronos_tui::SchedulingMutationDisposition::Refused => (
                                    ActivityLevel::Warning,
                                    "Chronos scheduling mutation was refused or stopped before dispatch",
                                ),
                                chronos_tui::SchedulingMutationDisposition::OutcomeUnknown => (
                                    ActivityLevel::Uncertain,
                                    "Chronos scheduling mutation outcome unknown; later children were not sent",
                                ),
                            };
                            (operation, level, message)
                        }
                    };
                    instrumentation.mark_peer_stage(operation, "peer_receipt", level);
                    instrumentation.settle_peer_operation(operation, level);
                    activity.push(level, "chronos", message);
                    if let WorkItemAgentJourney::AwaitingTaskReceipt {
                        draft,
                        lens,
                        task_id,
                    } = &work_item_agent
                    {
                        if level != ActivityLevel::Success {
                            work_item_agent = WorkItemAgentJourney::Failed;
                            notice = command_feedback(
                                "workitem.agent.new",
                                level,
                                "Chronos did not return an applied task receipt; the local agent was not invoked",
                            );
                            activity.push(level, "work-item", notice.clone());
                        } else {
                            work_item_binding_inspection =
                                Some(spawn_work_item_binding_inspection(
                                    current_profile.clone(),
                                    task_id.clone(),
                                    draft.clone(),
                                    *lens,
                                ));
                            work_item_agent = WorkItemAgentJourney::AwaitingTaskBinding;
                            notice = "WORKING · Chronos task created · Verifying the exact owner-published Daimon task binding before preparing any prompt"
                                .into();
                            activity.push(ActivityLevel::Info, "work-item", notice.clone());
                        }
                    }
                } else if let Some(operation) = refresh_operations.chronos.take() {
                    instrumentation.settle_peer_operation(operation, ActivityLevel::Success);
                    activity.push(
                        ActivityLevel::Success,
                        "chronos",
                        "Refresh settled to a current projection",
                    );
                }
            }
            if let Some(agora) = &mut agora {
                let settlement = agora.poll_settlement();
                health.settle("agora", "rooms/messages", settlement);
                settle_refresh(
                    &mut refresh_operations.agora,
                    settlement,
                    "agora",
                    &mut instrumentation,
                    &mut activity,
                );
            }
            if let Some(control) = &mut agora_create {
                control.poll();
                let current = control.notice();
                if current != last_agora_create_notice {
                    let level = match control.disposition() {
                        agora_tui::CreateRoomDisposition::Applied => ActivityLevel::Success,
                        agora_tui::CreateRoomDisposition::OutcomeUnknown => {
                            ActivityLevel::Uncertain
                        }
                        agora_tui::CreateRoomDisposition::Refused
                        | agora_tui::CreateRoomDisposition::Failed => ActivityLevel::Warning,
                        agora_tui::CreateRoomDisposition::Editing
                        | agora_tui::CreateRoomDisposition::Armed
                        | agora_tui::CreateRoomDisposition::Pending => ActivityLevel::Info,
                    };
                    notice = current.into();
                    activity.push(level, "agora", current);
                    if level != ActivityLevel::Info
                        && let Some(operation) = agora_operation.take()
                    {
                        instrumentation.settle_peer_operation(operation, level);
                    }
                    if control.disposition() == agora_tui::CreateRoomDisposition::Applied
                        && let Some(component) = &mut agora
                    {
                        component.update(agora_tui::Input::Refresh, true);
                    }
                    last_agora_create_notice = current.into();
                }
            }
            if let Some(control) = &mut agora_message {
                control.poll();
                let current = control.notice();
                if current != last_agora_message_notice {
                    let level = match control.disposition() {
                        agora_tui::ComposeMessageDisposition::Applied => ActivityLevel::Success,
                        agora_tui::ComposeMessageDisposition::OutcomeUnknown => {
                            ActivityLevel::Uncertain
                        }
                        agora_tui::ComposeMessageDisposition::Refused
                        | agora_tui::ComposeMessageDisposition::Failed => ActivityLevel::Warning,
                        agora_tui::ComposeMessageDisposition::Editing
                        | agora_tui::ComposeMessageDisposition::Armed
                        | agora_tui::ComposeMessageDisposition::Pending => ActivityLevel::Info,
                    };
                    notice = current.into();
                    activity.push(level, "agora", current);
                    if level != ActivityLevel::Info
                        && let Some(operation) = agora_message_operation.take()
                    {
                        instrumentation.mark_peer_stage(operation, "peer_receipt", level);
                        instrumentation.settle_peer_operation(operation, level);
                    }
                    if control.disposition() == agora_tui::ComposeMessageDisposition::Applied
                        && let Some(component) = &mut agora
                    {
                        component.update(agora_tui::Input::Refresh, true);
                    }
                    last_agora_message_notice = current.into();
                }
            }
            if let Some(moira) = &mut moira {
                let settlement = moira.poll_settlement();
                health.settle("moira", "sagas", settlement);
                settle_refresh(
                    &mut refresh_operations.moira,
                    settlement,
                    "moira",
                    &mut instrumentation,
                    &mut activity,
                );
            }
            if let Some(participation) = &mut moira_participation {
                participation.poll();
            }
            if let Some(control) = &mut moira_open {
                control.poll();
                let current = control.notice();
                if current != last_moira_open_notice {
                    let level = if control.recovery_required() {
                        ActivityLevel::Uncertain
                    } else if control.request_pending() {
                        ActivityLevel::Info
                    } else {
                        ActivityLevel::Success
                    };
                    notice = current.into();
                    activity.push(level, "moira", current);
                    last_moira_open_notice = current.into();
                }
            }
            if let Some(outcome) = moira_agent_worker.poll() {
                let level = match outcome {
                    MoiraAgentWorkerSettlement::Observed => ActivityLevel::Success,
                    MoiraAgentWorkerSettlement::Idle | MoiraAgentWorkerSettlement::Refused => {
                        ActivityLevel::Warning
                    }
                    MoiraAgentWorkerSettlement::FencedUnknown => ActivityLevel::Uncertain,
                };
                notice = moira_agent_worker.notice.clone();
                activity.push(level, "moira", notice.clone());
                if outcome != MoiraAgentWorkerSettlement::FencedUnknown
                    && let Some((socket, participation)) = &moira_participation_control
                {
                    match moira_client::unix::AgentParticipationUnixPort::connect(
                        socket,
                        moira_client::unix::ParticipationClientLimitsV1::default(),
                    ) {
                        Ok(port) => {
                            moira_participation = Some(
                                moira_tui::ParticipationComponent::new(port, *participation),
                            );
                            activity.push(
                                ActivityLevel::Info,
                                "moira",
                                "Reconnected a fresh participation observer after owner-worker settlement",
                            );
                        }
                        Err(error) => activity.push(
                            ActivityLevel::Warning,
                            "moira",
                            format!(
                                "Owner worker settled, but participation observer reconnect was refused: {error:?}"
                            ),
                        ),
                    }
                }
            }
            if let Some(reconciliation) = &mut moira_reconciliation {
                reconciliation.poll();
                let current = reconciliation.notice();
                if current != last_moira_reconciliation_notice {
                    let level = reconciliation_batch_level(reconciliation.disposition());
                    notice = current.into();
                    activity.push(level, "moira", current);
                    if !reconciliation.request_pending()
                        && let Some(operation) = moira_operation.take()
                    {
                        instrumentation.mark_peer_stage(
                            operation,
                            "child_receipts",
                            if level == ActivityLevel::Info {
                                ActivityLevel::Success
                            } else {
                                level
                            },
                        );
                        instrumentation.settle_peer_operation(
                            operation,
                            if level == ActivityLevel::Info {
                                ActivityLevel::Success
                            } else {
                                level
                            },
                        );
                    }
                    last_moira_reconciliation_notice = current.into();
                }
            }
            let daimon_settlement = daimon
                .as_mut()
                .and_then(daimon_tui::Component::poll_settlement);
            let daimon_agent_settlement = daimon_agent
                .as_mut()
                .and_then(daimon_tui::AgentSetComponent::poll_settlement);
            health.settle("daimon", "presence", daimon_settlement);
            health.settle("daimon", "sessions/events", daimon_agent_settlement);
            if daimon_recovery_refresh && let Some(settlement) = daimon_agent_settlement {
                let available = settlement.is_ok();
                if let Some(control) = &mut daimon_control {
                    control.complete_recovery_inspection(available);
                }
                let level = if available {
                    ActivityLevel::Uncertain
                } else {
                    ActivityLevel::Warning
                };
                if let Some(operation) = daimon_operation.take() {
                    instrumentation.mark_peer_stage(operation, "recovery_inspection", level);
                    instrumentation.settle_peer_operation(operation, level);
                }
                daimon_recovery_refresh = false;
                daimon_unknown_evidence = true;
                notice = if available {
                    "Daimon recovery inspection completed; no mutation was retried and the original outcome remains unknown"
                } else {
                    "Daimon recovery inspection was unavailable; retained evidence may now be closed without retrying"
                }
                .into();
                activity.push(level, "daimon", notice.clone());
            }
            if daimon_post_receipt_refresh && let Some(settlement) = daimon_agent_settlement {
                let level = if settlement.is_ok() {
                    ActivityLevel::Success
                } else {
                    ActivityLevel::Warning
                };
                if let Some(operation) = daimon_operation.take() {
                    instrumentation.mark_peer_stage(operation, "refreshed_projection", level);
                    instrumentation.settle_peer_operation(operation, level);
                }
                daimon_post_receipt_refresh = false;
                notice = if settlement.is_ok()
                    && matches!(
                        work_item_agent,
                        WorkItemAgentJourney::ConversationReady
                    )
                {
                    "DONE · Local-agent turn accepted and reconciled with the selected session conversation · Next: continue here, or run M-x Return to the Kanban board"
                } else if settlement.is_ok() {
                    "Daimon child receipts reconciled with refreshed session projections"
                } else {
                    "Daimon child receipts retained; refreshed session projection unavailable"
                }
                .into();
                activity.push(level, "daimon", notice.clone());
            }
            settle_refresh(
                &mut refresh_operations.daimon,
                if daimon_agent.is_some() {
                    daimon_agent_settlement
                } else {
                    daimon_settlement
                },
                "daimon",
                &mut instrumentation,
                &mut activity,
            );
            if let Some(control) = &mut daimon_control {
                control.poll();
                let current = control.notice();
                if current != last_daimon_control_notice {
                    let level = match control.disposition() {
                        daimon_tui::AgentControlDisposition::Applied => ActivityLevel::Success,
                        daimon_tui::AgentControlDisposition::OutcomeUnknown => {
                            ActivityLevel::Uncertain
                        }
                        daimon_tui::AgentControlDisposition::Refused
                        | daimon_tui::AgentControlDisposition::Failed => ActivityLevel::Warning,
                        daimon_tui::AgentControlDisposition::Proposed
                        | daimon_tui::AgentControlDisposition::Pending => ActivityLevel::Info,
                    };
                    notice = current.into();
                    activity.push(level, "daimon", current);
                    if active_daimon_action == Some("workitem.agent.prompt")
                        && level != ActivityLevel::Info
                    {
                        if control.disposition() == daimon_tui::AgentControlDisposition::Applied {
                            work_item_agent = WorkItemAgentJourney::ConversationReady;
                            notice = "DONE · Local-agent turn accepted; the selected Daimon timeline is the live conversation · Next: continue here, or run M-x Return to the Kanban board"
                                .into();
                            activity.push(ActivityLevel::Success, "work-item", notice.clone());
                        } else {
                            work_item_agent = WorkItemAgentJourney::Failed;
                            notice = command_feedback(
                                "workitem.agent.prompt",
                                level,
                                "The local-agent invocation did not produce an applied receipt; inspect the retained Daimon evidence before retrying",
                            );
                            activity.push(level, "work-item", notice.clone());
                        }
                    }
                    if level != ActivityLevel::Info {
                        if control.has_unknown() && !daimon_unknown_evidence {
                            if let Some(operation) = daimon_operation {
                                instrumentation.mark_peer_stage(
                                    operation,
                                    "child_receipts",
                                    ActivityLevel::Uncertain,
                                );
                            }
                            daimon_unknown_evidence = true;
                        }
                        if control.applied_count() != 0
                            && !control.has_unknown()
                            && !daimon_post_receipt_refresh
                            && let Some(agent) = &mut daimon_agent
                        {
                            if let Some(operation) = daimon_operation {
                                instrumentation.mark_peer_stage(operation, "child_receipts", level);
                            }
                            agent.update(daimon_tui::AgentSetInput::RefreshAll, true);
                            daimon_post_receipt_refresh = agent.request_pending();
                            if !daimon_post_receipt_refresh
                                && let Some(operation) = daimon_operation.take()
                            {
                                instrumentation.mark_peer_stage(
                                    operation,
                                    "refreshed_projection",
                                    ActivityLevel::Warning,
                                );
                                instrumentation
                                    .settle_peer_operation(operation, ActivityLevel::Warning);
                                activity.push(
                                    ActivityLevel::Warning,
                                    "daimon",
                                    "Daimon child receipts retained; projection refresh could not be queued",
                                );
                            }
                        }
                        if !control.has_unknown()
                            && !daimon_post_receipt_refresh
                            && let Some(operation) = daimon_operation.take()
                        {
                            instrumentation.mark_peer_stage(operation, "child_receipts", level);
                            instrumentation.settle_peer_operation(operation, level);
                        }
                    }
                    last_daimon_control_notice = current.into();
                }
            }
            if let Some(hermes) = &mut hermes {
                let settlement = hermes.poll_settlement();
                health.settle("hermes", "ledger/cursors", settlement);
                settle_refresh(
                    &mut refresh_operations.hermes,
                    settlement,
                    "hermes",
                    &mut instrumentation,
                    &mut activity,
                );
            }
            if let Some(control) = &mut hermes_compaction {
                control.poll();
                let current = control.notice();
                if current != last_hermes_compaction_notice {
                    let level = match control.disposition() {
                        hermes_tui::CompactionDisposition::Applied { .. } => ActivityLevel::Success,
                        hermes_tui::CompactionDisposition::OutcomeUnknown => {
                            ActivityLevel::Uncertain
                        }
                        hermes_tui::CompactionDisposition::Refused
                        | hermes_tui::CompactionDisposition::Failed => ActivityLevel::Warning,
                        hermes_tui::CompactionDisposition::Proposed
                        | hermes_tui::CompactionDisposition::Armed
                        | hermes_tui::CompactionDisposition::Pending => ActivityLevel::Info,
                    };
                    notice = current.into();
                    activity.push(level, "hermes", current);
                    let applied = matches!(
                        control.disposition(),
                        hermes_tui::CompactionDisposition::Applied { .. }
                    );
                    if level != ActivityLevel::Info
                        && !applied
                        && let Some(operation) = hermes_compaction_operation.take()
                    {
                        instrumentation.settle_peer_operation(operation, level);
                    }
                    if applied && hermes.is_some() {
                        if let Some(operation) = hermes_compaction_operation {
                            instrumentation.mark_peer_stage(
                                operation,
                                "peer_receipt",
                                ActivityLevel::Success,
                            );
                        }
                        hermes_compaction_post_receipt_refresh = true;
                    }
                    last_hermes_compaction_notice = current.into();
                }
            }
            if let Some(control) = &mut hermes_disclosure {
                control.poll();
                let current = control.notice();
                if current != last_hermes_disclosure_notice {
                    let level = match control.disposition() {
                        hermes_tui::DisclosureDisposition::Disclosed => ActivityLevel::Success,
                        hermes_tui::DisclosureDisposition::Refused
                        | hermes_tui::DisclosureDisposition::Failed => ActivityLevel::Warning,
                        hermes_tui::DisclosureDisposition::Proposed
                        | hermes_tui::DisclosureDisposition::Armed
                        | hermes_tui::DisclosureDisposition::Pending => ActivityLevel::Info,
                    };
                    // Only stable disposition text enters telemetry/activity; disclosed
                    // event metadata and payload preview remain inside the owner sheet.
                    notice = current.into();
                    activity.push(level, "hermes", current);
                    if level != ActivityLevel::Info
                        && let Some(operation) = hermes_disclosure_operation.take()
                    {
                        instrumentation.mark_peer_stage(operation, "peer_read_receipt", level);
                        instrumentation.settle_peer_operation(operation, level);
                    }
                    last_hermes_disclosure_notice = current.into();
                }
            }
            if let Some(control) = &mut hermes_acknowledge {
                control.poll();
                let current = control.notice();
                if current != last_hermes_acknowledge_notice {
                    let level = match control.disposition() {
                        hermes_tui::AcknowledgeDisposition::Applied { .. } => {
                            ActivityLevel::Success
                        }
                        hermes_tui::AcknowledgeDisposition::OutcomeUnknown => {
                            ActivityLevel::Uncertain
                        }
                        hermes_tui::AcknowledgeDisposition::Refused
                        | hermes_tui::AcknowledgeDisposition::Failed => ActivityLevel::Warning,
                        hermes_tui::AcknowledgeDisposition::Proposed
                        | hermes_tui::AcknowledgeDisposition::Armed
                        | hermes_tui::AcknowledgeDisposition::Pending => ActivityLevel::Info,
                    };
                    notice = current.into();
                    activity.push(level, "hermes", current);
                    let applied = matches!(
                        control.disposition(),
                        hermes_tui::AcknowledgeDisposition::Applied { .. }
                    );
                    if level != ActivityLevel::Info
                        && !applied
                        && let Some(operation) = hermes_operation.take()
                    {
                        instrumentation.settle_peer_operation(operation, level);
                    }
                    if applied && hermes.is_some() {
                        if let Some(operation) = hermes_operation {
                            instrumentation.mark_peer_stage(
                                operation,
                                "peer_receipt",
                                ActivityLevel::Success,
                            );
                        }
                        hermes_post_receipt_refresh = true;
                    }
                    last_hermes_acknowledge_notice = current.into();
                }
            }
            if hermes_post_receipt_refresh
                && let Some(component) = &mut hermes
                && !component.request_pending()
            {
                component.update(hermes_tui::Input::Refresh, true);
                if component.request_pending() {
                    refresh_operations.hermes = hermes_operation.take();
                    hermes_post_receipt_refresh = false;
                } else {
                    if let Some(operation) = hermes_operation.take() {
                        instrumentation.settle_peer_operation(operation, ActivityLevel::Warning);
                    }
                    hermes_post_receipt_refresh = false;
                    notice = "Hermes acknowledgement applied, but post-receipt cursor refresh could not be queued"
                        .into();
                    activity.push(ActivityLevel::Warning, "hermes", notice.clone());
                }
            }
            if hermes_compaction_post_receipt_refresh
                && let Some(component) = &mut hermes
                && !component.request_pending()
            {
                component.update(hermes_tui::Input::Refresh, true);
                if component.request_pending() {
                    refresh_operations.hermes = hermes_compaction_operation.take();
                    hermes_compaction_post_receipt_refresh = false;
                } else {
                    if let Some(operation) = hermes_compaction_operation.take() {
                        instrumentation.settle_peer_operation(operation, ActivityLevel::Warning);
                    }
                    hermes_compaction_post_receipt_refresh = false;
                    notice = "Hermes compaction applied, but post-receipt stream refresh could not be queued"
                        .into();
                    activity.push(ActivityLevel::Warning, "hermes", notice.clone());
                }
            }
            if let Some(aegis) = &mut aegis {
                let settlement = aegis.poll_settlement();
                health.settle("aegis", "authority", settlement);
                settle_refresh(
                    &mut refresh_operations.aegis,
                    settlement,
                    "aegis",
                    &mut instrumentation,
                    &mut activity,
                );
            }
            if let Some(control) = &mut aegis_revoke {
                control.poll();
                let current = control.notice();
                if current != last_aegis_revoke_notice {
                    let level = match control.disposition() {
                        aegis_tui::RevokeEpochDisposition::Applied { .. } => ActivityLevel::Success,
                        aegis_tui::RevokeEpochDisposition::OutcomeUnknown => {
                            ActivityLevel::Uncertain
                        }
                        aegis_tui::RevokeEpochDisposition::Refused
                        | aegis_tui::RevokeEpochDisposition::Failed => ActivityLevel::Warning,
                        aegis_tui::RevokeEpochDisposition::Proposed
                        | aegis_tui::RevokeEpochDisposition::Armed
                        | aegis_tui::RevokeEpochDisposition::Pending => ActivityLevel::Info,
                    };
                    notice = current.into();
                    activity.push(level, "aegis", current);
                    let applied = matches!(
                        control.disposition(),
                        aegis_tui::RevokeEpochDisposition::Applied { .. }
                    );
                    if level != ActivityLevel::Info
                        && !applied
                        && let Some(operation) = aegis_operation.take()
                    {
                        instrumentation.settle_peer_operation(operation, level);
                    }
                    if applied && let Some(component) = &mut aegis {
                        if let Some(operation) = aegis_operation {
                            instrumentation.mark_peer_stage(
                                operation,
                                "peer_receipt",
                                ActivityLevel::Success,
                            );
                        }
                        refresh_operations.aegis = aegis_operation.take();
                        component.update(aegis_tui::Input::Refresh, true);
                    }
                    last_aegis_revoke_notice = current.into();
                }
            }
            if let Some(control) = &mut aegis_managed {
                control.poll();
                let current = control.notice();
                if current != last_aegis_managed_notice {
                    let level = match control.disposition() {
                        aegis_tui::ManagedAuthorityDisposition::Applied => ActivityLevel::Success,
                        aegis_tui::ManagedAuthorityDisposition::Fenced => ActivityLevel::Uncertain,
                        aegis_tui::ManagedAuthorityDisposition::Refused
                        | aegis_tui::ManagedAuthorityDisposition::Failed => ActivityLevel::Warning,
                        aegis_tui::ManagedAuthorityDisposition::Connecting
                        | aegis_tui::ManagedAuthorityDisposition::Editing
                        | aegis_tui::ManagedAuthorityDisposition::Proposed
                        | aegis_tui::ManagedAuthorityDisposition::Armed
                        | aegis_tui::ManagedAuthorityDisposition::Pending => ActivityLevel::Info,
                    };
                    notice = current.into();
                    activity.push(level, "aegis", current);
                    if level != ActivityLevel::Info
                        && let Some(operation) = aegis_operation.take()
                    {
                        instrumentation.mark_peer_stage(operation, "peer_receipt", level);
                        instrumentation.settle_peer_operation(operation, level);
                    }
                    last_aegis_managed_notice = current.into();
                }
            }
            if let Some(tellus) = &mut tellus {
                let settlement = tellus.poll_settlement();
                health.settle("tellus", "provenance", settlement);
                settle_refresh(
                    &mut refresh_operations.tellus,
                    settlement,
                    "tellus",
                    &mut instrumentation,
                    &mut activity,
                );
            }
            if last_vellum_poll.elapsed() >= Duration::from_secs(1) {
                if let Some(vellum) = &mut vellum {
                    let started = Instant::now();
                    let renew_identity = vellum.issued_requests() >= 192;
                    let result: Result<(), String> = if renew_identity {
                        vellum
                            .renew_identity()
                            .map_err(|error| {
                                format!("Vellum client identity renewal refused: {error:?}")
                            })
                            .and_then(|()| vellum.poll().map_err(|error| error.to_string()))
                    } else {
                        vellum.poll().map_err(|error| error.to_string())
                    };
                    if renew_identity {
                        let level = if result.is_ok() {
                            ActivityLevel::Success
                        } else {
                            ActivityLevel::Warning
                        };
                        instrumentation.record(
                            "vellum.client.renew-identity",
                            "vellum",
                            level,
                            started.elapsed(),
                        );
                        activity.push(
                            level,
                            "vellum",
                            if result.is_ok() {
                                "Vellum replay identity renewed before request capacity exhaustion"
                            } else {
                                "Vellum replay identity renewal failed; editing is fenced until recovery"
                            },
                        );
                    }
                    health.settle(
                        "vellum",
                        "workspace",
                        Some(if result.is_ok() { Ok(()) } else { Err(()) }),
                    );
                    instrumentation.record(
                        "vellum.projection.poll",
                        "vellum",
                        if result.is_ok() {
                            ActivityLevel::Success
                        } else {
                            ActivityLevel::Warning
                        },
                        started.elapsed(),
                    );
                    if let Err(error) = result {
                        notice = error.to_string();
                    }
                }
                last_vellum_poll = Instant::now();
            }
            if let Some(control) = &mut vellum_durable_save {
                control.poll();
                if let Some(editor) = &mut vellum {
                    editor.admit_durable_saves(&control.durable_document_ordinals());
                }
                let current = control.notice();
                if current != last_vellum_durable_notice {
                    let level = match control.disposition() {
                        DurableSaveBatchDisposition::Durable => ActivityLevel::Success,
                        DurableSaveBatchDisposition::OutcomeUnknown => ActivityLevel::Uncertain,
                        DurableSaveBatchDisposition::Partial => ActivityLevel::Warning,
                        DurableSaveBatchDisposition::Proposed
                        | DurableSaveBatchDisposition::Armed
                        | DurableSaveBatchDisposition::Pending => ActivityLevel::Info,
                    };
                    notice = current.into();
                    activity.push(level, "vellum", current);
                    if !control.request_pending()
                        && !matches!(
                            control.disposition(),
                            DurableSaveBatchDisposition::Proposed
                                | DurableSaveBatchDisposition::Armed
                        )
                        && let Some(operation) = vellum_durable_operation.take()
                    {
                        instrumentation.mark_peer_stage(operation, "child_receipts", level);
                        instrumentation.settle_peer_operation(operation, level);
                    }
                    last_vellum_durable_notice = current.into();
                }
            }
            health.observing(
                "agora",
                "rooms/messages",
                agora
                    .as_ref()
                    .is_some_and(agora_tui::Component::request_pending),
            );
            health.observing(
                "moira",
                "sagas",
                moira
                    .as_ref()
                    .is_some_and(moira_tui::Component::request_pending),
            );
            health.observing(
                "daimon",
                "presence",
                daimon
                    .as_ref()
                    .is_some_and(daimon_tui::Component::request_pending),
            );
            health.observing(
                "daimon",
                "sessions/events",
                daimon_agent
                    .as_ref()
                    .is_some_and(daimon_tui::AgentSetComponent::request_pending),
            );
            health.observing(
                "hermes",
                "ledger/cursors",
                hermes
                    .as_ref()
                    .is_some_and(hermes_tui::Component::request_pending),
            );
            health.observing(
                "aegis",
                "authority",
                aegis
                    .as_ref()
                    .is_some_and(aegis_tui::Component::request_pending),
            );
            health.observing(
                "tellus",
                "provenance",
                tellus
                    .as_ref()
                    .is_some_and(tellus::Component::request_pending),
            );
            let peer_busy = |peer: &str| match peer {
                "chronos" => chronos_operation.is_some() || refresh_operations.chronos.is_some(),
                "vellum" => {
                    vellum_durable_save
                        .as_ref()
                        .is_some_and(DurableSaveBatchComponent::request_pending)
                        || vellum_atomic_operation.is_some()
                }
                "agora" => {
                    agora
                        .as_ref()
                        .is_some_and(agora_tui::Component::request_pending)
                        || agora_create
                            .as_ref()
                            .is_some_and(agora_tui::CreateRoomComponent::request_pending)
                        || agora_message
                            .as_ref()
                            .is_some_and(agora_tui::ComposeMessageComponent::request_pending)
                }
                "moira" => {
                    moira
                        .as_ref()
                        .is_some_and(moira_tui::Component::request_pending)
                        || moira_open
                            .as_ref()
                            .is_some_and(moira_tui::OpenControlComponent::request_pending)
                        || moira_agent_worker.pending
                        || moira_reconciliation
                            .as_ref()
                            .is_some_and(|control| control.request_pending())
                }
                "daimon" => {
                    daimon
                        .as_ref()
                        .is_some_and(daimon_tui::Component::request_pending)
                        || daimon_agent
                            .as_ref()
                            .is_some_and(daimon_tui::AgentSetComponent::request_pending)
                        || daimon_control
                            .as_ref()
                            .is_some_and(|control| control.request_pending())
                }
                "hermes" => {
                    hermes
                        .as_ref()
                        .is_some_and(hermes_tui::Component::request_pending)
                        || hermes_disclosure.as_ref().is_some_and(|control| {
                            control.disposition() == hermes_tui::DisclosureDisposition::Pending
                        })
                        || hermes_compaction.as_ref().is_some_and(|control| {
                            control.disposition() == hermes_tui::CompactionDisposition::Pending
                        })
                        || hermes_acknowledge.as_ref().is_some_and(|control| {
                            control.disposition() == hermes_tui::AcknowledgeDisposition::Pending
                        })
                }
                "aegis" => {
                    aegis
                        .as_ref()
                        .is_some_and(aegis_tui::Component::request_pending)
                        || aegis_revoke.as_ref().is_some_and(|control| {
                            control.disposition() == aegis_tui::RevokeEpochDisposition::Pending
                        })
                        || aegis_managed.as_ref().is_some_and(|control| {
                            control.disposition() == aegis_tui::ManagedAuthorityDisposition::Pending
                        })
                }
                "tellus" => tellus
                    .as_ref()
                    .is_some_and(tellus::Component::request_pending),
                _ => false,
            };
            let peer_uncertain = |peer: &str| match peer {
                "chronos" => chronos
                    .as_ref()
                    .is_some_and(Hub::mutation_recovery_required),
                "vellum" => {
                    vellum_durable_save
                        .as_ref()
                        .is_some_and(DurableSaveBatchComponent::recovery_required)
                        || vellum_atomic_replace.as_ref().is_some_and(|control| {
                            control.disposition() == AtomicReplaceDisposition::OutcomeUnknown
                        })
                }
                "agora" => {
                    agora_create
                        .as_ref()
                        .is_some_and(agora_tui::CreateRoomComponent::recovery_required)
                        || agora_message
                            .as_ref()
                            .is_some_and(agora_tui::ComposeMessageComponent::recovery_required)
                }
                "moira" => {
                    moira_open
                        .as_ref()
                        .is_some_and(moira_tui::OpenControlComponent::recovery_required)
                        || moira_agent_worker.uncertain
                        || moira_reconciliation
                            .as_ref()
                            .is_some_and(moira_tui::ReconciliationBatchComponent::recovery_required)
                }
                "daimon" => daimon_control.as_ref().is_some_and(|control| {
                    control.disposition() == daimon_tui::AgentControlDisposition::OutcomeUnknown
                }),
                "hermes" => {
                    hermes_compaction.as_ref().is_some_and(|control| {
                        control.disposition() == hermes_tui::CompactionDisposition::OutcomeUnknown
                    }) || hermes_acknowledge.as_ref().is_some_and(|control| {
                        control.disposition() == hermes_tui::AcknowledgeDisposition::OutcomeUnknown
                    })
                }
                "aegis" => {
                    aegis_revoke.as_ref().is_some_and(|control| {
                        control.disposition() == aegis_tui::RevokeEpochDisposition::OutcomeUnknown
                    }) || aegis_managed.as_ref().is_some_and(|control| {
                        control.disposition() == aegis_tui::ManagedAuthorityDisposition::Fenced
                    })
                }
                "tellus" => false,
                _ => false,
            };
            let health_view = health.view(peer_busy, peer_uncertain);
            let height = terminal.size()?.height.saturating_sub(4);
            let vellum_frame = vellum.as_ref().and_then(|editor| editor.frame(height));
            let operations_view = operations.view();
            let agenda_view = operator_agenda_view(chronos.as_ref(), moira_participation.as_ref());
            let render_started = Instant::now();
            terminal.draw(|frame| {
                let prompt_notice = vellum_open_prompt
                    .as_ref()
                    .or(vellum_find_prompt.as_ref())
                    .map(|prompt| {
                        prompt.display(usize::from(frame.area().width.saturating_sub(6)))
                    });
                draw_shell_with_moira(
                    frame,
                    &shell,
                    PeerViews {
                        chronos: chronos.as_ref(),
                        vellum: vellum_frame.as_ref(),
                        vellum_atomic_replace: vellum_atomic_replace.as_ref(),
                        vellum_durable_save: vellum_durable_save.as_ref(),
                        agora: agora.as_ref(),
                        agora_create: agora_create.as_ref(),
                        agora_message: agora_message.as_ref(),
                        moira: moira.as_ref(),
                        moira_participation: moira_participation.as_ref(),
                        moira_open: moira_open.as_ref(),
                        moira_reconciliation: moira_reconciliation.as_ref(),
                        daimon: daimon.as_ref(),
                        daimon_model: daimon_model.as_ref(),
                        daimon_agent: daimon_agent.as_ref(),
                        daimon_permission: daimon_permission_composer.as_ref(),
                        daimon_prompt: daimon_prompt_composer.as_ref(),
                        daimon_control: daimon_control.as_ref(),
                        hermes: hermes.as_ref(),
                        hermes_compaction: hermes_compaction.as_ref(),
                        hermes_disclosure: hermes_disclosure.as_ref(),
                        hermes_acknowledge: hermes_acknowledge.as_ref(),
                        aegis: aegis.as_ref(),
                        aegis_managed: aegis_managed.as_ref(),
                        aegis_revoke: aegis_revoke.as_ref(),
                        tellus: tellus.as_ref(),
                        agenda: Some(&agenda_view),
                        operations: Some(&operations_view),
                        health: Some(&health_view),
                        configuration: Some(&configuration),
                    },
                    focus,
                    prompt_notice.as_deref().unwrap_or(&notice),
                );
                match overlay {
                    WorkbenchOverlay::None => {}
                    WorkbenchOverlay::Commands => draw_command_palette(frame, &palette),
                    WorkbenchOverlay::Help => draw_help(frame, palette.commands()),
                    WorkbenchOverlay::Activity => draw_activity(frame, &activity),
                    WorkbenchOverlay::Configuration => {
                        draw_inspector(frame, "Session configuration", &configuration);
                    }
                    WorkbenchOverlay::LocalProviders => {
                        draw_inspector(
                            frame,
                            "Local provider configuration",
                            &provider_configuration_lines,
                        );
                    }
                    WorkbenchOverlay::ProviderEdit => {
                        if let Some(workflow) = &provider_edit {
                            draw_inspector(
                                frame,
                                "Add or replace local agent provider",
                                &workflow.lines(),
                            );
                        }
                    }
                    WorkbenchOverlay::ProviderQualification => {
                        if let Some(workflow) = &provider_qualification {
                            draw_inspector(
                                frame,
                                "Qualify local model provider",
                                &workflow.lines(),
                            );
                        }
                    }
                    WorkbenchOverlay::AcpSessions => {
                        draw_inspector(
                            frame,
                            "ACP session configuration",
                            &acp_configuration_lines,
                        );
                    }
                    WorkbenchOverlay::AgoraRooms => {
                        draw_inspector(
                            frame,
                            "Agora room configuration",
                            &agora_configuration_lines,
                        );
                    }
                    WorkbenchOverlay::Reconfiguration => {
                        if let Some(workflow) = &reconfiguration {
                            draw_inspector(frame, "Workspace profile restart", &workflow.lines());
                        }
                    }
                    WorkbenchOverlay::Instrumentation => {
                        draw_inspector(frame, "TUI instrumentation", &instrumentation.lines());
                    }
                }
            })?;
            instrumentation.record(
                "workbench.render",
                "pantheon",
                ActivityLevel::Success,
                render_started.elapsed(),
            );
            if let Some((operation, level)) = pending_ui_feedback.take() {
                instrumentation.mark_peer_stage(operation, "feedback_presented", level);
                instrumentation.mark_peer_stage(
                    operation,
                    "frame_rendered",
                    ActivityLevel::Success,
                );
                instrumentation.settle_peer_operation(operation, level);
            }
            if !event::poll(Duration::from_millis(25))? {
                continue;
            }
            let event = event::read()?;
            let input_started = Instant::now();
            let mut key_behavior = match &event {
                Event::Key(key) => Some(BehaviorFacade::key(
                    *key,
                    overlay,
                    shell.active(),
                    vellum_open_prompt.is_none()
                        && vellum_find_prompt.is_none()
                        && vellum_surface_active(shell.active(), focus),
                )),
                _ => None,
            };
            if let (Event::Key(key), Some(KeyBehavior::CommandPaletteInput)) =
                (&event, key_behavior)
            {
                key_behavior = Some(BehaviorFacade::palette(palette.key(*key)));
            }
            if key_behavior == Some(KeyBehavior::SafeExit) {
                // Safe exit crosses overlay boundaries, then reaches the one
                // guarded exit implementation at the bottom of this dispatch.
                overlay = WorkbenchOverlay::None;
            }
            match event {
                Event::Paste(text) if overlay == WorkbenchOverlay::Commands => {
                    palette.insert(&text);
                }
                Event::Key(_)
                    if matches!(key_behavior, Some(KeyBehavior::CommandUnavailable(_))) =>
                {
                    let Some(KeyBehavior::CommandUnavailable(reason)) = key_behavior else {
                        unreachable!("guarded above")
                    };
                    let operation = instrumentation
                        .begin_ui_operation_at("workbench.command.unavailable", input_started)
                        .map(|(operation, _)| operation);
                    if let Some(operation) = operation {
                        instrumentation.mark_peer_stage(
                            operation,
                            "command_dispatched",
                            ActivityLevel::Warning,
                        );
                        pending_ui_feedback = Some((operation, ActivityLevel::Warning));
                    }
                    notice = command_feedback(
                        "workbench.command.unavailable",
                        ActivityLevel::Warning,
                        reason,
                    );
                    activity.push(ActivityLevel::Warning, "palette", notice.clone());
                    overlay = WorkbenchOverlay::None;
                }
                Event::Key(_) if matches!(key_behavior, Some(KeyBehavior::ExecuteCommand(_))) => {
                    let Some(KeyBehavior::ExecuteCommand(action)) = key_behavior else {
                        unreachable!("guarded above")
                    };
                    let ui_operation = instrumentation.begin_ui_operation_at(action, input_started);
                    let ui_correlation = ui_operation.map(|(_, correlation)| correlation);
                    if let Some((operation, _)) = ui_operation {
                        instrumentation.mark_peer_stage(
                            operation,
                            "command_dispatched",
                            ActivityLevel::Info,
                        );
                    }
                    if action.starts_with("navigate.") {
                        vellum_open_prompt = None;
                        vellum_find_prompt = None;
                        confirming_discard = false;
                    }
                    if action == "configuration.providers.agent.upsert" {
                        if provider_edit
                            .as_ref()
                            .is_some_and(|workflow| workflow.worker.is_some())
                        {
                            notice = "QUEUED · A guarded provider owner operation is already running · Next: wait for its exact plan or receipt"
                                .into();
                            overlay = WorkbenchOverlay::ProviderEdit;
                            continue;
                        }
                        provider_edit = Some(ProviderEditWorkflow::new());
                        notice = "OPENED · Guarded local agent provider form · Next: complete the bounded fields and review Daimon's exact redacted plan"
                            .into();
                        activity.push(ActivityLevel::Info, "configuration", notice.clone());
                        pending_ui_feedback =
                            ui_operation.map(|(operation, _)| (operation, ActivityLevel::Info));
                        overlay = WorkbenchOverlay::ProviderEdit;
                        continue;
                    }
                    if action == "models.provider.qualify.v1" {
                        if provider_qualification.as_ref().is_some_and(|workflow| {
                            workflow.worker.is_some()
                                || workflow.phase == ProviderQualificationPhase::Qualifying
                        }) {
                            notice = "QUEUED · A provider qualification is already running · Next: wait for its exact owner receipt; it will not be cancelled or duplicated"
                                .into();
                            overlay = WorkbenchOverlay::ProviderQualification;
                            continue;
                        }
                        provider_qualification =
                            Some(ProviderQualificationWorkflow::new(current_profile.clone()));
                        notice = "OPENED · Local-model qualification review · Next: wait for the owner plan, inspect exact redacted facts, then press c to arm"
                            .into();
                        activity.push(ActivityLevel::Info, "daimon", notice.clone());
                        pending_ui_feedback =
                            ui_operation.map(|(operation, _)| (operation, ActivityLevel::Info));
                        overlay = WorkbenchOverlay::ProviderQualification;
                        continue;
                    }
                    let configuration_overlay = match action {
                        "configuration.providers.local" => Some((
                            WorkbenchOverlay::LocalProviders,
                            "Local provider configuration",
                        )),
                        "configuration.providers.refresh" => Some((
                            WorkbenchOverlay::LocalProviders,
                            "Local provider configuration",
                        )),
                        "configuration.acp.sessions" => {
                            Some((WorkbenchOverlay::AcpSessions, "ACP session configuration"))
                        }
                        "configuration.agora.rooms" => {
                            Some((WorkbenchOverlay::AgoraRooms, "Agora room configuration"))
                        }
                        _ => None,
                    };
                    if let Some((next_overlay, label)) = configuration_overlay {
                        if matches!(
                            action,
                            "configuration.providers.local" | "configuration.providers.refresh"
                        ) {
                            if provider_inspection.is_some() {
                                notice = "QUEUED · Local provider inspection is already running · Next: wait for the bounded owner projection or its refusal"
                                    .into();
                                activity.push(ActivityLevel::Info, "configuration", notice.clone());
                                pending_ui_feedback = ui_operation
                                    .map(|(operation, _)| (operation, ActivityLevel::Info));
                                overlay = next_overlay;
                                continue;
                            }
                            provider_configuration_lines = vec![
                                "INSPECTING · Asking the selected Daimon owner to validate and redact its provider catalogue."
                                    .into(),
                                "The TUI remains responsive; no provider effect or configuration mutation is being attempted."
                                    .into(),
                            ];
                            provider_inspection = Some(spawn_provider_inspection(
                                current_profile.clone(),
                                action == "configuration.providers.refresh",
                            ));
                        }
                        notice = if matches!(
                            action,
                            "configuration.providers.local" | "configuration.providers.refresh"
                        ) {
                            format!(
                                "QUEUED · {label} inspection · Next: wait for the bounded owner projection; the interface remains responsive"
                            )
                        } else {
                            format!(
                                "OPENED · {label} · Next: review owner/profile requirements or Esc to close"
                            )
                        };
                        activity.push(ActivityLevel::Info, "configuration", notice.clone());
                        pending_ui_feedback =
                            ui_operation.map(|(operation, _)| (operation, ActivityLevel::Info));
                        overlay = next_overlay;
                        continue;
                    }
                    if action == "workbench.help" {
                        notice =
                            "OPENED · Contextual help · Next: F1 or C-g returns to work".into();
                        activity.push(ActivityLevel::Info, "pantheon", notice.clone());
                        pending_ui_feedback =
                            ui_operation.map(|(operation, _)| (operation, ActivityLevel::Info));
                        overlay = WorkbenchOverlay::Help;
                        continue;
                    }
                    if action == "workbench.activity" {
                        notice =
                            "OPENED · Activity and receipts · Next: C-g returns to work".into();
                        pending_ui_feedback =
                            ui_operation.map(|(operation, _)| (operation, ActivityLevel::Info));
                        overlay = WorkbenchOverlay::Activity;
                        continue;
                    }
                    if action == "workbench.configuration" {
                        notice = "OPENED · Session configuration · Next: inspect requirements or run workspace reconfiguration from M-x".into();
                        activity.push(ActivityLevel::Info, "configuration", notice.clone());
                        pending_ui_feedback =
                            ui_operation.map(|(operation, _)| (operation, ActivityLevel::Info));
                        overlay = WorkbenchOverlay::Configuration;
                        continue;
                    }
                    if action == "workbench.reconfigure" {
                        if reconfiguration.is_some() {
                            notice = "READY · Workspace profile preflight · Next: enter a profile path and press Enter".into();
                            overlay = WorkbenchOverlay::Reconfiguration;
                        } else {
                            notice = command_feedback(
                                action,
                                ActivityLevel::Warning,
                                "This session was not launched from a workspace profile",
                            );
                        }
                        let level = if reconfiguration.is_some() {
                            ActivityLevel::Info
                        } else {
                            ActivityLevel::Warning
                        };
                        activity.push(level, "configuration", notice.clone());
                        pending_ui_feedback = ui_operation.map(|(operation, _)| (operation, level));
                        continue;
                    }
                    if action == "workbench.instrumentation" {
                        instrumentation.record(
                            action,
                            "pantheon",
                            ActivityLevel::Info,
                            Duration::ZERO,
                        );
                        notice = "OPENED · Presentation instrumentation · Next: inspect correlations and budget violations; C-g returns".into();
                        activity.push(ActivityLevel::Info, "presentation", notice.clone());
                        pending_ui_feedback =
                            ui_operation.map(|(operation, _)| (operation, ActivityLevel::Info));
                        overlay = WorkbenchOverlay::Instrumentation;
                        continue;
                    }
                    if action == "vellum.document.open" {
                        overlay = WorkbenchOverlay::None;
                        if vellum.is_none() {
                            notice = command_feedback(
                                action,
                                ActivityLevel::Warning,
                                "Vellum service is not configured",
                            );
                            activity.push(ActivityLevel::Warning, "vellum", notice.clone());
                            pending_ui_feedback = ui_operation
                                .map(|(operation, _)| (operation, ActivityLevel::Warning));
                            continue;
                        }
                        activate_peer_surface(&mut shell, Surface::Vellum);
                        focus = Focus::Vellum;
                        vellum_open_prompt = Some(TextPrompt::new(4096, "Open file"));
                        notice = command_feedback(
                            action,
                            ActivityLevel::Info,
                            "Vellum file admission prompt opened",
                        );
                        activity.push(ActivityLevel::Info, "vellum", notice.clone());
                        pending_ui_feedback =
                            ui_operation.map(|(operation, _)| (operation, ActivityLevel::Info));
                        continue;
                    }
                    if matches!(
                        action,
                        "ledger.compact.v1" | "ledger.events.disclose.v1" | "ledger.acknowledge.v1"
                    ) && (hermes_compaction.as_ref().is_some_and(|control| {
                        matches!(
                            control.disposition(),
                            hermes_tui::CompactionDisposition::Pending
                                | hermes_tui::CompactionDisposition::OutcomeUnknown
                        )
                    }) || hermes_compaction_post_receipt_refresh
                        || hermes_compaction_operation.is_some()
                        || hermes_disclosure.as_ref().is_some_and(|control| {
                            control.disposition() == hermes_tui::DisclosureDisposition::Pending
                        })
                        || hermes_disclosure_operation.is_some()
                        || hermes_acknowledge.as_ref().is_some_and(|control| {
                            matches!(
                                control.disposition(),
                                hermes_tui::AcknowledgeDisposition::Pending
                                    | hermes_tui::AcknowledgeDisposition::OutcomeUnknown
                            )
                        })
                        || hermes_post_receipt_refresh
                        || hermes_operation.is_some())
                    {
                        overlay = WorkbenchOverlay::None;
                        notice = "A Hermes control is still pending, reconciling, or outcome-unknown; settle it before opening another"
                                    .into();
                        activity.push(ActivityLevel::Warning, "hermes", notice.clone());
                        pending_ui_feedback =
                            ui_operation.map(|(operation, _)| (operation, ActivityLevel::Warning));
                        continue;
                    }
                    overlay = WorkbenchOverlay::None;
                    let refresh_surface = shell.active();
                    let refresh_focus = focus;
                    let refresh_pending_before = (
                        agora.as_ref().is_some_and(|value| value.request_pending()),
                        moira.as_ref().is_some_and(|value| value.request_pending()),
                        if let Some(value) = daimon_agent.as_ref() {
                            value.request_pending()
                        } else {
                            daimon.as_ref().is_some_and(|value| value.request_pending())
                        },
                        hermes.as_ref().is_some_and(|value| value.request_pending()),
                        aegis.as_ref().is_some_and(|value| value.request_pending()),
                        tellus.as_ref().is_some_and(|value| value.request_pending()),
                    );
                    let started = Instant::now();
                    let (level, source, message) = execute_workbench_action(
                        action,
                        &mut shell,
                        &mut focus,
                        &mut chronos,
                        &mut vellum,
                        vellum_control.as_ref(),
                        &mut vellum_durable_save,
                        &mut vellum_atomic_replace,
                        &mut agora,
                        agora_control.as_ref(),
                        &mut agora_create,
                        &mut agora_message,
                        &mut moira,
                        &mut moira_participation,
                        moira_participation_control.as_ref(),
                        &mut moira_open,
                        moira_agent_worker_config.as_ref(),
                        &mut moira_agent_worker,
                        moira_control_socket.as_deref(),
                        &mut moira_reconciliation,
                        &mut daimon,
                        &mut daimon_agent,
                        daimon_agent_control_socket.as_deref(),
                        daimon_execution_policy.as_ref(),
                        &mut daimon_prompt_composer,
                        &mut daimon_permission_composer,
                        &mut daimon_control,
                        &mut hermes,
                        hermes_operator_socket.as_deref(),
                        &mut hermes_compaction,
                        hermes_disclosure_socket.as_deref(),
                        &mut hermes_disclosure,
                        hermes_control_socket.as_deref(),
                        &mut hermes_acknowledge,
                        &mut aegis,
                        aegis_control_socket.as_deref(),
                        &mut aegis_revoke,
                        &mut aegis_managed,
                        &mut tellus,
                        &mut operations,
                    );
                    if action == "workitem.agent.new"
                        || agent_task_collaboration_lens(action).is_some()
                    {
                        work_item_agent = if level == ActivityLevel::Info {
                            WorkItemAgentJourney::EditingTask {
                                draft: String::new(),
                                lens: agent_task_collaboration_lens(action),
                            }
                        } else {
                            WorkItemAgentJourney::Failed
                        };
                    }
                    let peer_workflow_opened = level == ActivityLevel::Info
                        && (agent_task_collaboration_lens(action).is_some()
                            || matches!(
                                action,
                                "workitem.agent.new"
                                    | "agora.room.create"
                                    | "agora.message.compose"
                                    | "moira.reconciliation.keep-uncertain"
                                    | "moira.reconciliation.fail-terminal"
                                    | "moira.reconciliation.cancel"
                                    | "moira.reconciliation.compensate"
                                    | "moira.reconciliation.admit-failed"
                                    | "moira.reconciliation.admit-succeeded"
                                    | "moira.reconciliation.retry"
                                    | "daimon.session.prompt"
                                    | "daimon.session.permission"
                                    | "daimon.session.reattach"
                                    | "daimon.session.advance"
                                    | "daimon.session.cancel"
                                    | "daimon.session.close"
                                    | "ledger.compact.v1"
                                    | "ledger.events.disclose.v1"
                                    | "ledger.acknowledge.v1"
                                    | "aegis.epoch.revoke"
                                    | "aegis.authority.manage"
                                    | "vellum.save-selected"
                                    | "vellum.atomic-replace-selected"
                            ));
                    let mut correlated_refresh = false;
                    if matches!(
                        action,
                        "suite.refresh" | "view.refresh" | "workitem.agent.return"
                    ) {
                        let refresh_correlation =
                            ui_correlation.unwrap_or_else(|| instrumentation.new_correlation());
                        let suite = action == "suite.refresh";
                        let current = |surface| suite || refresh_surface == surface;
                        let chronos_selected = action == "workitem.agent.return"
                            || suite
                            || refresh_surface == Surface::Chronos
                            || (refresh_surface == Surface::Workspace
                                && refresh_focus == Focus::Chronos);
                        correlated_refresh |= begin_refresh_correlated(
                            chronos_selected && chronos.is_some(),
                            action,
                            "chronos",
                            &mut refresh_operations.chronos,
                            &mut instrumentation,
                            started,
                            Some(refresh_correlation),
                        );
                        correlated_refresh |= begin_refresh_correlated(
                            current(Surface::Agora)
                                && !refresh_pending_before.0
                                && agora.as_ref().is_some_and(|value| value.request_pending()),
                            action,
                            "agora",
                            &mut refresh_operations.agora,
                            &mut instrumentation,
                            started,
                            Some(refresh_correlation),
                        );
                        correlated_refresh |= begin_refresh_correlated(
                            current(Surface::Moira)
                                && !refresh_pending_before.1
                                && moira.as_ref().is_some_and(|value| value.request_pending()),
                            action,
                            "moira",
                            &mut refresh_operations.moira,
                            &mut instrumentation,
                            started,
                            Some(refresh_correlation),
                        );
                        let daimon_pending = if let Some(value) = daimon_agent.as_ref() {
                            value.request_pending()
                        } else {
                            daimon.as_ref().is_some_and(|value| value.request_pending())
                        };
                        correlated_refresh |= begin_refresh_correlated(
                            current(Surface::Daimon) && !refresh_pending_before.2 && daimon_pending,
                            action,
                            "daimon",
                            &mut refresh_operations.daimon,
                            &mut instrumentation,
                            started,
                            Some(refresh_correlation),
                        );
                        correlated_refresh |= begin_refresh_correlated(
                            current(Surface::Hermes)
                                && !refresh_pending_before.3
                                && hermes.as_ref().is_some_and(|value| value.request_pending()),
                            action,
                            "hermes",
                            &mut refresh_operations.hermes,
                            &mut instrumentation,
                            started,
                            Some(refresh_correlation),
                        );
                        correlated_refresh |= begin_refresh_correlated(
                            current(Surface::Aegis)
                                && !refresh_pending_before.4
                                && aegis.as_ref().is_some_and(|value| value.request_pending()),
                            action,
                            "aegis",
                            &mut refresh_operations.aegis,
                            &mut instrumentation,
                            started,
                            Some(refresh_correlation),
                        );
                        correlated_refresh |= begin_refresh_correlated(
                            current(Surface::Tellus)
                                && !refresh_pending_before.5
                                && tellus.as_ref().is_some_and(|value| value.request_pending()),
                            action,
                            "tellus",
                            &mut refresh_operations.tellus,
                            &mut instrumentation,
                            started,
                            Some(refresh_correlation),
                        );
                    }
                    if matches!(action, "agora.room.open" | "agora.room.back") {
                        correlated_refresh |= begin_refresh(
                            !refresh_pending_before.0
                                && agora.as_ref().is_some_and(|value| value.request_pending()),
                            action,
                            "agora",
                            &mut refresh_operations.agora,
                            &mut instrumentation,
                            started,
                        );
                    }
                    if matches!(action, "moira.saga.open" | "moira.saga.back") {
                        correlated_refresh |= begin_refresh(
                            !refresh_pending_before.1
                                && moira.as_ref().is_some_and(|value| value.request_pending()),
                            action,
                            "moira",
                            &mut refresh_operations.moira,
                            &mut instrumentation,
                            started,
                        );
                    }
                    if matches!(action, "aegis.page.next" | "aegis.page.previous") {
                        correlated_refresh |= begin_refresh(
                            !refresh_pending_before.4
                                && aegis.as_ref().is_some_and(|value| value.request_pending()),
                            action,
                            "aegis",
                            &mut refresh_operations.aegis,
                            &mut instrumentation,
                            started,
                        );
                    }
                    if !peer_workflow_opened && !correlated_refresh {
                        instrumentation.record(action, source, level, started.elapsed());
                    }
                    if matches!(
                        action,
                        "moira.reconciliation.keep-uncertain"
                            | "moira.reconciliation.fail-terminal"
                            | "moira.reconciliation.cancel"
                            | "moira.reconciliation.compensate"
                            | "moira.reconciliation.admit-failed"
                            | "moira.reconciliation.admit-succeeded"
                            | "moira.reconciliation.retry"
                    ) && level == ActivityLevel::Info
                        && moira_reconciliation.is_some()
                    {
                        active_moira_action = Some(action);
                        if moira_operation.is_none()
                            && moira_reconciliation
                                .as_ref()
                                .is_some_and(|control| control.request_pending())
                        {
                            moira_operation = instrumentation.begin_peer_operation(action, "moira");
                        }
                    }
                    if action.starts_with("daimon.session.")
                        && daimon_control.is_some()
                        && level == ActivityLevel::Info
                    {
                        active_daimon_action = Some(action);
                    }
                    notice = command_feedback(action, level, &message);
                    activity.push(level, source, notice.clone());
                    pending_ui_feedback = ui_operation.map(|(operation, _)| (operation, level));
                }
                Event::Paste(text) if overlay == WorkbenchOverlay::Reconfiguration => {
                    if let Some(workflow) = &mut reconfiguration {
                        workflow.insert(&text);
                    }
                }
                Event::Paste(text) if overlay == WorkbenchOverlay::ProviderEdit => {
                    if let Some(workflow) = &mut provider_edit {
                        workflow.paste(&text);
                    }
                }
                Event::Key(key) if key_behavior == Some(KeyBehavior::ProviderEditInput) => {
                    if let Some(workflow) = &mut provider_edit {
                        workflow.key(key, current_profile.clone());
                        notice = workflow.notice.clone();
                    } else {
                        overlay = WorkbenchOverlay::None;
                    }
                }
                Event::Key(key)
                    if key_behavior == Some(KeyBehavior::ProviderQualificationInput) =>
                {
                    if let Some(workflow) = &mut provider_qualification {
                        workflow.key(key, current_profile.clone());
                        notice = workflow.notice.clone();
                    } else {
                        overlay = WorkbenchOverlay::None;
                    }
                }
                Event::Key(key) if key_behavior == Some(KeyBehavior::ReconfigurationInput) => {
                    if is_key(key, KeyCode::Esc, KeyModifiers::NONE) {
                        overlay = WorkbenchOverlay::None;
                        continue;
                    }
                    let Some(workflow) = &mut reconfiguration else {
                        overlay = WorkbenchOverlay::None;
                        continue;
                    };
                    if is_key(key, KeyCode::Backspace, KeyModifiers::NONE) {
                        workflow.backspace();
                    } else if is_key(key, KeyCode::Char('e'), KeyModifiers::NONE)
                        && workflow.plan.is_some()
                    {
                        workflow.edit();
                    } else if is_key(key, KeyCode::Char('c'), KeyModifiers::NONE)
                        && workflow.plan.is_some()
                    {
                        workflow.arm();
                    } else if is_key(key, KeyCode::Enter, KeyModifiers::NONE)
                        && workflow.plan.is_none()
                    {
                        let started = Instant::now();
                        workflow.preflight();
                        let level = if workflow.plan.is_some() {
                            ActivityLevel::Success
                        } else {
                            ActivityLevel::Warning
                        };
                        instrumentation.record(
                            "workbench.reconfigure.preflight",
                            "pantheon",
                            level,
                            started.elapsed(),
                        );
                        activity.push(level, "configuration", workflow.notice.clone());
                    } else if is_key(key, KeyCode::Enter, KeyModifiers::NONE) {
                        let Some((path, digest)) = workflow.confirmed_profile() else {
                            workflow.notice =
                                "Press c to arm the preflighted restart before Enter".into();
                            continue;
                        };
                        match WorkspaceProfile::load_with_digest(&path) {
                            Ok((_, current_digest)) if current_digest == digest => {}
                            Ok(_) => {
                                workflow.armed = false;
                                workflow.notice = "Restart refused: profile bytes changed after preflight; review the new diff"
                                    .into();
                                activity.push(
                                    ActivityLevel::Warning,
                                    "configuration",
                                    workflow.notice.clone(),
                                );
                                continue;
                            }
                            Err(error) => {
                                workflow.armed = false;
                                workflow.notice = format!(
                                    "Restart refused: profile revalidation failed: {error}"
                                );
                                activity.push(
                                    ActivityLevel::Warning,
                                    "configuration",
                                    workflow.notice.clone(),
                                );
                                continue;
                            }
                        }
                        let agora_unresolved = agora_create.as_ref().is_some_and(|control| {
                            control.request_pending() || control.recovery_required()
                        }) || agora_message.as_ref().is_some_and(
                            |control| control.request_pending() || control.recovery_required(),
                        );
                        let moira_unresolved =
                            moira_reconciliation.as_ref().is_some_and(|control| {
                                control.request_pending() || control.recovery_required()
                            });
                        let daimon_unresolved = daimon_control.as_ref().is_some_and(|control| {
                            control.request_pending() || control.recovery_required()
                        }) || daimon_post_receipt_refresh
                            || daimon_recovery_refresh
                            || daimon_operation.is_some();
                        let aegis_unresolved =
                            aegis_revoke.as_ref().is_some_and(|control| {
                                matches!(
                                    control.disposition(),
                                    aegis_tui::RevokeEpochDisposition::Pending
                                        | aegis_tui::RevokeEpochDisposition::OutcomeUnknown
                                )
                            }) || aegis_managed.as_ref().is_some_and(|control| {
                                matches!(
                                    control.disposition(),
                                    aegis_tui::ManagedAuthorityDisposition::Pending
                                        | aegis_tui::ManagedAuthorityDisposition::Fenced
                                )
                            });
                        let hermes_unresolved =
                            hermes_acknowledge.as_ref().is_some_and(|control| {
                                matches!(
                                    control.disposition(),
                                    hermes_tui::AcknowledgeDisposition::Pending
                                        | hermes_tui::AcknowledgeDisposition::OutcomeUnknown
                                )
                            }) || hermes_post_receipt_refresh
                                || hermes_operation.is_some()
                                || hermes_disclosure.as_ref().is_some_and(|control| {
                                    control.disposition()
                                        == hermes_tui::DisclosureDisposition::Pending
                                })
                                || hermes_disclosure_operation.is_some()
                                || hermes_compaction.as_ref().is_some_and(|control| {
                                    matches!(
                                        control.disposition(),
                                        hermes_tui::CompactionDisposition::Pending
                                            | hermes_tui::CompactionDisposition::OutcomeUnknown
                                    )
                                })
                                || hermes_compaction_post_receipt_refresh
                                || hermes_compaction_operation.is_some();
                        let chronos_unresolved = chronos_operation.is_some()
                            || chronos
                                .as_ref()
                                .is_some_and(Hub::mutation_recovery_required);
                        let provider_inspection_pending = provider_inspection.is_some();
                        let provider_edit_pending = provider_edit
                            .as_ref()
                            .is_some_and(|workflow| workflow.worker.is_some());
                        let provider_qualification_pending = provider_qualification
                            .as_ref()
                            .is_some_and(|workflow| workflow.worker.is_some());
                        let vellum_unresolved =
                            vellum_durable_save.as_ref().is_some_and(|control| {
                                control.request_pending() || control.recovery_required()
                            }) || vellum_atomic_replace.as_ref().is_some_and(|control| {
                                control.disposition() == AtomicReplaceDisposition::OutcomeUnknown
                            }) || vellum_atomic_operation.is_some();
                        if chronos_unresolved
                            || provider_inspection_pending
                            || provider_edit_pending
                            || provider_qualification_pending
                            || vellum_unresolved
                            || agora_unresolved
                            || moira_unresolved
                            || daimon_unresolved
                            || hermes_unresolved
                            || aegis_unresolved
                            || operations.reply.is_some()
                        {
                            workflow.notice = "Restart refused: guarded work or topology drain is pending or outcome-unknown"
                                .into();
                            activity.push(
                                ActivityLevel::Warning,
                                "configuration",
                                workflow.notice.clone(),
                            );
                            continue;
                        }
                        match decide_exit(
                            vellum
                                .as_ref()
                                .map(ServiceEditor::aggregate_persistence)
                                .unwrap_or(
                                    vellum_sdk_client::workspace::PersistenceProjectionV1::Clean,
                                ),
                            false,
                        ) {
                            ExitDecision::Exit => {
                                restart_profile = Some(RestartProfileRequest {
                                    path,
                                    sha256: digest,
                                });
                                activity.push(
                                    ActivityLevel::Info,
                                    "configuration",
                                    "Preflighted process restart requested; draining owned topology",
                                );
                                break;
                            }
                            ExitDecision::ConfirmDiscard => {
                                workflow.notice = "Restart refused: save the dirty Vellum workset or explicitly exit and discard it first"
                                    .into();
                            }
                            ExitDecision::RefuseIndeterminate => {
                                workflow.notice = "Restart refused: Vellum persistence is indeterminate; recover the exact effect first"
                                    .into();
                            }
                        }
                    } else if let Some(character) = bounded_text_character(key) {
                        workflow.insert(&character.to_string());
                    }
                }
                Event::Key(_) if key_behavior == Some(KeyBehavior::DismissOverlay) => {
                    if overlay == WorkbenchOverlay::ProviderEdit
                        && provider_edit
                            .as_ref()
                            .is_some_and(|workflow| workflow.worker.is_some())
                    {
                        notice = "WAIT · Provider owner operation is pending · Next: remain here until the exact plan or receipt settles"
                            .into();
                    } else {
                        overlay = WorkbenchOverlay::None;
                    }
                }
                Event::Key(_) if key_behavior == Some(KeyBehavior::Suppress) => {}
                Event::Key(_) if key_behavior == Some(KeyBehavior::OpenCommands) => {
                    palette.reset();
                    overlay = WorkbenchOverlay::Commands;
                }
                Event::Mouse(mouse)
                    if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) =>
                {
                    if let Some(surface) =
                        surface_tab_at(&shell, terminal.size()?.width, mouse.column, mouse.row)
                        && shell.activate(surface)
                    {
                        vellum_open_prompt = None;
                        vellum_find_prompt = None;
                        confirming_discard = false;
                        notice = format!("surface moved to {} by mouse", surface.label());
                    }
                }
                Event::Mouse(mouse)
                    if mouse.row <= 2 && matches!(mouse.kind, MouseEventKind::ScrollUp) =>
                {
                    shell.previous();
                    vellum_open_prompt = None;
                    vellum_find_prompt = None;
                    confirming_discard = false;
                    notice = format!("surface moved to {} by mouse", shell.active().label());
                }
                Event::Mouse(mouse)
                    if mouse.row <= 2 && matches!(mouse.kind, MouseEventKind::ScrollDown) =>
                {
                    shell.next();
                    vellum_open_prompt = None;
                    vellum_find_prompt = None;
                    confirming_discard = false;
                    notice = format!("surface moved to {} by mouse", shell.active().label());
                }
                Event::Resize(..) => continue,
                Event::Paste(text) if vellum_open_prompt.is_some() => {
                    if let Some(prompt) = &mut vellum_open_prompt {
                        notice = match prompt.insert(&text) {
                            Ok(()) => "Vellum open path updated".into(),
                            Err(error) => error.to_string(),
                        };
                    }
                }
                Event::Paste(text) if vellum_find_prompt.is_some() => {
                    if let Some(prompt) = &mut vellum_find_prompt {
                        notice = match prompt.insert(&text) {
                            Ok(()) => "Vellum find query updated".into(),
                            Err(error) => error.to_string(),
                        };
                    }
                }
                Event::Paste(text)
                    if vellum_surface_active(shell.active(), focus)
                        && vellum_atomic_replace.is_some() =>
                {
                    let control = vellum_atomic_replace.as_mut().expect("matched above");
                    notice = if control.insert_text(&text, true) {
                        "Pasted bounded literal text into Vellum atomic replace".into()
                    } else {
                        control.notice().into()
                    };
                }
                Event::Paste(text)
                    if shell.active() == Surface::Daimon && daimon_prompt_composer.is_some() =>
                {
                    if let Some(composer) = &mut daimon_prompt_composer {
                        composer.insert_text(&text);
                        notice = "Pasted bounded text into the Daimon prompt editor".into();
                    }
                }
                Event::Paste(text)
                    if shell.active() == Surface::Agora && agora_message.is_some() =>
                {
                    if let Some(control) = &mut agora_message {
                        control.insert_text(&text);
                        notice = "Pasted bounded text into the active Agora message field".into();
                    }
                }
                Event::Paste(text)
                    if shell.active() == Surface::Agora && agora_create.is_some() =>
                {
                    if let Some(control) = &mut agora_create {
                        for character in text.chars() {
                            control.update(agora_tui::CreateRoomInput::Character(character), true);
                        }
                        notice = "Pasted bounded text into the active Agora room field".into();
                    }
                }
                Event::Paste(text)
                    if shell.active() == Surface::Aegis && aegis_managed.is_some() =>
                {
                    if let Some(control) = &mut aegis_managed {
                        for character in text.chars().filter(|value| !value.is_control()) {
                            control
                                .update(aegis_tui::ManagedAuthorityInput::Insert(character), true);
                        }
                    }
                }
                Event::Paste(text) if vellum_surface_active(shell.active(), focus) => {
                    if let Some(vellum) = &mut vellum {
                        let started = Instant::now();
                        let result = vellum.apply(ServiceAction::Insert(text));
                        if vellum.take_identity_renewal() {
                            instrumentation.record(
                                "vellum.client.renew-identity",
                                "vellum",
                                ActivityLevel::Success,
                                started.elapsed(),
                            );
                            activity.push(
                                ActivityLevel::Success,
                                "vellum",
                                "Vellum replay identity renewed before request capacity exhaustion",
                            );
                        }
                        let level = if result.is_ok() {
                            ActivityLevel::Success
                        } else {
                            ActivityLevel::Warning
                        };
                        instrumentation.record(
                            "vellum.edit.insert",
                            "vellum",
                            level,
                            started.elapsed(),
                        );
                        notice = result.map_or_else(
                            |error| error.to_string(),
                            |()| "Vellum edit accepted".into(),
                        );
                        confirming_discard = false;
                    } else {
                        notice = "Vellum service is not configured".into();
                    }
                }
                Event::Paste(_) => {
                    notice = "Paste is available only in the focused Vellum pane".into();
                }
                Event::Key(key)
                    if vellum_open_prompt.is_some()
                        && key_behavior == Some(KeyBehavior::SurfaceLocal) =>
                {
                    let mut prompt = vellum_open_prompt.take().expect("matched above");
                    match prompt.key(key) {
                        Ok(PromptInput::Pending) => vellum_open_prompt = Some(prompt),
                        Ok(PromptInput::Cancel) => {
                            notice = "Vellum open cancelled".into();
                        }
                        Ok(PromptInput::Submit(locator)) if locator.is_empty() => {
                            notice = "Vellum open refused: enter a file locator".into();
                            vellum_open_prompt = Some(prompt);
                        }
                        Ok(PromptInput::Submit(locator)) => {
                            let started = Instant::now();
                            let result = vellum
                                .as_mut()
                                .expect("open prompt exists only with Vellum")
                                .admit_file(locator);
                            let level = if result.is_ok() {
                                ActivityLevel::Success
                            } else {
                                ActivityLevel::Warning
                            };
                            instrumentation.record(
                                "vellum.document.open",
                                "vellum",
                                level,
                                started.elapsed(),
                            );
                            notice = result.map_or_else(
                                |error| error.to_string(),
                                |()| "Vellum admitted and opened the file".into(),
                            );
                            activity.push(level, "vellum", notice.clone());
                        }
                        Err(error) => {
                            notice = error.to_string();
                            vellum_open_prompt = Some(prompt);
                        }
                    }
                }
                Event::Key(key)
                    if vellum_find_prompt.is_some()
                        && key_behavior == Some(KeyBehavior::SurfaceLocal) =>
                {
                    let mut prompt = vellum_find_prompt.take().expect("matched above");
                    match prompt.key(key) {
                        Ok(PromptInput::Pending) => vellum_find_prompt = Some(prompt),
                        Ok(PromptInput::Cancel) => {
                            notice = "Vellum find cancelled".into();
                        }
                        Ok(PromptInput::Submit(query)) if query.is_empty() => {
                            notice = "Vellum find refused: enter a literal query".into();
                            vellum_find_prompt = Some(prompt);
                        }
                        Ok(PromptInput::Submit(query)) => {
                            let started = Instant::now();
                            let result = vellum
                                .as_mut()
                                .expect("find prompt exists only with Vellum")
                                .find_literal(&query);
                            let level = if matches!(result, Ok(true)) {
                                ActivityLevel::Success
                            } else {
                                ActivityLevel::Warning
                            };
                            instrumentation.record(
                                "vellum.document.find",
                                "vellum",
                                level,
                                started.elapsed(),
                            );
                            notice = match result {
                                Ok(true) => "Vellum literal match selected".into(),
                                Ok(false) => {
                                    "Vellum find found no literal match in the current document"
                                        .into()
                                }
                                Err(error) => error.to_string(),
                            };
                            activity.push(level, "vellum", notice.clone());
                        }
                        Err(error) => {
                            notice = error.to_string();
                            vellum_find_prompt = Some(prompt);
                        }
                    }
                }
                Event::Key(_) if matches!(key_behavior, Some(KeyBehavior::Navigate(_))) => {
                    let Some(KeyBehavior::Navigate(input)) = key_behavior else {
                        unreachable!("guarded above")
                    };
                    BehaviorFacade::navigate(&mut shell, input);
                    vellum_open_prompt = None;
                    vellum_find_prompt = None;
                    confirming_discard = false;
                    notice = format!("surface moved to {}", shell.active().label());
                }
                Event::Key(key)
                    if shell.active() == Surface::Agora
                        && key_behavior != Some(KeyBehavior::SafeExit) =>
                {
                    if agora_message.is_some() && is_key(key, KeyCode::Esc, KeyModifiers::NONE) {
                        if agora_message.as_ref().is_some_and(|control| {
                            control.request_pending() || control.recovery_required()
                        }) {
                            notice = "Agora message is pending or outcome-unknown; wait, or press r to replay the exact retained request"
                                .into();
                            activity.push(ActivityLevel::Warning, "agora", notice.clone());
                        } else {
                            agora_message = None;
                            last_agora_message_notice.clear();
                            notice =
                                "Closed Agora message composer without another submission".into();
                            activity.push(ActivityLevel::Info, "agora", notice.clone());
                        }
                    } else if let Some(control) = &mut agora_message {
                        let input = if is_key(key, KeyCode::Char('r'), KeyModifiers::NONE)
                            && control.recovery_required()
                        {
                            Some(agora_tui::ComposeMessageInput::RetryExact)
                        } else {
                            agora_tui::translate_compose_message_key(key)
                        };
                        if let Some(input) = input {
                            let pending_before = control.request_pending();
                            control.update(input, true);
                            if !pending_before
                                && control.request_pending()
                                && agora_message_operation.is_none()
                            {
                                agora_message_operation = instrumentation
                                    .begin_peer_operation("agora.message.send", "agora");
                            }
                        }
                    } else if agora_create.is_some()
                        && is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    {
                        if agora_create.as_ref().is_some_and(|control| {
                            control.request_pending() || control.recovery_required()
                        }) {
                            notice = "Agora room creation is pending or outcome-unknown; wait, or press r to replay the exact retained request"
                                .into();
                            activity.push(ActivityLevel::Warning, "agora", notice.clone());
                        } else {
                            agora_create = None;
                            last_agora_create_notice.clear();
                            notice = "Closed Agora room form without another submission".into();
                            activity.push(ActivityLevel::Info, "agora", notice.clone());
                        }
                    } else if let Some(control) = &mut agora_create {
                        if let Some(input) = agora_tui::translate_create_room_key(key) {
                            control.update(input, true);
                            if control.request_pending() && agora_operation.is_none() {
                                agora_operation = instrumentation
                                    .begin_peer_operation("agora.room.create", "agora");
                            }
                        }
                    } else if let Some(agora) = &mut agora {
                        if let Some(input) = agora_tui::translate_key(key) {
                            let started = Instant::now();
                            let pending_before = agora.request_pending();
                            let action = match input {
                                agora_tui::Input::Refresh => Some("view.refresh"),
                                agora_tui::Input::Open => Some("agora.room.open"),
                                agora_tui::Input::Back => Some("agora.room.back"),
                                _ => None,
                            };
                            if agora.update(input, true) == agora_tui::Lifecycle::CloseRequested {
                                notice = "Use Ctrl-Q to exit the complete workbench".into();
                            }
                            if let Some(action) = action {
                                begin_refresh(
                                    !pending_before && agora.request_pending(),
                                    action,
                                    "agora",
                                    &mut refresh_operations.agora,
                                    &mut instrumentation,
                                    started,
                                );
                            }
                        }
                    } else {
                        notice = Surface::Agora
                            .gap()
                            .unwrap_or("Agora presentation is unavailable")
                            .into();
                    }
                }
                Event::Key(key)
                    if shell.active() == Surface::Moira
                        && key_behavior != Some(KeyBehavior::SafeExit) =>
                {
                    if moira_open.is_some() && is_key(key, KeyCode::Esc, KeyModifiers::NONE) {
                        let control = moira_open.as_mut().expect("checked above");
                        if control.armed() {
                            control.update(moira_tui::OpenControlInput::Back, true);
                            notice = "Moira agent-session open disarmed".into();
                        } else if control.request_pending() || control.recovery_required() {
                            notice = "Moira agent-session open is pending or outcome-unknown; refresh owner state before closing"
                                .into();
                            activity.push(ActivityLevel::Warning, "moira", notice.clone());
                        } else {
                            moira_open = None;
                            last_moira_open_notice.clear();
                            notice = "Closed agent-session open review without another preparation"
                                .into();
                        }
                    } else if let Some(control) = &mut moira_open {
                        let input = if is_key(key, KeyCode::Char('r'), KeyModifiers::NONE) {
                            Some(moira_tui::OpenControlInput::Refresh)
                        } else if is_key(key, KeyCode::Char('c'), KeyModifiers::NONE) {
                            Some(moira_tui::OpenControlInput::Arm)
                        } else if is_key(key, KeyCode::Enter, KeyModifiers::NONE) {
                            Some(moira_tui::OpenControlInput::Confirm)
                        } else {
                            None
                        };
                        if let Some(input) = input {
                            control.update(input, true);
                            notice = match input {
                                moira_tui::OpenControlInput::Refresh => {
                                    "Moira open-state inspection queued"
                                }
                                moira_tui::OpenControlInput::Arm => {
                                    "Moira agent-session open arm requested"
                                }
                                moira_tui::OpenControlInput::Confirm => {
                                    "Moira agent-session open preparation submitted once"
                                }
                                moira_tui::OpenControlInput::Back
                                | moira_tui::OpenControlInput::Close => unreachable!(),
                            }
                            .into();
                            activity.push(ActivityLevel::Info, "moira", notice.clone());
                        }
                    } else if moira_reconciliation.is_some()
                        && is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    {
                        if moira_reconciliation.as_ref().is_some_and(|control| {
                            control.request_pending() || control.recovery_required()
                        }) {
                            notice = "Moira reconciliation is pending or fenced; wait, or press r to recover the exact receipt before closing"
                                .into();
                            activity.push(ActivityLevel::Warning, "moira", notice.clone());
                        } else {
                            moira_reconciliation = None;
                            last_moira_reconciliation_notice.clear();
                            active_moira_action = None;
                            notice =
                                "Closed reconciliation sheet without submitting another effect"
                                    .into();
                            activity.push(ActivityLevel::Info, "moira", notice.clone());
                        }
                    } else if let Some(reconciliation) = &mut moira_reconciliation {
                        if is_key(key, KeyCode::Up, KeyModifiers::NONE) {
                            reconciliation.move_selection(false);
                        } else if is_key(key, KeyCode::Down, KeyModifiers::NONE) {
                            reconciliation.move_selection(true);
                        } else if let Some(input) =
                            reconciliation_input(key, reconciliation.editing_logical_time())
                        {
                            reconciliation.update(input, true);
                            if reconciliation.request_pending() && moira_operation.is_none() {
                                moira_operation = instrumentation.begin_peer_operation(
                                    active_moira_action
                                        .unwrap_or("moira.reconciliation.keep-uncertain"),
                                    "moira",
                                );
                            }
                            notice = match input {
                                moira_tui::ReconciliationBatchInput::Refresh => {
                                    "Moira reconciliation recovery refresh queued"
                                }
                                moira_tui::ReconciliationBatchInput::Arm => {
                                    "Moira reconciliation arm requested"
                                }
                                moira_tui::ReconciliationBatchInput::Confirm => {
                                    "Moira reconciliation child sequence submitted once"
                                }
                                moira_tui::ReconciliationBatchInput::Digit(_)
                                | moira_tui::ReconciliationBatchInput::Backspace => {
                                    "Moira logical time edited"
                                }
                                moira_tui::ReconciliationBatchInput::Prepare => {
                                    "Moira time-bearing proposals prepared for owner inspection"
                                }
                                moira_tui::ReconciliationBatchInput::Back
                                | moira_tui::ReconciliationBatchInput::Close => {
                                    unreachable!("host maps only explicit control inputs")
                                }
                            }
                            .into();
                            activity.push(ActivityLevel::Info, "moira", notice.clone());
                        }
                    } else if let Some(moira) = &mut moira {
                        if let Some(input) = moira_tui::translate_key(key) {
                            if input == moira_tui::Input::Refresh
                                && let Some(participation) = &mut moira_participation
                            {
                                participation.update(moira_tui::Input::Refresh, true);
                            }
                            let started = Instant::now();
                            let pending_before = moira.request_pending();
                            let action = match input {
                                moira_tui::Input::Refresh => Some("view.refresh"),
                                moira_tui::Input::Open => Some("moira.saga.open"),
                                moira_tui::Input::Back => Some("moira.saga.back"),
                                _ => None,
                            };
                            if moira.update(input, true) == moira_tui::Lifecycle::CloseRequested {
                                notice = "Use Ctrl-Q to exit the complete workbench".into();
                            }
                            if let Some(action) = action {
                                begin_refresh(
                                    !pending_before && moira.request_pending(),
                                    action,
                                    "moira",
                                    &mut refresh_operations.moira,
                                    &mut instrumentation,
                                    started,
                                );
                            }
                        }
                    } else if let Some(participation) = &mut moira_participation {
                        if let Some(input) = moira_tui::translate_key(key)
                            && participation.update(input, true)
                                == moira_tui::Lifecycle::CloseRequested
                        {
                            notice = "Use Ctrl-Q to exit the complete workbench".into();
                        }
                    } else {
                        notice = Surface::Moira
                            .gap()
                            .unwrap_or("Moira presentation is unavailable")
                            .into();
                    }
                }
                Event::Key(key)
                    if shell.active() == Surface::Daimon
                        && key_behavior != Some(KeyBehavior::SafeExit) =>
                {
                    if daimon_permission_composer.is_some()
                        && is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    {
                        let outcome = daimon_permission_composer
                            .as_mut()
                            .expect("checked above")
                            .update(daimon_tui::AgentPermissionInput::Close, true);
                        if outcome == daimon_tui::AgentPermissionOutcome::CloseRequested {
                            daimon_permission_composer = None;
                            notice =
                                "Closed Daimon permission review; no decision was created".into();
                            activity.push(ActivityLevel::Info, "daimon", notice.clone());
                        }
                    } else if let Some(composer) = &mut daimon_permission_composer {
                        if let Some(input) = daimon_permission_input(key) {
                            match composer.update(input, true) {
                                daimon_tui::AgentPermissionOutcome::None => {
                                    notice = composer.notice().into();
                                }
                                daimon_tui::AgentPermissionOutcome::CloseRequested => {
                                    unreachable!("close is handled explicitly")
                                }
                                daimon_tui::AgentPermissionOutcome::Submit(selections) => {
                                    let Some(policy) = daimon_execution_policy.as_ref() else {
                                        notice =
                                            "Daimon execution policy became unavailable".into();
                                        activity.push(
                                            ActivityLevel::Warning,
                                            "daimon",
                                            notice.clone(),
                                        );
                                        continue;
                                    };
                                    let requests =
                                        match daimon_permission_requests(selections, policy) {
                                            Ok(requests) => requests,
                                            Err(error) => {
                                                notice = error;
                                                activity.push(
                                                    ActivityLevel::Warning,
                                                    "daimon",
                                                    notice.clone(),
                                                );
                                                continue;
                                            }
                                        };
                                    let socket = daimon_agent_control_socket
                                        .as_ref()
                                        .expect("permissions require the agent socket")
                                        .clone();
                                    daimon_control = match daimon_tui::AgentControlBatchComponent::new_with_connector(
                                        move || {
                                            daimon_client::unix::connect_agent(
                                                &socket,
                                                daimon_wire::LimitsV1::default(),
                                                daimon_wire::agent::AgentWireLimitsV1::default(),
                                                Duration::from_secs(5),
                                            )
                                        },
                                        requests,
                                    ) {
                                        Ok(control) => Some(control),
                                        Err(error) => {
                                            notice = error.into();
                                            activity.push(
                                                ActivityLevel::Warning,
                                                "daimon",
                                                notice.clone(),
                                            );
                                            continue;
                                        }
                                    };
                                    daimon_permission_composer = None;
                                    active_daimon_action = Some("daimon.session.permission");
                                    notice = "Exact Daimon permission-set proposal opened; c arms, Enter starts the bounded run".into();
                                    activity.push(ActivityLevel::Info, "daimon", notice.clone());
                                }
                            }
                        }
                    } else if daimon_prompt_composer.is_some()
                        && is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    {
                        let outcome = daimon_prompt_composer
                            .as_mut()
                            .expect("checked above")
                            .update(daimon_tui::AgentPromptInput::Close, true);
                        if outcome == daimon_tui::AgentPromptOutcome::CloseRequested {
                            daimon_prompt_composer = None;
                            notice = "Closed Daimon prompt composer; no request was created".into();
                            activity.push(ActivityLevel::Info, "daimon", notice.clone());
                        }
                    } else if let Some(composer) = &mut daimon_prompt_composer {
                        let reviewing = composer.reviewing();
                        if let Some(input) = daimon_prompt_input(key, reviewing) {
                            match composer.update(input, true) {
                                daimon_tui::AgentPromptOutcome::None => {
                                    notice = composer.notice().into();
                                }
                                daimon_tui::AgentPromptOutcome::CloseRequested => {
                                    unreachable!("close is handled explicitly")
                                }
                                daimon_tui::AgentPromptOutcome::Submit(prompt) => {
                                    let Some(agent) = daimon_agent.as_ref() else {
                                        notice = "Daimon session set became unavailable before prompt review"
                                            .into();
                                        activity.push(
                                            ActivityLevel::Warning,
                                            "daimon",
                                            notice.clone(),
                                        );
                                        continue;
                                    };
                                    let observations = agent.selected_observations();
                                    let Some(policy) = daimon_execution_policy.as_ref() else {
                                        notice =
                                            "Daimon prompt execution policy became unavailable"
                                                .into();
                                        activity.push(
                                            ActivityLevel::Warning,
                                            "daimon",
                                            notice.clone(),
                                        );
                                        continue;
                                    };
                                    let requests =
                                        match daimon_prompt_requests(&observations, policy, prompt)
                                        {
                                            Ok(requests) => requests,
                                            Err(error) => {
                                                notice = error;
                                                activity.push(
                                                    ActivityLevel::Warning,
                                                    "daimon",
                                                    notice.clone(),
                                                );
                                                continue;
                                            }
                                        };
                                    let socket = daimon_agent_control_socket
                                        .as_ref()
                                        .expect("prompting requires the agent socket")
                                        .clone();
                                    daimon_control = match daimon_tui::AgentControlBatchComponent::new_with_connector(
                                        move || {
                                            daimon_client::unix::connect_agent(
                                                &socket,
                                                daimon_wire::LimitsV1::default(),
                                                daimon_wire::agent::AgentWireLimitsV1::default(),
                                                Duration::from_secs(5),
                                            )
                                        },
                                        requests,
                                    ) {
                                        Ok(control) => Some(control),
                                        Err(error) => {
                                            notice = error.into();
                                            activity.push(
                                                ActivityLevel::Warning,
                                                "daimon",
                                                notice.clone(),
                                            );
                                            continue;
                                        }
                                    };
                                    daimon_prompt_composer = None;
                                    active_daimon_action = Some(
                                        if matches!(
                                            work_item_agent,
                                            WorkItemAgentJourney::ReviewingPrompt
                                        ) {
                                            work_item_agent = WorkItemAgentJourney::Invoking;
                                            "workitem.agent.prompt"
                                        } else {
                                            "daimon.session.prompt"
                                        },
                                    );
                                    notice = "Exact Daimon prompt-set proposal opened; c arms, Enter starts the bounded run".into();
                                    activity.push(ActivityLevel::Info, "daimon", notice.clone());
                                }
                            }
                        }
                    } else if daimon_control.is_some()
                        && is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    {
                        if daimon_control.as_ref().is_some_and(|control| {
                            control.request_pending() || control.recovery_required()
                        }) {
                            notice = "Daimon request is pending or outcome-unknown; wait, or press r to inspect the exact session before closing"
                                .into();
                            activity.push(ActivityLevel::Warning, "daimon", notice.clone());
                        } else {
                            daimon_control = None;
                            last_daimon_control_notice.clear();
                            active_daimon_action = None;
                            daimon_recovery_refresh = false;
                            daimon_unknown_evidence = false;
                            notice = "Closed Daimon control sheet; no additional request submitted"
                                .into();
                            activity.push(ActivityLevel::Info, "daimon", notice.clone());
                        }
                    } else if daimon_control
                        .as_ref()
                        .is_some_and(daimon_tui::AgentControlBatchComponent::recovery_required)
                        && is_key(key, KeyCode::Char('r'), KeyModifiers::NONE)
                    {
                        let sessions = daimon_control
                            .as_ref()
                            .expect("checked above")
                            .recovery_sessions();
                        let queued = daimon_agent
                            .as_mut()
                            .is_some_and(|agent| agent.refresh_sessions(&sessions));
                        if queued {
                            daimon_control
                                .as_mut()
                                .expect("checked above")
                                .recovery_inspection_started();
                            daimon_recovery_refresh = true;
                            notice = format!(
                                "Daimon exact recovery inspection queued for {} outcome-unknown session(s); no mutation will be retried",
                                sessions.len()
                            );
                            activity.push(ActivityLevel::Uncertain, "daimon", notice.clone());
                        } else {
                            notice = "Daimon recovery inspection could not be queued while another session refresh is pending"
                                .into();
                            activity.push(ActivityLevel::Warning, "daimon", notice.clone());
                        }
                    } else if let Some(control) = &mut daimon_control {
                        if let Some(input) = daimon_control_input(key) {
                            control.update(input, true);
                            if control.request_pending()
                                && daimon_operation.is_none()
                                && let Some(action) = active_daimon_action
                            {
                                daimon_operation =
                                    instrumentation.begin_peer_operation(action, "daimon");
                            }
                            notice = match input {
                                daimon_tui::AgentControlInput::Arm => "Daimon exact control armed",
                                daimon_tui::AgentControlInput::Confirm => {
                                    "Daimon exact control submitted once"
                                }
                                daimon_tui::AgentControlInput::Back
                                | daimon_tui::AgentControlInput::Close => {
                                    unreachable!("host maps only explicit control inputs")
                                }
                            }
                            .into();
                            activity.push(ActivityLevel::Info, "daimon", notice.clone());
                        }
                    } else if let Some(daimon) = &mut daimon_agent {
                        if let Some(input) = daimon_agent_set_input(key) {
                            let started = Instant::now();
                            let pending_before = daimon.request_pending();
                            if daimon.update(input, true) == daimon_tui::Lifecycle::CloseRequested {
                                notice = "Use Ctrl-Q to exit the complete workbench".into();
                            }
                            begin_refresh(
                                matches!(
                                    input,
                                    daimon_tui::AgentSetInput::RefreshSelected
                                        | daimon_tui::AgentSetInput::RefreshAll
                                ) && !pending_before
                                    && daimon.request_pending(),
                                "view.refresh",
                                "daimon",
                                &mut refresh_operations.daimon,
                                &mut instrumentation,
                                started,
                            );
                        }
                    } else if let Some(daimon) = &mut daimon {
                        if let Some(input) = daimon_tui::translate_key(key) {
                            let started = Instant::now();
                            let pending_before = daimon.request_pending();
                            if daimon.update(input, true) == daimon_tui::Lifecycle::CloseRequested {
                                notice = "Use Ctrl-Q to exit the complete workbench".into();
                            }
                            begin_refresh(
                                input == daimon_tui::Input::Refresh
                                    && !pending_before
                                    && daimon.request_pending(),
                                "view.refresh",
                                "daimon",
                                &mut refresh_operations.daimon,
                                &mut instrumentation,
                                started,
                            );
                        }
                    } else {
                        notice = Surface::Daimon
                            .gap()
                            .unwrap_or("Daimon presentation is unavailable")
                            .into();
                    }
                }
                Event::Key(key)
                    if shell.active() == Surface::Hermes
                        && key_behavior != Some(KeyBehavior::SafeExit) =>
                {
                    if hermes_compaction.is_some() && is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    {
                        if hermes_compaction.as_ref().is_some_and(|control| {
                            matches!(
                                control.disposition(),
                                hermes_tui::CompactionDisposition::Applied { .. }
                            )
                        }) && (hermes_compaction_post_receipt_refresh
                            || hermes_compaction_operation.is_some())
                        {
                            notice = "Hermes compaction receipt is still settling to refreshed stream state; wait before closing"
                                .into();
                            activity.push(ActivityLevel::Warning, "hermes", notice.clone());
                            continue;
                        }
                        let lifecycle = hermes_compaction
                            .as_mut()
                            .expect("checked above")
                            .update(hermes_tui::CompactionInput::Close, true);
                        if lifecycle == hermes_tui::Lifecycle::CloseRequested {
                            hermes_compaction = None;
                            last_hermes_compaction_notice.clear();
                            notice =
                                "Closed Hermes compaction sheet without another submission".into();
                        } else {
                            notice = "Hermes compaction is pending or outcome-unknown; wait, or press r to inspect exact stream state"
                                .into();
                            activity.push(ActivityLevel::Warning, "hermes", notice.clone());
                        }
                    } else if hermes_compaction.as_ref().is_some_and(|control| {
                        control.disposition() == hermes_tui::CompactionDisposition::OutcomeUnknown
                    }) && is_key(key, KeyCode::Char('r'), KeyModifiers::NONE)
                    {
                        if let Some(hermes) = &mut hermes {
                            let started = Instant::now();
                            let pending_before = hermes.request_pending();
                            hermes.update(hermes_tui::Input::Refresh, true);
                            begin_refresh(
                                !pending_before && hermes.request_pending(),
                                "ledger.compact.inspect",
                                "hermes",
                                &mut refresh_operations.hermes,
                                &mut instrumentation,
                                started,
                            );
                        }
                        hermes_compaction = None;
                        last_hermes_compaction_notice.clear();
                        notice = "Hermes compaction outcome remains unknown; exact stream inspection queued without retry"
                            .into();
                        activity.push(ActivityLevel::Uncertain, "hermes", notice.clone());
                    } else if let Some(control) = &mut hermes_compaction {
                        if let Some(input) = hermes_tui::translate_compaction_key(key) {
                            control.update(input, true);
                            if control.disposition() == hermes_tui::CompactionDisposition::Pending
                                && hermes_compaction_operation.is_none()
                            {
                                hermes_compaction_operation = instrumentation
                                    .begin_peer_operation("ledger.compact.v1", "hermes");
                            }
                        }
                    } else if hermes_disclosure.is_some()
                        && is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    {
                        let lifecycle = hermes_disclosure
                            .as_mut()
                            .expect("checked above")
                            .update(hermes_tui::DisclosureInput::Close, true);
                        if lifecycle == hermes_tui::Lifecycle::CloseRequested {
                            hermes_disclosure = None;
                            last_hermes_disclosure_notice.clear();
                            notice =
                                "Closed Hermes event disclosure; disclosed content was discarded"
                                    .into();
                        } else {
                            notice =
                                "Hermes disclosure read is pending; wait before closing".into();
                            activity.push(ActivityLevel::Warning, "hermes", notice.clone());
                        }
                    } else if let Some(control) = &mut hermes_disclosure {
                        if let Some(input) = hermes_tui::translate_disclosure_key(key) {
                            control.update(input, true);
                            if control.disposition() == hermes_tui::DisclosureDisposition::Pending
                                && hermes_disclosure_operation.is_none()
                            {
                                hermes_disclosure_operation = instrumentation
                                    .begin_peer_operation("ledger.events.disclose.v1", "hermes");
                            }
                        }
                    } else if hermes_acknowledge.is_some()
                        && is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    {
                        if hermes_acknowledge.as_ref().is_some_and(|control| {
                            matches!(
                                control.disposition(),
                                hermes_tui::AcknowledgeDisposition::Applied { .. }
                            )
                        }) && (hermes_post_receipt_refresh || hermes_operation.is_some())
                        {
                            notice = "Hermes acknowledgement receipt is still settling to a refreshed cursor; wait before closing"
                                .into();
                            activity.push(ActivityLevel::Warning, "hermes", notice.clone());
                            continue;
                        }
                        let lifecycle = hermes_acknowledge
                            .as_mut()
                            .expect("checked above")
                            .update(hermes_tui::AcknowledgeInput::Close, true);
                        if lifecycle == hermes_tui::Lifecycle::CloseRequested {
                            hermes_acknowledge = None;
                            last_hermes_acknowledge_notice.clear();
                            notice =
                                "Closed Hermes acknowledgement sheet without another submission"
                                    .into();
                        } else {
                            notice = "Hermes acknowledgement is pending or outcome-unknown; wait, or press r to inspect the exact cursor"
                                .into();
                            activity.push(ActivityLevel::Warning, "hermes", notice.clone());
                        }
                    } else if hermes_acknowledge.as_ref().is_some_and(|control| {
                        control.disposition() == hermes_tui::AcknowledgeDisposition::OutcomeUnknown
                    }) && is_key(key, KeyCode::Char('r'), KeyModifiers::NONE)
                    {
                        if let Some(hermes) = &mut hermes {
                            let started = Instant::now();
                            let pending_before = hermes.request_pending();
                            hermes.update(hermes_tui::Input::Refresh, true);
                            begin_refresh(
                                !pending_before && hermes.request_pending(),
                                "ledger.acknowledge.inspect",
                                "hermes",
                                &mut refresh_operations.hermes,
                                &mut instrumentation,
                                started,
                            );
                        }
                        hermes_acknowledge = None;
                        last_hermes_acknowledge_notice.clear();
                        notice = "Hermes acknowledgement outcome remains unknown; exact cursor inspection queued without retry"
                            .into();
                        activity.push(ActivityLevel::Uncertain, "hermes", notice.clone());
                    } else if let Some(control) = &mut hermes_acknowledge {
                        if let Some(input) = hermes_tui::translate_acknowledge_key(key) {
                            control.update(input, true);
                            if control.disposition() == hermes_tui::AcknowledgeDisposition::Pending
                                && hermes_operation.is_none()
                            {
                                hermes_operation = instrumentation
                                    .begin_peer_operation("ledger.acknowledge.v1", "hermes");
                            }
                        }
                    } else if let Some(hermes) = &mut hermes {
                        if let Some(input) = hermes_tui::translate_key(key) {
                            let started = Instant::now();
                            let pending_before = hermes.request_pending();
                            if hermes.update(input, true) == hermes_tui::Lifecycle::CloseRequested {
                                notice = "Use Ctrl-Q to exit the complete workbench".into();
                            }
                            begin_refresh(
                                input == hermes_tui::Input::Refresh
                                    && !pending_before
                                    && hermes.request_pending(),
                                "view.refresh",
                                "hermes",
                                &mut refresh_operations.hermes,
                                &mut instrumentation,
                                started,
                            );
                        }
                    } else {
                        notice = Surface::Hermes
                            .gap()
                            .unwrap_or("Hermes presentation is unavailable")
                            .into();
                    }
                }
                Event::Key(key)
                    if shell.active() == Surface::Aegis
                        && key_behavior != Some(KeyBehavior::SafeExit) =>
                {
                    if let Some(control) = &mut aegis_managed {
                        let before = control.disposition();
                        let input = if is_key(key, KeyCode::Esc, KeyModifiers::NONE) {
                            Some(aegis_tui::ManagedAuthorityInput::Close)
                        } else if is_key(key, KeyCode::Backspace, KeyModifiers::NONE) {
                            Some(aegis_tui::ManagedAuthorityInput::Backspace)
                        } else if is_key(key, KeyCode::Enter, KeyModifiers::NONE) {
                            Some(
                                if before == aegis_tui::ManagedAuthorityDisposition::Editing {
                                    aegis_tui::ManagedAuthorityInput::Prepare
                                } else {
                                    aegis_tui::ManagedAuthorityInput::Confirm
                                },
                            )
                        } else if is_key(key, KeyCode::Char('c'), KeyModifiers::NONE)
                            && before == aegis_tui::ManagedAuthorityDisposition::Proposed
                        {
                            Some(aegis_tui::ManagedAuthorityInput::Arm)
                        } else if is_key(key, KeyCode::Char('e'), KeyModifiers::NONE)
                            && before != aegis_tui::ManagedAuthorityDisposition::Editing
                        {
                            Some(aegis_tui::ManagedAuthorityInput::Edit)
                        } else if key.kind == KeyEventKind::Press
                            && key.modifiers.intersection(KeyModifiers::CONTROL).is_empty()
                            && let KeyCode::Char(character) = key.code
                        {
                            Some(aegis_tui::ManagedAuthorityInput::Insert(character))
                        } else {
                            None
                        };
                        if let Some(input) = input {
                            let lifecycle = control.update(input, true);
                            if control.disposition()
                                == aegis_tui::ManagedAuthorityDisposition::Pending
                                && before != aegis_tui::ManagedAuthorityDisposition::Pending
                                && aegis_operation.is_none()
                            {
                                aegis_operation = instrumentation
                                    .begin_peer_operation("aegis.authority.manage", "aegis");
                            }
                            if lifecycle == aegis_tui::Lifecycle::CloseRequested {
                                aegis_managed = None;
                                last_aegis_managed_notice.clear();
                                notice =
                                    "Closed Aegis managed session; private grants destroyed".into();
                                activity.push(ActivityLevel::Info, "aegis", notice.clone());
                            }
                        }
                    } else if aegis_revoke.is_some()
                        && is_key(key, KeyCode::Esc, KeyModifiers::NONE)
                    {
                        let lifecycle = aegis_revoke
                            .as_mut()
                            .expect("checked above")
                            .update(aegis_tui::RevokeEpochInput::Close, true);
                        if lifecycle == aegis_tui::Lifecycle::CloseRequested {
                            aegis_revoke = None;
                            last_aegis_revoke_notice.clear();
                            notice =
                                "Closed Aegis revocation sheet without another submission".into();
                        } else {
                            notice = "Aegis revocation is pending or outcome-unknown; wait, or press r to inspect current authority state"
                                .into();
                            activity.push(ActivityLevel::Warning, "aegis", notice.clone());
                        }
                    } else if aegis_revoke.as_ref().is_some_and(|control| {
                        control.disposition() == aegis_tui::RevokeEpochDisposition::OutcomeUnknown
                    }) && is_key(key, KeyCode::Char('r'), KeyModifiers::NONE)
                    {
                        if let Some(aegis) = &mut aegis {
                            let started = Instant::now();
                            let pending_before = aegis.request_pending();
                            aegis.update(aegis_tui::Input::Refresh, true);
                            begin_refresh(
                                !pending_before && aegis.request_pending(),
                                "aegis.epoch.revoke.inspect",
                                "aegis",
                                &mut refresh_operations.aegis,
                                &mut instrumentation,
                                started,
                            );
                        }
                        aegis_revoke = None;
                        last_aegis_revoke_notice.clear();
                        notice = "Aegis revocation outcome remains unknown; current authority inspection queued without retry"
                            .into();
                        activity.push(ActivityLevel::Uncertain, "aegis", notice.clone());
                    } else if let Some(control) = &mut aegis_revoke {
                        if let Some(input) = aegis_tui::translate_revoke_epoch_key(key) {
                            control.update(input, true);
                            if control.disposition() == aegis_tui::RevokeEpochDisposition::Pending
                                && aegis_operation.is_none()
                            {
                                aegis_operation = instrumentation
                                    .begin_peer_operation("aegis.epoch.revoke", "aegis");
                            }
                        }
                    } else if let Some(aegis) = &mut aegis {
                        if let Some(input) = aegis_tui::translate_key(key) {
                            let started = Instant::now();
                            let pending_before = aegis.request_pending();
                            let action = match input {
                                aegis_tui::Input::Refresh => Some("view.refresh"),
                                aegis_tui::Input::NextPage => Some("aegis.page.next"),
                                aegis_tui::Input::PreviousPage => Some("aegis.page.previous"),
                                aegis_tui::Input::Close => None,
                            };
                            if aegis.update(input, true) == aegis_tui::Lifecycle::CloseRequested {
                                notice = "Use Ctrl-Q to exit the complete workbench".into();
                            }
                            if let Some(action) = action {
                                begin_refresh(
                                    !pending_before && aegis.request_pending(),
                                    action,
                                    "aegis",
                                    &mut refresh_operations.aegis,
                                    &mut instrumentation,
                                    started,
                                );
                            }
                        }
                    } else {
                        notice = Surface::Aegis
                            .gap()
                            .unwrap_or("Aegis presentation is unavailable")
                            .into();
                    }
                }
                Event::Key(key)
                    if shell.active() == Surface::Operations
                        && key_behavior != Some(KeyBehavior::SafeExit) =>
                {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::NONE) => {
                            operations.arm();
                            notice = operations.notice.clone();
                            activity.push(ActivityLevel::Warning, "operations", notice.clone());
                        }
                        (KeyCode::Enter, KeyModifiers::NONE) => {
                            let started = Instant::now();
                            let hosted_peers = operations.hosted_peer_ids();
                            if operations.confirm() {
                                drain_operations = begin_drain_operations(
                                    hosted_peers,
                                    &mut instrumentation,
                                    started,
                                );
                                notice = operations.notice.clone();
                                activity.push(ActivityLevel::Info, "operations", notice.clone());
                            } else {
                                notice = operations.notice.clone();
                            }
                        }
                        (KeyCode::Esc, KeyModifiers::NONE) => {
                            operations.disarm();
                            notice = operations.notice.clone();
                        }
                        _ => {
                            notice =
                                "Operations uses c to arm, Enter to confirm, and Esc to disarm"
                                    .into();
                        }
                    }
                }
                Event::Key(key)
                    if shell.active() == Surface::Tellus
                        && key_behavior != Some(KeyBehavior::SafeExit) =>
                {
                    if let Some(tellus) = &mut tellus {
                        if let Some(input) = tellus::translate_key(key) {
                            let started = Instant::now();
                            let pending_before = tellus.request_pending();
                            if tellus.update(input, true) == tellus::Lifecycle::CloseRequested {
                                notice = "Use Ctrl-Q to exit the complete workbench".into();
                            }
                            begin_refresh(
                                input == tellus::Input::Refresh
                                    && !pending_before
                                    && tellus.request_pending(),
                                "view.refresh",
                                "tellus",
                                &mut refresh_operations.tellus,
                                &mut instrumentation,
                                started,
                            );
                        }
                    } else {
                        notice = Surface::Tellus
                            .gap()
                            .unwrap_or("Tellus grounding presentation is unavailable")
                            .into();
                    }
                }
                Event::Key(key)
                    if vellum_surface_active(shell.active(), focus)
                        && vellum_atomic_replace.is_some()
                        && key_behavior != Some(KeyBehavior::SafeExit) =>
                {
                    let control = vellum_atomic_replace.as_mut().expect("matched above");
                    let disposition_before = control.disposition();
                    let input = if is_key(key, KeyCode::Esc, KeyModifiers::NONE) {
                        if matches!(
                            disposition_before,
                            AtomicReplaceDisposition::Editing
                                | AtomicReplaceDisposition::OutcomeUnknown
                        ) {
                            Some(AtomicReplaceInput::Close)
                        } else {
                            Some(AtomicReplaceInput::Back)
                        }
                    } else if is_key(key, KeyCode::Tab, KeyModifiers::NONE) {
                        Some(AtomicReplaceInput::NextField)
                    } else if is_key(key, KeyCode::Backspace, KeyModifiers::NONE) {
                        Some(AtomicReplaceInput::Backspace)
                    } else if is_key(key, KeyCode::Enter, KeyModifiers::NONE) {
                        Some(if disposition_before == AtomicReplaceDisposition::Editing {
                            AtomicReplaceInput::Preview
                        } else {
                            AtomicReplaceInput::Confirm
                        })
                    } else if is_key(key, KeyCode::Char('c'), KeyModifiers::NONE)
                        && disposition_before != AtomicReplaceDisposition::Editing
                    {
                        Some(AtomicReplaceInput::Arm)
                    } else if is_key(key, KeyCode::Char('r'), KeyModifiers::NONE)
                        && disposition_before == AtomicReplaceDisposition::OutcomeUnknown
                    {
                        Some(AtomicReplaceInput::Recover)
                    } else if is_key(key, KeyCode::Char('q'), KeyModifiers::NONE)
                        && matches!(
                            disposition_before,
                            AtomicReplaceDisposition::Applied | AtomicReplaceDisposition::Refused
                        )
                    {
                        Some(AtomicReplaceInput::Close)
                    } else if let Some(character) = bounded_text_character(key) {
                        Some(AtomicReplaceInput::Character(character))
                    } else {
                        None
                    };
                    if let Some(input) = input {
                        let started = Instant::now();
                        let submitting = matches!(
                            input,
                            AtomicReplaceInput::Confirm | AtomicReplaceInput::Recover
                        ) && matches!(
                            disposition_before,
                            AtomicReplaceDisposition::Armed
                                | AtomicReplaceDisposition::OutcomeUnknown
                        );
                        if submitting && vellum_atomic_operation.is_none() {
                            vellum_atomic_operation = instrumentation.begin_peer_operation_at(
                                "vellum.atomic-replace-selected",
                                "vellum",
                                started,
                            );
                        }
                        let lifecycle = control.update(
                            input,
                            vellum.as_mut().expect("atomic replace requires Vellum"),
                            true,
                        );
                        let disposition = control.disposition();
                        let level = match disposition {
                            AtomicReplaceDisposition::Applied => ActivityLevel::Success,
                            AtomicReplaceDisposition::OutcomeUnknown => ActivityLevel::Uncertain,
                            AtomicReplaceDisposition::Refused => ActivityLevel::Warning,
                            AtomicReplaceDisposition::Editing
                            | AtomicReplaceDisposition::Preview
                            | AtomicReplaceDisposition::Armed => ActivityLevel::Info,
                        };
                        if submitting {
                            instrumentation.record(
                                "vellum.atomic-replace.commit",
                                "vellum",
                                level,
                                started.elapsed(),
                            );
                            if let Some(operation) = vellum_atomic_operation {
                                instrumentation.mark_peer_stage(
                                    operation,
                                    "owner_atomic_receipt",
                                    level,
                                );
                                if disposition != AtomicReplaceDisposition::OutcomeUnknown {
                                    instrumentation.settle_peer_operation(operation, level);
                                    vellum_atomic_operation = None;
                                }
                            }
                        }
                        notice = control.notice().into();
                        activity.push(level, "vellum", notice.clone());
                        if lifecycle == AtomicReplaceLifecycle::CloseRequested {
                            vellum_atomic_replace = None;
                            vellum_atomic_operation = None;
                            notice = "Closed Vellum atomic-replace composer".into();
                        }
                    }
                }
                Event::Key(key)
                    if vellum_surface_active(shell.active(), focus)
                        && vellum_durable_save.is_some()
                        && key_behavior != Some(KeyBehavior::SafeExit) =>
                {
                    let control = vellum_durable_save.as_mut().expect("matched above");
                    if is_key(key, KeyCode::Esc, KeyModifiers::NONE) {
                        if control.disposition() == DurableSaveBatchDisposition::Armed {
                            control.update(DurableSaveBatchInput::Back, true);
                            notice = "Vellum durable-save workset disarmed".into();
                        } else if control.request_pending() || control.recovery_required() {
                            notice = "Vellum durable save is pending or unresolved; use r for exact receipt lookup"
                                .into();
                            activity.push(ActivityLevel::Warning, "vellum", notice.clone());
                        } else {
                            vellum_durable_save = None;
                            last_vellum_durable_notice.clear();
                            notice = "Closed Vellum durable-save workset".into();
                        }
                    } else {
                        let input = if is_key(key, KeyCode::Char('c'), KeyModifiers::NONE) {
                            Some(DurableSaveBatchInput::Arm)
                        } else if is_key(key, KeyCode::Enter, KeyModifiers::NONE) {
                            Some(DurableSaveBatchInput::Confirm)
                        } else if is_key(key, KeyCode::Char('r'), KeyModifiers::NONE) {
                            Some(DurableSaveBatchInput::Recover)
                        } else {
                            None
                        };
                        if let Some(input) = input {
                            let pending_before = control.request_pending();
                            control.update(input, true);
                            if !pending_before
                                && control.request_pending()
                                && vellum_durable_operation.is_none()
                            {
                                let action = if input == DurableSaveBatchInput::Recover {
                                    "vellum.save-selected.recover"
                                } else {
                                    "vellum.save-selected"
                                };
                                vellum_durable_operation =
                                    instrumentation.begin_peer_operation(action, "vellum");
                            }
                            notice = control.notice().into();
                        }
                    }
                }
                Event::Key(key) => match route_key_for_surface(key, shell.active(), focus) {
                    RoutedInput::Exit => {
                        let agora_unresolved = agora_create.as_ref().is_some_and(|control| {
                            control.request_pending() || control.recovery_required()
                        }) || agora_message.as_ref().is_some_and(
                            |control| control.request_pending() || control.recovery_required(),
                        );
                        let moira_unresolved =
                            moira_reconciliation.as_ref().is_some_and(|control| {
                                control.request_pending() || control.recovery_required()
                            });
                        let daimon_unresolved = daimon_control.as_ref().is_some_and(|control| {
                            control.request_pending() || control.recovery_required()
                        }) || daimon_post_receipt_refresh
                            || daimon_recovery_refresh
                            || daimon_operation.is_some();
                        let aegis_unresolved =
                            aegis_revoke.as_ref().is_some_and(|control| {
                                matches!(
                                    control.disposition(),
                                    aegis_tui::RevokeEpochDisposition::Pending
                                        | aegis_tui::RevokeEpochDisposition::OutcomeUnknown
                                )
                            }) || aegis_managed.as_ref().is_some_and(|control| {
                                matches!(
                                    control.disposition(),
                                    aegis_tui::ManagedAuthorityDisposition::Pending
                                        | aegis_tui::ManagedAuthorityDisposition::Fenced
                                )
                            });
                        let hermes_unresolved =
                            hermes_acknowledge.as_ref().is_some_and(|control| {
                                matches!(
                                    control.disposition(),
                                    hermes_tui::AcknowledgeDisposition::Pending
                                        | hermes_tui::AcknowledgeDisposition::OutcomeUnknown
                                )
                            }) || hermes_post_receipt_refresh
                                || hermes_operation.is_some()
                                || hermes_disclosure.as_ref().is_some_and(|control| {
                                    control.disposition()
                                        == hermes_tui::DisclosureDisposition::Pending
                                })
                                || hermes_disclosure_operation.is_some()
                                || hermes_compaction.as_ref().is_some_and(|control| {
                                    matches!(
                                        control.disposition(),
                                        hermes_tui::CompactionDisposition::Pending
                                            | hermes_tui::CompactionDisposition::OutcomeUnknown
                                    )
                                })
                                || hermes_compaction_post_receipt_refresh
                                || hermes_compaction_operation.is_some();
                        let chronos_unresolved = chronos_operation.is_some()
                            || chronos
                                .as_ref()
                                .is_some_and(Hub::mutation_recovery_required);
                        let vellum_unresolved =
                            vellum_durable_save.as_ref().is_some_and(|control| {
                                control.request_pending() || control.recovery_required()
                            }) || vellum_atomic_replace.as_ref().is_some_and(|control| {
                                control.disposition() == AtomicReplaceDisposition::OutcomeUnknown
                            }) || vellum_atomic_operation.is_some();
                        let provider_edit_unresolved = provider_edit
                            .as_ref()
                            .is_some_and(|workflow| workflow.worker.is_some());
                        let provider_qualification_unresolved = provider_qualification
                            .as_ref()
                            .is_some_and(|workflow| workflow.worker.is_some());
                        if chronos_unresolved
                            || provider_edit_unresolved
                            || provider_qualification_unresolved
                            || vellum_unresolved
                            || agora_unresolved
                            || moira_unresolved
                            || daimon_unresolved
                            || hermes_unresolved
                            || aegis_unresolved
                            || operations.reply.is_some()
                        {
                            notice = "A guarded control or topology drain is pending or outcome-unknown; settle or explicitly recover it before exit"
                                .into();
                            activity.push(ActivityLevel::Warning, "pantheon", notice.clone());
                            continue;
                        }
                        match decide_exit(
                            vellum
                                .as_ref()
                                .map(ServiceEditor::aggregate_persistence)
                                .unwrap_or(
                                    vellum_sdk_client::workspace::PersistenceProjectionV1::Clean,
                                ),
                            confirming_discard,
                        ) {
                            ExitDecision::Exit => break,
                            ExitDecision::ConfirmDiscard => {
                                confirming_discard = true;
                                notice = "Unsaved Vellum edit; Ctrl-Q again discards, Esc cancels"
                                    .into();
                            }
                            ExitDecision::RefuseIndeterminate => {
                                notice = "Vellum effect is indeterminate; repeat the exact action before exit"
                                    .into();
                            }
                        }
                    }
                    RoutedInput::SwitchFocus => {
                        if shell.active() == Surface::Workspace {
                            if focus == Focus::Chronos
                                && let Some(peer) = &mut chronos
                                && peer.composing_task_batch()
                            {
                                peer.update(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
                                notice = "Added a row to the atomic Chronos task batch".into();
                            } else {
                                focus = focus.other();
                                confirming_discard = false;
                                notice = format!("focus moved to {focus:?}");
                            }
                        } else {
                            notice = "Tab is local to the Workspace surface".into();
                        }
                    }
                    RoutedInput::Chronos(input) => {
                        if !chronos_surface_active(shell.active(), focus) {
                            notice = format!("{} has no input peer", shell.active().label());
                        } else if let Some(chronos) = &mut chronos {
                            let started = Instant::now();
                            let refresh = is_key(input, KeyCode::Char('r'), KeyModifiers::NONE);
                            let submitting_task_batch = chronos.composing_task_batch()
                                && is_key(input, KeyCode::Enter, KeyModifiers::NONE);
                            let confirming_cancel_batch = chronos.confirming_cancel_batch()
                                && is_key(input, KeyCode::Char('y'), KeyModifiers::NONE);
                            let confirming_scheduling = chronos.confirming_scheduling()
                                && is_key(input, KeyCode::Char('y'), KeyModifiers::NONE);
                            let confirming_eligibility_batch =
                                chronos.confirming_eligibility_batch();
                            if submitting_task_batch
                                && let Err(reason) = work_item_agent.task_id_for_submission()
                            {
                                notice = command_feedback(
                                    "workitem.agent.new",
                                    ActivityLevel::Warning,
                                    reason,
                                );
                                activity.push(ActivityLevel::Warning, "work-item", notice.clone());
                                continue;
                            }
                            if chronos.composing_task_batch() {
                                work_item_agent.capture_task_key(input);
                            }
                            if chronos.update(input) == Lifecycle::CloseRequested {
                                notice = "Use Ctrl-Q to exit the complete workbench".into();
                            }
                            if submitting_task_batch && !chronos.composing_task_batch() {
                                work_item_agent.task_submitted();
                                chronos_operation = instrumentation
                                    .begin_peer_operation_at(
                                        "chronos.task.submit-batch",
                                        "chronos",
                                        started,
                                    )
                                    .map(ChronosOperation::Task);
                            } else if confirming_cancel_batch {
                                chronos_operation = instrumentation
                                    .begin_peer_operation_at(
                                        "chronos.task.cancel-batch",
                                        "chronos",
                                        started,
                                    )
                                    .map(ChronosOperation::Task);
                            } else if confirming_scheduling {
                                chronos_operation = instrumentation
                                    .begin_peer_operation_at(
                                        if confirming_eligibility_batch {
                                            "chronos.schedule.eligibility-batch"
                                        } else {
                                            "chronos.schedule.control"
                                        },
                                        "chronos",
                                        started,
                                    )
                                    .map(ChronosOperation::Scheduling);
                            }
                            begin_refresh(
                                refresh,
                                "view.refresh",
                                "chronos",
                                &mut refresh_operations.chronos,
                                &mut instrumentation,
                                started,
                            );
                        } else {
                            notice = "Chronos service is not configured".into();
                        }
                    }
                    RoutedInput::Vellum(_input)
                        if !vellum_surface_active(shell.active(), focus) =>
                    {
                        notice = format!("{} has no input peer", shell.active().label());
                    }
                    RoutedInput::Vellum(input) if vellum.is_none() => {
                        let _ = input;
                        notice = "Vellum service is not configured".into();
                    }
                    RoutedInput::Vellum(input) => match translate_service_input(input) {
                        ServiceInput::CancelQuit => {
                            confirming_discard = false;
                            notice = "discard cancelled".into();
                        }
                        ServiceInput::Apply(ServiceAction::Save) | ServiceInput::Open => {
                            unreachable!("Vellum open and save are behavior-facade commands")
                        }
                        ServiceInput::Find => {
                            vellum_find_prompt = Some(TextPrompt::new(1024, "Find literal"));
                            notice = "Vellum find prompt opened".into();
                        }
                        ServiceInput::Apply(action) => {
                            let action_id = vellum_action_id(&action);
                            let started = Instant::now();
                            let editor = vellum.as_mut().expect("checked above");
                            let result = editor.apply(action);
                            if editor.take_identity_renewal() {
                                instrumentation.record(
                                    "vellum.client.renew-identity",
                                    "vellum",
                                    ActivityLevel::Success,
                                    started.elapsed(),
                                );
                                activity.push(
                                    ActivityLevel::Success,
                                    "vellum",
                                    "Vellum replay identity renewed before request capacity exhaustion",
                                );
                            }
                            let level = if result.is_ok() {
                                ActivityLevel::Success
                            } else {
                                ActivityLevel::Warning
                            };
                            instrumentation.record(action_id, "vellum", level, started.elapsed());
                            notice = result.map_or_else(
                                |error| error.to_string(),
                                |()| "Vellum action accepted".into(),
                            );
                            confirming_discard = false;
                        }
                        ServiceInput::Quit => unreachable!("Ctrl-Q is host-owned"),
                        ServiceInput::Unsupported => {
                            notice = "Command is unavailable in Vellum service mode".into();
                        }
                    },
                },
                _ => {}
            }
        }
        Ok(())
    });
    let disconnect = vellum.map_or(Ok(()), |editor| {
        editor
            .disconnect()
            .map(|_| ())
            .map_err(|failure| format!("Vellum disconnect refused: {failure:?}"))
    });
    let shutdown = operations.finish();
    if let Err(error) = result {
        return match shutdown {
            Ok(_) => Err(error),
            Err(shutdown) => Err(format!("{error}; {shutdown}").into()),
        };
    }
    disconnect?;
    shutdown
        .map(|shutdown| ManagedTerminalOutcome {
            shutdown,
            restart_profile,
        })
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn behavior_facade_enforces_overlay_global_navigation_and_peer_precedence() {
        let ctrl = |character| KeyEvent::new(KeyCode::Char(character), KeyModifiers::CONTROL);
        let plain = |character| KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE);

        for overlay in [
            WorkbenchOverlay::None,
            WorkbenchOverlay::Commands,
            WorkbenchOverlay::Help,
            WorkbenchOverlay::Activity,
            WorkbenchOverlay::Configuration,
            WorkbenchOverlay::LocalProviders,
            WorkbenchOverlay::ProviderEdit,
            WorkbenchOverlay::ProviderQualification,
            WorkbenchOverlay::AcpSessions,
            WorkbenchOverlay::AgoraRooms,
            WorkbenchOverlay::Reconfiguration,
            WorkbenchOverlay::Instrumentation,
        ] {
            assert_eq!(
                BehaviorFacade::key(ctrl('q'), overlay, Surface::Vellum, true),
                KeyBehavior::SafeExit
            );
        }
        assert_eq!(
            BehaviorFacade::key(
                plain('x'),
                WorkbenchOverlay::Commands,
                Surface::Overview,
                false,
            ),
            KeyBehavior::CommandPaletteInput
        );
        assert_eq!(
            BehaviorFacade::key(
                KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                WorkbenchOverlay::Help,
                Surface::Overview,
                false,
            ),
            KeyBehavior::DismissOverlay
        );
        assert_eq!(
            BehaviorFacade::key(plain('x'), WorkbenchOverlay::Help, Surface::Overview, false,),
            KeyBehavior::Suppress
        );
        assert_eq!(
            BehaviorFacade::key(
                plain('r'),
                WorkbenchOverlay::LocalProviders,
                Surface::Configuration,
                false,
            ),
            KeyBehavior::ExecuteCommand("configuration.providers.refresh")
        );
        assert_eq!(
            BehaviorFacade::key(
                plain('x'),
                WorkbenchOverlay::ProviderEdit,
                Surface::Configuration,
                false,
            ),
            KeyBehavior::ProviderEditInput
        );
        assert_eq!(
            BehaviorFacade::key(
                plain('x'),
                WorkbenchOverlay::ProviderQualification,
                Surface::Configuration,
                false,
            ),
            KeyBehavior::ProviderQualificationInput
        );
        assert_eq!(
            BehaviorFacade::key(ctrl('p'), WorkbenchOverlay::None, Surface::Vellum, true),
            KeyBehavior::OpenCommands
        );
        assert_eq!(
            BehaviorFacade::key(plain('r'), WorkbenchOverlay::None, Surface::Overview, false),
            KeyBehavior::ExecuteCommand("view.refresh")
        );
        assert_eq!(
            BehaviorFacade::key(plain('r'), WorkbenchOverlay::None, Surface::Vellum, true),
            KeyBehavior::SurfaceLocal
        );
        assert_eq!(
            BehaviorFacade::key(plain(']'), WorkbenchOverlay::None, Surface::Vellum, true),
            KeyBehavior::SurfaceLocal
        );
        assert_eq!(
            BehaviorFacade::key(plain('['), WorkbenchOverlay::None, Surface::Vellum, true),
            KeyBehavior::SurfaceLocal
        );
        assert_eq!(
            BehaviorFacade::key(plain(']'), WorkbenchOverlay::None, Surface::Overview, false),
            KeyBehavior::Navigate(ShellInput::NextSurface)
        );
        assert_eq!(
            BehaviorFacade::key(
                KeyEvent::new(KeyCode::Char('3'), KeyModifiers::ALT),
                WorkbenchOverlay::None,
                Surface::Vellum,
                true,
            ),
            KeyBehavior::ExecuteCommand("navigate.agora")
        );
        assert_eq!(
            BehaviorFacade::key(
                KeyEvent::new(KeyCode::Char('r'), KeyModifiers::ALT),
                WorkbenchOverlay::None,
                Surface::Vellum,
                true,
            ),
            KeyBehavior::ExecuteCommand("navigate.agenda")
        );
        assert_eq!(
            BehaviorFacade::key(plain('o'), WorkbenchOverlay::None, Surface::Vellum, true),
            KeyBehavior::SurfaceLocal
        );
        assert_eq!(
            BehaviorFacade::key(ctrl('o'), WorkbenchOverlay::None, Surface::Vellum, true),
            KeyBehavior::ExecuteCommand("vellum.document.open")
        );
        assert_eq!(
            BehaviorFacade::key(
                KeyEvent::new(
                    KeyCode::Char('O'),
                    KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                ),
                WorkbenchOverlay::None,
                Surface::Vellum,
                true,
            ),
            KeyBehavior::ExecuteCommand("vellum.document.open")
        );
        assert_eq!(
            BehaviorFacade::key(
                KeyEvent::new(KeyCode::Char('\u{f}'), KeyModifiers::NONE),
                WorkbenchOverlay::None,
                Surface::Vellum,
                true,
            ),
            KeyBehavior::ExecuteCommand("vellum.document.open")
        );
        assert_eq!(
            BehaviorFacade::key(ctrl('s'), WorkbenchOverlay::None, Surface::Vellum, true),
            KeyBehavior::ExecuteCommand("vellum.save")
        );
        assert_eq!(
            BehaviorFacade::key(
                KeyEvent::new(KeyCode::Char('N'), KeyModifiers::SHIFT),
                WorkbenchOverlay::None,
                Surface::Chronos,
                false,
            ),
            KeyBehavior::ExecuteCommand("workitem.agent.new")
        );
    }

    #[test]
    fn provider_catalog_projection_is_bounded_redacted_and_owner_oriented() {
        let catalog = ProviderCatalogProjection {
            version: 1,
            owner: "daimon-local-agent".into(),
            document_digest: format!("sha256:{}", "11".repeat(32)),
            providers: vec![ProviderCatalogEntryProjection {
                provider_id: "22".repeat(16),
                harness_profile_id: "33".repeat(16),
                profile_document_digest: format!("sha256:{}", "44".repeat(32)),
                policy_digest: None,
                transport: None,
                isolation: None,
                executable_available: None,
                argument_count: None,
                inherited_environment_name_count: None,
                authentication_method_configured: None,
                wire: Some("openai_chat_completions".into()),
                model: Some("qwen/qwen3-coder-next".into()),
                endpoint_scope: Some("loopback".into()),
                endpoint_port: Some(1234),
                structured_output: Some(false),
                structured_tools: Some(true),
                sampling_temperature_milli: Some(0),
                bearer_token_configured: Some(false),
                qualification: None,
            }],
            task_bindings: vec![TaskBindingCatalogProjection {
                external_task_key: "55".repeat(16),
                chronos_task_id: "review-headless-boundary".into(),
            }],
            contexts: vec![ContextCatalogProjection {
                reference: "66".repeat(16),
                kind: "vellum_document".into(),
                resolution: Some("runtime_owner".into()),
                source_digest: None,
                bytes: None,
                workspace_id: Some(1),
                document_ordinal: Some(2),
                expected_revision: Some(3),
            }],
        };
        validate_provider_catalog(&catalog).unwrap();
        let binding =
            validate_work_item_binding(&catalog, "review-headless-boundary", &"55".repeat(16))
                .unwrap();
        assert_eq!(binding.task_id, "review-headless-boundary");
        assert_eq!(binding.external_task_key, "55".repeat(16));
        assert!(
            validate_work_item_binding(&catalog, "unbound-task", &"55".repeat(16))
                .unwrap_err()
                .contains("no exact binding")
        );
        assert!(
            validate_work_item_binding(&catalog, "review-headless-boundary", &"77".repeat(16))
                .unwrap_err()
                .contains("does not match")
        );
        let lines = provider_catalog_lines(catalog).join("\n");
        assert!(lines.contains("Daimon validated"));
        assert!(lines.contains("qwen/qwen3-coder-next"));
        assert!(lines.contains("loopback:1234"));
        assert!(lines.contains("structured tools true"));
        assert!(lines.contains("review-headless-boundary"));
        assert!(lines.contains("vellum_document"));
        assert!(lines.contains("runtime-resolved bytes"));
        assert!(lines.contains("workspace 1/document 2 @ revision 3"));
        assert!(!lines.contains("host_header"));
        assert!(!lines.contains("OPENAI_API_KEY"));
    }

    #[test]
    fn provider_qualification_receipt_is_exact_redacted_and_partial() {
        let qualification = ProviderQualificationProjection {
            version: 1,
            owner: "daimon-local-model".into(),
            operation_id: "11".repeat(16),
            document_digest: format!("sha256:{}", "22".repeat(32)),
            profile_document_digest: format!("sha256:{}", "33".repeat(32)),
            provider_id: "44".repeat(16),
            harness_profile_id: "55".repeat(16),
            model: "qwen/qwen3-coder-next".into(),
            host_observed_at_unix_ms: 1,
            disposition: "qualified_completion".into(),
            completion: true,
            tool_continuation: "not_exercised".into(),
            input_tokens: 17,
            output_tokens: 4,
            output_bytes: 30,
            evidence_digest: Some(format!("sha256:{}", "66".repeat(32))),
            known_deviations: vec!["tool_continuation_not_exercised".into()],
            replayed: false,
        };
        let provider = ProviderCatalogEntryProjection {
            provider_id: qualification.provider_id.clone(),
            harness_profile_id: qualification.harness_profile_id.clone(),
            profile_document_digest: qualification.profile_document_digest.clone(),
            policy_digest: None,
            transport: None,
            isolation: None,
            executable_available: None,
            argument_count: None,
            inherited_environment_name_count: None,
            authentication_method_configured: None,
            wire: Some("openai_chat_completions".into()),
            model: Some(qualification.model.clone()),
            endpoint_scope: Some("loopback".into()),
            endpoint_port: Some(1234),
            structured_output: Some(false),
            structured_tools: Some(true),
            sampling_temperature_milli: Some(0),
            bearer_token_configured: Some(false),
            qualification: Some(qualification.clone()),
        };
        validate_provider_qualification(&qualification, Some(&provider)).unwrap();
        let mut catalog = ProviderCatalogProjection {
            version: 1,
            owner: "daimon-local-model".into(),
            document_digest: qualification.document_digest.clone(),
            providers: vec![provider.clone()],
            task_bindings: vec![TaskBindingCatalogProjection {
                external_task_key: "77".repeat(16),
                chronos_task_id: "qualification-task".into(),
            }],
            contexts: Vec::new(),
        };
        validate_provider_catalog(&catalog).unwrap();
        catalog.document_digest = format!("sha256:{}", "88".repeat(32));
        assert!(validate_provider_catalog(&catalog).is_err());
        let workflow = ProviderQualificationWorkflow {
            phase: ProviderQualificationPhase::Complete,
            plan: Some(ProviderQualificationPlan {
                owner: "daimon-local-model".into(),
                document_digest: qualification.document_digest.clone(),
                provider,
            }),
            receipt: Some(qualification),
            armed: false,
            notice: "QUALIFIED".into(),
            worker: None,
        };
        let lines = workflow.lines().join("\n");
        assert!(lines.contains("qualified_completion"));
        assert!(lines.contains("17 input · 4 output tokens · 30 output bytes"));
        assert!(lines.contains("tool continuation    not_exercised"));
        assert!(lines.contains("not task invocation"));
        assert!(!lines.contains("bounded qualification complete"));
    }

    #[test]
    fn provider_edit_form_builds_a_complete_owner_request_without_claiming_delegation() {
        let mut workflow = ProviderEditWorkflow::new();
        workflow.values[0] = "22".repeat(16);
        workflow.values[1] = "33".repeat(16);
        workflow.values[2] = format!("sha256:{}", "44".repeat(32));
        workflow.values[3] = "/usr/bin/provider".into();
        workflow.values[4] = "serve\n--stdio".into();
        workflow.values[5] = "MODEL_HOME\nMODEL_CACHE".into();
        workflow.values[6] = "oauth-device".into();
        let provider = workflow.provider_value().unwrap();
        assert_eq!(provider["transport"], "acp");
        assert_eq!(provider["isolation"]["kind"], "direct");
        assert_eq!(
            provider["arguments"],
            serde_json::json!(["serve", "--stdio"])
        );
        assert_eq!(
            provider["inherited_environment_names"],
            serde_json::json!(["MODEL_HOME", "MODEL_CACHE"])
        );
        let lines = workflow.lines().join("\n");
        assert!(lines.contains("configuration only"));
        assert!(lines.contains("no qualification"));
        assert!(lines.contains("delegation"));
        assert!(lines.contains("invocation"));
    }

    #[test]
    fn provider_edit_plan_and_receipt_are_exact_owner_bound_facts() {
        let operation_id = "11".repeat(16);
        let document_digest = format!("sha256:{}", "22".repeat(32));
        let candidate_digest = format!("sha256:{}", "33".repeat(32));
        let plan = ProviderEditPlanProjection {
            version: 1,
            owner: "daimon-local-agent".into(),
            operation_id: operation_id.clone(),
            action: "replace".into(),
            expected_document_digest: document_digest.clone(),
            current_document_digest: document_digest.clone(),
            candidate_document_digest: candidate_digest.clone(),
            plan_digest: format!("sha256:{}", "44".repeat(32)),
            provider: ProviderCatalogEntryProjection {
                provider_id: "55".repeat(16),
                harness_profile_id: "66".repeat(16),
                profile_document_digest: format!("sha256:{}", "77".repeat(32)),
                policy_digest: Some(format!("sha256:{}", "88".repeat(32))),
                transport: Some("acp".into()),
                isolation: Some("direct".into()),
                executable_available: Some(true),
                argument_count: Some(2),
                inherited_environment_name_count: Some(0),
                authentication_method_configured: Some(false),
                wire: None,
                model: None,
                endpoint_scope: None,
                endpoint_port: None,
                structured_output: None,
                structured_tools: None,
                sampling_temperature_milli: None,
                bearer_token_configured: None,
                qualification: None,
            },
            changed_fields: vec!["policy_digest".into()],
            restart_required: true,
        };
        let expected_provider = serde_json::json!({
            "provider_id": "55".repeat(16),
            "harness_profile_id": "66".repeat(16),
            "policy_digest": format!("sha256:{}", "88".repeat(32)),
            "program": "/usr/bin/provider",
            "arguments": ["serve", "--stdio"],
            "inherited_environment_names": [],
            "acp_agent_authentication_method": null,
            "isolation": {"kind": "direct"},
            "transport": "acp"
        });
        validate_provider_edit_plan(&plan, &operation_id, &document_digest, &expected_provider)
            .unwrap();
        let receipt = ProviderEditReceiptProjection {
            version: 1,
            owner: "daimon-local-agent".into(),
            operation_id,
            disposition: "applied".into(),
            applied: true,
            replayed: false,
            document_digest: candidate_digest,
            provider_id: plan.provider.provider_id.clone(),
            restart_required: true,
        };
        validate_provider_edit_receipt(&receipt, &plan).unwrap();
    }

    #[test]
    fn behavior_facade_applies_navigation_in_one_place() {
        let mut shell = Shell::new(vec![
            Surface::Overview,
            Surface::Chronos,
            Surface::Configuration,
        ])
        .unwrap();
        BehaviorFacade::navigate(&mut shell, ShellInput::NextSurface);
        assert_eq!(shell.active(), Surface::Chronos);
        BehaviorFacade::navigate(
            &mut shell,
            ShellInput::ActivateSurface(Surface::Configuration),
        );
        assert_eq!(shell.active(), Surface::Configuration);
        BehaviorFacade::navigate(&mut shell, ShellInput::PreviousSurface);
        assert_eq!(shell.active(), Surface::Chronos);
    }

    #[test]
    fn behavior_facade_normalizes_palette_outcomes_into_shared_commands() {
        assert_eq!(
            BehaviorFacade::palette(PaletteOutcome::Execute("navigate.agora")),
            KeyBehavior::ExecuteCommand("navigate.agora")
        );
        assert_eq!(
            BehaviorFacade::palette(PaletteOutcome::Unavailable("not configured")),
            KeyBehavior::CommandUnavailable("not configured")
        );
        assert_eq!(
            BehaviorFacade::palette(PaletteOutcome::Dismiss),
            KeyBehavior::DismissOverlay
        );
        assert_eq!(
            BehaviorFacade::palette(PaletteOutcome::None),
            KeyBehavior::Suppress
        );
    }

    #[test]
    fn command_feedback_is_observable_and_orients_the_next_step() {
        let ready = command_feedback(
            "chronos.task.new",
            ActivityLevel::Info,
            "Opened the bounded Chronos new-task form",
        );
        assert!(ready.starts_with("READY ·"));
        assert!(ready.contains("Next:"));
        assert!(ready.contains("Tab adds a row"));

        let refused = command_feedback(
            "vellum.document.open",
            ActivityLevel::Warning,
            "Vellum service is not configured",
        );
        assert!(refused.starts_with("REFUSED ·"));
        assert!(refused.contains("runtime file admission"));
    }

    #[test]
    fn moira_owner_worker_accepts_only_an_exact_bounded_participation_receipt() {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!(
            "pantheon-moira-worker-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let executable = root.join("worker");
        let config = root.join("opaque.json");
        std::fs::write(&config, b"opaque").unwrap();
        std::fs::write(
            &executable,
            b"#!/bin/sh\nprintf '%s\\n' '{\"outcome\":\"observed\",\"participation\":\"01010101010101010101010101010101\",\"participation_revision\":2}'\n",
        )
        .unwrap();
        let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&executable, permissions).unwrap();

        let mut worker = MoiraAgentWorkerControl::new();
        worker
            .start(MoiraAgentWorkerConfig {
                executable: executable.clone(),
                config: config.clone(),
                participation: moira_protocol::agent_participation::ParticipationIdV1([1; 16]),
            })
            .unwrap();
        assert_eq!(
            worker.start(MoiraAgentWorkerConfig {
                executable: executable.clone(),
                config: config.clone(),
                participation: moira_protocol::agent_participation::ParticipationIdV1([1; 16]),
            }),
            Err("Moira owner worker is already pending")
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        let outcome = loop {
            if let Some(outcome) = worker.poll() {
                break outcome;
            }
            assert!(Instant::now() < deadline, "owner worker did not settle");
            std::thread::sleep(Duration::from_millis(2));
        };
        assert_eq!(outcome, MoiraAgentWorkerSettlement::Observed);
        assert!(worker.notice.contains("revision 2"));
        assert!(!worker.pending);
        assert!(!worker.uncertain);

        std::fs::write(
            &executable,
            b"#!/bin/sh\nprintf '%s\\n' '{\"outcome\":\"observed\",\"participation\":\"02020202020202020202020202020202\",\"participation_revision\":3}'\n",
        )
        .unwrap();
        worker
            .start(MoiraAgentWorkerConfig {
                executable,
                config,
                participation: moira_protocol::agent_participation::ParticipationIdV1([1; 16]),
            })
            .unwrap();
        while worker.pending && Instant::now() < deadline + Duration::from_secs(2) {
            worker.poll();
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(worker.notice.contains("different participation"));
        assert!(worker.notice.contains("outcome unknown"));
        assert!(worker.uncertain);
        assert_eq!(
            worker.start(MoiraAgentWorkerConfig {
                executable: root.join("worker"),
                config: root.join("opaque.json"),
                participation: moira_protocol::agent_participation::ParticipationIdV1([1; 16]),
            }),
            Err("Moira owner worker is fenced; inspect the participation before any retry")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn work_item_agent_journey_carries_bounded_task_text_into_the_prompt() {
        let mut journey = WorkItemAgentJourney::EditingTask {
            draft: String::new(),
            lens: None,
        };
        journey.capture_task_key(KeyEvent::new(KeyCode::Char('R'), KeyModifiers::SHIFT));
        journey.capture_task_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        journey.capture_task_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        journey.task_submitted();
        let WorkItemAgentJourney::AwaitingTaskReceipt {
            draft,
            lens,
            task_id,
        } = journey
        else {
            panic!("task submission should advance the journey");
        };
        assert_eq!(draft, "Re");
        assert_eq!(task_id, "Re");
        assert!(lens.is_none());
        let prompt = work_item_agent_prompt(&draft, lens);
        assert_eq!(prompt, "Re");
        assert!(
            single_work_item_task_id("one\ntwo")
                .unwrap_err()
                .contains("accepts one task")
        );
    }

    #[test]
    fn collaboration_lenses_are_one_turn_guidance_without_synthetic_authority() {
        for lens in AGENT_COLLABORATION_LENSES {
            let prompt = agent_collaboration_lens_prompt(lens);
            assert!(prompt.contains(lens.guidance));
            assert!(prompt.contains("one agent turn"));
            assert!(prompt.contains("not a human identity"));
            assert!(prompt.contains("not a human identity") && prompt.contains("authority"));
            assert!(agent_collaboration_lens(lens.action).is_some());
            assert!(agent_task_collaboration_lens(lens.new_task_action).is_some());
            let work_prompt = work_item_agent_prompt("inspect", Some(lens));
            assert!(work_prompt.contains("Work item:\ninspect"));
            assert!(work_prompt.contains(lens.guidance));
        }
        assert!(agent_collaboration_lens("workitem.agent.lens.unknown").is_none());
        assert!(agent_task_collaboration_lens("workitem.agent.new.unknown").is_none());
    }

    fn local_profile(name: &str, surfaces: Vec<Surface>) -> WorkspaceProfile {
        WorkspaceProfile {
            version: 1,
            name: name.into(),
            surfaces,
            agora: None,
            moira: None,
            daimon: None,
            hermes: None,
            aegis: None,
            tellus: None,
            chronos: None,
            vellum: None,
        }
    }

    #[test]
    fn reconfiguration_workflow_preflights_exact_changes_before_arming() {
        let current = local_profile("current", vec![Surface::Overview]);
        let path = std::env::temp_dir().join(format!(
            "pantheon-reconfiguration-{}-{}.json",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::write(
            &path,
            r#"{
                "version": 1,
                "name": "next",
                "surfaces": ["overview", "operations"],
                "agora": null,
                "moira": null,
                "daimon": null,
                "hermes": null,
                "aegis": null,
                "tellus": null,
                "chronos": null,
                "vellum": null
            }"#,
        )
        .unwrap();
        let mut workflow = ReconfigurationWorkflow::new(current);
        workflow.insert(path.to_str().unwrap());
        workflow.preflight();
        let plan = workflow.plan.as_ref().expect("valid preflight");
        assert_eq!(plan.minimum_action, "shell_restart");
        assert_eq!(plan.changes.len(), 2);
        assert!(!workflow.armed);
        workflow.arm();
        assert_eq!(
            workflow
                .confirmed_profile()
                .as_ref()
                .map(|value| value.0.as_path()),
            Some(path.as_path())
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn reconfiguration_path_accepts_shifted_text_but_not_control_input() {
        assert_eq!(
            bounded_text_character(KeyEvent::new(KeyCode::Char('T'), KeyModifiers::SHIFT)),
            Some('T')
        );
        assert_eq!(
            bounded_text_character(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL)),
            None
        );
    }

    #[test]
    fn standalone_peer_surfaces_keep_tab_local_while_legacy_workspace_switches_focus() {
        let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(matches!(
            route_key_for_surface(tab, Surface::Chronos, Focus::Chronos),
            RoutedInput::Chronos(key) if key == tab
        ));
        assert!(matches!(
            route_key_for_surface(tab, Surface::Vellum, Focus::Chronos),
            RoutedInput::Vellum(_)
        ));
        assert!(matches!(
            route_key_for_surface(tab, Surface::Workspace, Focus::Chronos),
            RoutedInput::SwitchFocus
        ));
    }

    #[test]
    fn daimon_filter_key_exposes_protocol_derived_event_views() {
        let key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
        assert_eq!(
            daimon_agent_set_input(key),
            Some(daimon_tui::AgentSetInput::CycleEventFilter)
        );
    }

    #[test]
    fn suite_health_requires_settlement_evidence_and_preserves_degradation() {
        let mut health =
            SuiteHealthLedger::new([("agora", "rooms/messages"), ("hermes", "ledger/cursors")]);
        health.observing("agora", "rooms/messages", true);
        health.settle("agora", "rooms/messages", Some(Ok(())));
        health.settle("hermes", "ledger/cursors", Some(Err(())));
        let view = health.view(|peer| peer == "agora", |peer| peer == "hermes");
        assert_eq!(view.peers[0].state, PeerHealthState::Current);
        assert!(view.peers[0].age_millis.is_some());
        assert!(view.peers[0].busy);
        assert_eq!(view.peers[1].state, PeerHealthState::Degraded);
        assert!(view.peers[1].uncertain);
    }

    #[test]
    fn command_catalog_combines_contextual_work_with_all_qualified_guards() {
        let shell = Shell::new(vec![Surface::Overview, Surface::Workspace, Surface::Moira])
            .expect("valid shell");
        let commands = workbench_commands(
            &shell, true, true, true, true, true, true, true, true, true, true, true, true, true,
            true, true, true, true, true, true, true, true, true, true, true,
        );
        for expected in [
            "navigate.overview",
            "navigate.workspace",
            "navigate.moira",
            "moira.participation.open.prepare",
            "moira.participation.worker.run",
            "view.refresh",
            "vellum.atomic-replace-selected",
            "suite.refresh",
            "workspace.focus.toggle",
            "vellum.save",
            "chronos.task.new",
            "workitem.agent.new",
            "workitem.agent.new.executive",
            "workitem.agent.new.product-owner",
            "workitem.agent.new.project-manager",
            "workitem.agent.new.ic-collaborator",
            "workitem.agent.lens.executive",
            "workitem.agent.lens.product-owner",
            "workitem.agent.lens.project-manager",
            "workitem.agent.lens.ic-collaborator",
            "workitem.agent.return",
            "workitem.agent.chat",
            "chronos.task.cancel",
            "chronos.schedule.open",
            "chronos.schedule.task-pause",
            "chronos.schedule.task-not-before",
            "agora.room.create",
            "agora.message.compose",
            "aegis.epoch.revoke",
            "aegis.authority.manage",
            "ledger.compact.v1",
            "ledger.events.disclose.v1",
            "ledger.acknowledge.v1",
            "operations.drain",
            "moira.saga.open",
            "moira.saga.back",
            "moira.reconciliation.keep-uncertain",
            "moira.reconciliation.fail-terminal",
            "moira.reconciliation.cancel",
            "moira.reconciliation.compensate",
            "moira.reconciliation.admit-failed",
            "moira.reconciliation.admit-succeeded",
            "moira.reconciliation.retry",
            "daimon.session.prompt",
            "models.provider.qualify.v1",
            "daimon.session.permission",
            "daimon.session.reattach",
            "daimon.session.advance",
            "daimon.session.cancel",
            "daimon.session.close",
            "workbench.activity",
            "workbench.help",
            "workbench.configuration",
            "configuration.providers.local",
            "configuration.providers.agent.upsert",
            "configuration.acp.sessions",
            "configuration.agora.rooms",
            "workbench.reconfigure",
            "workbench.instrumentation",
        ] {
            assert!(
                commands
                    .iter()
                    .any(|command| command.id == expected && command.enabled),
                "missing enabled workbench action {expected}"
            );
        }
        for route in GUARDED_CONTROL_ROUTES {
            if matches!(
                route.capability,
                "messages.speak.v2"
                    | "agent.open.prepare.v1"
                    | "agent.worker.step.v1"
                    | "agents.session.prompt.v1"
                    | "agents.session.permission.v1"
                    | "agents.session.reattach.v1"
                    | "models.provider.qualify.v1"
                    | "ledger.acknowledge.v1"
                    | "ledger.events.disclose.v1"
                    | "ledger.compact.v1"
            ) {
                let composed_id = match route.capability {
                    "messages.speak.v2" => "agora.message.compose",
                    "agent.open.prepare.v1" => "moira.participation.open.prepare",
                    "agent.worker.step.v1" => "moira.participation.worker.run",
                    "agents.session.prompt.v1" => "daimon.session.prompt",
                    "agents.session.permission.v1" => "daimon.session.permission",
                    "agents.session.reattach.v1" => "daimon.session.reattach",
                    "models.provider.qualify.v1" => "models.provider.qualify.v1",
                    _ => route.capability,
                };
                assert!(commands.iter().any(|command| {
                    command.id == composed_id
                        && command.kind == CommandKind::Guarded
                        && command.enabled
                }));
                continue;
            }
            assert!(
                commands.iter().any(|command| {
                    command.id == route.capability
                        && command.kind == CommandKind::Guarded
                        && !command.enabled
                }),
                "missing honest interactive disposition for {}",
                route.capability
            );
        }
    }

    #[test]
    fn unavailable_workspace_actions_explain_their_requirements() {
        let shell = Shell::new(vec![Surface::Overview]).expect("valid shell");
        let commands = workbench_commands(
            &shell, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false,
        );
        let save = commands
            .iter()
            .find(|command| command.id == "vellum.save")
            .unwrap();
        assert!(!save.enabled);
        assert_eq!(save.unavailable, Some("Vellum is not configured"));
    }

    #[test]
    fn operations_without_owned_topology_refuses_to_arm() {
        let mut operations = OperationsRuntime::new(None);
        operations.arm();
        let view = operations.view();
        assert_eq!(view.lifecycle, "external");
        assert!(!view.drain_armed);
        assert!(!view.drain_pending);
        assert!(view.notice.contains("No Pantheon-owned topology"));
    }

    #[test]
    fn operations_drains_owned_topology_and_retains_exact_receipt() {
        let mut host = NativeHost::new();
        host.start_agora(|| Ok::<_, String>(((), || ShutdownDisposition::Clean)))
            .unwrap();
        host.seal().unwrap();
        let mut operations = OperationsRuntime::new(Some(host));
        operations.arm();
        assert!(operations.confirm());
        let (level, shutdown) = (0..1_000)
            .find_map(|_| {
                let level = operations.poll();
                if level.is_none() {
                    std::thread::yield_now();
                }
                level
            })
            .expect("drain worker settled");
        assert_eq!(level, ActivityLevel::Success);
        assert!(shutdown.unwrap().clean());
        let view = operations.view();
        assert_eq!(view.lifecycle, "local_host_stopped");
        assert_eq!(view.receipts.len(), 1);
        assert_eq!(view.receipts[0].peer, "agora");
        assert_eq!(view.receipts[0].disposition, "clean");
        assert!(operations.finish().unwrap().unwrap().clean());
    }

    #[test]
    fn topology_drain_traces_each_peer_under_one_correlation() {
        let mut instrumentation = Instrumentation::default();
        let mut operations = begin_drain_operations(
            vec![HostedPeerId::Agora, HostedPeerId::Moira],
            &mut instrumentation,
            Instant::now(),
        );
        assert_eq!(operations.len(), 2);
        assert_eq!(instrumentation.traces().len(), 2);
        assert_eq!(
            instrumentation.traces()[0].correlation_id,
            instrumentation.traces()[1].correlation_id
        );
        let shutdown = TopologyShutdown {
            receipts: vec![
                crate::registry::ShutdownReceipt {
                    peer: HostedPeerId::Moira,
                    disposition: ShutdownDisposition::Clean,
                },
                crate::registry::ShutdownReceipt {
                    peer: HostedPeerId::Agora,
                    disposition: ShutdownDisposition::Refused,
                },
            ],
        };
        settle_drain_operations(
            &mut operations,
            Some(&shutdown),
            ActivityLevel::Uncertain,
            &mut instrumentation,
        );
        assert!(operations.is_empty());
        let agora = instrumentation
            .traces()
            .iter()
            .find(|trace| trace.peer == "agora")
            .unwrap();
        let moira = instrumentation
            .traces()
            .iter()
            .find(|trace| trace.peer == "moira")
            .unwrap();
        assert_eq!(agora.stages[2].level, Some(ActivityLevel::Warning));
        assert_eq!(moira.stages[2].level, Some(ActivityLevel::Success));
        assert!(agora.complete && moira.complete);
    }

    #[test]
    fn session_configuration_is_bounded_to_non_secret_launch_facts() {
        let shell = Shell::new(vec![Surface::Overview, Surface::Moira]).expect("valid shell");
        let lines = session_configuration(
            &shell, None, false, false, false, false, true, false, false, false, false, false,
            false, false, false, false, false, false, false,
        );
        let rendered = lines.join("\n");
        assert!(rendered.contains("Overview · Moira"));
        assert!(rendered.contains("Moira              configured"));
        assert!(rendered.contains("single user · local machine"));
        assert!(!rendered.contains("socket"));
        assert!(!rendered.contains("credential"));
    }

    #[test]
    fn reconciliation_activity_never_restylizes_unknown_as_success() {
        assert_eq!(
            reconciliation_level("Reconciliation outcome unknown; receipt inspection required"),
            ActivityLevel::Uncertain
        );
        assert_eq!(
            reconciliation_level("Reconciliation durably accepted"),
            ActivityLevel::Success
        );
        assert_eq!(
            reconciliation_level("Unknown outcome recovered from durable operator receipt"),
            ActivityLevel::Success
        );
        assert_eq!(
            reconciliation_level("Reconciliation refused"),
            ActivityLevel::Warning
        );
        assert_eq!(
            reconciliation_batch_level(moira_tui::ReconciliationBatchDisposition::OutcomeUnknown),
            ActivityLevel::Uncertain
        );
        assert_eq!(
            reconciliation_batch_level(moira_tui::ReconciliationBatchDisposition::Partial),
            ActivityLevel::Warning
        );
        assert_eq!(
            reconciliation_batch_level(moira_tui::ReconciliationBatchDisposition::Applied),
            ActivityLevel::Success
        );
    }

    #[test]
    fn peer_refresh_trace_runs_from_dispatch_to_projection_settlement() {
        let mut instrumentation = Instrumentation::default();
        let mut activity = ActivityLog::default();
        let started = Instant::now() - Duration::from_millis(7);
        let mut operation = None;
        assert!(begin_refresh(
            true,
            "view.refresh",
            "hermes",
            &mut operation,
            &mut instrumentation,
            started,
        ));
        assert_eq!(instrumentation.in_flight_count(), 1);
        settle_refresh(
            &mut operation,
            Some(Ok(())),
            "hermes",
            &mut instrumentation,
            &mut activity,
        );
        let metric = instrumentation
            .actions()
            .iter()
            .find(|metric| metric.action == "view.refresh" && metric.peer == "hermes")
            .expect("refresh metric");
        assert_eq!(metric.dispatches, 1);
        assert_eq!(metric.succeeded, 1);
        assert!(metric.last_latency_micros >= 7_000);
        assert_eq!(instrumentation.in_flight_count(), 0);
        assert_eq!(
            activity.entries().last().unwrap().level,
            ActivityLevel::Success
        );
    }

    #[test]
    fn hermes_disclosure_trace_spans_dispatch_read_receipt_and_settlement() {
        let mut instrumentation = Instrumentation::default();
        let operation = instrumentation
            .begin_peer_operation("ledger.events.disclose.v1", "hermes")
            .expect("trace capacity");
        assert!(instrumentation.mark_peer_stage(
            operation,
            "peer_read_receipt",
            ActivityLevel::Success,
        ));
        instrumentation.settle_peer_operation(operation, ActivityLevel::Success);

        let trace = instrumentation
            .traces()
            .iter()
            .find(|trace| trace.operation_id == operation)
            .expect("disclosure trace");
        assert_eq!(
            trace
                .stages
                .iter()
                .map(|stage| stage.stage)
                .collect::<Vec<_>>(),
            vec![
                "shell_dispatch",
                "peer_pending",
                "peer_read_receipt",
                "terminal_disposition",
            ]
        );
        assert!(trace.complete);
        let rendered = instrumentation.lines().join("\n");
        assert!(rendered.contains("ledger.events.disclose.v1"));
        for forbidden in ["payload", "preview", "correlation:", "producer", "schema"] {
            assert!(!rendered.contains(forbidden));
        }
    }

    #[test]
    fn daimon_controls_are_derived_only_from_eligible_exact_session_state() {
        use daimon_protocol::{
            AgentSessionIdV1, AgentTurnIdV1, ContentDigestV1,
            agent::{AgentCapabilitiesV1, AgentSessionViewV1},
        };

        let key = IdempotencyKeyV1::new([9; 16]);
        let mut view = AgentSessionViewV1 {
            session: AgentSessionIdV1::new([7; 16]),
            revision: 8,
            phase: AgentSessionPhaseV1::TurnActive,
            profile_document_digest: ContentDigestV1::new([3; 32]),
            active_turn: Some(AgentTurnIdV1::new([4; 16])),
            last_event_cursor: Some(2),
            last_context: None,
            capabilities: AgentCapabilitiesV1 {
                load_session: true,
                resume_session: true,
                permissions: true,
                filesystem_read: true,
                filesystem_write: false,
                terminal: true,
                session_modes: true,
                session_models: true,
            },
        };
        assert!(matches!(
            daimon_control_request("daimon.session.advance", &view, key, 42),
            Ok(daimon_tui::AgentControlRequest::Advance(_))
        ));
        assert!(matches!(
            daimon_control_request("daimon.session.cancel", &view, key, 42),
            Ok(daimon_tui::AgentControlRequest::Cancel(_))
        ));
        assert!(daimon_control_request("daimon.session.close", &view, key, 42).is_err());

        view.phase = AgentSessionPhaseV1::Ready;
        view.active_turn = None;
        assert!(matches!(
            daimon_control_request("daimon.session.close", &view, key, 42),
            Ok(daimon_tui::AgentControlRequest::Close(_))
        ));
        assert!(daimon_control_request("daimon.session.cancel", &view, key, 42).is_err());
    }

    #[test]
    fn daimon_prompt_batch_builds_fresh_exact_envelopes_for_every_session() {
        use daimon_protocol::{
            AgentSessionIdV1, ContentDigestV1,
            agent::{AgentCapabilitiesV1, AgentEventPageV1, AgentSessionViewV1},
        };

        let observation = |value| daimon_tui::AgentObservation {
            view: AgentSessionViewV1 {
                session: AgentSessionIdV1::new([value; 16]),
                revision: u64::from(value),
                phase: AgentSessionPhaseV1::Ready,
                profile_document_digest: ContentDigestV1::new([3; 32]),
                active_turn: None,
                last_event_cursor: None,
                last_context: None,
                capabilities: AgentCapabilitiesV1 {
                    load_session: true,
                    resume_session: true,
                    permissions: true,
                    filesystem_read: true,
                    filesystem_write: true,
                    terminal: true,
                    session_modes: true,
                    session_models: true,
                },
            },
            events: AgentEventPageV1 {
                session: AgentSessionIdV1::new([value; 16]),
                entries: Vec::new(),
                next_after: None,
            },
        };
        let mut observations = [observation(7), observation(8), observation(9)];
        let references = observations.iter().collect::<Vec<_>>();
        let policy = DaimonExecutionPolicyProfile {
            principal: "10101010101010101010101010101010".into(),
            profile: "11111111111111111111111111111111".into(),
            task_attempt: crate::profile::DaimonTaskAttemptProfile {
                task_key: "14141414141414141414141414141414".into(),
                task_incarnation: 4,
                attempt_number: 2,
            },
            context: vec!["12121212121212121212121212121212".into()],
            tools: vec!["13131313131313131313131313131313".into()],
            budget: crate::profile::DaimonExecutionBudgetProfile {
                max_elapsed_millis: 60_000,
                max_input_tokens: 4_096,
                max_output_tokens: 8_192,
                max_tool_calls: 32,
                max_context_bytes: 1_048_576,
                max_output_bytes: 1_048_576,
            },
        };
        let requests = daimon_prompt_requests(&references, &policy, b"do serious work".to_vec())
            .expect("valid prompt batch");
        assert_eq!(requests.len(), 3);
        let prompts = requests
            .iter()
            .map(|request| match request {
                daimon_tui::AgentControlRequest::Prompt(request) => request,
                _ => panic!("expected prompt request"),
            })
            .collect::<Vec<_>>();
        assert_eq!(prompts[0].expected_revision, 7);
        assert_eq!(prompts[2].expected_revision, 9);
        assert!(
            prompts
                .iter()
                .all(|request| request.prompt == b"do serious work")
        );
        assert_ne!(prompts[0].turn, prompts[1].turn);
        assert_ne!(
            prompts[0].execution.execution_id,
            prompts[1].execution.execution_id
        );
        assert_ne!(
            prompts[0].execution.idempotency_key,
            prompts[1].execution.idempotency_key
        );
        assert_eq!(
            prompts[0].execution.task_attempt,
            TaskAttemptRefV1 {
                task_key: [0x14; 16],
                task_incarnation: 4,
                attempt_number: 2,
            }
        );
        assert_eq!(
            prompts[0].execution.task_attempt,
            prompts[1].execution.task_attempt
        );
        assert_eq!(prompts[0].execution.context.len(), 1);
        assert_eq!(prompts[0].execution.tools.len(), 1);

        observations[1].view.phase = AgentSessionPhaseV1::ReattachRequired;
        observations[2].view.phase = AgentSessionPhaseV1::AwaitingPermission;
        observations[2].view.active_turn = Some(AgentTurnIdV1::new([22; 16]));
        let mixed = observations.iter().collect::<Vec<_>>();
        let error = daimon_prompt_requests(&mixed, &policy, b"do serious work".to_vec())
            .expect_err("mixed workset must be explained before identities are created");
        assert!(error.contains("2 of 3 selected session(s)"));
        assert!(error.contains("#2 ReattachRequired r8 no-active-turn"));
        assert!(error.contains("#3 AwaitingPermission r9 active-turn"));
        assert!(!error.contains("08080808"));
    }

    #[test]
    fn daimon_reattach_batch_builds_fresh_exact_envelopes_for_detached_sessions() {
        use daimon_protocol::{
            AgentSessionIdV1, ContentDigestV1,
            agent::{AgentCapabilitiesV1, AgentEventPageV1, AgentSessionViewV1},
        };

        let observation = |value| daimon_tui::AgentObservation {
            view: AgentSessionViewV1 {
                session: AgentSessionIdV1::new([value; 16]),
                revision: u64::from(value),
                phase: AgentSessionPhaseV1::ReattachRequired,
                profile_document_digest: ContentDigestV1::new([3; 32]),
                active_turn: None,
                last_event_cursor: None,
                last_context: None,
                capabilities: AgentCapabilitiesV1 {
                    load_session: true,
                    resume_session: true,
                    permissions: true,
                    filesystem_read: true,
                    filesystem_write: true,
                    terminal: true,
                    session_modes: true,
                    session_models: true,
                },
            },
            events: AgentEventPageV1 {
                session: AgentSessionIdV1::new([value; 16]),
                entries: Vec::new(),
                next_after: None,
            },
        };
        let observations = [observation(7), observation(8), observation(9)];
        let references = observations.iter().collect::<Vec<_>>();
        let policy = DaimonExecutionPolicyProfile {
            principal: "10101010101010101010101010101010".into(),
            profile: "11111111111111111111111111111111".into(),
            task_attempt: crate::profile::DaimonTaskAttemptProfile {
                task_key: "14141414141414141414141414141414".into(),
                task_incarnation: 4,
                attempt_number: 2,
            },
            context: vec!["12121212121212121212121212121212".into()],
            tools: vec!["13131313131313131313131313131313".into()],
            budget: crate::profile::DaimonExecutionBudgetProfile {
                max_elapsed_millis: 60_000,
                max_input_tokens: 4_096,
                max_output_tokens: 8_192,
                max_tool_calls: 32,
                max_context_bytes: 1_048_576,
                max_output_bytes: 1_048_576,
            },
        };
        let requests =
            daimon_reattach_requests(&references, &policy).expect("valid reattach batch");
        assert_eq!(requests.len(), 3);
        let reattachments = requests
            .iter()
            .map(|request| match request {
                daimon_tui::AgentControlRequest::Reattach(request) => request,
                _ => panic!("expected reattach request"),
            })
            .collect::<Vec<_>>();
        assert_eq!(reattachments[0].expected_revision, 7);
        assert_eq!(reattachments[2].expected_revision, 9);
        assert_ne!(
            reattachments[0].execution.execution_id,
            reattachments[1].execution.execution_id
        );
        assert_ne!(
            reattachments[0].execution.idempotency_key,
            reattachments[1].execution.idempotency_key
        );
    }

    #[test]
    fn daimon_permission_batch_binds_latest_challenge_and_fresh_execution_per_session() {
        use daimon_protocol::{
            AgentPermissionOptionIdV1, AgentPermissionRequestIdV1, AgentSessionIdV1, AgentTurnIdV1,
            ContentDigestV1,
            agent::{
                AgentCapabilitiesV1, AgentEventPageV1, AgentEventRecordV1,
                AgentPermissionOptionKindV1, AgentPermissionOptionV1, AgentPermissionRequestV1,
                AgentSessionViewV1,
            },
        };

        let observation = |value: u8| {
            let session = AgentSessionIdV1::new([value; 16]);
            let turn = AgentTurnIdV1::new([value + 10; 16]);
            let challenge = AgentPermissionRequestV1 {
                request: AgentPermissionRequestIdV1::new([value + 20; 16]),
                turn,
                capability: b"terminal.execute".to_vec(),
                description: b"run formatter".to_vec(),
                options: vec![AgentPermissionOptionV1 {
                    id: AgentPermissionOptionIdV1::new([value + 30; 16]),
                    kind: AgentPermissionOptionKindV1::RejectOnce,
                    label: b"Reject once".to_vec(),
                }],
            };
            daimon_tui::AgentObservation {
                view: AgentSessionViewV1 {
                    session,
                    revision: u64::from(value),
                    phase: AgentSessionPhaseV1::AwaitingPermission,
                    profile_document_digest: ContentDigestV1::new([3; 32]),
                    active_turn: Some(turn),
                    last_event_cursor: Some(5),
                    last_context: None,
                    capabilities: AgentCapabilitiesV1 {
                        load_session: true,
                        resume_session: true,
                        permissions: true,
                        filesystem_read: true,
                        filesystem_write: true,
                        terminal: true,
                        session_modes: true,
                        session_models: true,
                    },
                },
                events: AgentEventPageV1 {
                    session,
                    entries: vec![AgentEventRecordV1 {
                        cursor: 5,
                        event: AgentEventV1::Permission(challenge),
                    }],
                    next_after: None,
                },
            }
        };
        let observations = [observation(7), observation(8)];
        let references = observations.iter().collect::<Vec<_>>();
        let challenges = daimon_permission_challenges(&references).unwrap();
        let selections = challenges
            .into_iter()
            .map(|challenge| daimon_tui::AgentPermissionSelection {
                option: challenge.challenge.options[0].id,
                challenge,
            })
            .collect();
        let policy = DaimonExecutionPolicyProfile {
            principal: "10101010101010101010101010101010".into(),
            profile: "11111111111111111111111111111111".into(),
            task_attempt: crate::profile::DaimonTaskAttemptProfile {
                task_key: "14141414141414141414141414141414".into(),
                task_incarnation: 4,
                attempt_number: 2,
            },
            context: Vec::new(),
            tools: vec!["13131313131313131313131313131313".into()],
            budget: crate::profile::DaimonExecutionBudgetProfile {
                max_elapsed_millis: 60_000,
                max_input_tokens: 4_096,
                max_output_tokens: 8_192,
                max_tool_calls: 32,
                max_context_bytes: 1_048_576,
                max_output_bytes: 1_048_576,
            },
        };
        let requests = daimon_permission_requests(selections, &policy).unwrap();
        let decisions = requests
            .iter()
            .map(|request| match request {
                daimon_tui::AgentControlRequest::Permission(request) => request,
                _ => panic!("expected permission request"),
            })
            .collect::<Vec<_>>();
        assert_eq!(decisions[0].request.event_cursor, 5);
        assert_eq!(decisions[0].request.expected_revision, 7);
        assert_eq!(decisions[1].request.expected_revision, 8);
        assert_eq!(
            decisions[0].request.permission_request,
            decisions[0].challenge.request
        );
        assert_ne!(
            decisions[0].request.execution.execution_id,
            decisions[1].request.execution.execution_id
        );
        assert_ne!(
            decisions[0].request.execution.idempotency_key,
            decisions[1].request.execution.idempotency_key
        );
    }
}
