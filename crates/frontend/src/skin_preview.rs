//! Цикл превью скина: два независимых часа — покачивание конечностей идёт
//! всегда с одной скоростью, поворот стоит на месте, пока фигуру тянут мышью.

use crate::skin;
use crate::state::{LauncherUI, Page};
use gpui::Context;
use std::time::{Duration, Instant};

/// Период кадра. Замер: полный кадр (растеризация 560×680 + BGRA) — ~4 мс, так
/// что 60 к/с укладывается с запасом; на 30 к/с вращение заметно рвано.
const FRAME: Duration = Duration::from_millis(16);
/// Полный цикл взмаха рук и ног.
const SWAY_PERIOD_MS: f32 = 2400.0;
/// Скорость автоповорота — полный оборот примерно за 6 секунд.
const YAW_DEG_PER_SEC: f32 = 60.0;

/// Всё, что нужно фоновому рендеру для одного кадра.
pub(crate) struct FrameJob {
    skin: Vec<u8>,
    cape: Option<Vec<u8>>,
    yaw: f64,
    sway: f64,
}

impl LauncherUI {
    /// Сдвинуть часы и собрать задание на кадр. `None` — рендерить нечего.
    fn next_frame_job(&mut self, elapsed: Duration) -> Option<FrameJob> {
        let skin = self.skin_bytes.clone()?;
        if self.page != Page::Profile {
            return None;
        }
        let dt = elapsed.as_secs_f32().min(0.25); // после долгой паузы не прыгаем
        self.skin_sway = (self.skin_sway + dt * 1000.0 / SWAY_PERIOD_MS).fract();
        if !self.skin_dragging {
            self.skin_yaw = (self.skin_yaw + dt * YAW_DEG_PER_SEC).rem_euclid(360.0);
        }
        Some(FrameJob {
            skin,
            cape: self.cape_bytes.clone(),
            yaw: self.skin_yaw as f64,
            sway: self.skin_sway as f64,
        })
    }

    /// Довернуть фигуру рукой. Кадр подхватит цикл — отдельно рисовать не нужно.
    pub fn rotate_skin(&mut self, degrees: f32, cx: &mut Context<Self>) {
        self.skin_yaw = (self.skin_yaw + degrees).rem_euclid(360.0);
        cx.notify();
    }

    pub(crate) fn start_skin_animation(&mut self, cx: &mut Context<Self>) {
        if self.skin_anim_running {
            return;
        }
        self.skin_anim_running = true;
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let mut last = Instant::now();
            loop {
                let started = Instant::now();
                let elapsed = last.elapsed();
                last = started;
                let Ok(job) = this.update(cx, |state, _| state.next_frame_job(elapsed)) else {
                    break; // окно закрылось
                };
                let Some(job) = job else {
                    // Профиль закрыт или скина нет — цикл гасим совсем. Раньше
                    // он просыпался четырежды в секунду всё время работы
                    // лаунчера, хотя рисовать было нечего.
                    let _ = this.update(cx, |state, _| state.skin_anim_running = false);
                    break;
                };

                let frame = executor
                    .spawn(async move {
                        skin::render_view(&job.skin, job.cape.as_deref(), job.yaw, job.sway)
                    })
                    .await;

                let alive = this.update(cx, |state, cx| {
                    // Кадр не отрисовался — оставляем предыдущий, иначе моргнёт.
                    if let Some(frame) = frame {
                        state.skin_preview = Some(frame);
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    break;
                }
                // Пауза с учётом того, что кадр уже отнял. Раньше ждали FRAME
                // поверх рендера, и период выходил ~37 мс вместо 33 — то есть
                // частота всегда была ниже заявленной.
                executor
                    .timer(FRAME.saturating_sub(started.elapsed()))
                    .await;
            }
        })
        .detach();
    }
}
