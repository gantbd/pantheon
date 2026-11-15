use chronos_tui::{Hub, Lifecycle};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};
use vellum_sdk_client::protocol::ServicePort;
use vellum_tui::{ServiceAction, ServiceEditor, ServiceInput, translate_service_input};

use crate::{
    ExitDecision, Focus, RoutedInput, Shell, ShellInput, Surface, decide_exit, draw_shell,
    route_key, shell_input,
};

/// Runs the composite terminal for host-supplied sessions. A Vellum-owned
/// integration host can bind handoff on the Hub before entering this loop.
pub fn run_terminal<T: ServicePort>(
    chronos: Hub,
    vellum: ServiceEditor<T>,
) -> Result<(), Box<dyn std::error::Error>> {
    run_multi_peer_terminal(Shell::dual_peer(), Some(chronos), Some(vellum), None)
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
    mut shell: Shell,
    mut chronos: Option<Hub>,
    mut vellum: Option<ServiceEditor<T>>,
    mut agora: Option<agora_tui::Component>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut focus = Focus::Chronos;
    let mut notice = String::from("workspace profile loaded");
    let mut confirming_discard = false;
    let mut last_vellum_poll = Instant::now();

    let result = ratatui::run(|terminal| -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Some(chronos) = &mut chronos {
                chronos.poll();
            }
            if let Some(agora) = &mut agora {
                agora.poll();
            }
            if last_vellum_poll.elapsed() >= Duration::from_millis(250) {
                if let Some(vellum) = &mut vellum
                    && let Err(error) = vellum.poll()
                {
                    notice = error.to_string();
                }
                last_vellum_poll = Instant::now();
            }
            let height = terminal.size()?.height.saturating_sub(2);
            let vellum_frame = vellum.as_ref().and_then(|editor| editor.frame(height));
            terminal.draw(|frame| {
                draw_shell(
                    frame,
                    &shell,
                    chronos.as_ref(),
                    vellum_frame.as_ref(),
                    agora.as_ref(),
                    focus,
                    &notice,
                )
            })?;
            if !event::poll(Duration::from_millis(25))? {
                continue;
            }
            match event::read()? {
                Event::Resize(..) => continue,
                Event::Paste(text)
                    if shell.active() == Surface::Workspace && focus == Focus::Vellum =>
                {
                    if let Some(vellum) = &mut vellum {
                        notice = match vellum.apply(ServiceAction::Insert(text)) {
                            Ok(()) => "Vellum edit accepted".into(),
                            Err(error) => error.to_string(),
                        };
                        confirming_discard = false;
                    } else {
                        notice = "Vellum service is not configured".into();
                    }
                }
                Event::Paste(_) => {
                    notice = "Paste is available only in the focused Vellum pane".into();
                }
                Event::Key(key) if shell_input(key).is_some() => {
                    match shell_input(key).expect("matched above") {
                        ShellInput::NextSurface => shell.next(),
                        ShellInput::PreviousSurface => shell.previous(),
                    }
                    confirming_discard = false;
                    notice = format!("surface moved to {}", shell.active().label());
                }
                Event::Key(key)
                    if shell.active() == Surface::Agora
                        && !(key.kind == KeyEventKind::Press
                            && key.modifiers == KeyModifiers::CONTROL
                            && key.code == KeyCode::Char('q')) =>
                {
                    if let Some(agora) = &mut agora {
                        if let Some(input) = agora_tui::translate_key(key)
                            && agora.update(input, true) == agora_tui::Lifecycle::CloseRequested
                        {
                            notice = "Use Ctrl-Q to exit the complete workbench".into();
                        }
                    } else {
                        notice = Surface::Agora
                            .gap()
                            .unwrap_or("Agora presentation is unavailable")
                            .into();
                    }
                }
                Event::Key(key) => match route_key(key, focus) {
                    RoutedInput::Exit => match decide_exit(
                        vellum
                            .as_ref()
                            .map(|editor| editor.workspace().projection().persistence())
                            .unwrap_or(
                                vellum_sdk_client::workspace::PersistenceProjectionV1::Clean,
                            ),
                        confirming_discard,
                    ) {
                        ExitDecision::Exit => break,
                        ExitDecision::ConfirmDiscard => {
                            confirming_discard = true;
                            notice =
                                "Unsaved Vellum edit; Ctrl-Q again discards, Esc cancels".into();
                        }
                        ExitDecision::RefuseIndeterminate => {
                            notice = "Vellum effect is indeterminate; repeat the exact action before exit"
                                .into();
                        }
                    },
                    RoutedInput::SwitchFocus => {
                        if shell.active() == Surface::Workspace {
                            focus = focus.other();
                            confirming_discard = false;
                            notice = format!("focus moved to {focus:?}");
                        } else {
                            notice = "Tab is local to the Workspace surface".into();
                        }
                    }
                    RoutedInput::Chronos(input) => {
                        if shell.active() != Surface::Workspace {
                            notice = format!("{} has no input peer", shell.active().label());
                        } else if let Some(chronos) = &mut chronos {
                            if chronos.update(input) == Lifecycle::CloseRequested {
                                notice = "Use Ctrl-Q to exit the complete workbench".into();
                            }
                        } else {
                            notice = "Chronos service is not configured".into();
                        }
                    }
                    RoutedInput::Vellum(_input) if shell.active() != Surface::Workspace => {
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
                        ServiceInput::Apply(action) => {
                            notice = match vellum.as_mut().expect("checked above").apply(action) {
                                Ok(()) => "Vellum action accepted".into(),
                                Err(error) => error.to_string(),
                            };
                            confirming_discard = false;
                        }
                        ServiceInput::Open => {
                            notice = "Workspace admission remains service/operator-owned in the composite"
                                .into();
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
    result?;
    disconnect?;
    Ok(())
}
