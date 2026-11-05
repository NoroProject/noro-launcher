//! What streamer mode treats as private, where both sides of the bridge need
//! the same answer: the backend for the system toast, the window for the card.

use schema::notifications::categories;

/// Categories whose cards name people and quote them: private messages, and
/// the staff feed with its reporters, targets and reasons.
pub fn private_category(category: &str) -> bool {
    category == categories::MESSAGES || categories::STAFF.contains(&category)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_and_staff_feed_are_private() {
        assert!(private_category(categories::MESSAGES));
        assert!(private_category(categories::STAFF_REPORTS));
        assert!(!private_category(categories::SECURITY));
        assert!(!private_category(categories::PROJECT));
    }
}
