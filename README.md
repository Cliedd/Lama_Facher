# Forge

Learn Rust or Java in your terminal. Forge offers small exercises, compiler feedback, a colorful terminal editor, and local progress tracking.

## Installation

On Linux or macOS, run this in a terminal:

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/Cliedd/Lama_Facher/main/install.sh | bash
```

On Windows, run this in PowerShell:

```powershell
irm https://raw.githubusercontent.com/Cliedd/Lama_Facher/main/install.ps1 | iex
```

The installer downloads the latest GitHub release and places Forge and its exercises in your user account. It does not require administrator access. Prebuilt archives cover Linux x86-64, macOS Apple Silicon, and Windows x86-64. Other architectures, or a repository without a published release, use the source build: install [Rust/Cargo](https://rustup.rs/) and Git first. Linux/macOS also need `curl` and `tar`; a source build needs a C compiler. Windows source builds need a compatible C++ build environment (for example, Visual Studio Build Tools). The GitHub URL must be reachable during installation.

Open a new terminal, then run:

```sh
forge doctor
forge start
```

On Linux/macOS, the launcher is `~/.local/bin/forge`. If `forge` is not found, add it to your shell's `PATH`:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

Add that line to `~/.bashrc` or `~/.zshrc` to keep it for future sessions. Windows installs `forge.cmd` under `%LOCALAPPDATA%\Programs\Forge` and adds that directory to the user `PATH`; start a new PowerShell session to pick up the change.

Forge itself can open without language compilers. To compile Rust exercises, install the [Rust toolchain](https://rustup.rs/). To compile Java exercises, install a JDK with `javac` on `PATH` (for example, [Eclipse Temurin](https://adoptium.net/)). `forge doctor` displays the detected toolchains and exercise count. The editor needs an interactive terminal.

To inspect the installer before running it, download and read [`install.sh`](install.sh) or [`install.ps1`](install.ps1). You can also install from a clone: `git clone https://github.com/Cliedd/Lama_Facher.git`, `cd Lama_Facher`, then `bash install.sh` (or `.\install.ps1` in PowerShell). Re-running the installer updates the launcher and keeps previous releases available.

## Learning with Forge

| Command | Purpose |
| --- | --- |
| `forge start` | Open the guided terminal experience |
| `forge tui` | Open the exercise editor |
| `forge lesson java` / `forge lesson rust` | Show a language path |
| `forge list --language java` | List exercises for a language (`rust` also works) |
| `forge progress` | Show local completion counts |
| `forge doctor` | Check installed toolchains and exercise data |
| `forge run <id>` | Run the starter template for one exercise |
| `forge test` | Run all exercise templates |
| `forge --help` | Show all commands |

Progress and drafts are stored under `~/.config/forge` on Unix systems. The current lessons and generated exercise templates are still being refined; some templates contain complete sample solutions. `forge run` executes the exercise template, while editing happens in the TUI.

## Update and uninstall

Run the same install command again to update. To select a release, set `FORGE_VERSION` to a tag such as `v0.2.0` before running the installer. On Linux/macOS, `FORGE_INSTALL_DIR` and `FORGE_BIN_DIR` can override the default paths; Windows supports the same environment variables.

On Linux/macOS, the default installation can be removed with:

```sh
rm "$HOME/.local/bin/forge"
rm -r "$HOME/.local/share/forge"
```

On Windows PowerShell:

```powershell
Remove-Item "$env:LOCALAPPDATA\Programs\Forge\forge.cmd"
Remove-Item "$env:LOCALAPPDATA\Forge" -Recurse
```

You can also remove `%LOCALAPPDATA%\Programs\Forge` from your Windows user `PATH`. Uninstalling leaves local progress intact. Delete `~/.config/forge` separately on Unix if you also want to erase progress.

## Development

Install stable Rust and, for Java exercises, JDK 21. From a clone:

```sh
cargo build --locked
cargo test --locked
cargo run -- start
```

`cargo run -- doctor` checks the local environment. Run from the repository root so Forge finds `exercises/`, or set `FORGE_HOME` to a directory containing `exercises/`. CI runs builds and Rust tests on Linux, macOS, and Windows. A `v*` tag produces platform archives in GitHub Releases; installation from a release becomes available after that workflow succeeds.
