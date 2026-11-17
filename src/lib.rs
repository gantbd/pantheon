//! Host-owned layout and focus routing for independent engine presentations.
#![forbid(unsafe_code)]

mod shell;
mod workbench;

pub use shell::{Shell, ShellInput, Surface, shell_input};
pub use workbench::{
    ActionMetric, ActivityEntry, ActivityLevel, ActivityLog, CommandKind, CommandPalette,
    CommandSpec, Instrumentation, OperationTrace, PaletteOutcome, TraceStage, draw_activity,
    draw_command_palette, draw_help, draw_inspector,
};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use vellum_tui::EditorFrame;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Focus {
    Chronos,
    Vellum,
}

impl Focus {
    pub const fn other(self) -> Self {
        match self {
            Self::Chronos => Self::Vellum,
            Self::Vellum => Self::Chronos,
        }
    }
}

pub enum RoutedInput {
    Exit,
    SwitchFocus,
    Chronos(KeyEvent),
    Vellum(vellum_tui::Input),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitDecision {
    Exit,
    ConfirmDiscard,
    RefuseIndeterminate,
}

/// The composition root may terminate only after preserving Vellum's explicit
/// uncertainty. Dirty state requires one local confirmation; an indeterminate
/// effect is never converted into discard or success.
pub const fn decide_exit(
    persistence: vellum_sdk_client::workspace::PersistenceProjectionV1,
    discard_confirmed: bool,
) -> ExitDecision {
    use vellum_sdk_client::workspace::PersistenceProjectionV1;
    match persistence {
        PersistenceProjectionV1::Indeterminate => ExitDecision::RefuseIndeterminate,
        PersistenceProjectionV1::Dirty if !discard_confirmed => ExitDecision::ConfirmDiscard,
        PersistenceProjectionV1::Unknown
        | PersistenceProjectionV1::Clean
        | PersistenceProjectionV1::Dirty => ExitDecision::Exit,
    }
}

/// Routes one terminal event without granting either pane authority over the
/// other. The composition root alone owns global exit and focus transfer.
pub fn route_key(key: KeyEvent, focus: Focus) -> RoutedInput {
    if key.kind == KeyEventKind::Press
        && key.modifiers == KeyModifiers::CONTROL
        && key.code == KeyCode::Char('q')
    {
        return RoutedInput::Exit;
    }
    if key.kind == KeyEventKind::Press
        && key.modifiers == KeyModifiers::NONE
        && key.code == KeyCode::Tab
    {
        return RoutedInput::SwitchFocus;
    }
    match focus {
        Focus::Chronos => RoutedInput::Chronos(key),
        Focus::Vellum => RoutedInput::Vellum(vellum_tui::translate_key(key)),
    }
}

/// Draws two independent presentation peers under one host-owned frame. This
/// function owns geometry only; it cannot read or mutate either engine.
pub fn draw(
    frame: &mut Frame<'_>,
    chronos: &chronos_tui::Hub,
    vellum: Option<&EditorFrame>,
    focus: Focus,
    notice: &str,
) {
    draw_shell(
        frame,
        &Shell::dual_peer(),
        Some(chronos),
        vellum,
        None,
        focus,
        notice,
    );
}

/// Draws the host navigation and the selected explicit presentation surface.
/// Missing peers remain visible gaps; the host never fabricates their state.
pub fn draw_shell(
    frame: &mut Frame<'_>,
    shell: &Shell,
    chronos: Option<&chronos_tui::Hub>,
    vellum: Option<&EditorFrame>,
    agora: Option<&agora_tui::Component>,
    focus: Focus,
    notice: &str,
) {
    draw_shell_with_moira(
        frame,
        shell,
        PeerViews {
            chronos,
            vellum,
            vellum_atomic_replace: None,
            vellum_durable_save: None,
            agora,
            agora_create: None,
            agora_message: None,
            moira: None,
            moira_reconciliation: None,
            daimon: None,
            daimon_agent: None,
            daimon_permission: None,
            daimon_prompt: None,
            daimon_control: None,
            hermes: None,
            hermes_compaction: None,
            hermes_disclosure: None,
            hermes_acknowledge: None,
            aegis: None,
            aegis_managed: None,
            aegis_revoke: None,
            grounding: None,
            operations: None,
            health: None,
        },
        focus,
        notice,
    );
}

pub struct PeerViews<'a> {
    pub chronos: Option<&'a chronos_tui::Hub>,
    pub vellum: Option<&'a EditorFrame>,
    pub vellum_atomic_replace: Option<&'a vellum_tui::AtomicReplaceComponent>,
    pub vellum_durable_save: Option<&'a vellum_tui::DurableSaveBatchComponent>,
    pub agora: Option<&'a agora_tui::Component>,
    pub agora_create: Option<&'a agora_tui::CreateRoomComponent>,
    pub agora_message: Option<&'a agora_tui::ComposeMessageComponent>,
    pub moira: Option<&'a moira_tui::Component>,
    pub moira_reconciliation: Option<&'a moira_tui::ReconciliationBatchComponent>,
    pub daimon: Option<&'a daimon_tui::Component>,
    pub daimon_agent: Option<&'a daimon_tui::AgentSetComponent>,
    pub daimon_permission: Option<&'a daimon_tui::AgentPermissionComposer>,
    pub daimon_prompt: Option<&'a daimon_tui::AgentPromptComposer>,
    pub daimon_control: Option<&'a daimon_tui::AgentControlBatchComponent>,
    pub hermes: Option<&'a hermes_tui::Component>,
    pub hermes_compaction: Option<&'a hermes_tui::CompactionComponent>,
    pub hermes_disclosure: Option<&'a hermes_tui::DisclosureComponent>,
    pub hermes_acknowledge: Option<&'a hermes_tui::AcknowledgeComponent>,
    pub aegis: Option<&'a aegis_tui::Component>,
    pub aegis_managed: Option<&'a aegis_tui::ManagedAuthorityComponent>,
    pub aegis_revoke: Option<&'a aegis_tui::RevokeEpochComponent>,
    pub grounding: Option<&'a tellus::Component>,
    pub operations: Option<&'a OperationsView>,
    pub health: Option<&'a SuiteHealthView>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeerHealthState {
    Unobserved,
    Observing,
    Current,
    Degraded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeerHealthView {
    pub peer: &'static str,
    pub channel: &'static str,
    pub state: PeerHealthState,
    pub age_millis: Option<u64>,
    pub busy: bool,
    pub uncertain: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SuiteHealthView {
    pub peers: Vec<PeerHealthView>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationsReceiptView {
    pub peer: String,
    pub disposition: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationsView {
    pub lifecycle: String,
    pub reconfiguration_allowed: bool,
    pub hosted_peers: Vec<String>,
    pub drain_pending: bool,
    pub drain_armed: bool,
    pub receipts: Vec<OperationsReceiptView>,
    pub notice: String,
}

/// Draws the shell with an independently hosted Moira workflow peer.
/// This additive entry point preserves the original three-peer host contract.
pub fn draw_shell_with_moira(
    frame: &mut Frame<'_>,
    shell: &Shell,
    peers: PeerViews<'_>,
    focus: Focus,
    notice: &str,
) {
    let PeerViews {
        chronos,
        vellum,
        vellum_atomic_replace,
        vellum_durable_save,
        agora,
        agora_create,
        agora_message,
        moira,
        moira_reconciliation,
        daimon,
        daimon_agent,
        daimon_permission,
        daimon_prompt,
        daimon_control,
        hermes,
        hermes_compaction,
        hermes_disclosure,
        hermes_acknowledge,
        aegis,
        aegis_managed,
        aegis_revoke,
        grounding,
        operations,
        health,
    } = peers;
    let [brand, tabs, body, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " PANTHEON ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", shell.active().label()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  •  local operations workbench",
                Style::default().fg(Color::DarkGray),
            ),
        ])),
        brand,
    );
    let tab_line = shell
        .surfaces()
        .iter()
        .map(|surface| {
            if *surface == shell.active() {
                Span::styled(
                    format!(" {} ", surface.label()),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!(" {} ", surface.label()),
                    Style::default().fg(Color::DarkGray),
                )
            }
        })
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(Line::from(tab_line)), tabs);
    let [chronos_area, vellum_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(body);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" {notice}"),
                Style::default().fg(Color::LightCyan),
            ),
            Span::styled(
                format!(
                    "  │  focus={focus:?}  │  Ctrl-P actions  F1 help  Ctrl-G activity  [/] surfaces  Ctrl-Q exit"
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ])),
        status,
    );
    match shell.active() {
        Surface::Workspace => {
            if let Some(chronos) = chronos {
                chronos.draw_in(frame, chronos_area);
            } else {
                frame.render_widget(
                    Paragraph::new("Chronos service not configured")
                        .block(Block::default().borders(Borders::ALL).title("Chronos")),
                    chronos_area,
                );
            }
            if let Some(control) = vellum_atomic_replace {
                control.draw_in(frame, vellum_area, true);
            } else if let Some(control) = vellum_durable_save {
                control.draw_in(frame, vellum_area, true);
            } else if let Some(vellum) = vellum {
                vellum_tui::draw_in(frame, vellum_area, vellum, None);
            } else {
                vellum_tui::draw_empty_in(
                    frame,
                    vellum_area,
                    Some("Vellum service not configured"),
                );
            }
        }
        Surface::Overview => draw_overview(
            frame,
            body,
            shell,
            PeerAvailability {
                chronos: chronos.is_some(),
                vellum: vellum.is_some(),
                agora: agora.is_some(),
                moira: moira.is_some(),
                daimon: daimon.is_some() || daimon_agent.is_some(),
                hermes: hermes.is_some(),
                aegis: aegis.is_some(),
                grounding: grounding.is_some(),
            },
            health,
        ),
        Surface::Agora if agora_message.is_some() => {
            agora_message
                .expect("checked above")
                .draw_in(frame, body, true);
        }
        Surface::Agora if agora_create.is_some() => {
            agora_create
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Agora if agora.is_some() => {
            agora.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Moira if moira_reconciliation.is_some() => {
            moira_reconciliation
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Moira if moira.is_some() => {
            moira.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Daimon if daimon_permission.is_some() => {
            daimon_permission
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Daimon if daimon_prompt.is_some() => {
            daimon_prompt
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Daimon if daimon_control.is_some() => {
            daimon_control
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Daimon if daimon.is_some() && daimon_agent.is_some() => {
            let [presence, agent] =
                Layout::horizontal([Constraint::Percentage(38), Constraint::Percentage(62)])
                    .areas(body);
            daimon
                .expect("matched above")
                .draw_in(frame, presence, false);
            daimon_agent
                .expect("matched above")
                .draw_in(frame, agent, true);
        }
        Surface::Daimon if daimon_agent.is_some() => {
            daimon_agent
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Daimon if daimon.is_some() => {
            daimon.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Hermes if hermes_compaction.is_some() => {
            hermes_compaction
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Hermes if hermes_disclosure.is_some() => {
            hermes_disclosure
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Hermes if hermes_acknowledge.is_some() => {
            hermes_acknowledge
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Hermes if hermes.is_some() => {
            hermes.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Aegis if aegis_managed.is_some() => {
            aegis_managed
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Aegis if aegis_revoke.is_some() => {
            aegis_revoke
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Aegis if aegis.is_some() => {
            aegis.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Grounding if grounding.is_some() => {
            grounding.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Operations if operations.is_some() => {
            draw_operations(frame, body, operations.expect("matched above"));
        }
        surface => {
            frame.render_widget(
                Paragraph::new(surface.gap().unwrap_or("presentation unavailable")).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(surface.label()),
                ),
                body,
            );
        }
    }
}

fn draw_operations(frame: &mut Frame<'_>, area: ratatui::layout::Rect, view: &OperationsView) {
    let [summary, peers, receipts, footer] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .areas(area);
    let lifecycle_color = if view.lifecycle.contains("ready") {
        Color::Green
    } else if view.lifecycle.contains("stopped") {
        Color::DarkGray
    } else {
        Color::Yellow
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::raw("topology: "),
                Span::styled(
                    view.lifecycle.as_str(),
                    Style::default()
                        .fg(lifecycle_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(format!(
                "reconfiguration: {}",
                if view.reconfiguration_allowed {
                    "allowed"
                } else {
                    "fenced"
                }
            )),
            Line::from(format!(
                "drain: {}",
                if view.drain_pending {
                    "pending"
                } else if view.drain_armed {
                    "ARMED"
                } else {
                    "idle"
                }
            )),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Local topology "),
        ),
        summary,
    );
    let peer_lines = if view.hosted_peers.is_empty() {
        vec![Line::from(
            "No Pantheon-hosted peers; configured peers are externally owned",
        )]
    } else {
        view.hosted_peers
            .iter()
            .map(|peer| Line::from(format!("• {peer}")))
            .collect()
    };
    frame.render_widget(
        Paragraph::new(peer_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Hosted peers "),
        ),
        peers,
    );
    let receipt_lines = if view.receipts.is_empty() {
        vec![Line::from("No retained lifecycle receipts")]
    } else {
        view.receipts
            .iter()
            .map(|receipt| Line::from(format!("{}  {}", receipt.peer, receipt.disposition)))
            .collect()
    };
    frame.render_widget(
        Paragraph::new(receipt_lines)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Drain receipts "),
            ),
        receipts,
    );
    frame.render_widget(
        Paragraph::new(format!(
            "{} • c arm drain • Enter confirm • Ctrl-P actions",
            view.notice
        )),
        footer,
    );
}

struct PeerAvailability {
    chronos: bool,
    vellum: bool,
    agora: bool,
    moira: bool,
    daimon: bool,
    hermes: bool,
    aegis: bool,
    grounding: bool,
}

fn draw_overview(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    shell: &Shell,
    peers: PeerAvailability,
    health: Option<&SuiteHealthView>,
) {
    // Profile-selected gaps are actionable composition evidence. Keep them
    // ahead of the complete peer inventory so a short terminal cannot hide a
    // configured-but-unavailable surface as new peers are added.
    let mut lines = Vec::new();
    for surface in shell.surfaces() {
        let attached = match surface {
            Surface::Agora => peers.agora,
            Surface::Moira => peers.moira,
            Surface::Daimon => peers.daimon,
            Surface::Hermes => peers.hermes,
            Surface::Aegis => peers.aegis,
            Surface::Grounding => peers.grounding,
            Surface::Overview | Surface::Workspace | Surface::Operations => true,
        };
        if !attached && let Some(gap) = surface.gap() {
            lines.push(Line::from(format!("{}: gap — {gap}", surface.label())));
        }
    }
    if let Some(health) = health {
        let current = health
            .peers
            .iter()
            .filter(|peer| peer.state == PeerHealthState::Current)
            .count();
        let observing = health
            .peers
            .iter()
            .filter(|peer| peer.state == PeerHealthState::Observing)
            .count();
        let degraded = health
            .peers
            .iter()
            .filter(|peer| peer.state == PeerHealthState::Degraded)
            .count();
        let unobserved = health.peers.len() - current - observing - degraded;
        lines.push(Line::from(format!(
            "health: {current} current · {observing} observing · {degraded} degraded · {unobserved} unobserved"
        )));
        lines.push(Line::from(""));
        for peer in &health.peers {
            let symbol = match peer.state {
                PeerHealthState::Unobserved => "○",
                PeerHealthState::Observing => "…",
                PeerHealthState::Current => "✓",
                PeerHealthState::Degraded => "!",
            };
            let state = match peer.state {
                PeerHealthState::Unobserved => "configured; no settled observation",
                PeerHealthState::Observing => "observing",
                PeerHealthState::Current => "current",
                PeerHealthState::Degraded => "degraded; current projection unavailable",
            };
            let age = peer
                .age_millis
                .map_or(String::new(), |age| format!(" · evidence {age}ms ago"));
            let busy = if peer.busy { " · busy" } else { "" };
            let uncertain = if peer.uncertain {
                " · OUTCOME UNCERTAIN"
            } else {
                ""
            };
            lines.push(Line::from(format!(
                "{symbol} {} / {}: {state}{age}{busy}{uncertain}",
                peer.peer, peer.channel
            )));
        }
        lines.push(Line::from(""));
    }
    if health.is_none() {
        lines.extend([
            Line::from(format!(
                "Chronos: {}",
                if peers.chronos {
                    "attached"
                } else {
                    "not configured"
                }
            )),
            Line::from(format!(
                "Vellum: {}",
                if peers.vellum {
                    "attached"
                } else {
                    "not configured"
                }
            )),
            Line::from(format!(
                "Agora: {}",
                if peers.agora {
                    "attached"
                } else {
                    "not configured"
                }
            )),
            Line::from(format!(
                "Moira: {}",
                if peers.moira {
                    "attached"
                } else {
                    "not configured"
                }
            )),
            Line::from(format!(
                "Daimon: {}",
                if peers.daimon {
                    "attached"
                } else {
                    "not configured"
                }
            )),
            Line::from(format!(
                "Hermes: {}",
                if peers.hermes {
                    "attached"
                } else {
                    "not configured"
                }
            )),
            Line::from(format!(
                "Aegis: {}",
                if peers.aegis {
                    "attached"
                } else {
                    "not configured"
                }
            )),
            Line::from(format!(
                "Grounding: {}",
                if peers.grounding {
                    "attached"
                } else {
                    "not configured"
                }
            )),
        ]);
    }
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Workspace observations"),
        ),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_client::ServicePort as AegisServicePort;
    use aegis_protocol::{
        AuthorityStatusV1, CommandV1 as AegisCommandV1, OutcomeV1 as AegisOutcomeV1,
        ReplyV1 as AegisReplyV1, SessionHandleV1 as AegisSessionHandleV1,
    };
    use agora_client::v2::{ClientV2, PortFailureV2, ServicePortV2};
    use agora_protocol::v2::{ClientIdV2, OutcomeV2, ReplyV2, RequestV2, RoomPageV2};
    use daimon_client::ServicePort as DaimonServicePort;
    use daimon_protocol::{
        CommandV1 as DaimonCommandV1, OutcomeV1 as DaimonOutcomeV1, ReplyV1 as DaimonReplyV1,
        SessionHandleV1 as DaimonSessionHandleV1,
    };
    use moira_client::SagaServicePort;
    use moira_protocol::{SagaCensusV1, SagaCommandV1, SagaOutcomeV1, SagaPageV1, SagaReplyV1};
    use ratatui::{Terminal, backend::TestBackend};
    use std::{thread, time::Duration};

    struct AgoraPort;

    struct MoiraPort;
    struct DaimonPort;
    struct AegisPort;

    impl AegisServicePort for AegisPort {
        type Error = ();

        fn execute(
            &mut self,
            _: AegisSessionHandleV1,
            command: AegisCommandV1,
        ) -> Result<AegisReplyV1, Self::Error> {
            let outcome = match command {
                AegisCommandV1::Status => AegisOutcomeV1::Status(AuthorityStatusV1 {
                    epoch: 7,
                    active_grants: 0,
                    retained_decisions: 0,
                }),
                AegisCommandV1::GrantPage { .. } => AegisOutcomeV1::GrantPage {
                    entries: Vec::new(),
                    next_after: None,
                },
                AegisCommandV1::DecisionPage { .. } => AegisOutcomeV1::DecisionPage {
                    entries: Vec::new(),
                    next_after: None,
                },
                _ => panic!("read-only Aegis presentation attempted a mutation"),
            };
            Ok(AegisReplyV1::Accepted(outcome))
        }
    }
    impl DaimonServicePort for DaimonPort {
        type Error = ();

        fn execute(
            &mut self,
            _: DaimonSessionHandleV1,
            command: DaimonCommandV1,
        ) -> Result<DaimonReplyV1, Self::Error> {
            assert!(matches!(command, DaimonCommandV1::ReadPresence { .. }));
            Ok(DaimonReplyV1::Accepted(DaimonOutcomeV1::Presence {
                entries: Vec::new(),
                truncated: false,
            }))
        }
    }

    impl SagaServicePort for MoiraPort {
        type Error = ();

        fn execute(&mut self, command: SagaCommandV1) -> Result<SagaReplyV1, Self::Error> {
            let outcome = match command {
                SagaCommandV1::Inspect { .. } => SagaOutcomeV1::Page(SagaPageV1 {
                    entries: Vec::new(),
                    next_after: None,
                }),
                SagaCommandV1::Census => SagaOutcomeV1::Census(SagaCensusV1 {
                    total: 0,
                    runnable_effects: 0,
                    awaiting_decision: 0,
                    fenced: 0,
                    terminal: 0,
                    retries_consumed: 0,
                    saturated: false,
                }),
                _ => panic!("read-only Moira presentation attempted an unexpected command"),
            };
            Ok(SagaReplyV1::Accepted(outcome))
        }
    }

    impl ServicePortV2 for AgoraPort {
        type Error = ();

        fn execute(
            &mut self,
            _: ClientIdV2,
            _: RequestV2,
        ) -> Result<ReplyV2, PortFailureV2<Self::Error>> {
            Ok(ReplyV2::Accepted(OutcomeV2::Rooms(RoomPageV2 {
                revision: 3,
                rooms: Vec::new(),
                next: None,
            })))
        }
    }

    #[test]
    fn one_frame_contains_both_independent_presentations() {
        let chronos = chronos_tui::Hub::new(
            vec![chronos_tui::Workstream {
                name: "Chronos".into(),
                directory: "/nonexistent".into(),
            }],
            false,
            false,
        )
        .unwrap();
        let vellum = EditorFrame {
            lines: vec!["vellum source".into()],
            selection_cells: vec![Vec::new()],
            selection_count: 1,
            cursor: None,
            status: "Ready".into(),
            active: 0,
            documents: 1,
            dirty: false,
            editing_locked: false,
        };
        let mut terminal = Terminal::new(TestBackend::new(120, 16)).unwrap();
        terminal
            .draw(|frame| draw(frame, &chronos, Some(&vellum), Focus::Vellum, "qualified"))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("focus=Vellum"));
        assert!(rendered.contains("CHRONOS"));
        assert!(rendered.contains("Vellum"));
        assert!(rendered.contains("vellum source"));
    }

    #[test]
    fn shell_exposes_gaps_without_inventing_peer_state() {
        let shell = Shell::new(vec![Surface::Overview, Surface::Agora]).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(100, 12)).unwrap();
        terminal
            .draw(|frame| {
                draw_shell(
                    frame,
                    &shell,
                    None,
                    None,
                    None,
                    Focus::Chronos,
                    "profile loaded",
                )
            })
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("Agora: gap"));
        assert!(rendered.contains("no composable Ratatui"));
        assert!(rendered.contains("Chronos: not configured"));
    }

    #[test]
    fn overview_distinguishes_evidence_health_busy_and_uncertain_state() {
        let shell = Shell::new(vec![Surface::Overview]).unwrap();
        let health = SuiteHealthView {
            peers: vec![
                PeerHealthView {
                    peer: "chronos",
                    channel: "tasks",
                    state: PeerHealthState::Current,
                    age_millis: Some(12),
                    busy: false,
                    uncertain: false,
                },
                PeerHealthView {
                    peer: "daimon",
                    channel: "sessions/events",
                    state: PeerHealthState::Observing,
                    age_millis: Some(40),
                    busy: true,
                    uncertain: true,
                },
                PeerHealthView {
                    peer: "hermes",
                    channel: "ledger/cursors",
                    state: PeerHealthState::Degraded,
                    age_millis: Some(90),
                    busy: false,
                    uncertain: false,
                },
            ],
        };
        let mut terminal = Terminal::new(TestBackend::new(120, 16)).unwrap();
        terminal
            .draw(|frame| {
                draw_overview(
                    frame,
                    frame.area(),
                    &shell,
                    PeerAvailability {
                        chronos: true,
                        vellum: false,
                        agora: false,
                        moira: false,
                        daimon: true,
                        hermes: true,
                        aegis: false,
                        grounding: false,
                    },
                    Some(&health),
                );
            })
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("health: 1 current · 1 observing · 1 degraded"));
        assert!(rendered.contains("chronos / tasks: current · evidence 12ms ago"));
        assert!(rendered.contains(
            "daimon / sessions/events: observing · evidence 40ms ago · busy · OUTCOME UNCERTAIN"
        ));
        assert!(rendered.contains("hermes / ledger/cursors: degraded"));
        assert!(!rendered.contains("Chronos: attached"));
    }

    #[test]
    fn shell_renders_host_supplied_agora_peer_instead_of_gap() {
        let shell = Shell::new(vec![Surface::Agora]).unwrap();
        let mut agora = agora_tui::Component::new(ClientV2::new(ClientIdV2(1), AgoraPort));
        for _ in 0..100 {
            agora.poll();
            let mut terminal = Terminal::new(TestBackend::new(100, 12)).unwrap();
            terminal
                .draw(|frame| {
                    draw_shell(
                        frame,
                        &shell,
                        None,
                        None,
                        Some(&agora),
                        Focus::Chronos,
                        "host supplied",
                    )
                })
                .unwrap();
            let rendered: String = terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect();
            if rendered.contains("revision 3") {
                assert!(!rendered.contains("no composable Ratatui"));
                return;
            }
            thread::sleep(Duration::from_millis(1));
        }
        panic!("Agora peer did not render its accepted room page");
    }

    #[test]
    fn shell_renders_host_supplied_moira_peer_instead_of_gap() {
        let shell = Shell::new(vec![Surface::Moira]).unwrap();
        let mut moira = moira_tui::Component::new(MoiraPort);
        for _ in 0..100 {
            moira.poll();
            let mut terminal = Terminal::new(TestBackend::new(100, 12)).unwrap();
            terminal
                .draw(|frame| {
                    draw_shell_with_moira(
                        frame,
                        &shell,
                        PeerViews {
                            chronos: None,
                            vellum: None,
                            vellum_atomic_replace: None,
                            vellum_durable_save: None,
                            agora: None,
                            agora_create: None,
                            agora_message: None,
                            moira: Some(&moira),
                            moira_reconciliation: None,
                            daimon: None,
                            daimon_agent: None,
                            daimon_permission: None,
                            daimon_prompt: None,
                            daimon_control: None,
                            hermes: None,
                            hermes_compaction: None,
                            hermes_disclosure: None,
                            hermes_acknowledge: None,
                            aegis: None,
                            aegis_managed: None,
                            aegis_revoke: None,
                            grounding: None,
                            operations: None,
                            health: None,
                        },
                        Focus::Chronos,
                        "host supplied",
                    )
                })
                .unwrap();
            let rendered = terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            if rendered.contains("0 saga(s)") {
                assert!(!rendered.contains("workflow presentation is not configured"));
                return;
            }
            thread::sleep(Duration::from_millis(1));
        }
        panic!("Moira peer did not render its accepted saga page");
    }

    #[test]
    fn shell_renders_host_supplied_daimon_peer_instead_of_gap() {
        let shell = Shell::new(vec![Surface::Daimon]).unwrap();
        let mut daimon = daimon_tui::Component::new(DaimonSessionHandleV1(1), DaimonPort);
        for _ in 0..100 {
            daimon.poll();
            let mut terminal = Terminal::new(TestBackend::new(100, 12)).unwrap();
            terminal
                .draw(|frame| {
                    draw_shell_with_moira(
                        frame,
                        &shell,
                        PeerViews {
                            chronos: None,
                            vellum: None,
                            vellum_atomic_replace: None,
                            vellum_durable_save: None,
                            agora: None,
                            agora_create: None,
                            agora_message: None,
                            moira: None,
                            moira_reconciliation: None,
                            daimon: Some(&daimon),
                            daimon_agent: None,
                            daimon_permission: None,
                            daimon_prompt: None,
                            daimon_control: None,
                            hermes: None,
                            hermes_compaction: None,
                            hermes_disclosure: None,
                            hermes_acknowledge: None,
                            aegis: None,
                            aegis_managed: None,
                            aegis_revoke: None,
                            grounding: None,
                            operations: None,
                            health: None,
                        },
                        Focus::Chronos,
                        "host supplied",
                    )
                })
                .unwrap();
            let rendered = terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            if rendered.contains("0 agent(s)") {
                assert!(!rendered.contains("operations presentation is not configured"));
                return;
            }
            thread::sleep(Duration::from_millis(1));
        }
        panic!("Daimon peer did not render its accepted presence page");
    }

    #[test]
    fn shell_renders_host_supplied_aegis_peer_without_mutation_authority() {
        let shell = Shell::new(vec![Surface::Aegis]).unwrap();
        let mut aegis = aegis_tui::Component::new(AegisSessionHandleV1(1), AegisPort);
        for _ in 0..100 {
            aegis.poll();
            let mut terminal = Terminal::new(TestBackend::new(100, 12)).unwrap();
            terminal
                .draw(|frame| {
                    draw_shell_with_moira(
                        frame,
                        &shell,
                        PeerViews {
                            chronos: None,
                            vellum: None,
                            vellum_atomic_replace: None,
                            vellum_durable_save: None,
                            agora: None,
                            agora_create: None,
                            agora_message: None,
                            moira: None,
                            moira_reconciliation: None,
                            daimon: None,
                            daimon_agent: None,
                            daimon_permission: None,
                            daimon_prompt: None,
                            daimon_control: None,
                            hermes: None,
                            hermes_compaction: None,
                            hermes_disclosure: None,
                            hermes_acknowledge: None,
                            aegis: Some(&aegis),
                            aegis_managed: None,
                            aegis_revoke: None,
                            grounding: None,
                            operations: None,
                            health: None,
                        },
                        Focus::Chronos,
                        "host supplied",
                    )
                })
                .unwrap();
            let rendered = terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            if rendered.contains("epoch: 7") {
                assert!(!rendered.contains("authority presentation is not configured"));
                return;
            }
            thread::sleep(Duration::from_millis(1));
        }
        panic!("Aegis peer did not render its accepted authority observation");
    }

    #[test]
    fn host_routes_only_to_focus_and_reserves_global_controls() {
        let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(matches!(
            route_key(tab, Focus::Chronos),
            RoutedInput::SwitchFocus
        ));
        let exit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert!(matches!(route_key(exit, Focus::Vellum), RoutedInput::Exit));
        let text = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(matches!(
            route_key(text, Focus::Chronos),
            RoutedInput::Chronos(key) if key == text
        ));
        assert!(matches!(
            route_key(text, Focus::Vellum),
            RoutedInput::Vellum(vellum_tui::Input::Edit(_))
        ));
    }

    #[test]
    fn exit_preserves_dirty_confirmation_and_indeterminate_effects() {
        use vellum_sdk_client::workspace::PersistenceProjectionV1;
        assert_eq!(
            decide_exit(PersistenceProjectionV1::Clean, false),
            ExitDecision::Exit
        );
        assert_eq!(
            decide_exit(PersistenceProjectionV1::Unknown, false),
            ExitDecision::Exit
        );
        assert_eq!(
            decide_exit(PersistenceProjectionV1::Dirty, false),
            ExitDecision::ConfirmDiscard
        );
        assert_eq!(
            decide_exit(PersistenceProjectionV1::Dirty, true),
            ExitDecision::Exit
        );
        assert_eq!(
            decide_exit(PersistenceProjectionV1::Indeterminate, true),
            ExitDecision::RefuseIndeterminate
        );
    }
}
