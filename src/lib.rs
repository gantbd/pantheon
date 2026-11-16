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
            daimon: None,
            daimon_agent: None,
            hermes: None,
            aegis: None,
            grounding: None,
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
    pub daimon: Option<&'a daimon_tui::Component>,
    pub daimon_agent: Option<&'a daimon_tui::AgentComponent>,
    pub hermes: Option<&'a hermes_tui::Component>,
    pub aegis: Option<&'a aegis_tui::Component>,
    pub grounding: Option<&'a grounding_tui::Component>,
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
        daimon,
        daimon_agent,
        hermes,
        aegis,
        grounding,
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
        ),
        Surface::Agora if agora.is_some() => {
            agora.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Moira if moira.is_some() => {
            moira.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Daimon if daimon_agent.is_some() => {
            daimon_agent
                .expect("matched above")
                .draw_in(frame, body, true);
        }
        Surface::Daimon if daimon.is_some() => {
            daimon.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Hermes if hermes.is_some() => {
            hermes.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Aegis if aegis.is_some() => {
            aegis.expect("matched above").draw_in(frame, body, true);
        }
        Surface::Grounding if grounding.is_some() => {
            grounding.expect("matched above").draw_in(frame, body, true);
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
            Surface::Overview | Surface::Workspace => true,
        };
        if !attached && let Some(gap) = surface.gap() {
            lines.push(Line::from(format!("{}: gap — {gap}", surface.label())));
        }
    }
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
                            daimon: None,
                            daimon_agent: None,
                            hermes: None,
                            aegis: None,
                            grounding: None,
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
                            agora: None,
                            moira: None,
                            daimon: Some(&daimon),
                            daimon_agent: None,
                            hermes: None,
                            aegis: None,
                            grounding: None,
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
                            agora: None,
                            moira: None,
                            daimon: None,
                            daimon_agent: None,
                            hermes: None,
                            aegis: Some(&aegis),
                            grounding: None,
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
