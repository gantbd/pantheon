use chronos_tui::{HandoffRequest, HandoffResult, Hub};
use std::{error::Error, sync::mpsc, thread};
use vellum_sdk_client::{
    protocol::{
        ClientId, ReplyV1, RequestV1, ServiceAccessErrorV1, ServiceControlErrorV1, ServicePort,
    },
    workspace::WorkspaceClientV1,
};
use vellum_tui::ServiceEditor;

use crate::{Shell, Surface, run_multi_peer_terminal, run_terminal};

enum Request {
    Connect(mpsc::SyncSender<Result<ClientId, ServiceControlErrorV1>>),
    Execute(
        RequestV1,
        mpsc::SyncSender<Result<ReplyV1, ServiceAccessErrorV1>>,
    ),
    Disconnect(
        ClientId,
        mpsc::SyncSender<Result<(), ServiceControlErrorV1>>,
    ),
    Handoff(HandoffRequest, mpsc::SyncSender<HandoffResult>),
    Stop,
}

#[derive(Clone)]
struct HostPort(mpsc::SyncSender<Request>);

impl HostPort {
    fn call<R>(
        &self,
        request: impl FnOnce(mpsc::SyncSender<R>) -> Request,
    ) -> Result<R, ServiceAccessErrorV1> {
        let (reply, receiver) = mpsc::sync_channel(1);
        self.0
            .send(request(reply))
            .map_err(|_| ServiceAccessErrorV1::NotSent)?;
        receiver
            .recv()
            .map_err(|_| ServiceAccessErrorV1::OutcomeUnknown)
    }
}

impl ServicePort for HostPort {
    fn connect(&mut self) -> Result<ClientId, ServiceControlErrorV1> {
        self.call(Request::Connect)
            .map_err(ServiceControlErrorV1::Access)?
    }

    fn execute(&mut self, request: RequestV1) -> Result<ReplyV1, ServiceAccessErrorV1> {
        self.call(|reply| Request::Execute(request, reply))?
    }

    fn disconnect(&mut self, client: ClientId) -> Result<(), ServiceControlErrorV1> {
        self.call(|reply| Request::Disconnect(client, reply))
            .map_err(ServiceControlErrorV1::Access)?
    }
}

/// Launches Workbench with one Vellum-owned service and handoff handler.
/// The factory creates both on the service's owner thread. The handler can
/// capture the live editor state through that same service. For an export,
/// transport uncertainty is reported as an unknown publication outcome.
pub fn run_hosted_terminal<S, H>(
    mut chronos: Hub,
    start: impl FnOnce() -> Result<(S, H), String> + Send + 'static,
) -> Result<(), Box<dyn Error>>
where
    S: ServicePort,
    H: FnMut(&mut S, HandoffRequest) -> HandoffResult,
{
    let (send, receive) = mpsc::sync_channel(1);
    let (started, ready) = mpsc::sync_channel(1);
    let owner = thread::spawn(move || {
        let (mut service, mut handoff) = match start() {
            Ok(parts) => parts,
            Err(error) => {
                let _ = started.send(Err(error));
                return;
            }
        };
        if started.send(Ok(())).is_err() {
            return;
        }
        while let Ok(request) = receive.recv() {
            match request {
                Request::Connect(reply) => {
                    let _ = reply.send(service.connect());
                }
                Request::Execute(request, reply) => {
                    let _ = reply.send(service.execute(request));
                }
                Request::Disconnect(client, reply) => {
                    let _ = reply.send(service.disconnect(client));
                }
                Request::Handoff(request, reply) => {
                    let _ = reply.send(handoff(&mut service, request));
                }
                Request::Stop => break,
            }
        }
    });
    let port = HostPort(send);
    let result = (|| -> Result<(), Box<dyn Error>> {
        ready.recv()??;
        let workspace = WorkspaceClientV1::connect(port.clone())
            .map_err(|error| format!("Vellum host refused: {error:?}"))?;
        let editor = ServiceEditor::open(workspace)?;
        let handoff_port = port.clone();
        chronos.bind_handoff(move |request| {
            match handoff_port.call(|reply| Request::Handoff(request, reply)) {
                Ok(result) => result,
                Err(ServiceAccessErrorV1::NotSent) => HandoffResult::Unavailable,
                Err(_) => HandoffResult::PublicationUnknown,
            }
        });
        run_terminal(chronos, editor)
    })();
    let _ = port.0.send(Request::Stop);
    owner
        .join()
        .map_err(|_| "Vellum owner thread stopped unexpectedly")?;
    result
}

/// Launches the hosted Vellum/Chronos composition with an independently
/// constructed Agora presentation peer. Agora's port and authentication
/// choices remain with the caller that constructed the component.
pub fn run_hosted_terminal_with_agora<S, H>(
    mut chronos: Hub,
    agora: agora_tui::Component,
    start: impl FnOnce() -> Result<(S, H), String> + Send + 'static,
) -> Result<(), Box<dyn Error>>
where
    S: ServicePort,
    H: FnMut(&mut S, HandoffRequest) -> HandoffResult,
{
    let (send, receive) = mpsc::sync_channel(1);
    let (started, ready) = mpsc::sync_channel(1);
    let owner = thread::spawn(move || {
        let (mut service, mut handoff) = match start() {
            Ok(parts) => parts,
            Err(error) => {
                let _ = started.send(Err(error));
                return;
            }
        };
        if started.send(Ok(())).is_err() {
            return;
        }
        while let Ok(request) = receive.recv() {
            match request {
                Request::Connect(reply) => {
                    let _ = reply.send(service.connect());
                }
                Request::Execute(request, reply) => {
                    let _ = reply.send(service.execute(request));
                }
                Request::Disconnect(client, reply) => {
                    let _ = reply.send(service.disconnect(client));
                }
                Request::Handoff(request, reply) => {
                    let _ = reply.send(handoff(&mut service, request));
                }
                Request::Stop => break,
            }
        }
    });
    let port = HostPort(send);
    let result = (|| -> Result<(), Box<dyn Error>> {
        ready.recv()??;
        let workspace = WorkspaceClientV1::connect(port.clone())
            .map_err(|error| format!("Vellum host refused: {error:?}"))?;
        let editor = ServiceEditor::open(workspace)?;
        let handoff_port = port.clone();
        chronos.bind_handoff(move |request| {
            match handoff_port.call(|reply| Request::Handoff(request, reply)) {
                Ok(result) => result,
                Err(ServiceAccessErrorV1::NotSent) => HandoffResult::Unavailable,
                Err(_) => HandoffResult::PublicationUnknown,
            }
        });
        let shell = Shell::new(vec![Surface::Workspace, Surface::Agora])?;
        run_multi_peer_terminal(shell, Some(chronos), Some(editor), Some(agora))
    })();
    let _ = port.0.send(Request::Stop);
    owner
        .join()
        .map_err(|_| "Vellum owner thread stopped unexpectedly")?;
    result
}
