//! Host-owned layout and focus routing for independent engine presentations.
#![forbid(unsafe_code)]

mod shell;
mod theme;
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
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table, Tabs, Wrap},
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
    let shell = Shell::new(vec![Surface::Chronos, Surface::Vellum])
        .expect("static presentation surface set is valid");
    draw_shell(frame, &shell, Some(chronos), vellum, None, focus, notice);
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
            moira_participation: None,
            moira_open: None,
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
            configuration: None,
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
    pub moira_participation: Option<&'a moira_tui::ParticipationComponent>,
    pub moira_open: Option<&'a moira_tui::OpenControlComponent>,
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
    pub configuration: Option<&'a [String]>,
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
        moira_participation,
        moira_open,
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
        configuration,
    } = peers;
    frame.render_widget(Block::default().style(theme::base()), frame.area());
    let [brand, tabs, body, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let active = shell.active();
    let accent = theme::surface_accent(active);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " PANTHEON ",
                Style::default()
                    .fg(theme::VOID)
                    .bg(accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}  {}", active.glyph(), active.label().to_uppercase()),
                Style::default()
                    .fg(theme::TEXT)
                    .bg(theme::VOID)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  ──  {}", active.tagline()), theme::muted()),
        ]))
        .style(theme::base()),
        brand,
    );
    let compact_tabs = tabs.width < 86;
    let decorated_tabs = tabs.width >= 132;
    let tab_titles = shell
        .surfaces()
        .iter()
        .map(|surface| {
            if compact_tabs {
                Line::from(format!("{}{}", surface.direct_key(), surface.glyph()))
            } else if decorated_tabs {
                Line::from(format!("{} {}", surface.glyph(), surface.label()))
            } else {
                Line::from(format!("{} {}", surface.direct_key(), surface.label()))
            }
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Tabs::new(tab_titles)
            .select(shell.active_index())
            .divider(Span::styled("  ", theme::muted()))
            .style(theme::muted())
            .highlight_style(
                Style::default()
                    .fg(accent)
                    .bg(theme::PANEL_RAISED)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(theme::FAINT).bg(theme::VOID)),
            ),
        tabs,
    );
    let body = body.inner(Margin {
        horizontal: 1,
        vertical: 0,
    });
    let status_focus = match active {
        Surface::Chronos => Focus::Chronos,
        Surface::Vellum => Focus::Vellum,
        _ => focus,
    };
    draw_status_bar(frame, status, status_focus, notice);
    match shell.active() {
        Surface::Chronos => {
            if let Some(chronos) = chronos {
                chronos.draw_in(frame, body);
            } else {
                frame.render_widget(
                    Paragraph::new("Chronos service not configured")
                        .style(theme::panel())
                        .block(theme::card(" CHRONOS ", theme::BLUE)),
                    body,
                );
            }
        }
        Surface::Vellum => {
            if let Some(control) = vellum_atomic_replace {
                control.draw_in(frame, body, true);
            } else if let Some(control) = vellum_durable_save {
                control.draw_in(frame, body, true);
            } else if let Some(vellum) = vellum {
                vellum_tui::draw_in(frame, body, vellum, None);
            } else {
                vellum_tui::draw_empty_in(frame, body, Some("Vellum service not configured"));
            }
        }
        Surface::Workspace => {
            let [chronos_area, vellum_area] =
                Layout::horizontal([Constraint::Percentage(44), Constraint::Percentage(56)])
                    .spacing(1)
                    .areas(body);
            let chronos_area = draw_workspace_panel(
                frame,
                chronos_area,
                "◷  CHRONOS · PLAN",
                focus == Focus::Chronos,
                theme::BLUE,
            );
            let vellum_area = draw_workspace_panel(
                frame,
                vellum_area,
                "✦  VELLUM · CRAFT",
                focus == Focus::Vellum,
                theme::VIOLET,
            );
            if let Some(chronos) = chronos {
                chronos.draw_in(frame, chronos_area);
            } else {
                frame.render_widget(
                    Paragraph::new("Chronos service not configured")
                        .style(theme::panel())
                        .block(theme::card(" CHRONOS ", theme::BLUE)),
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
                moira: moira.is_some() || moira_participation.is_some(),
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
        Surface::Moira if moira_open.is_some() => {
            moira_open
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Moira if moira_reconciliation.is_some() => {
            moira_reconciliation
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Moira if moira.is_some() && moira_participation.is_some() => {
            let [sagas, participation] =
                Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
                    .spacing(1)
                    .areas(body);
            moira.expect("matched above").draw_in(frame, sagas, false);
            moira_participation
                .expect("matched above")
                .draw_in(frame, participation, true);
        }
        Surface::Moira if moira_participation.is_some() => {
            moira_participation
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
        Surface::Configuration => {
            draw_configuration(frame, body, configuration.unwrap_or(&[]));
        }
        surface => {
            draw_unconfigured_surface(frame, body, surface);
        }
    }
}

/// Resolves a click in the two-row surface rail using the same title geometry
/// as [`draw_shell_with_moira`]. The host owns only shell hit-testing; peer
/// widgets remain keyboard-driven until their presentation APIs expose mouse
/// intents explicitly.
pub fn surface_tab_at(shell: &Shell, width: u16, column: u16, row: u16) -> Option<Surface> {
    if !(1..=2).contains(&row) || column >= width {
        return None;
    }
    let compact = width < 86;
    let decorated = width >= 132;
    let mut start = 0usize;
    let column = usize::from(column);
    for surface in shell.surfaces() {
        let title = if compact {
            format!("{}{}", surface.direct_key(), surface.glyph())
        } else if decorated {
            format!("{} {}", surface.glyph(), surface.label())
        } else {
            format!("{} {}", surface.direct_key(), surface.label())
        };
        let end = start + Line::from(title).width();
        if (start..end).contains(&column) {
            return Some(*surface);
        }
        // Ratatui's configured divider is two spaces.
        start = end + 2;
    }
    None
}

fn draw_configuration(frame: &mut Frame<'_>, area: Rect, lines: &[String]) {
    let [intro, facts_area] = Layout::vertical([Constraint::Length(4), Constraint::Min(4)])
        .spacing(1)
        .areas(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                "Session truth, not ambient machine discovery.",
                Style::default()
                    .fg(theme::GREEN)
                    .bg(theme::PANEL)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::styled(
                "Secrets and grant handles stay redacted. Ctrl-P → preflight restart changes the profile deliberately.",
                theme::muted(),
            ),
        ])
        .wrap(Wrap { trim: true })
        .block(theme::card(" CONFIGURATION · KNOW YOUR BOUNDARIES ", theme::GREEN)),
        intro,
    );
    let facts = if lines.is_empty() {
        vec![Line::styled(
            "No session configuration projection is available.",
            theme::muted(),
        )]
    } else {
        lines
            .iter()
            .map(|line| {
                let (label, value) = line.split_once("  ").unwrap_or((line.as_str(), ""));
                Line::from(vec![
                    Span::styled(
                        format!("{label:<20}"),
                        Style::default()
                            .fg(theme::CYAN)
                            .bg(theme::PANEL)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        value.trim(),
                        Style::default().fg(theme::TEXT).bg(theme::PANEL),
                    ),
                ])
            })
            .collect()
    };
    frame.render_widget(
        Paragraph::new(facts)
            .wrap(Wrap { trim: false })
            .block(theme::card(" LIVE SESSION FACTS ", theme::CYAN)),
        facts_area,
    );
}

fn draw_status_bar(frame: &mut Frame<'_>, area: Rect, focus: Focus, notice: &str) {
    let [message, shortcuts] =
        Layout::horizontal([Constraint::Percentage(48), Constraint::Percentage(52)]).areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(theme::CYAN).bg(theme::VOID)),
            Span::styled(notice, Style::default().fg(theme::TEXT).bg(theme::VOID)),
        ]))
        .style(theme::base()),
        message,
    );
    let hints = if area.width >= 132 {
        format!("⌃P act · F1 help · ⌃G log · ⌃0 home · ⌃, cfg · ⌃Q · focus={focus:?} ")
    } else {
        format!("⌃P actions  ·  F1 help  ·  [ ] tabs  ·  ⌃Q exit  ·  focus={focus:?} ")
    };
    frame.render_widget(
        Paragraph::new(hints)
            .alignment(Alignment::Right)
            .style(theme::muted()),
        shortcuts,
    );
}

fn draw_workspace_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    active: bool,
    accent: Color,
) -> Rect {
    let border = if active { accent } else { theme::FAINT };
    let marker = if active { "  ● FOCUSED " } else { "" };
    let block = theme::card(title, accent)
        .border_style(Style::default().fg(border).bg(theme::PANEL))
        .title_bottom(Line::styled(
            marker,
            Style::default()
                .fg(accent)
                .bg(theme::PANEL)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner
}

fn draw_unconfigured_surface(frame: &mut Frame<'_>, area: Rect, surface: Surface) {
    let accent = theme::surface_accent(surface);
    let (noun, workflows, profile_hint) = match surface {
        Surface::Agora => (
            "conversation workspace",
            [
                "Browse rooms and bounded message history",
                "Compose revision-bound messages",
                "Create rooms through explicit owner control",
            ],
            r#"profile \"agora\" · socket + participant"#,
        ),
        Surface::Moira => (
            "saga operations desk",
            [
                "Inspect saga state, effects, and receipts",
                "Review exact reconciliation proposals",
                "Recover uncertainty without blind retry",
            ],
            r#"profile \"moira\" · socket (+ host when owned)"#,
        ),
        Surface::Daimon => (
            "agent control room",
            [
                "Observe presence, sessions, and event evidence",
                "Prompt or advance selected sessions",
                "Review permissions and guarded cancellation",
            ],
            r#"profile \"daimon\" · agent socket + selection"#,
        ),
        Surface::Grounding => (
            "evidence workbench",
            [
                "Inspect sources, claims, and provenance",
                "Track freshness and bounded load state",
                "Refresh through the typed presentation endpoint",
            ],
            r#"profile \"grounding\" · endpoint (+ token)"#,
        ),
        Surface::Aegis => (
            "authority console",
            [
                "Inspect grants and decision evidence",
                "Page through bounded authority state",
                "Manage or revoke with explicit admin control",
            ],
            r#"profile \"aegis\" · socket (+ administrator host)"#,
        ),
        Surface::Hermes => (
            "ledger observatory",
            [
                "Inspect streams, cursors, and retention state",
                "Disclose bounded event previews",
                "Acknowledge or compact exact guarded ranges",
            ],
            r#"profile \"hermes\" · socket + stream/consumer"#,
        ),
        _ => (
            "presentation peer",
            [
                "Attach the peer-owned presentation component",
                "Refresh owner-published state",
                "Use the action palette for guarded work",
            ],
            "select and configure this surface in the workspace profile",
        ),
    };
    let [hero, content, footer] = Layout::vertical([
        Constraint::Length(6),
        Constraint::Min(9),
        Constraint::Length(3),
    ])
    .spacing(1)
    .areas(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    format!(" {}  {} ", surface.glyph(), surface.label().to_uppercase()),
                    Style::default()
                        .fg(theme::VOID)
                        .bg(accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  AWAITING CONNECTION", theme::muted()),
            ]),
            Line::from(""),
            Line::styled(
                format!("This {noun} is ready; its peer is not attached to this session."),
                theme::panel(),
            ),
        ])
        .wrap(Wrap { trim: true })
        .block(theme::card(" OPERATIONAL LANDING ", accent)),
        hero,
    );
    let workflow_lines = workflows
        .into_iter()
        .flat_map(|workflow| {
            [
                Line::from(vec![
                    Span::styled(" ◇  ", Style::default().fg(accent).bg(theme::PANEL)),
                    Span::styled(workflow, theme::panel()),
                ]),
                Line::from(""),
            ]
        })
        .collect::<Vec<_>>();
    let connection = vec![
        Line::styled(
            "CONNECTION CONTRACT",
            Style::default()
                .fg(theme::CYAN)
                .bg(theme::PANEL)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::styled(profile_hint, theme::panel()),
        Line::from(""),
        Line::styled(
            "Pantheon renders owner-published state here; it never fabricates readiness or authority.",
            Style::default().fg(theme::MUTED).bg(theme::PANEL),
        ),
    ];
    if content.width >= 88 {
        let [workflows_area, connection_area] =
            Layout::horizontal([Constraint::Percentage(56), Constraint::Percentage(44)])
                .spacing(1)
                .areas(content);
        frame.render_widget(
            Paragraph::new(workflow_lines).block(theme::card(" AVAILABLE WORKFLOWS ", accent)),
            workflows_area,
        );
        frame.render_widget(
            Paragraph::new(connection)
                .wrap(Wrap { trim: true })
                .block(theme::card(" CONNECT THIS PEER ", theme::CYAN)),
            connection_area,
        );
    } else {
        let [workflows_area, connection_area] =
            Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)])
                .areas(content);
        frame.render_widget(
            Paragraph::new(workflow_lines).block(theme::card(" WORKFLOWS ", accent)),
            workflows_area,
        );
        frame.render_widget(
            Paragraph::new(connection)
                .wrap(Wrap { trim: true })
                .block(theme::card(" CONNECT ", theme::CYAN)),
            connection_area,
        );
    }
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Ctrl-P ", theme::key()),
            Span::styled(" inspect actions  ·  ", theme::muted()),
            Span::styled("Configuration", Style::default().fg(accent)),
            Span::styled(" shows the exact session topology", theme::muted()),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(theme::muted()),
        ),
        footer,
    );
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
        theme::GREEN
    } else if view.lifecycle.contains("stopped") {
        theme::MUTED
    } else {
        theme::YELLOW
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("◆  topology  ", theme::muted()),
                Span::styled(
                    view.lifecycle.as_str(),
                    Style::default()
                        .fg(lifecycle_color)
                        .bg(theme::PANEL)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("◇  reconfigure  ", theme::muted()),
                Span::styled(
                    if view.reconfiguration_allowed {
                        "allowed"
                    } else {
                        "fenced"
                    },
                    Style::default()
                        .fg(if view.reconfiguration_allowed {
                            theme::GREEN
                        } else {
                            theme::YELLOW
                        })
                        .bg(theme::PANEL),
                ),
            ]),
            Line::from(vec![
                Span::styled("▽  drain  ", theme::muted()),
                Span::styled(
                    if view.drain_pending {
                        "pending"
                    } else if view.drain_armed {
                        "ARMED"
                    } else {
                        "idle"
                    },
                    Style::default()
                        .fg(if view.drain_pending || view.drain_armed {
                            theme::YELLOW
                        } else {
                            theme::MUTED
                        })
                        .bg(theme::PANEL)
                        .add_modifier(if view.drain_armed {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
            ]),
        ])
        .style(theme::panel())
        .block(theme::card(" LOCAL TOPOLOGY ", theme::CYAN)),
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
        Paragraph::new(peer_lines)
            .style(theme::panel())
            .block(theme::card(" HOSTED PEERS ", theme::VIOLET)),
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
            .style(theme::panel())
            .wrap(Wrap { trim: false })
            .block(theme::card(" DRAIN RECEIPTS ", theme::GREEN)),
        receipts,
    );
    frame.render_widget(
        Paragraph::new(format!(
            "{} • c arm drain • Enter confirm • Ctrl-P actions",
            view.notice
        ))
        .style(theme::muted()),
        footer,
    );
}

#[derive(Clone, Copy)]
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
    let mut gaps = Vec::new();
    for surface in shell.surfaces() {
        let attached = match surface {
            Surface::Agora => peers.agora,
            Surface::Moira => peers.moira,
            Surface::Daimon => peers.daimon,
            Surface::Hermes => peers.hermes,
            Surface::Aegis => peers.aegis,
            Surface::Grounding => peers.grounding,
            Surface::Overview
            | Surface::Chronos
            | Surface::Vellum
            | Surface::Workspace
            | Surface::Operations
            | Surface::Configuration => true,
        };
        if !attached && let Some(gap) = surface.gap() {
            gaps.push((*surface, gap));
        }
    }
    if area.height < 14 {
        draw_overview_compact(frame, area, &gaps, peers, health);
        return;
    }
    let gap_height = if gaps.is_empty() {
        0
    } else {
        u16::try_from(gaps.len().min(3) + 2).unwrap_or(5)
    };
    let [hero, gap_area, detail] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(gap_height),
        Constraint::Min(3),
    ])
    .spacing(if gaps.is_empty() { 0 } else { 1 })
    .areas(area);

    draw_overview_hero(frame, hero, health);
    if !gaps.is_empty() {
        let lines = gaps
            .iter()
            .map(|(surface, gap)| {
                Line::from(vec![
                    Span::styled(
                        format!("{} {}: gap  ", surface.glyph(), surface.label()),
                        Style::default()
                            .fg(theme::surface_accent(*surface))
                            .bg(theme::PANEL)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(*gap, Style::default().fg(theme::MUTED).bg(theme::PANEL)),
                ])
            })
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: true })
                .block(theme::card(
                    " COMPOSITION GAPS · VISIBLE BY DESIGN ",
                    theme::YELLOW,
                )),
            gap_area,
        );
    }

    if let Some(health) = health {
        draw_health_detail(frame, detail, health);
    } else {
        draw_availability_detail(frame, detail, peers);
    }
}

fn draw_overview_compact(
    frame: &mut Frame<'_>,
    area: Rect,
    gaps: &[(Surface, &str)],
    peers: PeerAvailability,
    health: Option<&SuiteHealthView>,
) {
    let mut lines = gaps
        .iter()
        .map(|(surface, gap)| {
            Line::from(vec![
                Span::styled(
                    format!("{} {}: gap", surface.glyph(), surface.label()),
                    Style::default()
                        .fg(theme::surface_accent(*surface))
                        .bg(theme::PANEL)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" — {gap}"),
                    Style::default().fg(theme::MUTED).bg(theme::PANEL),
                ),
            ])
        })
        .collect::<Vec<_>>();
    if let Some(health) = health {
        let counts = health_counts(health);
        lines.push(Line::styled(
            format!(
                "health: {} current · {} observing · {} degraded · {} unobserved",
                counts.0, counts.1, counts.2, counts.3
            ),
            Style::default().fg(theme::TEXT).bg(theme::PANEL),
        ));
        lines.extend(health.peers.iter().map(|peer| {
            let (symbol, state, color) = health_state_style(peer.state);
            let age = peer
                .age_millis
                .map_or(String::new(), |age| format!(" · evidence {age}ms ago"));
            let busy = if peer.busy { " · busy" } else { "" };
            let uncertain = if peer.uncertain {
                " · OUTCOME UNCERTAIN"
            } else {
                ""
            };
            Line::styled(
                format!(
                    "{symbol} {} / {}: {state}{age}{busy}{uncertain}",
                    peer.peer, peer.channel
                ),
                Style::default().fg(color).bg(theme::PANEL),
            )
        }));
    } else {
        for (name, attached) in [
            ("Chronos", peers.chronos),
            ("Vellum", peers.vellum),
            ("Agora", peers.agora),
            ("Moira", peers.moira),
            ("Daimon", peers.daimon),
            ("Hermes", peers.hermes),
            ("Aegis", peers.aegis),
            ("Grounding", peers.grounding),
        ] {
            lines.push(Line::styled(
                format!(
                    "{name}: {}",
                    if attached {
                        "attached"
                    } else {
                        "not configured"
                    }
                ),
                Style::default()
                    .fg(if attached { theme::GREEN } else { theme::MUTED })
                    .bg(theme::PANEL),
            ));
        }
    }
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(theme::card(" OVERVIEW · COMPACT SIGNAL ", theme::CYAN)),
        area,
    );
}

fn health_counts(health: &SuiteHealthView) -> (usize, usize, usize, usize) {
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
    (
        current,
        observing,
        degraded,
        health.peers.len() - current - observing - degraded,
    )
}

fn draw_overview_hero(frame: &mut Frame<'_>, area: Rect, health: Option<&SuiteHealthView>) {
    let (current, observing, degraded, unobserved) = health.map_or((0, 0, 0, 0), health_counts);
    let cards = Layout::horizontal([
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
    ])
    .spacing(1)
    .split(area);
    for (index, (value, label, glyph, color)) in [
        (current, "CURRENT", "✓", theme::GREEN),
        (observing, "OBSERVING", "◌", theme::BLUE),
        (degraded, "DEGRADED", "!", theme::RED),
        (unobserved, "UNOBSERVED", "○", theme::MUTED),
    ]
    .into_iter()
    .enumerate()
    {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{glyph} {value}"),
                    Style::default()
                        .fg(color)
                        .bg(theme::PANEL)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {label}"),
                    Style::default().fg(theme::MUTED).bg(theme::PANEL),
                ),
            ]))
            .alignment(Alignment::Center)
            .block(theme::card("", color)),
            cards[index],
        );
    }
}

fn draw_health_detail(frame: &mut Frame<'_>, area: Rect, health: &SuiteHealthView) {
    let [table_area, posture_area] = if area.width >= 96 {
        Layout::horizontal([Constraint::Percentage(68), Constraint::Percentage(32)])
            .spacing(1)
            .areas(area)
    } else {
        Layout::vertical([Constraint::Percentage(68), Constraint::Percentage(32)])
            .spacing(1)
            .areas(area)
    };
    let rows = health.peers.iter().map(|peer| {
        let (symbol, state, color) = health_state_style(peer.state);
        let age = peer
            .age_millis
            .map_or_else(|| "—".to_owned(), |age| format!("{age} ms"));
        let flags = match (peer.busy, peer.uncertain) {
            (_, true) => "OUTCOME UNCERTAIN",
            (true, false) => "busy",
            (false, false) => "quiet",
        };
        Row::new(vec![
            format!("{symbol} {}", peer.peer),
            peer.channel.to_owned(),
            state.to_owned(),
            age,
            flags.to_owned(),
        ])
        .style(Style::default().fg(color).bg(theme::PANEL))
    });
    frame.render_widget(
        Table::new(
            rows,
            [
                Constraint::Percentage(20),
                Constraint::Percentage(27),
                Constraint::Percentage(18),
                Constraint::Percentage(13),
                Constraint::Percentage(22),
            ],
        )
        .header(
            Row::new(["PEER", "CHANNEL", "STATE", "EVIDENCE", "POSTURE"])
                .style(theme::muted())
                .bottom_margin(1),
        )
        .column_spacing(1)
        .block(theme::card(" LIVE SIGNALS ", theme::CYAN)),
        table_area,
    );

    let settled = health
        .peers
        .iter()
        .filter(|peer| peer.state == PeerHealthState::Current)
        .count();
    let total = health.peers.len();
    let counts = health_counts(health);
    let ratio = if total == 0 {
        0.0
    } else {
        settled as f64 / total as f64
    };
    let block = theme::card(" CONSTELLATION POSTURE ", theme::VIOLET);
    let inner = block.inner(posture_area);
    frame.render_widget(block, posture_area);
    let [copy, gauge, legend] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(2),
    ])
    .spacing(1)
    .areas(inner);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(format!("{} current · {} observing", counts.0, counts.1)),
            Line::styled(
                format!("{} degraded · {} unobserved", counts.2, counts.3),
                Style::default().fg(theme::MUTED).bg(theme::PANEL),
            ),
        ])
        .style(Style::default().fg(theme::TEXT).bg(theme::PANEL)),
        copy,
    );
    frame.render_widget(
        Gauge::default()
            .ratio(ratio)
            .label(format!("{settled}/{total} current"))
            .use_unicode(true)
            .style(Style::default().fg(theme::MUTED).bg(theme::PANEL_RAISED))
            .gauge_style(
                Style::default()
                    .fg(theme::VOID)
                    .bg(theme::GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
        gauge,
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled("✓ current     ◌ observing", theme::muted()),
            Line::styled("! degraded    ○ unobserved", theme::muted()),
            Line::styled(
                "Uncertainty stays loud.",
                Style::default().fg(theme::YELLOW).bg(theme::PANEL),
            ),
        ]),
        legend,
    );
}

const fn health_state_style(state: PeerHealthState) -> (&'static str, &'static str, Color) {
    match state {
        PeerHealthState::Unobserved => ("○", "unobserved", theme::MUTED),
        PeerHealthState::Observing => ("◌", "observing", theme::BLUE),
        PeerHealthState::Current => ("✓", "current", theme::GREEN),
        PeerHealthState::Degraded => ("!", "degraded", theme::RED),
    }
}

fn draw_availability_detail(frame: &mut Frame<'_>, area: Rect, peers: PeerAvailability) {
    let services = [
        (
            "Chronos",
            "planning and workstreams",
            peers.chronos,
            theme::BLUE,
        ),
        ("Vellum", "documents and craft", peers.vellum, theme::VIOLET),
        (
            "Agora",
            "rooms and conversation",
            peers.agora,
            theme::VIOLET,
        ),
        ("Moira", "sagas and recovery", peers.moira, theme::PINK),
        ("Daimon", "agents and presence", peers.daimon, theme::GREEN),
        ("Hermes", "streams and cursors", peers.hermes, theme::CYAN),
        ("Aegis", "authority and grants", peers.aegis, theme::YELLOW),
        (
            "Grounding",
            "evidence and context",
            peers.grounding,
            theme::YELLOW,
        ),
    ];
    let rows = services
        .into_iter()
        .map(|(name, purpose, attached, color)| {
            Row::new(vec![
                format!(
                    "{} {name}: {}",
                    if attached { "◆" } else { "◇" },
                    if attached {
                        "attached"
                    } else {
                        "not configured"
                    }
                ),
                purpose.to_owned(),
            ])
            .style(
                Style::default()
                    .fg(if attached { color } else { theme::MUTED })
                    .bg(theme::PANEL),
            )
        });
    frame.render_widget(
        Table::new(
            rows,
            [Constraint::Percentage(42), Constraint::Percentage(58)],
        )
        .header(
            Row::new(["PEER", "PURPOSE"])
                .style(theme::muted())
                .bottom_margin(1),
        )
        .column_spacing(2)
        .block(theme::card(" CONFIGURED PEERS ", theme::CYAN)),
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
    use moira_client::{AgentParticipationServicePort, SagaServicePort};
    use moira_protocol::{
        SagaCensusV1, SagaCommandV1, SagaOutcomeV1, SagaPageV1, SagaReplyV1,
        agent_participation as participation,
    };
    use ratatui::{Terminal, backend::TestBackend};
    use std::{thread, time::Duration};

    struct AgoraPort;

    struct MoiraPort;
    struct MoiraParticipationPort;
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

    impl AgentParticipationServicePort for MoiraParticipationPort {
        type Error = ();

        fn execute_participation(
            &mut self,
            command: participation::CommandV1,
        ) -> Result<participation::ReplyV1, Self::Error> {
            assert!(matches!(
                command,
                participation::CommandV1::Participation { .. }
            ));
            Ok(participation::ReplyV1::Accepted(Box::new(
                participation::OutcomeV1::Participation(None),
            )))
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
    fn chronos_and_vellum_have_independent_full_frame_surfaces() {
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
        let mut shell = Shell::new(vec![Surface::Chronos, Surface::Vellum]).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(120, 16)).unwrap();
        terminal
            .draw(|frame| {
                draw_shell(
                    frame,
                    &shell,
                    Some(&chronos),
                    Some(&vellum),
                    None,
                    Focus::Chronos,
                    "qualified",
                )
            })
            .unwrap();
        let chronos_rendered: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(chronos_rendered.contains("CHRONOS"));
        assert!(!chronos_rendered.contains("vellum source"));

        assert!(shell.activate(Surface::Vellum));
        terminal
            .draw(|frame| {
                draw_shell(
                    frame,
                    &shell,
                    Some(&chronos),
                    Some(&vellum),
                    None,
                    Focus::Vellum,
                    "qualified",
                )
            })
            .unwrap();
        let vellum_rendered = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(vellum_rendered.contains("VELLUM"));
        assert!(vellum_rendered.contains("vellum source"));
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
    fn every_unconfigured_peer_tab_has_a_distinct_operational_landing() {
        for (surface, marker) in [
            (Surface::Agora, "conversation workspace"),
            (Surface::Moira, "saga operations desk"),
            (Surface::Daimon, "agent control room"),
            (Surface::Grounding, "evidence workbench"),
            (Surface::Aegis, "authority console"),
            (Surface::Hermes, "ledger observatory"),
        ] {
            let shell = Shell::new(vec![surface]).unwrap();
            let mut terminal = Terminal::new(TestBackend::new(120, 24)).unwrap();
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
            let rendered = terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            assert!(rendered.contains("OPERATIONAL LANDING"));
            assert!(
                rendered.contains(marker),
                "{} landing is generic",
                surface.label()
            );
            assert!(rendered.contains("CONNECTION CONTRACT"));
            assert!(!rendered.contains("is quiet"));
        }
    }

    #[test]
    fn surface_rail_is_named_when_wide_and_compact_when_narrow() {
        let surfaces = vec![
            Surface::Overview,
            Surface::Chronos,
            Surface::Vellum,
            Surface::Agora,
            Surface::Moira,
            Surface::Daimon,
            Surface::Grounding,
            Surface::Aegis,
            Surface::Hermes,
            Surface::Operations,
            Surface::Configuration,
        ];
        let shell = Shell::new(surfaces.clone()).unwrap();
        let mut wide = Terminal::new(TestBackend::new(160, 20)).unwrap();
        wide.draw(|frame| {
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
        let rendered = wide
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        for surface in &surfaces {
            assert!(
                rendered.contains(surface.label()),
                "wide rail omitted {}",
                surface.label()
            );
        }

        let mut narrow = Terminal::new(TestBackend::new(80, 12)).unwrap();
        narrow
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
        let rendered = narrow
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(rendered.contains("0◈"));
        assert!(rendered.contains("8≋"));
        assert!(rendered.contains("OVERVIEW"));

        assert_eq!(surface_tab_at(&shell, 80, 0, 1), Some(Surface::Overview));
        assert_eq!(surface_tab_at(&shell, 80, 4, 1), Some(Surface::Chronos));
        assert_eq!(surface_tab_at(&shell, 80, 0, 3), None);
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
        assert!(rendered.contains("1 current · 1 observing"));
        assert!(rendered.contains("1 degraded · 0 unobserved"));
        assert!(rendered.contains("LIVE SIGNALS"));
        assert!(rendered.contains("chronos"));
        assert!(rendered.contains("sessions/events"));
        assert!(rendered.contains("OUTCOME UNCERTAIN"));
        assert!(rendered.contains("ledger/cursors"));
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
                            moira_participation: None,
                            moira_open: None,
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
                            configuration: None,
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
    fn shell_renders_selected_moira_participation_without_copying_owner_state() {
        let shell = Shell::new(vec![Surface::Moira]).unwrap();
        let mut participation = moira_tui::ParticipationComponent::new(
            MoiraParticipationPort,
            participation::ParticipationIdV1([1; 16]),
        );
        for _ in 0..100 {
            participation.poll();
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
                            moira_participation: Some(&participation),
                            moira_open: None,
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
                            configuration: None,
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
            if rendered.contains("Governed participation not found") {
                assert!(rendered.contains("No current governed participation"));
                assert!(!rendered.contains("workflow presentation is not configured"));
                return;
            }
            thread::sleep(Duration::from_millis(1));
        }
        panic!("Moira participation did not render its owner observation");
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
                            moira_participation: None,
                            moira_open: None,
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
                            configuration: None,
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
                            moira_participation: None,
                            moira_open: None,
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
                            configuration: None,
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
