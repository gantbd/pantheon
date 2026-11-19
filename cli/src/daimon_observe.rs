//! Exact, content-safe Daimon agent-session observation for bounded tool profiles.

use serde_json::{Value, json};
use std::{ffi::OsString, path::PathBuf};

pub const CAPABILITY: &str = "agents.session.inspect.v1";

pub fn run(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let usage = "daimon agent observe requires SOCKET AGENT_SESSION_HEX";
    let socket = PathBuf::from(args.next().ok_or(usage)?);
    let session_text = args
        .next()
        .ok_or(usage)?
        .into_string()
        .map_err(|_| "Daimon agent session must be UTF-8")?;
    if args.next().is_some() {
        return Err(usage.into());
    }
    let session = daimon_protocol::AgentSessionIdV1::new(parse_hex_16(&session_text)?);
    let (observation, complete) = crate::headless::observe_daimon_agent(socket, session);
    let status = observation
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("invalid_reply");
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "peer": "daimon",
            "capability": CAPABILITY,
            "complete": complete,
            "status": status,
            "session": session_text,
            "observation": observation,
            "boundary": "one exact owner session and its stable content-free event window; no prompt, assistant text, thought, terminal output, tool detail, file path, mutation, provider, filesystem, terminal, permission, or execution authority is transferred",
        }))?
    );
    if complete {
        Ok(())
    } else {
        Err(std::io::Error::other("Daimon agent observation unavailable").into())
    }
}

fn parse_hex_16(value: &str) -> Result<[u8; 16], Box<dyn std::error::Error>> {
    if value.len() != 32 {
        return Err("Daimon agent session must be exactly 32 lowercase hex characters".into());
    }
    let mut bytes = [0_u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let offset = index * 2;
        let pair = &value[offset..offset + 2];
        if !pair
            .bytes()
            .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value))
        {
            return Err("Daimon agent session must be exactly 32 lowercase hex characters".into());
        }
        *byte = u8::from_str_radix(pair, 16)?;
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_lowercase_session_identity_is_required() {
        assert_eq!(
            parse_hex_16("00112233445566778899aabbccddeeff").unwrap()[15],
            255
        );
        assert!(parse_hex_16("00112233445566778899AABBCCDDEEFF").is_err());
        assert!(parse_hex_16("00").is_err());
        assert!(parse_hex_16("zz112233445566778899aabbccddeeff").is_err());
    }
}
