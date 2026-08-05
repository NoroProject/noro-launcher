//! Сопоставление нашей платформы с обозначениями Mojang.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    LinuxX64,
    LinuxArm64,
    MacX64,
    MacArm64,
    WindowsX64,
}

impl Platform {
    /// Все платформы, под которые бывает клиент: сборка готовится сразу для всех.
    pub const ALL: [Platform; 5] = [
        Platform::LinuxX64,
        Platform::LinuxArm64,
        Platform::MacX64,
        Platform::MacArm64,
        Platform::WindowsX64,
    ];

    pub fn host() -> Self {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "aarch64") => Platform::LinuxArm64,
            ("linux", _) => Platform::LinuxX64,
            ("macos", "x86_64") => Platform::MacX64,
            ("macos", _) => Platform::MacArm64,
            ("windows", _) => Platform::WindowsX64,
            _ => Platform::LinuxX64,
        }
    }

    /// Наш тег платформы ("linux-x86_64").
    pub fn tag(&self) -> &'static str {
        match self {
            Platform::LinuxX64 => "linux-x86_64",
            Platform::LinuxArm64 => "linux-aarch64",
            Platform::MacX64 => "macos-x86_64",
            Platform::MacArm64 => "macos-aarch64",
            Platform::WindowsX64 => "windows-x86_64",
        }
    }

    /// Ключ ОС в правилах version.json ("osx"/"linux"/"windows").
    pub fn mojang_os_name(&self) -> &'static str {
        match self {
            Platform::MacX64 | Platform::MacArm64 => "osx",
            Platform::LinuxX64 | Platform::LinuxArm64 => "linux",
            Platform::WindowsX64 => "windows",
        }
    }

    /// Архитектура для правил ("x86_64"/"arm64"/"x86").
    pub fn mojang_arch(&self) -> &'static str {
        match self {
            Platform::LinuxArm64 | Platform::MacArm64 => "aarch64",
            _ => "x86_64",
        }
    }

    /// Ключ компонента java-runtime ОС у Mojang.
    pub fn java_os_key(&self) -> &'static str {
        match self {
            Platform::LinuxX64 => "linux",
            Platform::LinuxArm64 => "linux", // официального arm64-рантайма может не быть
            Platform::MacX64 => "mac-os",
            Platform::MacArm64 => "mac-os-arm64",
            Platform::WindowsX64 => "windows-x64",
        }
    }

    /// Классификатор natives в старых version.json ("natives-osx" и т.п.).
    pub fn legacy_natives_classifier(&self) -> &'static str {
        match self {
            Platform::MacX64 | Platform::MacArm64 => "natives-macos",
            Platform::LinuxX64 | Platform::LinuxArm64 => "natives-linux",
            Platform::WindowsX64 => "natives-windows",
        }
    }
}
