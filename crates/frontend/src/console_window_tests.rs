use super::*;

fn line(level: GameLogLevel, text: &str) -> LogEntry {
    LogEntry {
        timestamp: 0,
        level,
        text: text.into(),
        thread: None,
        logger: None,
        body: text.into(),
        continuation: false,
    }
}

fn window() -> ConsoleWindow {
    let (_backend_rx, backend, _frontend_rx, _frontend) = bridge::create_pair();
    ConsoleWindow::new(
        Uuid::nil(),
        "GTMH".into(),
        Vec::new(),
        ConsoleSettings::default(),
        backend,
    )
}

/// What is shown has to stay what the list was told about.
fn assert_consistent(view: &ConsoleWindow) {
    assert_eq!(view.list_state.item_count(), view.visible.len());
    assert!(view.visible.iter().all(|&i| i < view.logs.len()));
    assert!(view.visible.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn new_lines_are_shown_as_they_come() {
    let mut view = window();
    view.append(vec![
        line(GameLogLevel::Info, "loading"),
        line(GameLogLevel::Warn, "odd jar"),
        line(GameLogLevel::Error, "crash"),
    ]);
    assert_eq!(view.visible, vec![0, 1, 2]);
    assert_consistent(&view);
}

#[test]
fn the_level_filter_and_search_narrow_the_log() {
    let mut view = window();
    view.append(vec![
        line(GameLogLevel::Info, "loading"),
        line(GameLogLevel::Warn, "odd jar"),
        line(GameLogLevel::Error, "crash in jar"),
    ]);
    view.settings.show_info = false;
    view.refilter();
    assert_eq!(view.visible, vec![1, 2]);

    view.set_search("JAR".into());
    assert_eq!(view.visible, vec![1, 2]);
    view.set_search("crash".into());
    assert_eq!(view.visible, vec![2]);
    assert_consistent(&view);

    // Lines that arrive while filtered are filtered too.
    view.append(vec![
        line(GameLogLevel::Error, "another crash"),
        line(GameLogLevel::Info, "crash?"),
    ]);
    assert_eq!(view.visible, vec![2, 3]);
    assert_consistent(&view);
}

#[test]
fn the_buffer_drops_the_oldest_lines() {
    let mut view = window();
    let batch = |from: usize, n: usize| {
        (from..from + n)
            .map(|i| {
                let level = if i % 3 == 0 {
                    GameLogLevel::Warn
                } else {
                    GameLogLevel::Info
                };
                line(level, &format!("line {i}"))
            })
            .collect::<Vec<_>>()
    };
    view.settings.show_info = false;
    view.refilter();
    view.append(batch(0, MAX_LOG_LINES - 5));
    view.append(batch(MAX_LOG_LINES - 5, 20));
    assert_eq!(view.logs.len(), MAX_LOG_LINES);
    assert_eq!(view.logs[0].text, "line 15");
    assert!(view
        .visible
        .iter()
        .all(|&i| view.logs[i].level == GameLogLevel::Warn));
    assert_consistent(&view);

    // One batch bigger than the whole buffer.
    view.append(batch(10_000, MAX_LOG_LINES + 7));
    assert_eq!(view.logs.len(), MAX_LOG_LINES);
    assert_consistent(&view);
}
