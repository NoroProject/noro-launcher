//! Канал с клиентским модом разбора: слушатель на loopback.
//!
//! Мод не получает ни токена, ни URL админки — он шлёт намерение, лаунчер
//! ходит к мастеру сам. Слушатель поднимается на запуск игры и гаснет вместе с
//! ней: чем короче окно, тем меньше смысла в нём для чужой локальной страницы.

mod handshake;
mod intents;
#[cfg(test)]
mod live_tests;
mod master;
mod push;
mod server;
mod store;

pub use store::CaseStore;

use crate::backend::Ctx;
use mod_link::ToMod;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use uuid::Uuid;

#[derive(Default)]
struct Inner {
    /// Ключ из файла рукопожатия. Пуст, пока слушателя нет.
    key: String,
    instance_dir: Option<PathBuf>,
    /// Куда слать кадры, пока мод подключён. Соединение одно: панель у
    /// модератора тоже одна.
    tx: Option<UnboundedSender<ToMod>>,
    accept: Option<JoinHandle<()>>,
}

#[derive(Clone, Default)]
pub struct ModLink {
    inner: Arc<Mutex<Inner>>,
    store: CaseStore,
}

impl ModLink {
    pub fn store(&self) -> CaseStore {
        self.store.clone()
    }

    /// Поднять слушатель и положить файл рукопожатия в каталог инстанса.
    ///
    /// Ошибка здесь не повод не пустить игрока: без канала мод просто не
    /// найдёт лаунчер и останется без панели, а игра запустится как раньше.
    pub async fn start(&self, ctx: &Ctx, instance_dir: PathBuf) {
        self.stop().await;
        let listener = match TcpListener::bind("127.0.0.1:0").await {
            Ok(l) => l,
            Err(e) => return tracing::warn!("mod_link: порт не открылся: {e}"),
        };
        let port = match listener.local_addr() {
            Ok(a) => a.port(),
            Err(e) => return tracing::warn!("mod_link: адрес не прочитался: {e}"),
        };
        let key = handshake::new_key();
        if let Err(e) = handshake::write(&instance_dir, port, &key).await {
            return tracing::warn!("mod_link: рукопожатие не записано: {e}");
        }

        let accept = tokio::spawn(server::serve(ctx.clone(), self.clone(), listener));
        let mut guard = self.inner.lock();
        guard.key = key;
        guard.instance_dir = Some(instance_dir);
        guard.accept = Some(accept);
        tracing::info!(port, "mod_link: канал открыт");
    }

    /// Игра закрылась: гасим слушатель и убираем ключ.
    pub async fn stop(&self) {
        let (accept, dir) = {
            let mut guard = self.inner.lock();
            guard.key.clear();
            guard.tx = None;
            (guard.accept.take(), guard.instance_dir.take())
        };
        if let Some(accept) = accept {
            accept.abort();
        }
        if let Some(dir) = dir {
            handshake::remove(&dir).await;
        }
        self.store.close();
    }

    /// Мастер сказал, что дело изменилось. Перечитываем то, что на экране.
    pub fn case_updated(&self, ctx: &Ctx, case_id: Uuid) {
        let open = self.store.is_open(case_id);
        let (ctx, link) = (ctx.clone(), self.clone());
        tokio::spawn(async move {
            // Дело не на экране — обновится, когда его откроют. Очередь всё
            // равно перечитываем: там могла смениться строка.
            if open {
                push::refresh_case(&ctx, &link, case_id).await;
            }
            push::refresh_queue_in_place(&ctx, &link).await;
        });
    }

    pub fn send(&self, frame: ToMod) {
        if let Some(tx) = &self.inner.lock().tx {
            let _ = tx.send(frame);
        }
    }

    fn key(&self) -> String {
        self.inner.lock().key.clone()
    }

    fn attach(&self, tx: UnboundedSender<ToMod>) {
        self.inner.lock().tx = Some(tx);
    }

    fn detach(&self) {
        self.inner.lock().tx = None;
        self.store.close();
    }
}
