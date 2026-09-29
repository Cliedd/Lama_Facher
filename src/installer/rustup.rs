use crate::core::errors::{ForgeError, Result};
use std::process::Command;

pub struct RustupInstaller;

impl RustupInstaller {
    pub fn is_installed() -> bool {
        Command::new("rustc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn check_status() -> String {
        if Self::is_installed() {
            if let Ok(out) = Command::new("rustc").arg("--version").output() {
                return String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
            "Installed".into()
        } else {
            "Not installed".into()
        }
    }

    pub fn install() -> Result<()> {
        let status = Command::new("sh")
            .arg("-c")
            .arg("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y")
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(ForgeError::Toolchain(
                "Failed to run rustup installer".into(),
            ))
        }
    }
}
