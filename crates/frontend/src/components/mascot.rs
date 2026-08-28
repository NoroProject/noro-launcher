use gpui::{img, prelude::*, px, AnyElement};

/// Mirrors what `assets/make-mascot.py` draws, not just what the UI currently
/// uses. Dropping an unused variant would leave a sprite with no way to show it.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Mood {
    Idle,
    Happy,
    Loading,
    Thinking,
    Sleeping,
}

impl Mood {
    fn file(self) -> &'static str {
        match self {
            Mood::Idle => "mascot-idle.png",
            Mood::Happy => "mascot-happy.png",
            Mood::Loading => "mascot-loading.png",
            Mood::Thinking => "mascot-thinking.png",
            Mood::Sleeping => "mascot-sleeping.png",
        }
    }
}

/// Sized by height; the width follows the sprite's aspect ratio.
pub fn mascot(mood: Mood, height: f32) -> AnyElement {
    img(mood.file())
        .h(px(height))
        .w(px(height * 44. / 52.))
        .flex_shrink_0()
        .into_any_element()
}
