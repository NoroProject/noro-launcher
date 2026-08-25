//! Маскот лаунчера — картинка состояния.
//!
//! Пустой экран и долгое ожидание без него выглядели как поломка: игрок видел
//! иконку-заглушку и не понимал, ждать ему или уже всё сломалось.

use gpui::{img, prelude::*, px, AnyElement};

/// Настроение маскота. Файлы лежат в `assets/mascot-*.png`.
///
/// Перечисление зеркалит набор, который рисует `assets/make-mascot.py`, а не
/// только то, что сейчас показывается. Выкинуть неиспользуемый вариант значило
/// бы развести enum с генератором: картинка осталась бы, а способа её показать
/// не стало.
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

/// Картинка маскота заданной высоты; ширина подбирается по пропорции спрайта.
pub fn mascot(mood: Mood, height: f32) -> AnyElement {
    img(mood.file())
        .h(px(height))
        .w(px(height * 44. / 52.))
        .flex_shrink_0()
        .into_any_element()
}
