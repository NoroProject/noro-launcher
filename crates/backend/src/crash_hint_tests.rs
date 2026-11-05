use super::*;

fn scan(lines: &[&str]) -> Option<CrashHint> {
    let mut s = CrashScan::default();
    for line in lines {
        s.feed(line);
    }
    s.hint()
}

#[test]
fn a_clean_log_has_no_hint() {
    assert_eq!(
        scan(&[
            "[12:00:00] [Render thread/INFO]: Stopping!",
            "[12:00:00] [Render thread/INFO]: Saving worlds",
        ]),
        None
    );
}

#[test]
fn heap_errors_from_the_jvm() {
    assert_eq!(
        scan(&[
            "Error occurred during initialization of VM",
            "Could not reserve enough space for 8388608KB object heap",
        ]),
        Some(CrashHint::HeapTooLarge)
    );
    assert_eq!(
        scan(&["Initial heap size set to a larger value than the maximum heap size"]),
        Some(CrashHint::HeapTooLarge)
    );
}

#[test]
fn out_of_memory_in_game() {
    assert_eq!(
        scan(&["java.lang.OutOfMemoryError: Java heap space"]),
        Some(CrashHint::OutOfMemory)
    );
}

#[test]
fn wrong_java() {
    assert_eq!(
        scan(&["Exception in thread \"main\" java.lang.UnsupportedClassVersionError: net/minecraft/client/main/Main has been compiled by a more recent version of the Java Runtime (class file version 65.0)"]),
        Some(CrashHint::WrongJava)
    );
}

#[test]
fn graphics_driver_in_a_native_crash() {
    assert_eq!(
        scan(&[
            "# EXCEPTION_ACCESS_VIOLATION (0xc0000005) at pc=0x00007ffb, pid=1234",
            "# C  [atio6axx.dll+0x1a2b3c]",
        ]),
        Some(CrashHint::GraphicsDriver)
    );
    assert_eq!(
        scan(&["GLFW error 65542: WGL: The driver does not appear to support OpenGL"]),
        Some(CrashHint::GraphicsDriver)
    );
}

#[test]
fn mod_conflicts() {
    assert_eq!(
        scan(&["org.spongepowered.asm.mixin.transformer.throwables.MixinTransformerError: An unexpected critical error was encountered"]),
        Some(CrashHint::ModConflict)
    );
    assert_eq!(
        scan(&["net.fabricmc.loader.impl.FormattedException: Mod resolution encountered an incompatible mod set!"]),
        Some(CrashHint::ModConflict)
    );
}

#[test]
fn the_surest_cause_wins_whatever_the_order() {
    // Out of memory often drags a mixin failure behind it; memory is the cause.
    assert_eq!(
        scan(&[
            "java.lang.OutOfMemoryError: Java heap space",
            "Mixin apply failed mixins.foo.json:Bar",
        ]),
        Some(CrashHint::OutOfMemory)
    );
    assert_eq!(
        scan(&[
            "Mixin apply failed mixins.foo.json:Bar",
            "java.lang.OutOfMemoryError: Java heap space",
        ]),
        Some(CrashHint::OutOfMemory)
    );
}

#[test]
fn every_hint_has_a_distinct_key() {
    let hints = [
        CrashHint::ModConflict,
        CrashHint::GraphicsDriver,
        CrashHint::DiskFull,
        CrashHint::OutOfMemory,
        CrashHint::JvmFlags,
        CrashHint::WrongJava,
        CrashHint::HeapTooLarge,
    ];
    let keys: std::collections::HashSet<_> = hints.iter().map(|h| h.key()).collect();
    assert_eq!(keys.len(), hints.len());
}
