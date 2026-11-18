use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use serde::{Deserialize, Serialize};

/// Closed presentation surfaces known by this Pantheon revision.
///
/// This is host navigation, not a universal component or semantic protocol.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Surface {
    Overview,
    Agenda,
    Chronos,
    Vellum,
    /// Legacy side-by-side composition retained for version-one profiles.
    Workspace,
    Agora,
    Moira,
    Daimon,
    Tellus,
    Aegis,
    Hermes,
    Operations,
    Configuration,
}

impl Surface {
    /// Stable direct-navigation key shown in the surface rail. These are
    /// semantic rather than profile-relative, so a shortcut never changes
    /// meaning when a profile omits a surface.
    pub const fn direct_key(self) -> &'static str {
        match self {
            Self::Overview => "0",
            Self::Agenda => "R",
            Self::Chronos => "1",
            Self::Vellum => "2",
            Self::Agora => "3",
            Self::Moira => "4",
            Self::Daimon => "5",
            Self::Tellus => "6",
            Self::Aegis => "7",
            Self::Hermes => "8",
            Self::Operations => "9",
            Self::Configuration => ",",
            Self::Workspace => "w",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Agenda => "Agenda",
            Self::Chronos => "Chronos",
            Self::Vellum => "Vellum",
            Self::Workspace => "Workspace",
            Self::Agora => "Agora",
            Self::Moira => "Moira",
            Self::Daimon => "Daimon",
            Self::Tellus => "Tellus",
            Self::Aegis => "Aegis",
            Self::Hermes => "Hermes",
            Self::Operations => "Operations",
            Self::Configuration => "Configuration",
        }
    }

    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Overview => "◈",
            Self::Agenda => "☷",
            Self::Chronos => "◷",
            Self::Vellum => "✦",
            Self::Workspace => "⌘",
            Self::Agora => "◌",
            Self::Moira => "◇",
            Self::Daimon => "◆",
            Self::Tellus => "⌁",
            Self::Aegis => "⬡",
            Self::Hermes => "≋",
            Self::Operations => "⚙",
            Self::Configuration => "⌬",
        }
    }

    pub const fn tagline(self) -> &'static str {
        match self {
            Self::Overview => "the whole constellation, at a glance",
            Self::Agenda => "what needs a human—and why",
            Self::Chronos => "plans, workstreams, and temporal truth",
            Self::Vellum => "documents, selection, and deliberate craft",
            Self::Workspace => "plan with Chronos · shape with Vellum",
            Self::Agora => "rooms, conversation, and composition",
            Self::Moira => "sagas, decisions, and recovery",
            Self::Daimon => "agents, presence, and exact control",
            Self::Tellus => "grounded methods, context, and provenance",
            Self::Aegis => "authority, grants, and deliberate trust",
            Self::Hermes => "streams, cursors, and durable motion",
            Self::Operations => "topology, lifecycle, and clean exits",
            Self::Configuration => "what this session is—and is not—allowed to do",
        }
    }

    pub const fn gap(self) -> Option<&'static str> {
        match self {
            Self::Overview
            | Self::Agenda
            | Self::Chronos
            | Self::Vellum
            | Self::Workspace
            | Self::Operations
            | Self::Configuration => None,
            Self::Agora => Some("Agora has no composable Ratatui presentation peer"),
            Self::Moira => Some("Moira workflow presentation is not configured"),
            Self::Daimon => Some("Daimon operations presentation is not configured"),
            Self::Tellus => Some("Tellus grounding presentation is not configured"),
            Self::Aegis => Some("Aegis authority presentation is not configured"),
            Self::Hermes => Some("Hermes ledger presentation is not configured"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Shell {
    surfaces: Vec<Surface>,
    active: usize,
}

impl Shell {
    pub fn new(surfaces: Vec<Surface>) -> Result<Self, &'static str> {
        if surfaces.is_empty() {
            return Err("workspace profile must enable at least one surface");
        }
        if surfaces.len() > 12 {
            return Err("workspace profile enables too many surfaces");
        }
        for (index, surface) in surfaces.iter().enumerate() {
            if surfaces[..index].contains(surface) {
                return Err("workspace profile contains a duplicate surface");
            }
        }
        Ok(Self {
            surfaces,
            active: 0,
        })
    }

    pub fn workbench() -> Self {
        Self::new(vec![
            Surface::Overview,
            Surface::Agenda,
            Surface::Chronos,
            Surface::Vellum,
            Surface::Agora,
            Surface::Moira,
            Surface::Daimon,
            Surface::Tellus,
            Surface::Aegis,
            Surface::Hermes,
            Surface::Operations,
            Surface::Configuration,
        ])
        .expect("static surface set is valid")
    }

    /// Legacy two-peer shell retained for embedders that explicitly request
    /// only the independent Chronos and Vellum presentations.
    pub fn dual_peer() -> Self {
        Self::new(vec![Surface::Chronos, Surface::Vellum]).expect("static surface set is valid")
    }

    pub fn active(&self) -> Surface {
        self.surfaces[self.active]
    }

    pub const fn active_index(&self) -> usize {
        self.active
    }

    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    pub fn activate(&mut self, surface: Surface) -> bool {
        let Some(index) = self
            .surfaces
            .iter()
            .position(|candidate| *candidate == surface)
        else {
            return false;
        };
        self.active = index;
        true
    }

    pub fn activate_index(&mut self, index: usize) -> bool {
        if index >= self.surfaces.len() {
            return false;
        }
        self.active = index;
        true
    }

    pub fn next(&mut self) {
        self.active = (self.active + 1) % self.surfaces.len();
    }

    pub fn previous(&mut self) {
        self.active = self
            .active
            .checked_sub(1)
            .unwrap_or(self.surfaces.len() - 1);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellInput {
    NextSurface,
    PreviousSurface,
    Activate(usize),
    ActivateSurface(Surface),
}

pub fn shell_input(key: KeyEvent) -> Option<ShellInput> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    if matches!(key.modifiers, KeyModifiers::ALT | KeyModifiers::CONTROL) {
        let direct = match key.code {
            KeyCode::Char('0') => Some(Surface::Overview),
            KeyCode::Char('r') => Some(Surface::Agenda),
            KeyCode::Char('1') => Some(Surface::Chronos),
            KeyCode::Char('2') => Some(Surface::Vellum),
            KeyCode::Char('3') => Some(Surface::Agora),
            KeyCode::Char('4') => Some(Surface::Moira),
            KeyCode::Char('5') => Some(Surface::Daimon),
            KeyCode::Char('6') => Some(Surface::Tellus),
            KeyCode::Char('7') => Some(Surface::Aegis),
            KeyCode::Char('8') => Some(Surface::Hermes),
            KeyCode::Char('9') => Some(Surface::Operations),
            KeyCode::Char(',') => Some(Surface::Configuration),
            _ => None,
        };
        if let Some(surface) = direct {
            return Some(ShellInput::ActivateSurface(surface));
        }
    }
    if key.modifiers == KeyModifiers::ALT {
        let surface = match key.code {
            KeyCode::Char('h') => Surface::Overview,
            KeyCode::Char('r') => Surface::Agenda,
            KeyCode::Char('c') => Surface::Chronos,
            KeyCode::Char('v') => Surface::Vellum,
            KeyCode::Char('a') => Surface::Agora,
            KeyCode::Char('m') => Surface::Moira,
            KeyCode::Char('d') => Surface::Daimon,
            KeyCode::Char('g') => Surface::Tellus,
            KeyCode::Char('e') => Surface::Aegis,
            KeyCode::Char('l') => Surface::Hermes,
            KeyCode::Char('o') => Surface::Operations,
            KeyCode::Char('w') => Surface::Workspace,
            KeyCode::Char(',') => Surface::Configuration,
            _ => return None,
        };
        return Some(ShellInput::ActivateSurface(surface));
    }
    if key.modifiers != KeyModifiers::NONE {
        return None;
    }
    match key.code {
        KeyCode::Char(']') => Some(ShellInput::NextSurface),
        KeyCode::Char('[') => Some(ShellInput::PreviousSurface),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_is_closed_bounded_and_wraps() {
        let mut shell = Shell::new(vec![Surface::Overview, Surface::Workspace, Surface::Agora])
            .expect("valid shell");
        assert_eq!(shell.active(), Surface::Overview);
        shell.previous();
        assert_eq!(shell.active(), Surface::Agora);
        shell.next();
        assert_eq!(shell.active(), Surface::Overview);
        assert!(shell.activate(Surface::Workspace));
        assert_eq!(shell.active(), Surface::Workspace);
        assert!(!shell.activate(Surface::Hermes));
        assert_eq!(shell.active(), Surface::Workspace);
        assert!(Shell::new(Vec::new()).is_err());
        assert!(Shell::new(vec![Surface::Agora, Surface::Agora]).is_err());
    }

    #[test]
    fn direct_keys_are_semantic_and_do_not_steal_peer_number_keys() {
        assert_eq!(
            shell_input(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::ALT)),
            Some(ShellInput::ActivateSurface(Surface::Agora))
        );
        assert_eq!(
            shell_input(KeyEvent::new(KeyCode::Char('0'), KeyModifiers::CONTROL)),
            Some(ShellInput::ActivateSurface(Surface::Overview))
        );
        assert_eq!(
            shell_input(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE)),
            None
        );
        assert_eq!(
            shell_input(KeyEvent::new(KeyCode::Char(','), KeyModifiers::ALT)),
            Some(ShellInput::ActivateSurface(Surface::Configuration))
        );
        assert_eq!(
            shell_input(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::ALT)),
            Some(ShellInput::ActivateSurface(Surface::Agenda))
        );
    }
}
