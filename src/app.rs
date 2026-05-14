use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Utc};

use crate::model::{RateLimit, SessionRecord, TokenUsage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateRange {
    Today,
    Week,
    All,
}

impl DateRange {
    pub fn label(self) -> &'static str {
        match self {
            DateRange::Today => "today",
            DateRange::Week => "week",
            DateRange::All => "all",
        }
    }
}

#[derive(Debug, Clone)]
pub struct App {
    pub all_records: Vec<SessionRecord>,
    pub visible: Vec<SessionRecord>,
    pub selected: usize,
    pub range: DateRange,
    pub search: String,
    pub search_mode: bool,
    pub status: String,
}

impl App {
    pub fn new(records: Vec<SessionRecord>, range: DateRange) -> Self {
        let mut app = App {
            all_records: records,
            visible: Vec::new(),
            selected: 0,
            range,
            search: String::new(),
            search_mode: false,
            status: String::new(),
        };
        app.recompute();
        app
    }

    pub fn replace_records(&mut self, records: Vec<SessionRecord>) {
        self.all_records = records;
        self.recompute();
    }

    pub fn recompute(&mut self) {
        let now = Utc::now();
        self.visible = filter_records(&self.all_records, self.range, &self.search, now)
            .into_iter()
            .cloned()
            .collect();
        if self.visible.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.visible.len() {
            self.selected = self.visible.len() - 1;
        }
    }

    pub fn selected_record(&self) -> Option<&SessionRecord> {
        self.visible.get(self.selected)
    }

    pub fn move_next(&mut self) {
        if !self.visible.is_empty() {
            self.selected = (self.selected + 1).min(self.visible.len() - 1);
        }
    }

    pub fn move_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn totals(&self) -> TokenUsage {
        self.visible
            .iter()
            .fold(TokenUsage::default(), |total, record| {
                total.combined_with(record.effective_usage())
            })
    }

    pub fn latest_rate_limit(&self) -> Option<RateLimit> {
        self.visible.iter().find_map(|record| record.rate_limit)
    }
}

pub fn filter_records<'a>(
    records: &'a [SessionRecord],
    range: DateRange,
    query: &str,
    now: DateTime<Utc>,
) -> Vec<&'a SessionRecord> {
    let query = query.trim().to_lowercase();
    records
        .iter()
        .filter(|record| record_in_range(record.summary.updated_at, range, now))
        .filter(|record| {
            if query.is_empty() {
                return true;
            }
            let haystack = format!(
                "{} {} {} {}",
                record.summary.title,
                record.summary.cwd,
                record.summary.model.as_deref().unwrap_or(""),
                record.summary.id
            )
            .to_lowercase();
            haystack.contains(&query)
        })
        .collect()
}

fn record_in_range(updated_at: i64, range: DateRange, now: DateTime<Utc>) -> bool {
    match range {
        DateRange::All => true,
        DateRange::Today => {
            let local_now = now.with_timezone(&Local);
            let start = Local
                .with_ymd_and_hms(
                    local_now.year(),
                    local_now.month(),
                    local_now.day(),
                    0,
                    0,
                    0,
                )
                .single()
                .unwrap_or(local_now);
            updated_at >= start.timestamp()
        }
        DateRange::Week => updated_at >= (now - Duration::days(7)).timestamp(),
    }
}
