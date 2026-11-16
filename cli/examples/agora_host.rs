#![forbid(unsafe_code)]

use agora_client::v2::{ClientV2, PortFailureV2, ServicePortV2};
use agora_protocol::v2::{
    AdmissionPolicyV2, ClientIdV2, CommandV2, MessagePageV2, MessageV2, OutcomeV2, ReplyV2,
    RequestV2, RoleV2, RoomPageV2, RoomSummaryV2,
};
use pantheon::{Shell, Surface};
use pantheon_cli::run_multi_peer_terminal;
use vellum_sdk_client::protocol::{
    ClientId, ReplyV1, RequestV1, ServiceAccessErrorV1, ServiceControlErrorV1, ServicePort,
};

struct FixtureAgora;

impl ServicePortV2 for FixtureAgora {
    type Error = ();

    fn execute(
        &mut self,
        _: ClientIdV2,
        request: RequestV2,
    ) -> Result<ReplyV2, PortFailureV2<Self::Error>> {
        let outcome = match request.command {
            CommandV2::Rooms { .. } => OutcomeV2::Rooms(RoomPageV2 {
                revision: 7,
                rooms: vec![RoomSummaryV2 {
                    room_id: "atlas".into(),
                    name: "Atlas Room".into(),
                    admission: AdmissionPolicyV2::InviteOnly,
                    observer_role: RoleV2::Member,
                    member_count: 3,
                    retained_message_count: 1,
                    next_message_sequence: 2,
                }],
                next: None,
            }),
            CommandV2::Messages { room_id, .. } => OutcomeV2::Messages(MessagePageV2 {
                revision: 7,
                room_id,
                messages: vec![MessageV2 {
                    sequence: 1,
                    author: "Ada".into(),
                    logical_time: 1,
                    body: "Pantheon Agora peer is live".into(),
                    addressed_to: Vec::new(),
                }],
                oldest_retained_sequence: 1,
                next_message_sequence: 2,
                next: None,
            }),
            _ => return Ok(ReplyV2::Refused(agora_protocol::v2::RefusalV2::Unavailable)),
        };
        Ok(ReplyV2::Accepted(outcome))
    }
}

struct AbsentVellum;

impl ServicePort for AbsentVellum {
    fn connect(&mut self) -> Result<ClientId, ServiceControlErrorV1> {
        Err(ServiceControlErrorV1::Access(ServiceAccessErrorV1::NotSent))
    }

    fn execute(&mut self, _: RequestV1) -> Result<ReplyV1, ServiceAccessErrorV1> {
        Err(ServiceAccessErrorV1::NotSent)
    }

    fn disconnect(&mut self, _: ClientId) -> Result<(), ServiceControlErrorV1> {
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientV2::new(ClientIdV2(1), FixtureAgora);
    let agora = agora_tui::Component::new(client);
    let shell = Shell::new(vec![Surface::Agora])?;
    run_multi_peer_terminal::<AbsentVellum>(shell, None, None, Some(agora))
}
