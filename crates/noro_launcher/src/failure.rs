//! What went wrong, sorted by what the player can do about it.

/// One line that fits under the message.
pub fn short(detail: &str) -> String {
    const MAX: usize = 140;
    let line = detail.lines().next().unwrap_or_default();
    if line.chars().count() <= MAX {
        line.to_string()
    } else {
        let cut: String = line.chars().take(MAX).collect();
        format!("{cut}…")
    }
}

/// What went wrong, in terms of what the player can do about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Couldn't talk to the master: check the connection, try again.
    Network,
    /// The master answered but has nothing usable for this system.
    Unavailable,
    /// The download doesn't match its hash or signature.
    Corrupt,
    /// Writing the files failed: a full disk, permissions, an antivirus.
    Disk,
    /// Core is in place but wouldn't start.
    Start,
}

impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Kind::Network => "boot-error-network",
            Kind::Unavailable => "boot-error-unavailable",
            Kind::Corrupt => "boot-error-corrupt",
            Kind::Disk => "boot-error-disk",
            Kind::Start => "boot-error-start",
        }
    }
}

pub struct Failure {
    pub kind: Kind,
    pub error: anyhow::Error,
}

impl Failure {
    pub fn new(kind: Kind, error: impl Into<anyhow::Error>) -> Self {
        Self {
            kind,
            error: error.into(),
        }
    }

    /// A 4xx or an unreadable answer won't change on retry; the rest is the
    /// link.
    pub fn http(e: reqwest::Error) -> Self {
        let kind = if e.is_decode() || e.status().is_some_and(|s| s.is_client_error()) {
            Kind::Unavailable
        } else {
            Kind::Network
        };
        Self::new(kind, e)
    }
}
