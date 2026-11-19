use serde_json::Value;
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    process::Command,
    thread,
};

#[test]
fn headless_sme_plan_preserves_review_and_provenance_boundary() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        let mut chunk = [0_u8; 4 * 1024];
        loop {
            let count = stream.read(&mut chunk).unwrap();
            request.extend_from_slice(&chunk[..count]);
            let Some(separator) = request.windows(4).position(|part| part == b"\r\n\r\n") else {
                continue;
            };
            let headers = std::str::from_utf8(&request[..separator]).unwrap();
            let content_length = headers
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
                .unwrap()
                .parse::<usize>()
                .unwrap();
            if request.len() >= separator + 4 + content_length {
                break;
            }
        }
        let request = std::str::from_utf8(&request).unwrap();
        assert!(request.starts_with("POST /tellus/v1/sme-plan HTTP/1.1\r\n"));
        assert!(request.contains("\"fold_load_id\":\"fold\""));
        assert!(request.contains("\"tags\":[\"data-handling\"]"));

        let body = serde_json::json!({
            "version": 1,
            "capability": "tellus.sme-plan.v1",
            "status": "review_required",
            "complete": false,
            "next_action": "add independent coverage",
            "fold": {"load_id": "fold", "sha256": "01".repeat(32)},
            "lattice": {"load_id": "lattice", "sha256": "02".repeat(32)},
            "tags": ["data-handling"],
            "roles": [{
                "role": "privacy", "ref": "rune:privacy",
                "authority_sha256": "03".repeat(32),
                "surface_sha256": "04".repeat(32),
                "depth": "ambient", "surface_version": 2,
                "entries": [{"id": "purpose-boundary", "kind": "guidance", "text": "retain purpose"}]
            }],
            "orthogonality": {"all_orthogonal": false, "sufficient_roles": false, "pairs": []},
            "ensemble_question": "do we have the right ensemble?",
            "boundary": "SME routing and retrieval are advisory evidence, not stakeholder equity, independence, approval, authority, or proof"
        })
        .to_string();
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .unwrap();
    });

    let request_path = std::env::temp_dir().join(format!(
        "pantheon-sme-plan-{}-{}.json",
        std::process::id(),
        endpoint.port()
    ));
    fs::write(
        &request_path,
        r#"{"version":1,"fold_load_id":"fold","lattice_load_id":"lattice","tags":["data-handling"],"depth":"ambient","max_roles":4,"max_surface_bytes":65536}"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_pantheon"))
        .args([
            "tellus-sme-plan",
            &endpoint.to_string(),
            request_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let _ = fs::remove_file(request_path);
    server.join().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["capability"], "tellus.sme-plan.v1");
    assert_eq!(report["status"], "review_required");
    assert_eq!(report["complete"], false);
    assert_eq!(
        report["ensemble_question"],
        "do we have the right ensemble?"
    );
    assert_eq!(report["roles"][0]["ref"], "rune:privacy");
    assert_eq!(report["roles"][0]["authority_sha256"], "03".repeat(32));
    assert!(
        report["boundary"]
            .as_str()
            .unwrap()
            .contains("not stakeholder equity")
    );
}
