use crate::core::errors::{ForgeError, Result};
use std::process::Command;

pub struct SdkmanInstaller;

impl SdkmanInstaller {
    pub fn is_installed() -> bool {
        Command::new("javac")
            .arg("-version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn check_status() -> String {
        if Self::is_installed() {
            if let Ok(out) = Command::new("javac").arg("-version").output() {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                if !stdout.is_empty() {
                    return stdout;
                }
                return stderr;
            }
            "Installed".into()
        } else {
            "Not installed".into()
        }
    }

    pub fn install() -> Result<()> {
        let status = Command::new("sh")
            .arg("-c")
            .arg("curl -s \"https://get.sdkman.io\" | bash")
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(ForgeError::Toolchain(
                "Failed to run SDKMAN installer".into(),
            ))
        }
    }
}
