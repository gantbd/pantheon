//! Host-owned layout and focus routing for independent engine presentations.
#![forbid(unsafe_code)]

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    widgets::Paragraph,
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
    Chronos(chronos_tui::Input),
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
        Focus::Chronos => RoutedInput::Chronos(chronos_tui::translate_key(key)),
        Focus::Vellum => RoutedInput::Vellum(vellum_tui::translate_key(key)),
    }
}

/// Draws two independent presentation peers under one host-owned frame. This
/// function owns geometry only; it cannot read or mutate either engine.
pub fn draw(
    frame: &mut Frame<'_>,
    chronos: &chronos_tui::Component,
    vellum: Option<&EditorFrame>,
    focus: Focus,
    notice: &str,
) {
    let [header, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(frame.area());
    let [chronos_area, vellum_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(body);
    frame.render_widget(
        Paragraph::new(format!(
            "Workbench | focus={focus:?} | Tab switches | Ctrl-Q exits | {notice}"
        )),
        header,
    );
    chronos.draw_in(frame, chronos_area);
    if let Some(vellum) = vellum {
        vellum_tui::draw_in(frame, vellum_area, vellum, None);
    } else {
        vellum_tui::draw_empty_in(frame, vellum_area, Some("Vellum service not attached"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chronos_client::WireClientV1;
    use ratatui::{Terminal, backend::TestBackend};

    struct Unavailable;

    impl WireClientV1 for Unavailable {
        type Error = ();

        fn request(
            &mut self,
            _: chronos_protocol::Request,
        ) -> Result<chronos_protocol::Response, Self::Error> {
            Err(())
        }
    }

    #[test]
    fn one_frame_contains_both_independent_presentations() {
        let chronos = chronos_tui::Component::new(Unavailable, false);
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
        assert!(rendered.contains("Chronos"));
        assert!(rendered.contains("Vellum"));
        assert!(rendered.contains("vellum source"));
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
            RoutedInput::Chronos(chronos_tui::Input::Text('x'))
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
