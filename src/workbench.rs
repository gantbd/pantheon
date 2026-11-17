use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use std::time::{Duration, Instant};

pub const MAX_COMMAND_QUERY_BYTES: usize = 64;
pub const MAX_ACTIVITY_ENTRIES: usize = 64;
pub const MAX_INSTRUMENTED_ACTIONS: usize = 64;
pub const MAX_IN_FLIGHT_OPERATIONS: usize = 32;
pub const MAX_OPERATION_TRACES: usize = 64;
pub const MAX_TRACE_STAGES: usize = 8;
pub const LOCAL_ACTION_BUDGET: Duration = Duration::from_millis(50);
pub const PEER_ROUND_TRIP_BUDGET: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandKind {
    Navigate,
    Observe,
    Work,
    Guarded,
    Lifecycle,
}

impl CommandKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Navigate => "NAVIGATE",
            Self::Observe => "OBSERVE",
            Self::Work => "WORK",
            Self::Guarded => "GUARDED",
            Self::Lifecycle => "LIFECYCLE",
        }
    }

    const fn color(self) -> Color {
        match self {
            Self::Navigate => Color::Cyan,
            Self::Observe => Color::Blue,
            Self::Work => Color::Green,
            Self::Guarded => Color::Yellow,
            Self::Lifecycle => Color::Magenta,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandSpec {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub group: &'static str,
    pub shortcut: Option<&'static str>,
    pub kind: CommandKind,
    pub enabled: bool,
    pub unavailable: Option<&'static str>,
}

impl CommandSpec {
    pub const fn available(
        id: &'static str,
        title: &'static str,
        description: &'static str,
        group: &'static str,
        shortcut: Option<&'static str>,
        kind: CommandKind,
    ) -> Self {
        Self {
            id,
            title,
            description,
            group,
            shortcut,
            kind,
            enabled: true,
            unavailable: None,
        }
    }

    pub const fn unavailable(mut self, reason: &'static str) -> Self {
        self.enabled = false;
        self.unavailable = Some(reason);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaletteOutcome {
    None,
    Dismiss,
    Execute(&'static str),
    Unavailable(&'static str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPalette {
    commands: Vec<CommandSpec>,
    query: String,
    selected: usize,
}

impl CommandPalette {
    pub fn new(commands: Vec<CommandSpec>) -> Self {
        Self {
            commands,
            query: String::new(),
            selected: 0,
        }
    }

    pub fn commands(&self) -> &[CommandSpec] {
        &self.commands
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn reset(&mut self) {
        self.query.clear();
        self.selected = 0;
    }

    pub fn visible(&self) -> Vec<&CommandSpec> {
        self.commands
            .iter()
            .filter(|command| command_matches(command, &self.query))
            .collect()
    }

    pub fn selected(&self) -> Option<&CommandSpec> {
        self.visible().get(self.selected).copied()
    }

    pub fn insert(&mut self, text: &str) {
        for character in text.chars().filter(|character| !character.is_control()) {
            if self.query.len() + character.len_utf8() > MAX_COMMAND_QUERY_BYTES {
                break;
            }
            self.query.push(character);
        }
        self.selected = 0;
    }

    pub fn key(&mut self, key: KeyEvent) -> PaletteOutcome {
        if key.kind != KeyEventKind::Press {
            return PaletteOutcome::None;
        }
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) => PaletteOutcome::Dismiss,
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => PaletteOutcome::Dismiss,
            (KeyModifiers::NONE, KeyCode::Up) => {
                let count = self.visible().len();
                if count != 0 {
                    self.selected = self.selected.checked_sub(1).unwrap_or(count - 1);
                }
                PaletteOutcome::None
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                let count = self.visible().len();
                if count != 0 {
                    self.selected = (self.selected + 1) % count;
                }
                PaletteOutcome::None
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.selected().map_or(PaletteOutcome::None, |command| {
                    if command.enabled {
                        PaletteOutcome::Execute(command.id)
                    } else {
                        PaletteOutcome::Unavailable(
                            command.unavailable.unwrap_or("action unavailable"),
                        )
                    }
                })
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.query.pop();
                self.selected = 0;
                PaletteOutcome::None
            }
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(character)) => {
                self.insert(&character.to_string());
                PaletteOutcome::None
            }
            _ => PaletteOutcome::None,
        }
    }
}

fn command_matches(command: &CommandSpec, query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return true;
    }
    let candidate = format!(
        "{} {} {} {} {}",
        command.id,
        command.title,
        command.description,
        command.group,
        command.kind.label()
    )
    .to_ascii_lowercase();
    query
        .split_whitespace()
        .all(|token| candidate.contains(token))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityLevel {
    Info,
    Success,
    Warning,
    Uncertain,
}

impl ActivityLevel {
    const fn symbol(self) -> &'static str {
        match self {
            Self::Info => "•",
            Self::Success => "✓",
            Self::Warning => "!",
            Self::Uncertain => "?",
        }
    }

    const fn color(self) -> Color {
        match self {
            Self::Info => Color::Cyan,
            Self::Success => Color::Green,
            Self::Warning => Color::Yellow,
            Self::Uncertain => Color::Red,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityEntry {
    pub sequence: u64,
    pub level: ActivityLevel,
    pub source: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActivityLog {
    next_sequence: u64,
    entries: Vec<ActivityEntry>,
}

impl ActivityLog {
    pub fn push(
        &mut self,
        level: ActivityLevel,
        source: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.next_sequence = self.next_sequence.saturating_add(1);
        if self.entries.len() == MAX_ACTIVITY_ENTRIES {
            self.entries.remove(0);
        }
        self.entries.push(ActivityEntry {
            sequence: self.next_sequence,
            level,
            source: source.into(),
            message: message.into(),
        });
    }

    pub fn entries(&self) -> &[ActivityEntry] {
        &self.entries
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionMetric {
    pub action: &'static str,
    pub peer: &'static str,
    pub dispatches: u64,
    pub submitted: u64,
    pub succeeded: u64,
    pub warnings: u64,
    pub uncertain: u64,
    pub last_latency_micros: u64,
    pub max_latency_micros: u64,
    pub in_flight: u64,
    pub max_in_flight: u64,
    pub budget_violations: u64,
}

#[derive(Clone, Debug)]
struct InFlightOperation {
    id: u64,
    action: &'static str,
    peer: &'static str,
    started: Instant,
    budget: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceStage {
    pub stage: &'static str,
    pub elapsed_micros: u64,
    pub level: Option<ActivityLevel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationTrace {
    pub operation_id: u64,
    pub correlation_id: u64,
    pub action: &'static str,
    pub peer: &'static str,
    pub stages: Vec<TraceStage>,
    pub complete: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Instrumentation {
    actions: Vec<ActionMetric>,
    in_flight: Vec<InFlightOperation>,
    traces: Vec<OperationTrace>,
    next_operation_id: u64,
    next_correlation_id: u64,
    dropped_operations: u64,
}

impl Instrumentation {
    /// Records only stable action/peer identifiers and timing. Messages,
    /// credentials, document text, prompts, and authority material are never
    /// admitted to this projection.
    pub fn record(
        &mut self,
        action: &'static str,
        peer: &'static str,
        level: ActivityLevel,
        latency: Duration,
    ) {
        self.record_with_budget(action, peer, level, latency, LOCAL_ACTION_BUDGET);
    }

    pub fn record_with_budget(
        &mut self,
        action: &'static str,
        peer: &'static str,
        level: ActivityLevel,
        latency: Duration,
        budget: Duration,
    ) {
        let index = self.metric_index(action, peer);
        let metric = &mut self.actions[index];
        metric.dispatches = metric.dispatches.saturating_add(1);
        Self::observe_level(metric, level);
        Self::observe_latency(metric, latency, budget);
    }

    /// Starts one correlated shell-to-peer operation. The returned identifier
    /// is local-only and carries no peer authority or request content.
    pub fn begin_peer_operation(
        &mut self,
        action: &'static str,
        peer: &'static str,
    ) -> Option<u64> {
        self.begin_peer_operation_at(action, peer, Instant::now())
    }

    /// Starts a correlated operation at a host-captured dispatch instant so
    /// time spent routing and queueing is included in the peer round trip.
    pub fn begin_peer_operation_at(
        &mut self,
        action: &'static str,
        peer: &'static str,
        started: Instant,
    ) -> Option<u64> {
        let correlation_id = self.new_correlation();
        self.begin_correlated_peer_operation_at(action, peer, started, correlation_id)
    }

    /// Allocates one local correlation shared by every subsystem span caused
    /// by the same user operation. It carries no authority or request data.
    pub fn new_correlation(&mut self) -> u64 {
        self.next_correlation_id = self.next_correlation_id.saturating_add(1);
        self.next_correlation_id
    }

    pub fn begin_correlated_peer_operation_at(
        &mut self,
        action: &'static str,
        peer: &'static str,
        started: Instant,
        correlation_id: u64,
    ) -> Option<u64> {
        if self.in_flight.len() == MAX_IN_FLIGHT_OPERATIONS {
            self.dropped_operations = self.dropped_operations.saturating_add(1);
            return None;
        }
        self.next_operation_id = self.next_operation_id.saturating_add(1);
        let id = self.next_operation_id;
        let index = self.metric_index(action, peer);
        let metric = &mut self.actions[index];
        metric.dispatches = metric.dispatches.saturating_add(1);
        metric.submitted = metric.submitted.saturating_add(1);
        metric.in_flight = metric.in_flight.saturating_add(1);
        metric.max_in_flight = metric.max_in_flight.max(metric.in_flight);
        self.in_flight.push(InFlightOperation {
            id,
            action,
            peer,
            started,
            budget: PEER_ROUND_TRIP_BUDGET,
        });
        if self.traces.len() == MAX_OPERATION_TRACES {
            self.traces.remove(0);
        }
        self.traces.push(OperationTrace {
            operation_id: id,
            correlation_id,
            action,
            peer,
            stages: vec![
                TraceStage {
                    stage: "shell_dispatch",
                    elapsed_micros: 0,
                    level: None,
                },
                TraceStage {
                    stage: "peer_pending",
                    elapsed_micros: u64::try_from(started.elapsed().as_micros())
                        .unwrap_or(u64::MAX),
                    level: Some(ActivityLevel::Info),
                },
            ],
            complete: false,
        });
        Some(id)
    }

    pub fn mark_peer_stage(&mut self, id: u64, stage: &'static str, level: ActivityLevel) -> bool {
        let Some(operation) = self.in_flight.iter().find(|operation| operation.id == id) else {
            return false;
        };
        let elapsed_micros =
            u64::try_from(operation.started.elapsed().as_micros()).unwrap_or(u64::MAX);
        let Some(trace) = self
            .traces
            .iter_mut()
            .find(|trace| trace.operation_id == id)
        else {
            return false;
        };
        if trace.stages.len() == MAX_TRACE_STAGES {
            return false;
        }
        trace.stages.push(TraceStage {
            stage,
            elapsed_micros,
            level: Some(level),
        });
        true
    }

    /// Settles a correlated peer operation at the first explicit terminal
    /// disposition. Outcome-unknown is terminal evidence, never success.
    pub fn settle_peer_operation(&mut self, id: u64, level: ActivityLevel) -> Option<Duration> {
        let index = self
            .in_flight
            .iter()
            .position(|operation| operation.id == id)?;
        let operation = self.in_flight.remove(index);
        let latency = operation.started.elapsed();
        if let Some(trace) = self
            .traces
            .iter_mut()
            .find(|trace| trace.operation_id == id)
        {
            if trace.stages.len() == MAX_TRACE_STAGES {
                trace.stages.remove(1);
            }
            trace.stages.push(TraceStage {
                stage: "terminal_disposition",
                elapsed_micros: u64::try_from(latency.as_micros()).unwrap_or(u64::MAX),
                level: Some(level),
            });
            trace.complete = true;
        }
        let metric_index = self.metric_index(operation.action, operation.peer);
        let metric = &mut self.actions[metric_index];
        metric.in_flight = metric.in_flight.saturating_sub(1);
        Self::observe_level(metric, level);
        Self::observe_latency(metric, latency, operation.budget);
        Some(latency)
    }

    pub fn actions(&self) -> &[ActionMetric] {
        &self.actions
    }

    pub fn traces(&self) -> &[OperationTrace] {
        &self.traces
    }

    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    pub fn budget_violations(&self) -> u64 {
        self.actions
            .iter()
            .map(|metric| metric.budget_violations)
            .sum::<u64>()
            .saturating_add(self.dropped_operations)
            .saturating_add(
                self.in_flight
                    .iter()
                    .filter(|operation| operation.started.elapsed() > operation.budget)
                    .count() as u64,
            )
    }

    /// Adds a later peer-owned outcome to an existing dispatch without
    /// pretending that the observation was another request.
    pub fn observe_outcome(
        &mut self,
        action: &'static str,
        peer: &'static str,
        level: ActivityLevel,
    ) {
        let Some(metric) = self
            .actions
            .iter_mut()
            .find(|metric| metric.action == action && metric.peer == peer)
        else {
            return;
        };
        match level {
            ActivityLevel::Info => metric.submitted = metric.submitted.saturating_add(1),
            ActivityLevel::Success => metric.succeeded = metric.succeeded.saturating_add(1),
            ActivityLevel::Warning => metric.warnings = metric.warnings.saturating_add(1),
            ActivityLevel::Uncertain => metric.uncertain = metric.uncertain.saturating_add(1),
        }
    }

    fn metric_index(&mut self, action: &'static str, peer: &'static str) -> usize {
        self.actions
            .iter()
            .position(|metric| metric.action == action && metric.peer == peer)
            .unwrap_or_else(|| {
                if self.actions.len() == MAX_INSTRUMENTED_ACTIONS {
                    self.actions.remove(0);
                }
                self.actions.push(ActionMetric {
                    action,
                    peer,
                    dispatches: 0,
                    submitted: 0,
                    succeeded: 0,
                    warnings: 0,
                    uncertain: 0,
                    last_latency_micros: 0,
                    max_latency_micros: 0,
                    in_flight: 0,
                    max_in_flight: 0,
                    budget_violations: 0,
                });
                self.actions.len() - 1
            })
    }

    fn observe_level(metric: &mut ActionMetric, level: ActivityLevel) {
        match level {
            ActivityLevel::Info => metric.submitted = metric.submitted.saturating_add(1),
            ActivityLevel::Success => metric.succeeded = metric.succeeded.saturating_add(1),
            ActivityLevel::Warning => metric.warnings = metric.warnings.saturating_add(1),
            ActivityLevel::Uncertain => metric.uncertain = metric.uncertain.saturating_add(1),
        }
    }

    fn observe_latency(metric: &mut ActionMetric, latency: Duration, budget: Duration) {
        let micros = u64::try_from(latency.as_micros()).unwrap_or(u64::MAX);
        metric.last_latency_micros = micros;
        metric.max_latency_micros = metric.max_latency_micros.max(micros);
        if latency > budget {
            metric.budget_violations = metric.budget_violations.saturating_add(1);
        }
    }

    pub fn lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!(
                "{} in flight · {} budget violations · local ≤{}ms · peer ≤{}ms",
                self.in_flight_count(),
                self.budget_violations(),
                LOCAL_ACTION_BUDGET.as_millis(),
                PEER_ROUND_TRIP_BUDGET.as_millis()
            ),
            "action · peer                 calls  fly  ok  warn  ?  slow  last/max μs".into(),
            String::new(),
        ];
        if self.actions.is_empty() {
            lines.push("No instrumented workbench actions yet.".into());
            return lines;
        }
        lines.extend(self.actions.iter().rev().map(|metric| {
            format!(
                "{:<24} {:<9} {:>5} {:>4} {:>3} {:>5} {:>2} {:>5} {:>7}/{:<7}",
                metric.action,
                metric.peer,
                metric.dispatches,
                metric.in_flight,
                metric.succeeded,
                metric.warnings,
                metric.uncertain,
                metric.budget_violations,
                metric.last_latency_micros,
                metric.max_latency_micros,
            )
        }));
        if !self.traces.is_empty() {
            lines.push(String::new());
            lines.push("recent end-to-end traces (correlation/operation · peer · stages)".into());
            lines.extend(self.traces.iter().rev().take(8).map(|trace| {
                let stages = trace
                    .stages
                    .iter()
                    .map(|stage| format!("{}@{}µs", stage.stage, stage.elapsed_micros))
                    .collect::<Vec<_>>()
                    .join(" → ");
                format!(
                    "{}/{} · {} · {} · {}",
                    trace.correlation_id, trace.operation_id, trace.peer, trace.action, stages
                )
            }));
        }
        lines
    }
}

pub fn draw_command_palette(frame: &mut Frame<'_>, palette: &CommandPalette) {
    let area = centered(frame.area(), 84, 72);
    frame.render_widget(Clear, area);
    let [query, results, detail, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(4),
        Constraint::Length(4),
        Constraint::Length(1),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("⌕ ", Style::default().fg(Color::Cyan)),
            Span::raw(palette.query()),
            Span::styled("█", Style::default().fg(Color::DarkGray)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Command palette "),
        ),
        query,
    );
    let visible = palette.visible();
    let items = visible
        .iter()
        .enumerate()
        .map(|(index, command)| {
            let selected = index == palette.selected;
            let marker = if selected { "›" } else { " " };
            let availability = if command.enabled { "" } else { "  unavailable" };
            let line = Line::from(vec![
                Span::styled(
                    format!("{marker} {:<10} ", command.kind.label()),
                    Style::default().fg(command.kind.color()),
                ),
                Span::styled(
                    command.title,
                    if selected {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else if command.enabled {
                        Style::default()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::styled(availability, Style::default().fg(Color::DarkGray)),
            ]);
            ListItem::new(line)
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(if items.is_empty() {
            vec![ListItem::new("  No matching actions")]
        } else {
            items
        })
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT)),
        results,
    );
    let detail_text = palette.selected().map_or_else(
        || "Try a peer, capability, or verb such as ‘vellum save’.".to_owned(),
        |command| {
            let shortcut = command
                .shortcut
                .map_or_else(String::new, |value| format!(" • {value}"));
            let unavailable = command
                .unavailable
                .map_or_else(String::new, |value| format!("\nUnavailable: {value}"));
            format!(
                "{} • {}{}\n{}{}",
                command.group,
                command.kind.label(),
                shortcut,
                command.description,
                unavailable
            )
        },
    );
    frame.render_widget(
        Paragraph::new(detail_text)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title(" Action ")),
        detail,
    );
    frame.render_widget(
        Paragraph::new(" ↑↓ select  Enter run  Esc close  •  type to filter ")
            .style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}

pub fn draw_help(frame: &mut Frame<'_>, commands: &[CommandSpec]) {
    let area = centered(frame.area(), 82, 78);
    frame.render_widget(Clear, area);
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Ctrl-P", Style::default().fg(Color::Cyan)),
            Span::raw("  Search every available action"),
        ]),
        Line::from(vec![
            Span::styled("F1", Style::default().fg(Color::Cyan)),
            Span::raw("      Toggle this contextual help"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl-G", Style::default().fg(Color::Cyan)),
            Span::raw("  Show activity and receipts"),
        ]),
        Line::from(vec![
            Span::styled("[ / ]", Style::default().fg(Color::Cyan)),
            Span::raw("   Previous / next surface"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl-Q", Style::default().fg(Color::Cyan)),
            Span::raw("  Safely leave the workbench"),
        ]),
        Line::from(""),
        Line::styled(
            "AVAILABLE ACTIONS",
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];
    lines.extend(
        commands
            .iter()
            .filter(|command| command.enabled)
            .map(|command| {
                Line::from(vec![
                    Span::styled(
                        format!("{:<10}", command.kind.label()),
                        Style::default().fg(command.kind.color()),
                    ),
                    Span::raw(format!("{:<28}", command.title)),
                    Span::styled(
                        command.shortcut.unwrap_or("palette"),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            }),
    );
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Pantheon help "),
        ),
        area,
    );
}

pub fn draw_activity(frame: &mut Frame<'_>, activity: &ActivityLog) {
    let area = centered(frame.area(), 84, 76);
    frame.render_widget(Clear, area);
    let lines = if activity.entries().is_empty() {
        vec![Line::from(
            "No activity yet. Actions and exact outcomes appear here.",
        )]
    } else {
        activity
            .entries()
            .iter()
            .rev()
            .map(|entry| {
                Line::from(vec![
                    Span::styled(
                        format!("{} ", entry.level.symbol()),
                        Style::default().fg(entry.level.color()),
                    ),
                    Span::styled(
                        format!("#{:<4} {:<10}", entry.sequence, entry.source),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(entry.message.as_str()),
                ])
            })
            .collect()
    };
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Activity & receipts • newest first • Esc closes "),
        ),
        area,
    );
}

/// Draws a bounded read-only sheet from already-redacted host facts.
pub fn draw_inspector(frame: &mut Frame<'_>, title: &str, lines: &[String]) {
    let area = centered(frame.area(), 78, 72);
    frame.render_widget(Clear, area);
    let body = if lines.is_empty() {
        vec![Line::from("No session facts are available")]
    } else {
        lines.iter().cloned().map(Line::from).collect()
    };
    frame.render_widget(
        Paragraph::new(body).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {title} "))
                .title_bottom(" Esc close "),
        ),
        area,
    );
}

fn centered(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let [vertical] = Layout::vertical([Constraint::Percentage(height_percent)])
        .flex(Flex::Center)
        .areas(area);
    let [horizontal] = Layout::horizontal([Constraint::Percentage(width_percent)])
        .flex(Flex::Center)
        .areas(vertical);
    horizontal
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn commands() -> Vec<CommandSpec> {
        vec![
            CommandSpec::available(
                "view.refresh",
                "Refresh current view",
                "Ask the selected peer for a current observation.",
                "Current view",
                Some("r"),
                CommandKind::Observe,
            ),
            CommandSpec::available(
                "vellum.save",
                "Save current document",
                "Submit Vellum's guarded save for the selected document.",
                "Vellum",
                Some("Ctrl-S"),
                CommandKind::Guarded,
            )
            .unavailable("Vellum is not configured"),
        ]
    }

    #[test]
    fn palette_search_is_bounded_and_matches_across_action_metadata() {
        let mut palette = CommandPalette::new(commands());
        palette.insert("vellum guard");
        assert_eq!(palette.visible().len(), 1);
        assert_eq!(palette.visible()[0].id, "vellum.save");
        palette.insert(&"x".repeat(100));
        assert!(palette.query().len() <= MAX_COMMAND_QUERY_BYTES);
    }

    #[test]
    fn palette_never_executes_an_unavailable_action() {
        let mut palette = CommandPalette::new(commands());
        palette.insert("vellum");
        assert_eq!(
            palette.key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            PaletteOutcome::Unavailable("Vellum is not configured")
        );
    }

    #[test]
    fn activity_is_bounded_and_preserves_uncertainty() {
        let mut activity = ActivityLog::default();
        for index in 0..MAX_ACTIVITY_ENTRIES + 4 {
            activity.push(
                if index + 1 == MAX_ACTIVITY_ENTRIES + 4 {
                    ActivityLevel::Uncertain
                } else {
                    ActivityLevel::Info
                },
                "test",
                format!("event {index}"),
            );
        }
        assert_eq!(activity.entries().len(), MAX_ACTIVITY_ENTRIES);
        assert_eq!(activity.entries()[0].sequence, 5);
        assert_eq!(
            activity.entries().last().unwrap().level,
            ActivityLevel::Uncertain
        );
    }

    #[test]
    fn instrumentation_is_bounded_content_free_and_counts_uncertainty() {
        let mut instrumentation = Instrumentation::default();
        instrumentation.record(
            "moira.reconcile",
            "moira",
            ActivityLevel::Uncertain,
            Duration::from_micros(17),
        );
        let metric = &instrumentation.actions()[0];
        assert_eq!(metric.dispatches, 1);
        assert_eq!(metric.uncertain, 1);
        assert_eq!(metric.last_latency_micros, 17);
        for index in 0..MAX_INSTRUMENTED_ACTIONS + 3 {
            let action = Box::leak(format!("action-{index}").into_boxed_str());
            instrumentation.record(action, "test", ActivityLevel::Info, Duration::ZERO);
        }
        assert_eq!(instrumentation.actions().len(), MAX_INSTRUMENTED_ACTIONS);
        assert!(
            instrumentation
                .lines()
                .iter()
                .all(|line| !line.contains("secret"))
        );
    }

    #[test]
    fn instrumentation_correlates_peer_settlement_and_enforces_budgets() {
        let mut instrumentation = Instrumentation::default();
        let operation = instrumentation
            .begin_peer_operation("agora.room.create", "agora")
            .expect("bounded operation slot");
        assert_eq!(
            instrumentation.traces()[0].stages[0].stage,
            "shell_dispatch"
        );
        assert_eq!(instrumentation.traces()[0].stages[1].stage, "peer_pending");
        assert_eq!(instrumentation.in_flight_count(), 1);
        let latency = instrumentation
            .settle_peer_operation(operation, ActivityLevel::Uncertain)
            .expect("known operation");
        assert_eq!(instrumentation.in_flight_count(), 0);
        let metric = &instrumentation.actions()[0];
        assert_eq!(metric.dispatches, 1);
        assert_eq!(metric.submitted, 1);
        assert_eq!(metric.uncertain, 1);
        assert_eq!(metric.in_flight, 0);
        assert_eq!(metric.max_in_flight, 1);
        assert_eq!(metric.last_latency_micros, latency.as_micros() as u64);
        let trace = &instrumentation.traces()[0];
        assert!(trace.complete);
        assert_eq!(trace.stages.last().unwrap().stage, "terminal_disposition");
        assert_eq!(
            trace.stages.last().unwrap().level,
            Some(ActivityLevel::Uncertain)
        );

        let correlation = instrumentation.new_correlation();
        let first = instrumentation
            .begin_correlated_peer_operation_at(
                "suite.refresh",
                "moira",
                Instant::now(),
                correlation,
            )
            .unwrap();
        let second = instrumentation
            .begin_correlated_peer_operation_at(
                "suite.refresh",
                "hermes",
                Instant::now(),
                correlation,
            )
            .unwrap();
        let traces = instrumentation.traces();
        assert_eq!(traces[traces.len() - 2].correlation_id, correlation);
        assert_eq!(traces[traces.len() - 1].correlation_id, correlation);
        instrumentation.settle_peer_operation(first, ActivityLevel::Success);
        instrumentation.settle_peer_operation(second, ActivityLevel::Warning);

        instrumentation.record_with_budget(
            "qualification.slow",
            "pantheon",
            ActivityLevel::Success,
            Duration::from_millis(51),
            LOCAL_ACTION_BUDGET,
        );
        assert_eq!(instrumentation.budget_violations(), 1);

        let overdue = instrumentation
            .begin_peer_operation_at(
                "view.refresh",
                "hermes",
                Instant::now() - PEER_ROUND_TRIP_BUDGET - Duration::from_millis(1),
            )
            .expect("bounded operation slot");
        assert_eq!(instrumentation.budget_violations(), 2);
        instrumentation.settle_peer_operation(overdue, ActivityLevel::Warning);
        assert_eq!(instrumentation.in_flight_count(), 0);
    }

    #[test]
    fn reference_palette_and_render_path_meets_measured_budget() {
        let started = Instant::now();
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let mut palette = CommandPalette::new(commands());
        for _ in 0..1_000 {
            palette.reset();
            palette.insert("vellum guard");
            assert_eq!(palette.visible().len(), 1);
        }
        for _ in 0..100 {
            terminal
                .draw(|frame| draw_command_palette(frame, &palette))
                .unwrap();
        }
        let elapsed = started.elapsed();
        let qualification_budget = Duration::from_secs(2);
        let mut instrumentation = Instrumentation::default();
        instrumentation.record_with_budget(
            "qualification.palette-render",
            "pantheon",
            ActivityLevel::Success,
            elapsed,
            qualification_budget,
        );
        assert_eq!(
            instrumentation.budget_violations(),
            0,
            "reference palette/render path took {elapsed:?}, budget {qualification_budget:?}"
        );
    }

    #[test]
    fn overlays_render_in_small_terminals_without_panicking() {
        let mut terminal = Terminal::new(TestBackend::new(72, 24)).unwrap();
        let palette = CommandPalette::new(commands());
        terminal
            .draw(|frame| draw_command_palette(frame, &palette))
            .unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(rendered.contains("Command palette"));
        assert!(rendered.contains("Refresh current view"));
        terminal
            .draw(|frame| draw_inspector(frame, "Configuration", &["peer: ready".into()]))
            .unwrap();
    }
}
