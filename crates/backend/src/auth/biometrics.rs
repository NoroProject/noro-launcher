//! Native biometric authentication (Touch ID on macOS, Windows Hello on Windows).

use anyhow::{anyhow, Result};

// Only macOS and Windows shell out; on Linux this import is unused and `-D
// warnings` turns that into a build failure.
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

/// Built outside `cfg(windows)` on purpose. The crate can't be cross-compiled
/// for Windows from macOS or Linux, so the one part that's easy to get wrong —
/// the quoting — would never be exercised. Compiled under `test` everywhere, it
/// is.
#[cfg(any(target_os = "windows", test))]
fn windows_hello_script(reason: &str) -> String {
    // PowerShell escapes a quote inside a string by doubling it.
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

        // Quotes in the reason are doubled, not left to terminate the string.
        assert!(script.contains(r#"RequestVerificationAsync("Sign in to ""Noro""")"#));
        // `{{` and `}}` in the format string have to come out as single braces,
        // or PowerShell gets a script with no blocks in it.
        assert!(script.contains("Run({ $asyncOp.GetResults() })"));
        assert!(script.contains("{ exit 0 } else { exit 1 }"));
        assert!(
            !script.contains("{{"),
            "doubled braces should not reach the script"
        );
    }
}
