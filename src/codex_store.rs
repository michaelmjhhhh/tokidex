use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;

use crate::model::{RateLimit, SessionRecord, ThreadSummary, TokenUsage};

pub fn resolve_codex_home(cli_value: Option<PathBuf>) -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not locate current user's home directory")?;
    Ok(resolve_codex_home_from(
        cli_value,
        std::env::var_os("CODEX_HOME").as_deref().map(Path::new),
        &home,
    ))
}

pub fn resolve_codex_home_from(
    cli_value: Option<PathBuf>,
    env_value: Option<&Path>,
    home_dir: &Path,
) -> PathBuf {
    if let Some(path) = cli_value {
        return expand_tilde(path, home_dir);
    }
    if let Some(path) = env_value {
        return expand_tilde(path.to_path_buf(), home_dir);
    }
    home_dir.join(".codex")
}

fn expand_tilde(path: PathBuf, home_dir: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if text == "~" {
        return home_dir.to_path_buf();
    }
    if let Some(rest) = text.strip_prefix("~/") {
        return home_dir.join(rest);
    }
    path
}

pub fn load_records(codex_home: &Path) -> Result<Vec<SessionRecord>> {
    let db_path = codex_home.join("state_5.sqlite");
    let conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .with_context(|| {
        format!(
            "failed to open Codex state database at {}",
            db_path.display()
        )
    })?;

    let mut stmt = conn.prepare(
        r#"
        SELECT id, rollout_path, created_at, updated_at, cwd, title, tokens_used, model
        FROM threads
        ORDER BY updated_at DESC, id DESC
        "#,
    )?;

    let summaries = stmt
        .query_map([], |row| {
            Ok(ThreadSummary {
                id: row.get(0)?,
                rollout_path: PathBuf::from(row.get::<_, String>(1)?),
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                cwd: row.get(4)?,
                title: row.get(5)?,
                tokens_used: row.get(6)?,
                model: row.get(7)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(summaries
        .into_iter()
        .map(|summary| {
            let parsed = read_last_token_count(&summary.rollout_path).unwrap_or_default();
            SessionRecord {
                summary,
                usage: parsed.usage,
                rate_limit: parsed.rate_limit,
            }
        })
        .collect())
}

#[derive(Default)]
struct ParsedSession {
    usage: Option<TokenUsage>,
    rate_limit: Option<RateLimit>,
}

fn read_last_token_count(path: &Path) -> Result<ParsedSession> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut parsed = ParsedSession::default();

    for line in reader.lines() {
        let Ok(line) = line else {
            continue;
        };
        let Ok(event) = serde_json::from_str::<RolloutEvent>(&line) else {
            continue;
        };
        if event.kind != "event_msg" || event.payload.kind != "token_count" {
            continue;
        }
        if let Some(rate_limit) = event.payload.rate_limits.map(Into::into) {
            parsed.rate_limit = Some(rate_limit);
        }
        if let Some(info) = event.payload.info {
            parsed.usage = Some(info.total_token_usage.into());
        }
    }

    Ok(parsed)
}

#[derive(Debug, Deserialize)]
struct RolloutEvent {
    #[serde(rename = "type")]
    kind: String,
    payload: EventPayload,
}

#[derive(Debug, Deserialize)]
struct EventPayload {
    #[serde(rename = "type")]
    kind: String,
    info: Option<TokenInfo>,
    rate_limits: Option<RateLimitsJson>,
}

#[derive(Debug, Deserialize)]
struct TokenInfo {
    total_token_usage: TokenUsageJson,
}

#[derive(Debug, Deserialize)]
struct TokenUsageJson {
    input_tokens: i64,
    cached_input_tokens: i64,
    output_tokens: i64,
    reasoning_output_tokens: i64,
    total_tokens: i64,
}

impl From<TokenUsageJson> for TokenUsage {
    fn from(value: TokenUsageJson) -> Self {
        TokenUsage {
            input_tokens: value.input_tokens,
            cached_input_tokens: value.cached_input_tokens,
            output_tokens: value.output_tokens,
            reasoning_output_tokens: value.reasoning_output_tokens,
            total_tokens: value.total_tokens,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RateLimitsJson {
    primary: Option<RateLimitWindowJson>,
    secondary: Option<RateLimitWindowJson>,
}

#[derive(Debug, Deserialize)]
struct RateLimitWindowJson {
    used_percent: Option<f64>,
    resets_at: Option<i64>,
}

impl From<RateLimitsJson> for RateLimit {
    fn from(value: RateLimitsJson) -> Self {
        RateLimit {
            primary_used_percent: value.primary.as_ref().and_then(|limit| limit.used_percent),
            primary_resets_at: value.primary.as_ref().and_then(|limit| limit.resets_at),
            secondary_used_percent: value
                .secondary
                .as_ref()
                .and_then(|limit| limit.used_percent),
            secondary_resets_at: value.secondary.as_ref().and_then(|limit| limit.resets_at),
        }
    }
}
