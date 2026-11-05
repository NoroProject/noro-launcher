//! Why the game died, in words a player can act on.
//!
//! The launcher used to say "the game exited with an error" and nothing more,
//! while the log a few lines up named the cause: too little memory, too much
//! memory, the wrong Java, a graphics driver, two mods that don't get along.
//! The scan watches the lines of one game as they go by and keeps the most
//! telling cause it saw.

/// Ordered by how sure a match makes us: a JVM that refused to start says
/// exactly what is wrong, while a mixin error may be a symptom of something
/// earlier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CrashHint {
    ModConflict,
    GraphicsDriver,
    DiskFull,
    OutOfMemory,
    JvmFlags,
    WrongJava,
    HeapTooLarge,
}

impl CrashHint {
    /// Translation key of the advice.
    pub fn key(self) -> &'static str {
        match self {
            CrashHint::ModConflict => "crash-hint-mod-conflict",
            CrashHint::GraphicsDriver => "crash-hint-graphics-driver",
            CrashHint::DiskFull => "crash-hint-disk-full",
            CrashHint::OutOfMemory => "crash-hint-out-of-memory",
            CrashHint::JvmFlags => "crash-hint-jvm-flags",
            CrashHint::WrongJava => "crash-hint-wrong-java",
            CrashHint::HeapTooLarge => "crash-hint-heap-too-large",
        }
    }
}

/// Lower-case fragments and what they mean. Checked against lower-cased lines.
const PATTERNS: &[(&str, CrashHint)] = &[
    // The JVM could not get the memory it was told to take: more than the
    // computer has free, or a 32-bit Java.
    (
        "could not reserve enough space for",
        CrashHint::HeapTooLarge,
    ),
    ("invalid maximum heap size", CrashHint::HeapTooLarge),
    (
        "insufficient memory for the java runtime",
        CrashHint::HeapTooLarge,
    ),
    (
        "initial heap size set to a larger value than the maximum",
        CrashHint::HeapTooLarge,
    ),
    ("unsupportedclassversionerror", CrashHint::WrongJava),
    (
        "compiled by a more recent version of the java runtime",
        CrashHint::WrongJava,
    ),
    ("unrecognized vm option", CrashHint::JvmFlags),
    ("unrecognized option:", CrashHint::JvmFlags),
    ("java.lang.outofmemoryerror", CrashHint::OutOfMemory),
    ("no space left on device", CrashHint::DiskFull),
    ("there is not enough space on the disk", CrashHint::DiskFull),
    ("pixel format not accelerated", CrashHint::GraphicsDriver),
    ("glfw error 65542", CrashHint::GraphicsDriver),
    ("glfw error 65543", CrashHint::GraphicsDriver),
    (
        "driver does not appear to support opengl",
        CrashHint::GraphicsDriver,
    ),
    // Driver libraries in the frame a native crash happened in: AMD, NVIDIA,
    // Intel.
    ("atio6axx", CrashHint::GraphicsDriver),
    ("atioglxx", CrashHint::GraphicsDriver),
    ("nvoglv", CrashHint::GraphicsDriver),
    ("icd64.dll", CrashHint::GraphicsDriver),
    ("icd32.dll", CrashHint::GraphicsDriver),
    ("mixin apply failed", CrashHint::ModConflict),
    ("mixinapplyerror", CrashHint::ModConflict),
    ("mixintransformererror", CrashHint::ModConflict),
    ("incompatible mod set", CrashHint::ModConflict),
    ("incompatible mods found", CrashHint::ModConflict),
    ("duplicate mods found", CrashHint::ModConflict),
    (
        "missing or unsupported mandatory dependencies",
        CrashHint::ModConflict,
    ),
];

#[derive(Debug, Default)]
pub struct CrashScan {
    found: Option<CrashHint>,
}

impl CrashScan {
    pub fn feed(&mut self, line: &str) {
        // Every pattern is ASCII, so the cheap check rules most lines out
        // before anything is allocated.
        if line.len() < 8 {
            return;
        }
        let lower = line.to_ascii_lowercase();
        for (fragment, hint) in PATTERNS {
            if lower.contains(fragment) && self.found.is_none_or(|f| *hint > f) {
                self.found = Some(*hint);
            }
        }
    }

    pub fn hint(&self) -> Option<CrashHint> {
        self.found
    }
}

#[cfg(test)]
#[path = "crash_hint_tests.rs"]
mod tests;
