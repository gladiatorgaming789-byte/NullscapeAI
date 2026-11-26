use nullscape_core::protocol::handle_line;
use nullscape_core::{Simulation, SimulationConfig};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn sim() -> Simulation {
    Simulation::new(SimulationConfig::default())
}

fn call(sim: &mut Simulation, line: &str) -> Value {
    serde_json::from_str(&handle_line(sim, line)).expect("response must be valid JSON")
}

fn is_error(v: &Value) -> bool {
    v.get("error").is_some()
}

#[test]
fn partial_actions_default_missing_fields() {
    let mut s = sim();
    let r = call(&mut s, r#"{"0":{"jump":true}}"#);
    assert!(!is_error(&r), "{r}");
    assert_eq!(r[0]["tick"], 1);
    assert_eq!(r.as_array().unwrap().len(), 2);
}

#[test]
fn empty_object_advances_one_tick() {
    let mut s = sim();
    assert_eq!(call(&mut s, "{}")[0]["tick"], 1);
}

#[test]
fn invalid_requests_return_errors_and_do_not_advance() {
    let mut s = sim();
    for bad in [
        "not json",
        "",
        "[1,2,3]",
        r#"{"0":{"jmp":true}}"#,          // typo'd field
        r#"{"0":{"move_x":"left"}}"#,     // wrong type
        r#"{"7":{}}"#,                    // unknown agent
        r#"{"zero":{}}"#,                 // non-numeric id
        r#"{"-1":{}}"#,                   // negative id
        r#"{"cmd":"explode"}"#,           // unknown command
        r#"{"cmd":5}"#,                   // cmd not a string
        r#"{"cmd":"reset","seed":-1}"#,
        r#"{"cmd":"reset","seed":4294967296}"#,
        r#"{"cmd":"reset","seed":"x"}"#,
    ] {
        let r = call(&mut s, bad);
        assert!(is_error(&r), "expected error for {bad:?}, got {r}");
    }
    assert_eq!(s.tick, 0, "failed requests must not advance the simulation");
    assert_eq!(call(&mut s, "{}")[0]["tick"], 1, "a valid request still works afterwards");
}

#[test]
fn one_bad_agent_action_prevents_the_whole_step() {
    let mut s = sim();
    let r = call(&mut s, r#"{"0":{"jump":true},"1":{"nope":1}}"#);
    assert!(is_error(&r));
    assert_eq!(s.tick, 0);
}

#[test]
fn error_messages_are_helpful() {
    let mut s = sim();
    let r = call(&mut s, r#"{"0":{"jmp":true}}"#);
    let msg = r["error"].as_str().unwrap();
    assert!(msg.contains("jmp") && msg.contains("agent 0"), "{msg}");
    let r = call(&mut s, r#"{"5":{}}"#);
    assert!(r["error"].as_str().unwrap().contains("unknown agent id 5"));
}

#[test]
fn reset_restores_tick_zero_and_matches_a_fresh_simulation() {
    let mut s = sim();
    for _ in 0..30 {
        call(&mut s, r#"{"0":{"move_z":-1.0}}"#);
    }
    let raw = handle_line(&mut s, r#"{"cmd":"reset","seed":42}"#);
    let fresh = Simulation::new(SimulationConfig { seed: 42, ..Default::default() });
    // byte-identical to a freshly constructed simulation's observations
    assert_eq!(raw, fresh.deterministic_fingerprint());
    let r: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(r[0]["tick"], 0);

    // reset without a seed keeps the current one
    call(&mut s, "{}");
    let raw = handle_line(&mut s, r#"{"cmd":"reset"}"#);
    assert_eq!(raw, fresh.deterministic_fingerprint());
}

#[test]
fn success_responses_use_the_original_wire_format() {
    // Field order follows the struct definition and f32 values print in their
    // shortest form, exactly as the pre-protocol-module binary did.
    let mut s = sim();
    let raw = handle_line(&mut s, "{}");
    assert!(raw.starts_with(r#"[{"tick":1,"agent_id":0,"position":["#), "{raw}");
}

/// End to end through the real binary: N request lines in => exactly N
/// response lines out, no matter how many of them are invalid. This is the
/// property that used to hang the Python client.
#[test]
fn binary_replies_once_per_line_even_for_bad_input() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nullscape-headless"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn headless binary");

    let requests = [
        r#"{"0":{"move_z":-1.0}}"#,
        "this is not json",
        r#"{"0":{"jump":true}}"#, // partial action: used to be rejected silently
        "",
        r#"{"cmd":"reset","seed":5}"#,
        r#"{"1":{"attack":true}}"#,
    ];
    {
        let stdin = child.stdin.as_mut().unwrap();
        for r in requests {
            writeln!(stdin, "{r}").unwrap();
        }
    }
    drop(child.stdin.take()); // EOF => binary exits

    let lines: Vec<String> = BufReader::new(child.stdout.take().unwrap())
        .lines()
        .map(|l| l.unwrap())
        .collect();
    child.wait().unwrap();

    assert_eq!(lines.len(), requests.len(), "responses: {lines:#?}");
    let parsed: Vec<Value> = lines.iter().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert!(!is_error(&parsed[0]));
    assert!(is_error(&parsed[1]));
    assert!(!is_error(&parsed[2]));
    assert!(is_error(&parsed[3]));
    assert_eq!(parsed[4][0]["tick"], 0);
    assert_eq!(parsed[5][0]["tick"], 1);
}
