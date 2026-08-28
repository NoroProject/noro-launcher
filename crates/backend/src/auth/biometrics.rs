//! Native biometric authentication (Touch ID on macOS, Windows Hello on Windows).

use anyhow::{anyhow, Result};

// Внешний процесс зовут только macOS и Windows: на Linux импорт оставался
// неиспользованным и валил сборку с `-D warnings`.
#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::process::Command;

pub fn authenticate_biometrics(reason: &str) -> Result<bool> {
    #[cfg(target_os = "macos")]
    {
        authenticate_macos_touch_id(reason)
    }
    #[cfg(target_os = "windows")]
    {
        authenticate_windows_hello(reason)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = reason;
        Err(anyhow!("Biometrics not supported on this OS"))
    }
}

#[cfg(target_os = "macos")]
fn authenticate_macos_touch_id(reason: &str) -> Result<bool> {
    let script = format!(
        r#"import LocalAuthentication
import Foundation
let context = LAContext()
var error: NSError?
if context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) {{
    let semaphore = DispatchSemaphore(value: 0)
    var success = false
    context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: "{}") {{ result, _ in
        success = result
        semaphore.signal()
    }}
    _ = semaphore.wait(timeout: .now() + 60)
    if success {{ exit(0) }} else {{ exit(1) }}
}} else {{
    exit(2)
}}"#,
        reason.replace('"', "\\\"")
    );

    let output = Command::new("swift").arg("-e").arg(&script).output()?;

    if output.status.code() == Some(0) {
        Ok(true)
    } else if output.status.code() == Some(1) {
        Ok(false)
    } else {
        Err(anyhow!(
            "Touch ID is not available or disabled on this device"
        ))
    }
}

/// Сборка PowerShell-скрипта вынесена из `cfg(windows)` намеренно: под Windows
/// этот крейт с macOS/Linux не собирается (у зависимостей C-код под MSVC), и
/// единственная ошибкоопасная часть — экранирование — иначе осталась бы вообще
/// без проверки. Здесь её покрывает тест на любой платформе.
#[cfg(any(target_os = "windows", test))]
fn windows_hello_script(reason: &str) -> String {
    // Кавычки удваиваются — так PowerShell экранирует их внутри строки.
    format!(
        r#"[Windows.Security.Credentials.UI.UserConsentVerifier, Windows.Security.Credentials.UI, ContentType=WindowsRuntime]
$asyncOp = [Windows.Security.Credentials.UI.UserConsentVerifier]::RequestVerificationAsync("{}")
$task = [System.Threading.Tasks.Task]::Run({{ $asyncOp.GetResults() }})
$task.Wait()
if ($task.Result -eq [Windows.Security.Credentials.UI.UserConsentVerificationResult]::Verified) {{ exit 0 }} else {{ exit 1 }}"#,
        reason.replace('"', "\"\"")
    )
}

#[cfg(target_os = "windows")]
fn authenticate_windows_hello(reason: &str) -> Result<bool> {
    // Текст запроса берётся из аргумента, как и на macOS: раньше здесь была
    // зашита русская строка, хотя интерфейс лаунчера англоязычный.
    let script = windows_hello_script(reason);

    let output = Command::new("powershell")
        .arg("-Command")
        .arg(&script)
        .output()?;

    Ok(output.status.code() == Some(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_prompt_escapes_quotes_and_keeps_braces() {
        let script = windows_hello_script(r#"Sign in to "Noro""#);

        // Кавычки в тексте удвоены, а не оборвали строку PowerShell.
        assert!(script.contains(r#"RequestVerificationAsync("Sign in to ""Noro""")"#));
        // Блоки скрипта остались блоками: `{{`/`}}` в format! дают одну скобку.
        assert!(script.contains("Run({ $asyncOp.GetResults() })"));
        assert!(script.contains("{ exit 0 } else { exit 1 }"));
        assert!(
            !script.contains("{{"),
            "двойные скобки не должны утечь в скрипт"
        );
    }
}
