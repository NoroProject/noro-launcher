//! Новости.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewsItem {
    pub id: Uuid,
    pub title: String,
    /// Markdown.
    pub body: String,
    pub preview_img_url: Option<String>,
    pub author_name: Option<String>,
    pub pinned: bool,
    pub published_at: DateTime<Utc>,
}
