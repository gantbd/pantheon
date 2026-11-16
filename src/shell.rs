use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use serde::{Deserialize, Serialize};

/// Closed presentation surfaces known by this Pantheon revision.
///
/// This is host navigation, not a universal component or semantic protocol.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Surface {
    Overview,
    Workspace,
    Agora,
    Moira,
    Daimon,
    Grounding,
    Aegis,
    Hermes,
}

impl Surface {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Workspace => "Workspace",
            Self::Agora => "Agora",
            Self::Moira => "Moira",
            Self::Daimon => "Daimon",
            Self::Grounding => "Grounding",
            Self::Aegis => "Aegis",
            Self::Hermes => "Hermes",
        }
    }

    pub const fn gap(self) -> Option<&'static str> {
        match self {
            Self::Overview | Self::Workspace => None,
            Self::Agora => Some("Agora has no composable Ratatui presentation peer"),
            Self::Moira => Some("Moira workflow presentation is not configured"),
            Self::Daimon => Some("Daimon operations presentation is not configured"),
            Self::Grounding => Some("Grounding presentation is not configured"),
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
        if surfaces.len() > 8 {
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

    pub fn dual_peer() -> Self {
        Self::new(vec![Surface::Workspace]).expect("static surface set is valid")
    }

    pub fn active(&self) -> Surface {
        self.surfaces[self.active]
    }

    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
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
}

pub fn shell_input(key: KeyEvent) -> Option<ShellInput> {
    if key.kind != KeyEventKind::Press || key.modifiers != KeyModifiers::NONE {
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
        assert!(Shell::new(Vec::new()).is_err());
        assert!(Shell::new(vec![Surface::Agora, Surface::Agora]).is_err());
    }
}
