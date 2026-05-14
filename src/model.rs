use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadSummary {
    pub id: String,
    pub title: String,
    pub cwd: String,
    pub model: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub tokens_used: i64,
    pub rollout_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TokenUsage {
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
}

impl TokenUsage {
    pub fn combined_with(self, other: TokenUsage) -> TokenUsage {
        TokenUsage {
            input_tokens: self.input_tokens + other.input_tokens,
            cached_input_tokens: self.cached_input_tokens + other.cached_input_tokens,
            output_tokens: self.output_tokens + other.output_tokens,
            reasoning_output_tokens: self.reasoning_output_tokens + other.reasoning_output_tokens,
            total_tokens: self.total_tokens + other.total_tokens,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RateLimit {
    pub primary_used_percent: Option<f64>,
    pub primary_resets_at: Option<i64>,
    pub secondary_used_percent: Option<f64>,
    pub secondary_resets_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionRecord {
    pub summary: ThreadSummary,
    pub usage: Option<TokenUsage>,
    pub rate_limit: Option<RateLimit>,
}

impl SessionRecord {
    pub fn effective_usage(&self) -> TokenUsage {
        self.usage.unwrap_or(TokenUsage {
            total_tokens: self.summary.tokens_used,
            ..TokenUsage::default()
        })
    }
}
