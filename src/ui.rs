use chrono::{Datelike, Duration, Local, TimeZone};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Alignment, Frame, Line, Modifier, Span, Style};
use ratatui::style::Color;
use ratatui::text::Text;
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListItem, ListState, Padding, Paragraph, Wrap,
};

use crate::app::{App, PrivacyMode, display_cwd, display_id, display_rollout, display_title};
use crate::model::{RateLimit, SessionRecord, TokenUsage};

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_summary(frame, root[0], app);
    draw_body(frame, root[1], app);
    draw_footer(frame, root[2], app);
}

fn draw_summary(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let totals = app.totals();
    let rate = if app.privacy.enabled() {
        "RATE LIMIT hidden in privacy mode".to_string()
    } else {
        app.latest_rate_limit()
            .map(format_rate_limit)
            .unwrap_or_else(|| "RATE LIMIT unavailable".to_string())
    };
    let privacy = if app.privacy.enabled() {
        Span::styled("  PRIVACY ON", subtle_badge_style())
    } else {
        Span::raw("")
    };
    let trend = trend_summary(&app.all_records);
    let text = vec![
        Line::from(vec![
            Span::styled(
                "TOKIDEX",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Codex token usage dashboard", muted_style()),
            privacy,
        ]),
        Line::from(vec![
            metric_span("TOTAL", totals.total_tokens, Color::Yellow),
            divider_span(),
            metric_span("INPUT", totals.input_tokens, Color::Green),
            divider_span(),
            metric_span("CACHED", totals.cached_input_tokens, Color::Blue),
        ]),
        Line::from(vec![
            metric_span("OUTPUT", totals.output_tokens, Color::Magenta),
            divider_span(),
            metric_span("REASONING", totals.reasoning_output_tokens, Color::LightRed),
            divider_span(),
            Span::styled(format!("RANGE {}", app.range.label()), label_style()),
            Span::styled(format!("  SESSIONS {}", app.visible.len()), label_style()),
        ]),
        trend_line(&trend, &rate),
    ];

    frame.render_widget(
        Paragraph::new(text).block(panel_block(" Summary ").padding(Padding::horizontal(1))),
        area,
    );
}

fn draw_body(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(49), Constraint::Percentage(51)])
        .split(area);

    if app.visible.is_empty() {
        let empty = Paragraph::new(Text::from(vec![
            Line::from(Span::styled("No Codex token records found", value_style())),
            Line::from(Span::styled(
                "Try another range or Codex home.",
                muted_style(),
            )),
        ]))
        .alignment(Alignment::Center)
        .block(panel_block(" Sessions "));
        frame.render_widget(empty, columns[0]);
        draw_detail(frame, columns[1], None, app);
        return;
    }

    let sessions_block = panel_block(" Sessions ");
    let sessions_inner = sessions_block.inner(columns[0]);
    frame.render_widget(sessions_block, columns[0]);

    let rows_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(4)])
        .split(sessions_inner);

    let show_session_name = rows_area[1].width >= 44;
    let header = if show_session_name {
        Line::from(vec![
            Span::styled("UPDATED", header_style()),
            Span::styled("    TOKENS", header_style()),
            Span::styled("     MODEL", header_style()),
            Span::styled("  SESSION", header_style()),
        ])
    } else {
        Line::from(vec![
            Span::styled("UPDATED", header_style()),
            Span::styled("    TOKENS", header_style()),
            Span::styled("     MODEL", header_style()),
        ])
    };
    let header = Paragraph::new(header);
    frame.render_widget(header, rows_area[0]);

    let items = app
        .visible
        .iter()
        .enumerate()
        .map(|item| session_row(item, app.privacy, show_session_name))
        .collect::<Vec<_>>();

    let mut state = ListState::default();
    state.select(Some(app.selected));
    frame.render_stateful_widget(
        List::new(items)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ "),
        rows_area[1],
        &mut state,
    );

    draw_detail(frame, columns[1], app.selected_record(), app);
}

fn draw_detail(frame: &mut Frame<'_>, area: Rect, record: Option<&SessionRecord>, app: &App) {
    let Some(record) = record else {
        frame.render_widget(
            Paragraph::new("Select a session").block(panel_block(" Details ")),
            area,
        );
        return;
    };

    let usage = record.usage;
    let effective = record.effective_usage();
    let detail_state = if usage.is_some() {
        "JSONL token_count"
    } else {
        "SQLite total only"
    };
    let rate = record
        .rate_limit
        .map(format_rate_limit)
        .unwrap_or_else(|| "rate limit: unavailable".to_string());
    let title_width = usize::from(area.width.saturating_sub(16)).min(36);
    let field_width = usize::from(area.width.saturating_sub(20)).max(12);
    let title = truncate_chars(
        &single_line(&display_title(record, app.selected, app.privacy)),
        title_width,
    );
    let lines = vec![
        Line::from(Span::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        section_line("Metadata"),
        clipped_field_line("id", display_id(record, app.privacy), field_width),
        field_line(
            "model",
            record.summary.model.as_deref().unwrap_or("unknown"),
        ),
        clipped_field_line("cwd", display_cwd(record, app.privacy), field_width),
        field_line("created", format_time(record.summary.created_at)),
        field_line("updated", format_time(record.summary.updated_at)),
        clipped_field_line("rollout", display_rollout(record, app.privacy), field_width),
        Line::from(""),
        section_line("Token Breakdown"),
        field_line("source", detail_state),
        field_line("total", format_number(effective.total_tokens)),
        field_line("input", format_number(effective.input_tokens)),
        field_line("cached input", format_number(effective.cached_input_tokens)),
        field_line("output", format_number(effective.output_tokens)),
        field_line(
            "reasoning output",
            format_number(effective.reasoning_output_tokens),
        ),
        field_line(
            "rate limit",
            if app.privacy.enabled() {
                "hidden in privacy mode".to_string()
            } else {
                rate
            },
        ),
    ];

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: false })
            .block(panel_block(" Details ").padding(Padding::horizontal(1))),
        area,
    );
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let suffix = if app.privacy.enabled() {
        "  PRIVACY"
    } else {
        ""
    };
    let prompt = if app.search_mode {
        Line::from(vec![
            Span::styled("SEARCH ", header_style()),
            Span::styled(format!("/{}", app.search), value_style()),
        ])
    } else if app.search.is_empty() {
        help_line(suffix)
    } else {
        Line::from(vec![
            Span::styled("FILTER ", header_style()),
            Span::styled(&app.search, value_style()),
            Span::styled(
                "   Esc clear  / edit  d/w/a range  r refresh",
                muted_style(),
            ),
            Span::styled(suffix, subtle_badge_style()),
        ])
    };
    frame.render_widget(
        Paragraph::new(prompt).block(panel_block(" Keys ").padding(Padding::horizontal(1))),
        area,
    );
}

fn session_row(
    (index, record): (usize, &SessionRecord),
    privacy: PrivacyMode,
    show_session_name: bool,
) -> ListItem<'static> {
    let model = short_model(record.summary.model.as_deref().unwrap_or("unknown"));
    let title = single_line(&display_title(record, index, privacy));
    let mut spans = vec![
        Span::styled(format_time(record.summary.updated_at), muted_style()),
        Span::raw("  "),
        Span::styled(
            format!(
                "{:>11}",
                format_number(record.effective_usage().total_tokens)
            ),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(format!("{model:<8}"), Style::default().fg(Color::Green)),
    ];
    if show_session_name {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(title, value_style()));
    }
    ListItem::new(Line::from(spans))
}

fn panel_block(title: &'static str) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(title, header_style()))
}

fn metric_span(label: &'static str, value: i64, color: Color) -> Span<'static> {
    Span::styled(
        format!("{label} {}", format_number(value)),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn divider_span() -> Span<'static> {
    Span::styled("  │  ", muted_style())
}

fn section_line(title: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled("▌ ", Style::default().fg(Color::Cyan)),
        Span::styled(title, header_style()),
    ])
}

fn field_line(label: &'static str, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<14}"), muted_style()),
        Span::styled(value.into(), value_style()),
    ])
}

fn clipped_field_line(
    label: &'static str,
    value: impl Into<String>,
    max_chars: usize,
) -> Line<'static> {
    field_line(label, truncate_chars(&value.into(), max_chars))
}

fn help_line(suffix: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("q", key_style()),
        Span::styled(" quit   ", muted_style()),
        Span::styled("↑/↓", key_style()),
        Span::styled(" move   ", muted_style()),
        Span::styled("/", key_style()),
        Span::styled(" search   ", muted_style()),
        Span::styled("d/w/a", key_style()),
        Span::styled(" range   ", muted_style()),
        Span::styled("r", key_style()),
        Span::styled(" refresh", muted_style()),
        Span::styled(suffix.to_string(), subtle_badge_style()),
    ])
}

fn header_style() -> Style {
    Style::default()
        .fg(Color::Gray)
        .add_modifier(Modifier::BOLD)
}

fn label_style() -> Style {
    Style::default().fg(Color::LightCyan)
}

fn value_style() -> Style {
    Style::default().fg(Color::White)
}

fn muted_style() -> Style {
    Style::default().fg(Color::Gray)
}

fn key_style() -> Style {
    Style::default()
        .fg(Color::LightYellow)
        .add_modifier(Modifier::BOLD)
}

fn subtle_badge_style() -> Style {
    Style::default()
        .fg(Color::LightCyan)
        .add_modifier(Modifier::BOLD)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrendSummary {
    sparkline: String,
    peak: i64,
    today: i64,
}

fn trend_summary(records: &[SessionRecord]) -> TrendSummary {
    let totals = recent_daily_totals(records, 7);
    TrendSummary {
        sparkline: sparkline(&totals),
        peak: totals.iter().copied().max().unwrap_or_default(),
        today: totals.last().copied().unwrap_or_default(),
    }
}

fn trend_line(trend: &TrendSummary, rate: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("7D TREND ", label_style()),
        Span::styled(
            trend.sparkline.clone(),
            Style::default()
                .fg(Color::LightMagenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  peak ", muted_style()),
        Span::styled(format_compact_number(trend.peak), value_style()),
        Span::styled("  today ", muted_style()),
        Span::styled(format_compact_number(trend.today), value_style()),
        Span::styled("  ", muted_style()),
        Span::styled(compact_rate(rate), muted_style()),
    ])
}

fn recent_daily_totals(records: &[SessionRecord], days: usize) -> Vec<i64> {
    if days == 0 {
        return Vec::new();
    }

    let today = Local::now().date_naive();
    let start = today - Duration::days(days.saturating_sub(1) as i64);
    let mut totals = vec![0; days];

    for record in records {
        let Some(updated_at) = Local.timestamp_opt(record.summary.updated_at, 0).single() else {
            continue;
        };
        let day = updated_at.date_naive();
        let offset = day.num_days_from_ce() - start.num_days_from_ce();
        if (0..days as i32).contains(&offset) {
            totals[offset as usize] += record.effective_usage().total_tokens;
        }
    }

    totals
}

fn sparkline(values: &[i64]) -> String {
    const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let peak = values.iter().copied().max().unwrap_or_default();
    if peak <= 0 {
        return "▁".repeat(values.len());
    }

    values
        .iter()
        .map(|value| {
            let ratio = (*value as f64 / peak as f64).clamp(0.0, 1.0);
            let index = (ratio * (BLOCKS.len() - 1) as f64).round() as usize;
            BLOCKS[index]
        })
        .collect()
}

fn format_rate_limit(rate: RateLimit) -> String {
    let primary = rate
        .primary_used_percent
        .map(|value| format!("{value:.0}%"))
        .unwrap_or_else(|| "n/a".to_string());
    let secondary = rate
        .secondary_used_percent
        .map(|value| format!("{value:.0}%"))
        .unwrap_or_else(|| "n/a".to_string());
    format!("rate limit: primary {primary}, secondary {secondary}")
}

fn format_time(timestamp: i64) -> String {
    Local
        .timestamp_opt(timestamp, 0)
        .single()
        .map(|value| value.format("%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "invalid".to_string())
}

fn single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn short_model(model: &str) -> String {
    let compact = model
        .strip_prefix("codex-auto-")
        .map(|rest| format!("auto-{rest}"))
        .or_else(|| model.strip_prefix("codex-").map(str::to_string))
        .unwrap_or_else(|| model.to_string());
    truncate_chars(&compact, 8)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

fn format_number(value: i64) -> String {
    let mut digits = value.abs().to_string();
    let mut out = String::new();
    while digits.len() > 3 {
        let chunk = digits.split_off(digits.len() - 3);
        if out.is_empty() {
            out = chunk;
        } else {
            out = format!("{chunk},{out}");
        }
    }
    if out.is_empty() {
        out = digits;
    } else if !digits.is_empty() {
        out = format!("{digits},{out}");
    }
    if value < 0 { format!("-{out}") } else { out }
}

fn format_compact_number(value: i64) -> String {
    let abs = value.abs() as f64;
    let (scaled, suffix) = if abs >= 1_000_000_000.0 {
        (value as f64 / 1_000_000_000.0, "B")
    } else if abs >= 1_000_000.0 {
        (value as f64 / 1_000_000.0, "M")
    } else if abs >= 1_000.0 {
        (value as f64 / 1_000.0, "K")
    } else {
        return value.to_string();
    };
    format!("{scaled:.1}{suffix}")
}

fn compact_rate(rate: &str) -> String {
    rate.strip_prefix("rate limit: ")
        .map(|value| format!("rate {value}"))
        .or_else(|| {
            rate.strip_prefix("RATE LIMIT ")
                .map(|value| format!("rate {}", value.to_lowercase()))
        })
        .unwrap_or_else(|| rate.to_string())
}

#[allow(dead_code)]
fn _usage_for_docs(_: TokenUsage) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn sparkline_scales_values_to_blocks() {
        assert_eq!(sparkline(&[0, 50, 100]), "▁▅█");
        assert_eq!(sparkline(&[0, 0, 0]), "▁▁▁");
    }

    #[test]
    fn recent_daily_totals_groups_by_local_updated_day() {
        let today = Local::now().date_naive();
        let yesterday = today - Duration::days(1);
        let older = today - Duration::days(8);
        let records = vec![
            record_at(
                today
                    .and_hms_opt(9, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .timestamp(),
                10,
            ),
            record_at(
                today
                    .and_hms_opt(18, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .timestamp(),
                15,
            ),
            record_at(
                yesterday
                    .and_hms_opt(12, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .timestamp(),
                20,
            ),
            record_at(
                older
                    .and_hms_opt(12, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .timestamp(),
                99,
            ),
        ];

        let totals = recent_daily_totals(&records, 7);

        assert_eq!(totals[5], 20);
        assert_eq!(totals[6], 25);
        assert_eq!(totals.iter().sum::<i64>(), 45);
    }

    #[test]
    fn compact_numbers_keep_summary_short() {
        assert_eq!(format_compact_number(999), "999");
        assert_eq!(format_compact_number(12_300), "12.3K");
        assert_eq!(format_compact_number(12_300_000), "12.3M");
        assert_eq!(format_compact_number(1_230_000_000), "1.2B");
    }

    #[test]
    fn compact_rate_shortens_status_line() {
        assert_eq!(
            compact_rate("rate limit: primary 14%, secondary 35%"),
            "rate primary 14%, secondary 35%"
        );
        assert_eq!(
            compact_rate("RATE LIMIT hidden in privacy mode"),
            "rate hidden in privacy mode"
        );
    }

    fn record_at(updated_at: i64, total_tokens: i64) -> SessionRecord {
        SessionRecord {
            summary: crate::model::ThreadSummary {
                id: "id".to_string(),
                title: "title".to_string(),
                cwd: "/tmp".to_string(),
                model: Some("gpt-5.5".to_string()),
                created_at: updated_at,
                updated_at,
                tokens_used: total_tokens,
                rollout_path: PathBuf::from("/tmp/session.jsonl"),
            },
            usage: None,
            rate_limit: None,
        }
    }
}
