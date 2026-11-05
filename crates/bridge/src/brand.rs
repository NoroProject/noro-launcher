//! The instance's name before the master has said it.
//!
//! One launcher build serves whichever network stamped it, so the name the
//! player sees comes from the master (`/api/public/settings`). These are the
//! stand-ins for the moments before that answer.

/// The product's own name. The master calls an instance nobody has named the
/// same, so the launcher and the site agree on it.
pub const DEFAULT_INSTANCE_NAME: &str = "Noro";

/// What the bootstrapper was stamped with, handed to the core at start.
pub fn stamped_instance_name() -> Option<String> {
    std::env::var("NORO_INSTANCE_NAME")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// The best name known without asking the master.
pub fn initial_instance_name() -> String {
    stamped_instance_name().unwrap_or_else(|| DEFAULT_INSTANCE_NAME.to_string())
}
