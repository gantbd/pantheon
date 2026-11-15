//! Host-owned layout and focus routing for independent engine presentations.
#![forbid(unsafe_code)]

mod shell;

pub use shell::{Shell, ShellInput, Surface, shell_input};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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
            agora,
            moira: None,
        },
        focus,
        notice,
    );
}

pub struct PeerViews<'a> {
    pub chronos: Option<&'a chronos_tui::Hub>,
    pub vellum: Option<&'a EditorFrame>,
    pub agora: Option<&'a agora_tui::Component>,
    pub moira: Option<&'a moira_tui::Component>,
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
        agora,
        moira,
    } = peers;
    let [tabs, header, body] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(frame.area());
    let tab_line = shell
        .surfaces()
        .iter()
        .map(|surface| {
            if *surface == shell.active() {
                Span::raw(format!("[{}] ", surface.label()))
            } else {
                Span::raw(format!(" {}  ", surface.label()))
            }
        })
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(Line::from(tab_line)), tabs);
    let [chronos_area, vellum_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(body);
    frame.render_widget(
        Paragraph::new(format!(
            "Workbench | [/] surfaces | focus={focus:?} | Tab switches | Ctrl-Q exits | {notice}"
        )),
        header,
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
            if let Some(vellum) = vellum {
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
            chronos.is_some(),
            vellum.is_some(),
            agora.is_some(),
            moira.is_some(),
        ),
        Surface::Agora if agora.is_some() => {
            agora.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Moira if moira.is_some() => {
            moira.expect("matched above").draw_in(frame, body, true);
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

fn draw_overview(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    shell: &Shell,
    chronos_attached: bool,
    vellum_attached: bool,
    agora_attached: bool,
    moira_attached: bool,
) {
    let mut lines = vec![
        Line::from(format!(
            "Chronos: {}",
            if chronos_attached {
                "attached"
            } else {
                "not configured"
            }
        )),
        Line::from(format!(
            "Vellum: {}",
            if vellum_attached {
                "attached"
            } else {
                "not configured"
            }
        )),
        Line::from(format!(
            "Agora: {}",
            if agora_attached {
                "attached"
            } else {
                "not configured"
            }
        )),
        Line::from(format!(
            "Moira: {}",
            if moira_attached {
                "attached"
            } else {
                "not configured"
            }
        )),
    ];
    for surface in shell.surfaces() {
        if *surface == Surface::Agora && agora_attached {
            continue;
        }
        if *surface == Surface::Moira && moira_attached {
            continue;
        }
        if let Some(gap) = surface.gap() {
            lines.push(Line::from(format!("{}: gap — {gap}", surface.label())));
        }
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
    use agora_client::v2::{ClientV2, PortFailureV2, ServicePortV2};
    use agora_protocol::v2::{ClientIdV2, OutcomeV2, ReplyV2, RequestV2, RoomPageV2};
    use moira_client::SagaServicePort;
    use moira_protocol::{SagaCommandV1, SagaOutcomeV1, SagaPageV1, SagaReplyV1};
    use ratatui::{Terminal, backend::TestBackend};
    use std::{thread, time::Duration};

    struct AgoraPort;

    struct MoiraPort;

    impl SagaServicePort for MoiraPort {
        type Error = ();

        fn execute(&mut self, command: SagaCommandV1) -> Result<SagaReplyV1, Self::Error> {
            assert!(matches!(command, SagaCommandV1::Inspect { .. }));
            Ok(SagaReplyV1::Accepted(SagaOutcomeV1::Page(SagaPageV1 {
                entries: Vec::new(),
                next_after: None,
            })))
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
                            agora: None,
                            moira: Some(&moira),
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
