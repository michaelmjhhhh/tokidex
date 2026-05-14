use std::fs;
use std::path::Path;

use rusqlite::Connection;
use tempfile::TempDir;
use tokidex::app::{
    DateRange, PrivacyMode, display_cwd, display_id, display_rollout, display_title, filter_records,
};
use tokidex::codex_store::{load_records, resolve_codex_home_from};

fn create_state_db(codex_home: &Path) {
    let conn = Connection::open(codex_home.join("state_5.sqlite")).unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE threads (
            id TEXT PRIMARY KEY,
            rollout_path TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            source TEXT NOT NULL DEFAULT '',
            model_provider TEXT NOT NULL DEFAULT '',
            cwd TEXT NOT NULL,
            title TEXT NOT NULL,
            sandbox_policy TEXT NOT NULL DEFAULT '',
            approval_mode TEXT NOT NULL DEFAULT '',
            tokens_used INTEGER NOT NULL DEFAULT 0,
            has_user_event INTEGER NOT NULL DEFAULT 0,
            archived INTEGER NOT NULL DEFAULT 0,
            model TEXT
        );
        "#,
    )
    .unwrap();
}

fn insert_thread(
    codex_home: &Path,
    id: &str,
    rollout_path: &Path,
    created_at: i64,
    updated_at: i64,
    tokens_used: i64,
    model: &str,
    title: &str,
    cwd: &str,
) {
    let conn = Connection::open(codex_home.join("state_5.sqlite")).unwrap();
    conn.execute(
        r#"
        INSERT INTO threads (id, rollout_path, created_at, updated_at, cwd, title, tokens_used, model)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        (
            id,
            rollout_path.to_string_lossy().to_string(),
            created_at,
            updated_at,
            cwd,
            title,
            tokens_used,
            model,
        ),
    )
    .unwrap();
}

#[test]
fn load_records_reads_threads_sorted_by_updated_at_and_token_count_detail() {
    let temp = TempDir::new().unwrap();
    let codex_home = temp.path();
    create_state_db(codex_home);

    let rollout = codex_home.join("sessions/2026/05/14/thread.jsonl");
    fs::create_dir_all(rollout.parent().unwrap()).unwrap();
    fs::write(
        &rollout,
        r#"{"timestamp":"2026-05-14T15:00:00Z","type":"event_msg","payload":{"type":"token_count","info":null,"rate_limits":{"primary":{"used_percent":10.0},"secondary":{"used_percent":20.0}}}}
{"this is":"bad enough"}
not json
{"timestamp":"2026-05-14T15:01:00Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":10,"cached_input_tokens":4,"output_tokens":2,"reasoning_output_tokens":1,"total_tokens":12},"last_token_usage":{"input_tokens":10,"cached_input_tokens":4,"output_tokens":2,"reasoning_output_tokens":1,"total_tokens":12},"model_context_window":258400},"rate_limits":{"primary":{"used_percent":14.0,"window_minutes":300,"resets_at":1778786097},"secondary":{"used_percent":19.0,"window_minutes":10080,"resets_at":1779197553}}}}
{"timestamp":"2026-05-14T15:02:00Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":20,"cached_input_tokens":8,"output_tokens":3,"reasoning_output_tokens":2,"total_tokens":23}},"rate_limits":{"primary":{"used_percent":16.0},"secondary":{"used_percent":21.0}}}}"#,
    )
    .unwrap();

    let missing_rollout = codex_home.join("sessions/missing.jsonl");
    insert_thread(
        codex_home,
        "older",
        &missing_rollout,
        1_700_000_000,
        1_700_000_010,
        99,
        "gpt-5.5",
        "Older Session",
        "/tmp/older",
    );
    insert_thread(
        codex_home,
        "newer",
        &rollout,
        1_700_000_020,
        1_700_000_030,
        23,
        "gpt-5.5",
        "Newer Session",
        "/tmp/newer",
    );

    let records = load_records(codex_home).unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].summary.id, "newer");
    assert_eq!(records[0].usage.as_ref().unwrap().input_tokens, 20);
    assert_eq!(records[0].usage.as_ref().unwrap().cached_input_tokens, 8);
    assert_eq!(records[0].usage.as_ref().unwrap().output_tokens, 3);
    assert_eq!(
        records[0].usage.as_ref().unwrap().reasoning_output_tokens,
        2
    );
    assert_eq!(records[0].usage.as_ref().unwrap().total_tokens, 23);
    assert_eq!(
        records[0].rate_limit.as_ref().unwrap().primary_used_percent,
        Some(16.0)
    );
    assert!(records[1].usage.is_none());
    assert_eq!(records[1].summary.tokens_used, 99);
}

#[test]
fn filter_records_matches_query_and_date_ranges() {
    let temp = TempDir::new().unwrap();
    let codex_home = temp.path();
    create_state_db(codex_home);

    let first = codex_home.join("first.jsonl");
    let second = codex_home.join("second.jsonl");
    insert_thread(
        codex_home,
        "first",
        &first,
        1_700_000_000,
        1_700_000_000,
        10,
        "gpt-5.5",
        "Raycast review",
        "/work/raycast",
    );
    insert_thread(
        codex_home,
        "second",
        &second,
        1_700_086_400,
        1_700_086_400,
        20,
        "codex-auto-review",
        "Token tui",
        "/work/tokidex",
    );

    let records = load_records(codex_home).unwrap();
    let filtered = filter_records(
        &records,
        DateRange::All,
        "raycast",
        chrono::DateTime::from_timestamp(1_700_086_400, 0).unwrap(),
    );

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].summary.title, "Raycast review");

    let today_only = filter_records(
        &records,
        DateRange::Today,
        "",
        chrono::DateTime::from_timestamp(1_700_086_500, 0).unwrap(),
    );
    assert_eq!(today_only.len(), 1);
    assert_eq!(today_only[0].summary.id, "second");

    let week = filter_records(
        &records,
        DateRange::Week,
        "",
        chrono::DateTime::from_timestamp(1_700_086_500, 0).unwrap(),
    );
    assert_eq!(week.len(), 2);
}

#[test]
fn resolve_codex_home_prefers_cli_then_env_then_home_default() {
    let home = tempfile::tempdir().unwrap();
    let env_home = tempfile::tempdir().unwrap();
    let cli_home = tempfile::tempdir().unwrap();

    assert_eq!(
        resolve_codex_home_from(
            Some(cli_home.path().to_path_buf()),
            Some(env_home.path()),
            home.path()
        ),
        cli_home.path()
    );
    assert_eq!(
        resolve_codex_home_from(None, Some(env_home.path()), home.path()),
        env_home.path()
    );
    assert_eq!(
        resolve_codex_home_from(None, None, home.path()),
        home.path().join(".codex")
    );
}

#[test]
fn privacy_display_redacts_sensitive_session_fields() {
    let temp = TempDir::new().unwrap();
    let codex_home = temp.path();
    create_state_db(codex_home);

    let rollout = codex_home.join("sessions/2026/05/14/rollout-019e-secret.jsonl");
    insert_thread(
        codex_home,
        "019e1f50-9388-7cb2-9825-6a1eea43f79c",
        &rollout,
        1_700_000_000,
        1_700_000_000,
        10,
        "gpt-5.5",
        "Private client roadmap",
        "/Users/example/Documents/Codex/private-client",
    );

    let records = load_records(codex_home).unwrap();
    let record = &records[0];

    assert_eq!(display_title(record, 0, PrivacyMode::On), "Session 1");
    assert_eq!(
        display_id(record, PrivacyMode::On),
        "019e...f79c".to_string()
    );
    assert_eq!(display_cwd(record, PrivacyMode::On), "private-client");
    assert_eq!(
        display_rollout(record, PrivacyMode::On),
        "hidden in privacy mode"
    );
    assert!(!display_cwd(record, PrivacyMode::On).contains("/Users/example"));
    assert!(!display_rollout(record, PrivacyMode::On).contains("/Users/example"));
}
