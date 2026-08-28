//! Сборка support bundle: логи и окружение для разбора проблемы.
//!
//! Состав — фиксированный allowlist в коде, а не «всё из папки»: игрок должен
//! мочь заранее сказать, что именно уедет, и список не должен меняться сам
//! собой при появлении нового файла в каталоге.
//!
//! Не собирается никогда: `saves/`, `screenshots/`, `servers.dat`.
//!
//! Всё содержимое проходит через [`crate::redact`] до упаковки — и тот же текст
//! показывается игроку по кнопке «Посмотреть, что отправится». Без неё фича
//! неотличима от слежки; с ней игрок видит `C:\Users\*****` вместо своего имени.

mod collect;
mod environment;
mod pack;
mod send;

pub use collect::collect;
pub use pack::pack;
pub use send::{send, send_for_request};

use serde::{Deserialize, Serialize};

/// Один файл бандла — уже очищенный.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleFile {
    /// Имя внутри архива: `logs/latest.log`.
    pub name: String,
    pub text: String,
    /// Размер исходного файла на диске — чтобы показать игроку до отправки.
    pub original_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub files: Vec<BundleFile>,
    /// ОС, версия лаунчера, сборка, набор модов — то, что не лежит в логах.
    pub environment: String,
}

impl Bundle {
    /// Ровно тот текст, который уедет: и в предпросмотре, и в архиве.
    pub fn preview(&self) -> String {
        let mut out = format!("=== environment ===\n{}\n", self.environment);
        for f in &self.files {
            out.push_str(&format!(
                "\n=== {} ({} bytes on disk) ===\n{}\n",
                f.name, f.original_bytes, f.text
            ));
        }
        out
    }

    pub fn total_bytes(&self) -> u64 {
        self.files.iter().map(|f| f.text.len() as u64).sum()
    }
}
