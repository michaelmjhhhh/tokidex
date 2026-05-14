use chrono::{Local, TimeZone};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Alignment, Frame, Line, Modifier, Span, Style};
use ratatui::style::Color;
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::App;
use crate::model::{RateLimit, SessionRecord, TokenUsage};

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_summary(frame, root[0], app);
    draw_body(frame, root[1], app);
    draw_footer(frame, root[2], app);
}

fn draw_summary(frame: &mut Frame<'_>, area: ratatui::layout::Rect, app: &App) {
    let totals = app.totals();
    let rate = app
        .latest_rate_limit()
        .map(format_rate_limit)
        .unwrap_or_else(|| "rate limit: unavailable".to_string());
    let text = vec![
        Line::from(vec![
            Span::styled(
                "tokidex",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "  range: {}  sessions: {}",
                app.range.label(),
                app.visible.len()
            )),
        ]),
        Line::from(format!(
            "total: {}  input: {}  cached: {}  output: {}  reasoning: {}",
            format_number(totals.total_tokens),
            format_number(totals.input_tokens),
            format_number(totals.cached_input_tokens),
            format_number(totals.output_tokens),
            format_number(totals.reasoning_output_tokens)
        )),
        Line::from(rate),
    ];

    frame.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Codex token usage"),
        ),
        area,
    );
}

fn draw_body(frame: &mut Frame<'_>, area: ratatui::layout::Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(area);

    if app.visible.is_empty() {
        let empty = Paragraph::new("No Codex token records found")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Sessions"));
        frame.render_widget(empty, columns[0]);
        draw_detail(frame, columns[1], None);
        return;
    }

    let items = app
        .visible
        .iter()
        .map(|record| {
            let model = record.summary.model.as_deref().unwrap_or("unknown");
            let title = single_line(&record.summary.title);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format_time(record.summary.updated_at),
                    Style::default().fg(Color::Gray),
                ),
                Span::raw("  "),
                Span::styled(
                    format!(
                        "{:>10}",
                        format_number(record.effective_usage().total_tokens)
                    ),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  "),
                Span::styled(model.to_string(), Style::default().fg(Color::Green)),
                Span::raw("  "),
                Span::raw(title),
            ]))
        })
        .collect::<Vec<_>>();

    let mut state = ListState::default();
    state.select(Some(app.selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Sessions"))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> "),
        columns[0],
        &mut state,
    );

    draw_detail(frame, columns[1], app.selected_record());
}

fn draw_detail(frame: &mut Frame<'_>, area: ratatui::layout::Rect, record: Option<&SessionRecord>) {
    let Some(record) = record else {
        frame.render_widget(
            Paragraph::new("Select a session")
                .block(Block::default().borders(Borders::ALL).title("Details")),
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
    let lines = vec![
        Line::from(vec![Span::styled(
            single_line(&record.summary.title),
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(format!("id: {}", record.summary.id)),
        Line::from(format!(
            "model: {}",
            record.summary.model.as_deref().unwrap_or("unknown")
        )),
        Line::from(format!("cwd: {}", record.summary.cwd)),
        Line::from(format!(
            "created: {}",
            format_time(record.summary.created_at)
        )),
        Line::from(format!(
            "updated: {}",
            format_time(record.summary.updated_at)
        )),
        Line::from(format!(
            "rollout: {}",
            record.summary.rollout_path.display()
        )),
        Line::from(""),
        Line::from(format!("source: {detail_state}")),
        Line::from(format!("total: {}", format_number(effective.total_tokens))),
        Line::from(format!("input: {}", format_number(effective.input_tokens))),
        Line::from(format!(
            "cached input: {}",
            format_number(effective.cached_input_tokens)
        )),
        Line::from(format!(
            "output: {}",
            format_number(effective.output_tokens)
        )),
        Line::from(format!(
            "reasoning output: {}",
            format_number(effective.reasoning_output_tokens)
        )),
        Line::from(rate),
    ];

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Details")),
        area,
    );
}

fn draw_footer(frame: &mut Frame<'_>, area: ratatui::layout::Rect, app: &App) {
    let prompt = if app.search_mode {
        format!("/{}", app.search)
    } else if app.search.is_empty() {
        "q quit  ↑/↓ move  / search  d today  w week  a all  r refresh".to_string()
    } else {
        format!(
            "search: {}   q quit  ↑/↓ move  / edit  Esc clear  d/w/a range  r refresh",
            app.search
        )
    };
    frame.render_widget(
        Paragraph::new(prompt).block(Block::default().borders(Borders::ALL).title("Keys")),
        area,
    );
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

#[allow(dead_code)]
fn _usage_for_docs(_: TokenUsage) {}
