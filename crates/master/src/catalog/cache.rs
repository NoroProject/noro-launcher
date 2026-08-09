//! Кеш ответов каталогов модов.
//!
//! Modrinth и CurseForge рейт-лимитят, а админов немного и ходят они по одним и
//! тем же запросам: та же страница поиска, та же карточка, тот же список
//! версий. Пять минут в памяти дешевле, чем поймать 429 в момент, когда кто-то
//! собирает сборку.

use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Дольше держать нельзя: свежая версия мода должна появляться в списке в тот
/// же заход, а не через полчаса.
const TTL: Duration = Duration::from_secs(300);
/// Потолок записей: кеш живёт в памяти процесса и расти без границы не должен.
const MAX_ENTRIES: usize = 512;

struct Entry {
    stored_at: Instant,
    value: Arc<Value>,
}

#[derive(Clone, Default)]
pub struct HttpCache {
    entries: Arc<DashMap<String, Entry>>,
}

impl HttpCache {
    pub fn get(&self, key: &str) -> Option<Arc<Value>> {
        let entry = self.entries.get(key)?;
        if entry.stored_at.elapsed() <= TTL {
            return Some(entry.value.clone());
        }
        // Ссылку надо отпустить до remove: DashMap шардирован, и удаление из
        // того же шарда под живым guard'ом встанет намертво.
        drop(entry);
        self.entries.remove(key);
        None
    }

    pub fn put(&self, key: String, value: Value) -> Arc<Value> {
        let value = Arc::new(value);
        if self.entries.len() >= MAX_ENTRIES {
            self.evict();
        }
        self.entries.insert(
            key,
            Entry {
                stored_at: Instant::now(),
                value: value.clone(),
            },
        );
        value
    }

    /// Сносим протухшее, а если всё живое — чистим целиком. LRU здесь не нужен:
    /// записи живут пять минут и стоят один повторный запрос, а бухгалтерия
    /// обращений стоила бы больше.
    fn evict(&self) {
        self.entries.retain(|_, e| e.stored_at.elapsed() <= TTL);
        if self.entries.len() >= MAX_ENTRIES {
            self.entries.clear();
        }
    }
}
