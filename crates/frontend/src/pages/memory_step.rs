//! Memory steppers move in whole gigabytes, the way players think of RAM.

/// One step of the memory steppers.
pub const STEP_MB: u32 = 1024;

/// The new `(min, max)` after one click on either stepper.
///
/// Values snap to whole gigabytes: older launchers stepped by 512 MB, and a
/// saved 1536 would otherwise stay half a gigabyte off forever. The ceiling is
/// floored to a gigabyte too, so the top step is a round number. The maximum
/// never drops below the minimum.
pub fn step(min: u32, max: u32, is_min: bool, up: bool, ceiling: u32) -> (u32, u32) {
    let top = (ceiling / STEP_MB).max(1) * STEP_MB;
    let next = |value: u32| {
        let gb = if up {
            value / STEP_MB + 1
        } else {
            value.div_ceil(STEP_MB).saturating_sub(1)
        };
        (gb * STEP_MB).clamp(STEP_MB, top)
    };
    let (mut min, mut max) = (min, max);
    if is_min {
        min = next(min);
    } else {
        max = next(max);
    }
    (min, max.max(min))
}

/// `10240` → "10 GB", `1536` → "1.5 GB".
pub fn label(mb: u32) -> String {
    let gb = if mb.is_multiple_of(STEP_MB) {
        (mb / STEP_MB).to_string()
    } else {
        format!("{:.1}", mb as f64 / STEP_MB as f64)
    };
    let mut args = i18n::FluentArgs::new();
    args.set("gb", gb);
    i18n::t_args("settings-memory-gb", &args)
}

#[cfg(test)]
#[path = "memory_step_tests.rs"]
mod tests;
