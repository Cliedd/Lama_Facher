use std::env;

pub struct PathManager;

impl PathManager {
    pub fn ensure_toolchain_paths() {
        let Some(home) = dirs::home_dir() else { return };
        let mut paths: Vec<_> =
            env::split_paths(&env::var_os("PATH").unwrap_or_default()).collect();
        for path in [
            home.join(".cargo/bin"),
            home.join(".sdkman/candidates/java/current/bin"),
        ] {
            if path.is_dir() && !paths.contains(&path) {
                paths.insert(0, path);
            }
        }
        if let Ok(value) = env::join_paths(paths) {
            env::set_var("PATH", value);
        }
    }
}
