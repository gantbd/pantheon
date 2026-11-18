use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use std::time::{Duration, Instant};

use crate::theme;

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
            Self::Navigate => theme::CYAN,
            Self::Observe => theme::BLUE,
            Self::Work => theme::GREEN,
            Self::Guarded => theme::YELLOW,
            Self::Lifecycle => theme::VIOLET,
        }
    }

    const fn search_aliases(self) -> &'static str {
        match self {
            Self::Navigate => "open go navigate switch show",
            Self::Observe => "inspect refresh observe view",
            Self::Work => "work create edit",
            Self::Guarded => "review confirm guarded",
            Self::Lifecycle => "start stop restart drain lifecycle",
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
    cursor: usize,
    selected: usize,
    history: Vec<String>,
    history_cursor: Option<usize>,
}

impl CommandPalette {
    pub fn new(commands: Vec<CommandSpec>) -> Self {
        Self {
            commands,
            query: String::new(),
            cursor: 0,
            selected: 0,
            history: Vec::new(),
            history_cursor: None,
        }
    }

    pub fn commands(&self) -> &[CommandSpec] {
        &self.commands
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn reset(&mut self) {
        self.query.clear();
        self.cursor = 0;
        self.selected = 0;
        self.history_cursor = None;
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
            self.query.insert(self.cursor, character);
            self.cursor += character.len_utf8();
        }
        self.selected = 0;
        self.history_cursor = None;
    }

    pub fn key(&mut self, key: KeyEvent) -> PaletteOutcome {
        if key.kind != KeyEventKind::Press {
            return PaletteOutcome::None;
        }
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) => PaletteOutcome::Dismiss,
            (KeyModifiers::CONTROL, KeyCode::Char('g')) => PaletteOutcome::Dismiss,
            (KeyModifiers::NONE, KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                self.select_previous()
            }
            (KeyModifiers::NONE, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                self.select_next()
            }
            (KeyModifiers::ALT, KeyCode::Char('p')) => self.history_previous(),
            (KeyModifiers::ALT, KeyCode::Char('n')) => self.history_next(),
            (KeyModifiers::CONTROL, KeyCode::Char('a')) | (KeyModifiers::NONE, KeyCode::Home) => {
                self.cursor = 0;
                PaletteOutcome::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('e')) | (KeyModifiers::NONE, KeyCode::End) => {
                self.cursor = self.query.len();
                PaletteOutcome::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('b')) | (KeyModifiers::NONE, KeyCode::Left) => {
                self.move_cursor_left();
                PaletteOutcome::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('f')) | (KeyModifiers::NONE, KeyCode::Right) => {
                self.move_cursor_right();
                PaletteOutcome::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                self.query.clear();
                self.cursor = 0;
                self.selected = 0;
                PaletteOutcome::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                self.query.truncate(self.cursor);
                self.selected = 0;
                PaletteOutcome::None
            }
            (KeyModifiers::NONE, KeyCode::Tab) => {
                if let Some(completion) = self.selected().map(|command| command.id) {
                    self.query.clear();
                    self.query.push_str(completion);
                    self.cursor = self.query.len();
                    self.selected = 0;
                }
                PaletteOutcome::None
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                let Some((id, enabled, unavailable)) = self
                    .selected()
                    .map(|command| (command.id, command.enabled, command.unavailable))
                else {
                    return PaletteOutcome::None;
                };
                if enabled {
                    self.remember(id);
                    PaletteOutcome::Execute(id)
                } else {
                    PaletteOutcome::Unavailable(unavailable.unwrap_or("action unavailable"))
                }
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                if self.cursor != 0 {
                    let previous = self.query[..self.cursor]
                        .char_indices()
                        .next_back()
                        .map_or(0, |(index, _)| index);
                    self.query.drain(previous..self.cursor);
                    self.cursor = previous;
                }
                self.selected = 0;
                self.history_cursor = None;
                PaletteOutcome::None
            }
            (KeyModifiers::NONE, KeyCode::Delete) => {
                if self.cursor < self.query.len() {
                    let next = self.query[self.cursor..]
                        .char_indices()
                        .nth(1)
                        .map_or(self.query.len(), |(index, _)| self.cursor + index);
                    self.query.drain(self.cursor..next);
                }
                self.selected = 0;
                self.history_cursor = None;
                PaletteOutcome::None
            }
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(character)) => {
                self.insert(&character.to_string());
                PaletteOutcome::None
            }
            _ => PaletteOutcome::None,
        }
    }

    fn select_previous(&mut self) -> PaletteOutcome {
        let count = self.visible().len();
        if count != 0 {
            self.selected = self.selected.checked_sub(1).unwrap_or(count - 1);
        }
        PaletteOutcome::None
    }

    fn select_next(&mut self) -> PaletteOutcome {
        let count = self.visible().len();
        if count != 0 {
            self.selected = (self.selected + 1) % count;
        }
        PaletteOutcome::None
    }

    fn history_previous(&mut self) -> PaletteOutcome {
        if self.history.is_empty() {
            return PaletteOutcome::None;
        }
        let index = self
            .history_cursor
            .map_or(self.history.len() - 1, |index| index.saturating_sub(1));
        self.history_cursor = Some(index);
        self.query.clone_from(&self.history[index]);
        self.cursor = self.query.len();
        self.selected = 0;
        PaletteOutcome::None
    }

    fn history_next(&mut self) -> PaletteOutcome {
        let Some(index) = self.history_cursor else {
            return PaletteOutcome::None;
        };
        if index + 1 < self.history.len() {
            self.history_cursor = Some(index + 1);
            self.query.clone_from(&self.history[index + 1]);
        } else {
            self.history_cursor = None;
            self.query.clear();
        }
        self.cursor = self.query.len();
        self.selected = 0;
        PaletteOutcome::None
    }

    fn move_cursor_left(&mut self) {
        if self.cursor != 0 {
            self.cursor = self.query[..self.cursor]
                .char_indices()
                .next_back()
                .map_or(0, |(index, _)| index);
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor < self.query.len() {
            self.cursor = self.query[self.cursor..]
                .char_indices()
                .nth(1)
                .map_or(self.query.len(), |(index, _)| self.cursor + index);
        }
    }

    fn remember(&mut self, command: &'static str) {
        let entry = if self.query.trim().is_empty() {
            command.to_owned()
        } else {
            self.query.clone()
        };
        if self.history.last() != Some(&entry) {
            if self.history.len() == 32 {
                self.history.remove(0);
            }
            self.history.push(entry);
        }
        self.history_cursor = None;
    }
}

fn command_matches(command: &CommandSpec, query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return true;
    }
    let candidate = format!(
        "{} {} {} {} {} {}",
        command.id,
        command.title,
        command.description,
        command.group,
        command.kind.label(),
        command.kind.search_aliases()
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
            Self::Info => theme::CYAN,
            Self::Success => theme::GREEN,
            Self::Warning => theme::YELLOW,
            Self::Uncertain => theme::RED,
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

    /// Starts one presentation-owned span at the input boundary. The caller
    /// keeps it open through command resolution, visible feedback, and the
    /// first frame containing that feedback.
    pub fn begin_ui_operation_at(
        &mut self,
        action: &'static str,
        started: Instant,
    ) -> Option<(u64, u64)> {
        if self.in_flight.len() == MAX_IN_FLIGHT_OPERATIONS {
            self.dropped_operations = self.dropped_operations.saturating_add(1);
            return None;
        }
        let correlation_id = self.new_correlation();
        self.next_operation_id = self.next_operation_id.saturating_add(1);
        let id = self.next_operation_id;
        let index = self.metric_index(action, "presentation");
        let metric = &mut self.actions[index];
        metric.dispatches = metric.dispatches.saturating_add(1);
        metric.submitted = metric.submitted.saturating_add(1);
        metric.in_flight = metric.in_flight.saturating_add(1);
        metric.max_in_flight = metric.max_in_flight.max(metric.in_flight);
        self.in_flight.push(InFlightOperation {
            id,
            action,
            peer: "presentation",
            started,
            budget: LOCAL_ACTION_BUDGET,
        });
        if self.traces.len() == MAX_OPERATION_TRACES {
            self.traces.remove(0);
        }
        self.traces.push(OperationTrace {
            operation_id: id,
            correlation_id,
            action,
            peer: "presentation",
            stages: vec![
                TraceStage {
                    stage: "input_classified",
                    elapsed_micros: 0,
                    level: None,
                },
                TraceStage {
                    stage: "command_resolved",
                    elapsed_micros: u64::try_from(started.elapsed().as_micros())
                        .unwrap_or(u64::MAX),
                    level: Some(ActivityLevel::Info),
                },
            ],
            complete: false,
        });
        Some((id, correlation_id))
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
        let mut lines = vec![format!(
            "{} in flight · {} budget violations · local ≤{}ms · peer ≤{}ms",
            self.in_flight_count(),
            self.budget_violations(),
            LOCAL_ACTION_BUDGET.as_millis(),
            PEER_ROUND_TRIP_BUDGET.as_millis()
        )];
        if self.actions.is_empty() {
            lines.push("No instrumented workbench actions yet.".into());
            return lines;
        }
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
        lines.extend([
            String::new(),
            "action · peer                 calls  fly  ok  warn  ?  slow  last/max μs".into(),
        ]);
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
        lines
    }
}

pub fn draw_command_palette(frame: &mut Frame<'_>, palette: &CommandPalette) {
    let frame_area = frame.area();
    let width = frame_area.width.min(108);
    let height = frame_area.height.saturating_sub(2).clamp(6, 14);
    let area = Rect::new(
        frame_area.x + frame_area.width.saturating_sub(width) / 2,
        frame_area.y + frame_area.height.saturating_sub(height),
        width,
        height,
    );
    frame.render_widget(Clear, area);
    let [results, detail, query, footer] = Layout::vertical([
        Constraint::Min(4),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .areas(area);
    let visible = palette.visible();
    let capacity = usize::from(results.height.saturating_sub(2)).max(1);
    let start = palette
        .selected
        .saturating_sub(capacity.saturating_sub(1))
        .min(visible.len().saturating_sub(capacity));
    let items = visible
        .iter()
        .enumerate()
        .skip(start)
        .take(capacity)
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
                        Style::default()
                            .fg(theme::TEXT)
                            .bg(theme::PANEL_RAISED)
                            .add_modifier(Modifier::BOLD)
                    } else if command.enabled {
                        Style::default().fg(theme::TEXT).bg(theme::PANEL_RAISED)
                    } else {
                        Style::default().fg(theme::MUTED).bg(theme::PANEL_RAISED)
                    },
                ),
                Span::styled(
                    availability,
                    Style::default().fg(theme::MUTED).bg(theme::PANEL_RAISED),
                ),
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
        .style(theme::raised())
        .block(
            Block::default()
                .title(" *Completions* · commands and guarded work ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::FAINT).bg(theme::PANEL_RAISED))
                .style(theme::raised()),
        ),
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
            .style(theme::panel())
            .wrap(Wrap { trim: true })
            .block(theme::card(" SELECTED ACTION ", theme::VIOLET)),
        detail,
    );
    let before = &palette.query()[..palette.cursor()];
    let (cursor, after) = palette.query()[palette.cursor()..].chars().next().map_or(
        (" ".to_owned(), ""),
        |character| {
            let next = palette.cursor() + character.len_utf8();
            (character.to_string(), &palette.query()[next..])
        },
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "M-x ",
                Style::default()
                    .fg(theme::CYAN)
                    .bg(theme::PANEL_RAISED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                before,
                Style::default().fg(theme::TEXT).bg(theme::PANEL_RAISED),
            ),
            Span::styled(
                cursor,
                Style::default()
                    .fg(theme::PANEL)
                    .bg(theme::PINK)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                after,
                Style::default().fg(theme::TEXT).bg(theme::PANEL_RAISED),
            ),
        ]))
        .style(theme::raised())
        .block(theme::card(" MINIBUFFER ", theme::CYAN)),
        query,
    );
    frame.render_widget(
        Paragraph::new(" C-p/C-n select · TAB complete · RET run · C-g quit · M-p/M-n history ")
            .style(theme::muted()),
        footer,
    );
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum KeybindingGuide {
    #[default]
    Pantheon,
    Emacs,
}

pub fn draw_help(frame: &mut Frame<'_>, commands: &[CommandSpec], keybindings: KeybindingGuide) {
    let area = centered(frame.area(), 82, 78);
    frame.render_widget(Clear, area);
    let mut lines = match keybindings {
        KeybindingGuide::Pantheon => vec![
            Line::from(vec![
                Span::styled("Ctrl-P", theme::key()),
                Span::raw("  Search every available action"),
            ]),
            Line::from(vec![
                Span::styled("F1", theme::key()),
                Span::raw("      Toggle this contextual help"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl-G", theme::key()),
                Span::raw("  Show activity and receipts"),
            ]),
            Line::from(vec![
                Span::styled("[ / ]", theme::key()),
                Span::raw("   Previous / next surface"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl/Alt 0–9", theme::key()),
                Span::raw(" Dashboard / direct component surface"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl-,", theme::key()),
                Span::raw("   Open live session configuration"),
            ]),
            Line::from(vec![
                Span::styled("Mouse", theme::key()),
                Span::raw("    Click or wheel the surface rail"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl-Q", theme::key()),
                Span::raw("  Safely leave the workbench"),
            ]),
            Line::from(""),
        ],
        KeybindingGuide::Emacs => vec![
            Line::from(vec![
                Span::styled("M-x", theme::key()),
                Span::raw("      Search every available action"),
            ]),
            Line::from(vec![
                Span::styled("C-g", theme::key()),
                Span::raw("      Cancel the active prompt or prefix"),
            ]),
            Line::from(vec![
                Span::styled("C-h b", theme::key()),
                Span::raw("    Show this binding guide"),
            ]),
            Line::from(vec![
                Span::styled("C-h l", theme::key()),
                Span::raw("    Show activity and receipts"),
            ]),
            Line::from(vec![
                Span::styled("C-x C-f", theme::key()),
                Span::raw("  Open a file through Vellum"),
            ]),
            Line::from(vec![
                Span::styled("C-x C-s", theme::key()),
                Span::raw("  Save through Vellum"),
            ]),
            Line::from(vec![
                Span::styled("C-x C-c", theme::key()),
                Span::raw("  Safely leave the workbench"),
            ]),
            Line::from(vec![
                Span::styled("Mouse", theme::key()),
                Span::raw("      Click or wheel the surface rail"),
            ]),
            Line::from(""),
        ],
    };
    lines.push(Line::styled(
        "AVAILABLE ACTIONS",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    lines.extend(
        commands
            .iter()
            .filter(|command| command.enabled)
            .map(|command| {
                let shortcut = match (keybindings, command.id) {
                    (KeybindingGuide::Emacs, "workbench.activity") => "C-h l",
                    (KeybindingGuide::Emacs, "workbench.help") => "C-h b",
                    (KeybindingGuide::Emacs, "vellum.document.open") => "C-x C-f",
                    (KeybindingGuide::Emacs, "vellum.save") => "C-x C-s",
                    (KeybindingGuide::Emacs, _) => command.shortcut.unwrap_or("M-x"),
                    (KeybindingGuide::Pantheon, _) => command.shortcut.unwrap_or("palette"),
                };
                Line::from(vec![
                    Span::styled(
                        format!("{:<10}", command.kind.label()),
                        Style::default().fg(command.kind.color()),
                    ),
                    Span::raw(format!("{:<28}", command.title)),
                    Span::styled(
                        shortcut,
                        Style::default().fg(theme::MUTED).bg(theme::PANEL_RAISED),
                    ),
                ])
            }),
    );
    frame.render_widget(
        Paragraph::new(lines)
            .style(theme::raised())
            .wrap(Wrap { trim: false })
            .block(theme::modal(" PANTHEON · FIELD GUIDE ")),
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
                        Style::default().fg(theme::MUTED).bg(theme::PANEL_RAISED),
                    ),
                    Span::raw(entry.message.as_str()),
                ])
            })
            .collect()
    };
    frame.render_widget(
        Paragraph::new(lines)
            .style(theme::raised())
            .wrap(Wrap { trim: false })
            .block(theme::modal(
                " ACTIVITY & RECEIPTS · THE WORKBENCH REMEMBERS ",
            )),
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
        Paragraph::new(body)
            .style(theme::raised())
            .wrap(Wrap { trim: false })
            .block(
                theme::modal(format!(" {title} ")).title_bottom(Line::styled(
                    " Esc close ",
                    Style::default().fg(theme::MUTED).bg(theme::PANEL_RAISED),
                )),
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
    fn palette_natural_verbs_resolve_through_the_same_behavior_command() {
        let mut commands = commands();
        commands.push(CommandSpec::available(
            "navigate.tellus",
            "Go to Tellus",
            "Inspect grounding provenance.",
            "Tellus",
            Some("Ctrl-6"),
            CommandKind::Navigate,
        ));
        let mut palette = CommandPalette::new(commands);
        palette.insert("open tellus");
        assert_eq!(palette.visible().len(), 1);
        assert_eq!(
            palette.key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            PaletteOutcome::Execute("navigate.tellus")
        );
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
    fn minibuffer_supports_completion_editing_selection_and_history() {
        let mut palette = CommandPalette::new(commands());
        palette.insert("refresh");
        assert_eq!(palette.cursor(), "refresh".len());
        assert_eq!(
            palette.key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
            PaletteOutcome::None
        );
        assert_eq!(palette.query(), "view.refresh");
        assert_eq!(
            palette.key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            PaletteOutcome::Execute("view.refresh")
        );
        palette.reset();
        palette.key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::ALT));
        assert_eq!(palette.query(), "view.refresh");
        palette.key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));
        palette.insert("x");
        assert_eq!(palette.query(), "xview.refresh");
        palette.key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert!(palette.query().is_empty());
        assert_eq!(
            palette.key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL)),
            PaletteOutcome::Dismiss
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
    fn instrumentation_traces_input_through_visible_feedback_frame() {
        let mut instrumentation = Instrumentation::default();
        let (operation, correlation) = instrumentation
            .begin_ui_operation_at("chronos.task.new", Instant::now())
            .expect("bounded UI span");
        assert!(instrumentation.mark_peer_stage(
            operation,
            "command_dispatched",
            ActivityLevel::Info,
        ));
        assert!(instrumentation.mark_peer_stage(
            operation,
            "feedback_presented",
            ActivityLevel::Info,
        ));
        assert!(instrumentation.mark_peer_stage(
            operation,
            "frame_rendered",
            ActivityLevel::Success,
        ));
        instrumentation.settle_peer_operation(operation, ActivityLevel::Info);
        let trace = instrumentation.traces().last().expect("UI trace");
        assert_eq!(trace.correlation_id, correlation);
        assert_eq!(trace.peer, "presentation");
        assert!(trace.complete);
        assert_eq!(
            trace
                .stages
                .iter()
                .map(|stage| stage.stage)
                .collect::<Vec<_>>(),
            [
                "input_classified",
                "command_resolved",
                "command_dispatched",
                "feedback_presented",
                "frame_rendered",
                "terminal_disposition",
            ]
        );
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
        assert!(rendered.contains("MINIBUFFER"));
        assert!(rendered.contains("M-x"));
        assert!(rendered.contains("Refresh current view"));
        terminal
            .draw(|frame| draw_inspector(frame, "Configuration", &["peer: ready".into()]))
            .unwrap();
    }
}
