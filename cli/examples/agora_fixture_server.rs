#![forbid(unsafe_code)]

// Protocol fixture for the standalone Pantheon application package.

use std::{
    fs,
    io::{self, Read, Write},
    os::unix::{fs::PermissionsExt, net::UnixListener},
    path::PathBuf,
};

use agora_protocol::v2::{
    AdmissionPolicyV2, ClientIdV2, CommandV2, MessagePageV2, MessageV2, OutcomeV2, ReplyV2, RoleV2,
    RoomPageV2, RoomSummaryV2,
};
use agora_wire::{
    WireLimits,
    local_handshake::{
        CLIENT_HELLO_BYTES_V1, ServerIncarnationV1, decode_client_hello_v1, encode_server_hello_v1,
    },
    v2::{WireReplyV2, WireRequestV2, decode_request_v2, encode_reply_v2},
};

fn limits() -> WireLimits {
    WireLimits {
        max_frame_bytes: 8_192,
        max_text_bytes: 16 * 1_024,
        max_fields: 100_000,
        max_list_items: 4_096,
    }
}

fn read_frame(stream: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut length = [0_u8; 4];
    stream.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > limits().max_frame_bytes {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame limit"));
    }
    let mut frame = vec![0_u8; length];
    stream.read_exact(&mut frame)?;
    Ok(frame)
}

fn write_frame(stream: &mut impl Write, frame: &[u8]) -> io::Result<()> {
    stream.write_all(&(frame.len() as u32).to_be_bytes())?;
    stream.write_all(frame)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("socket path required")?;
    let listener = UnixListener::bind(&socket)?;
    fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))?;
    let (mut stream, _) = listener.accept()?;
    let mut hello = [0_u8; CLIENT_HELLO_BYTES_V1];
    stream.read_exact(&mut hello)?;
    decode_client_hello_v1(&hello).map_err(|_| "invalid client hello")?;
    stream.write_all(
        &encode_server_hello_v1(ServerIncarnationV1::new([9; 16]), 8_192, 8)
            .map_err(|_| "invalid server hello")?,
    )?;
    let WireRequestV2::Connect { participant_id } =
        decode_request_v2(&read_frame(&mut stream)?, limits()).map_err(|_| "connect decode")?
    else {
        return Err("connect expected".into());
    };
    if participant_id != "operator" {
        return Err("unexpected participant".into());
    }
    write_frame(
        &mut stream,
        &encode_reply_v2(&WireReplyV2::Connected(ClientIdV2(4)), limits())
            .map_err(|_| "connect encode")?,
    )?;
    loop {
        let frame = match read_frame(&mut stream) {
            Ok(frame) => frame,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(error) => return Err(error.into()),
        };
        let request = decode_request_v2(&frame, limits()).map_err(|_| "request decode")?;
        let (client, request) = match request {
            WireRequestV2::Execute { client, request } => (client, request),
            WireRequestV2::Disconnect(client) => {
                if client != ClientIdV2(4) {
                    return Err("unexpected client".into());
                }
                write_frame(
                    &mut stream,
                    &encode_reply_v2(&WireReplyV2::Disconnected, limits())
                        .map_err(|_| "disconnect encode")?,
                )?;
                break;
            }
            _ => return Err("execute or disconnect expected".into()),
        };
        if client != ClientIdV2(4) {
            return Err("unexpected client".into());
        }
        let outcome = match request.command {
            CommandV2::Rooms { .. } => OutcomeV2::Rooms(RoomPageV2 {
                revision: 11,
                rooms: vec![RoomSummaryV2 {
                    room_id: "profile-room".into(),
                    name: "Profile Room".into(),
                    admission: AdmissionPolicyV2::InviteOnly,
                    observer_role: RoleV2::Member,
                    member_count: 2,
                    retained_message_count: 1,
                    next_message_sequence: 2,
                }],
                next: None,
            }),
            CommandV2::Messages { room_id, .. } => OutcomeV2::Messages(MessagePageV2 {
                revision: 11,
                room_id,
                messages: vec![MessageV2 {
                    sequence: 1,
                    author: "Grace".into(),
                    logical_time: 1,
                    body: "Profile Agora transport is live".into(),
                    addressed_to: Vec::new(),
                }],
                oldest_retained_sequence: 1,
                next_message_sequence: 2,
                next: None,
            }),
            _ => return Err("unexpected command".into()),
        };
        write_frame(
            &mut stream,
            &encode_reply_v2(&WireReplyV2::Executed(ReplyV2::Accepted(outcome)), limits())
                .map_err(|_| "reply encode")?,
        )?;
    }
    Ok(())
}
