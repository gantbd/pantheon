#![forbid(unsafe_code)]

use chronos_tui::Lifecycle;
use crossterm::event::{self, Event};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use vellum_sdk_client::{
    transport::{
        AuthenticatedUnixConnector, FrameExchange, RemoteTransport, TransportError,
        TransportLimits, UnixConnector,
    },
    workspace::WorkspaceClientV1,
};
use vellum_tui::{ServiceAction, ServiceEditor, ServiceInput, translate_service_input};
use workbench_tui::{ExitDecision, Focus, RoutedInput, decide_exit, draw, route_key};

enum VellumConnector {
    Plain(UnixConnector),
    Authenticated(AuthenticatedUnixConnector),
}

impl FrameExchange for VellumConnector {
    fn exchange(
        &mut self,
        request: &[u8],
        max_reply_bytes: usize,
    ) -> Result<Vec<u8>, TransportError> {
        match self {
            Self::Plain(connector) => connector.exchange(request, max_reply_bytes),
            Self::Authenticated(connector) => connector.exchange(request, max_reply_bytes),
        }
    }
}

struct Options {
    chronos_directory: PathBuf,
    vellum_socket: PathBuf,
    vellum_credential: Option<PathBuf>,
    chronos_writes: bool,
    snapshot: bool,
}

fn options() -> Result<Options, Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let Some(first) = args.next() else {
        return Err(usage().into());
    };
    if first == "--help" {
        println!("{}", usage());
        std::process::exit(0);
    }
    let chronos_directory = PathBuf::from(first);
    let mut vellum_socket = None;
    let mut vellum_credential = None;
    let mut chronos_writes = false;
    let mut snapshot = false;
    while let Some(argument) = args.next() {
        if argument == "--vellum-service" && vellum_socket.is_none() {
            vellum_socket = Some(PathBuf::from(
                args.next().ok_or("--vellum-service requires SOCKET")?,
            ));
        } else if argument == "--vellum-credential" && vellum_credential.is_none() {
            vellum_credential = Some(PathBuf::from(
                args.next().ok_or("--vellum-credential requires PATH")?,
            ));
        } else if argument == "--chronos-allow-writes" && !chronos_writes {
            chronos_writes = true;
        } else if argument == "--snapshot" && !snapshot {
            snapshot = true;
        } else {
            return Err("unknown or duplicate option".into());
        }
    }
    if vellum_credential.is_some() && vellum_socket.is_none() {
        return Err("--vellum-credential requires --vellum-service".into());
    }
    Ok(Options {
        chronos_directory,
        vellum_socket: vellum_socket.ok_or("--vellum-service is required")?,
        vellum_credential,
        chronos_writes,
        snapshot,
    })
}

fn usage() -> &'static str {
    "workbench-tui CHRONOS_PRIVATE_SERVICE_DIRECTORY --vellum-service SOCKET [--vellum-credential PATH] [--chronos-allow-writes] [--snapshot]\nTab switches focus; Ctrl-Q exits after Vellum persistence checks. Each pane otherwise retains its own bindings and authority. --snapshot performs one bounded dual-service attachment probe without terminal mode."
}

fn vellum_connector(options: &Options) -> Result<VellumConnector, Box<dyn std::error::Error>> {
    if let Some(path) = &options.vellum_credential {
        let mut store = vellum_adapter_local_authentication::LocalCredentialStoreV1::default();
        let record = store
            .load(path)
            .map_err(|error| format!("Vellum credential refused: {error:?}"))?;
        let provider =
            vellum_adapter_local_authentication::LocalCredentialClaimProviderV1::new(record);
        Ok(VellumConnector::Authenticated(
            AuthenticatedUnixConnector::with_provider(options.vellum_socket.clone(), provider),
        ))
    } else {
        Ok(VellumConnector::Plain(UnixConnector::new(
            options.vellum_socket.clone(),
        )))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = options()?;
    let connector = vellum_connector(&options)?;
    let chronos_client = chronos_local::LocalClient::new(options.chronos_directory);
    let mut chronos = chronos_tui::Component::new(chronos_client, options.chronos_writes);
    let transport = RemoteTransport::new(connector, TransportLimits::reference())
        .map_err(|error| format!("Vellum transport refused: {error:?}"))?;
    let workspace = WorkspaceClientV1::connect(transport)
        .map_err(|error| format!("Vellum service refused: {error:?}"))?;
    let mut vellum = ServiceEditor::open(workspace)?;
    if options.snapshot {
        return snapshot(chronos, vellum);
    }
    let mut focus = Focus::Chronos;
    let mut notice = String::from("both services attached; no cross-engine workflow authority");
    let mut confirming_discard = false;
    let mut last_vellum_poll = Instant::now();

    let result = ratatui::run(|terminal| -> Result<(), Box<dyn std::error::Error>> {
        loop {
            chronos.poll();
            if last_vellum_poll.elapsed() >= Duration::from_millis(250) {
                if let Err(error) = vellum.poll() {
                    notice = error.to_string();
                }
                last_vellum_poll = Instant::now();
            }
            let height = terminal.size()?.height.saturating_sub(2);
            let vellum_frame = vellum.frame(height);
            terminal.draw(|frame| draw(frame, &chronos, vellum_frame.as_ref(), focus, &notice))?;
            if !event::poll(Duration::from_millis(25))? {
                continue;
            }
            match event::read()? {
                Event::Resize(..) => continue,
                Event::Paste(text) if focus == Focus::Vellum => {
                    notice = match vellum.apply(ServiceAction::Insert(text)) {
                        Ok(()) => "Vellum edit accepted".into(),
                        Err(error) => error.to_string(),
                    };
                    confirming_discard = false;
                }
                Event::Paste(_) => {
                    notice = "Paste is available only in the focused Vellum pane".into();
                }
                Event::Key(key) => match route_key(key, focus) {
                    RoutedInput::Exit => match decide_exit(
                        vellum.workspace().projection().persistence(),
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
                        focus = focus.other();
                        confirming_discard = false;
                        notice = format!("focus moved to {focus:?}");
                    }
                    RoutedInput::Chronos(input) => {
                        if chronos.update(input, true) == Lifecycle::CloseRequested {
                            notice = "Use Ctrl-Q to exit the complete workbench".into();
                        }
                    }
                    RoutedInput::Vellum(input) => match translate_service_input(input) {
                        ServiceInput::CancelQuit => {
                            confirming_discard = false;
                            notice = "discard cancelled".into();
                        }
                        ServiceInput::Apply(action) => {
                            notice = match vellum.apply(action) {
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
    let disconnect = vellum
        .disconnect()
        .map_err(|failure| format!("Vellum disconnect refused: {failure:?}"));
    result?;
    disconnect?;
    Ok(())
}

fn snapshot(
    mut chronos: chronos_tui::Component,
    vellum: ServiceEditor<RemoteTransport<VellumConnector>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !chronos.poll() {
        if Instant::now() >= deadline {
            return Err("Chronos attachment timed out".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let vellum_frame = vellum.frame(12);
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 18))?;
    terminal.draw(|frame| {
        draw(
            frame,
            &chronos,
            vellum_frame.as_ref(),
            Focus::Chronos,
            "dual-service snapshot",
        )
    })?;
    let rendered: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    if !rendered.contains("Observed") || !rendered.contains("Vellum") {
        return Err("dual-service snapshot was not current".into());
    }
    vellum
        .disconnect()
        .map_err(|failure| format!("Vellum disconnect refused: {failure:?}"))?;
    println!("dual-service snapshot current");
    Ok(())
}
