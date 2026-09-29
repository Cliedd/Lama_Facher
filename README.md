# Forge

Forge is a terminal course for learning Rust and Java through short exercises. Pick a language, read an objective, edit code in the colorized terminal editor, run it against the expected result, reveal a hint when needed, and return later to your saved draft. Progress stays on your computer.

```text
┌─ FORGE ─ Apprendre en pratiquant ───────────────────────────────┐
│  Choisir un langage                    0/… terminés             │
├─ Bienvenue ─────────────────────────────────────────────────────┤
│  Choisis un langage, ouvre un exercice et écris ton code.       │
├─ Choisis ton parcours ──────────────────────────────────────────┤
│  ▸ JAVA   0/… exercices terminés                                │
│    RUST   0/… exercices terminés                                │
└─────────────────────────────────────────────────────────────────┘
  ↑↓ choisir   Entrée ouvrir   F1 aide   q quitter
```

This is an illustrative terminal preview. The real layout adapts to your terminal width and shows the current exercise counts. The editor has separate panels for the objective and hints, code, compiler diagnostics, and program output.

## Install and start

Linux or macOS:

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/Cliedd/Lama_Facher/main/install.sh | bash
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/Cliedd/Lama_Facher/main/install.ps1 | iex
```

Open a new terminal and run `forge doctor`, then `forge start`. The release installer places Forge and the exercises in your user account without administrator access, Git or Cargo. It verifies the archive against the published SHA-256 checksums before installing. Prebuilt releases cover Linux x86-64, macOS Apple Silicon and Intel, and Windows x86-64. A published release is required for one-command binary installation; if none exists yet, use the source build under [Development](#development). Network access is required for installation.

On Linux/macOS the launcher is `~/.local/bin/forge`. If the command is not found, add it to your shell `PATH`:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

Add that line to `~/.bashrc` or `~/.zshrc` for future sessions. On Windows, the installer puts `forge.cmd` in `%LOCALAPPDATA%\Programs\Forge` and adds that directory to the user `PATH`; open a new PowerShell session afterward.

Forge can open without compilers, but completing Rust exercises requires [Rust](https://rustup.rs/) and Java exercises require a JDK with `javac` on `PATH`, such as [Eclipse Temurin](https://adoptium.net/). The editor requires an interactive terminal. Read [`install.sh`](install.sh) or [`install.ps1`](install.ps1) before running them if you prefer to inspect installer code.

## Learning path

1. Run `forge start` and choose Rust or Java with ↑/↓ and Enter.
2. Open an exercise. The list shows completed, in-progress and untouched items.
3. Edit the starter code. Press F2 for the next hint, Ctrl+S to save a draft, and Ctrl+R to compile and run. F1 shows all shortcuts.
4. Review the result and compiler diagnostics, then continue to the next exercise. `forge progress` shows the next item for each language.

You can also work in your own editor:

```sh
forge list --language rust
forge show <exercise-id>
forge run <exercise-id> --file solution.rs
forge progress
```

Use a `.java` file for Java exercises. `forge run <id>` without `--file` uses the saved draft if one exists, otherwise the starter code. `forge test` checks that exercise starter code compiles; it does not grade learner solutions.

| Command | Purpose |
| --- | --- |
| `forge start` or `forge tui` | Open the terminal course |
| `forge lesson java` / `forge lesson rust` | Show the next lesson in a language |
| `forge list --language java` | List exercises and completion status |
| `forge show <id>` | Show an objective and hints |
| `forge run <id> --file solution.java` | Compile and check your solution |
| `forge progress` | Show completion counts and next exercises |
| `forge progress export --format json` | Export a lossless backup to standard output |
| `forge progress export --format csv` | Export a spreadsheet-friendly report |
| `forge progress reset --exercise <id>` | Clear one exercise, including its draft |
| `forge progress reset --yes` | Clear all local progress and drafts |
| `forge doctor` | Check compilers, exercise data and progress storage |

To save a backup, redirect the JSON export to a new file before resetting:

```sh
forge progress export --format json > forge-progress-backup.json
forge progress export --format csv > forge-progress.csv
forge progress reset --yes
```

JSON retains exercise IDs, statuses, attempt counts and saved source code. CSV has the columns `exercise_id,status,attempts,last_code`; it quotes commas, quotes and multiline code for spreadsheet import. Exports include saved records for exercises that have since been removed from the course. The export contains your code, so handle it as a personal file. The CLI does not currently include an import command. Progress normally lives at `~/.config/forge/progress.json` on Unix; `FORGE_PROGRESS_DIR` can override the directory. On Windows, `forge doctor` prints the exact path.

## Update and uninstall

Run the install command again to update. Set `FORGE_VERSION` to a published tag such as `v0.1.0` to select a specific release. `FORGE_INSTALL_DIR` and `FORGE_BIN_DIR` override the install paths on both installer scripts.

Linux/macOS default uninstall:

```sh
rm "$HOME/.local/bin/forge"
rm -r "$HOME/.local/share/forge"
```

Windows PowerShell default uninstall:

```powershell
Remove-Item "$env:LOCALAPPDATA\Programs\Forge\forge.cmd"
Remove-Item "$env:LOCALAPPDATA\Forge" -Recurse
```

Uninstalling keeps progress. Use `forge progress reset --yes` before uninstalling if you also want to clear attempts and drafts.

## Development and release

Install stable Rust and JDK 21, clone this repository, then:

```sh
cargo build --locked
cargo test --locked
cargo run -- doctor
cargo run -- start
```

Run from the repository root, or point `FORGE_HOME` at a directory containing `exercises/`. The terminal rendering smoke test uses Ratatui's in-memory backend, so it runs in CI without a real terminal:

```sh
cargo test --locked --test tui_render
```

For a manual visual check, open `forge tui` in a terminal at least 100 columns by 32 rows and again at a smaller size. Check the language picker, exercise list, editor panels, F1 help, F2 hints, Ctrl+S, Ctrl+R compiler feedback, and Esc. Repeat in the terminals you support (for example GNOME Terminal, macOS Terminal, and Windows Terminal); the in-memory test cannot verify font rendering, color fidelity or key handling in each emulator.

The workflow in [`.github/workflows/ci.yml`](.github/workflows/ci.yml) builds and tests on Linux, macOS Apple Silicon and Intel, and Windows. To publish a release, first ensure the version in `Cargo.toml` matches the tag (for example `0.1.0` and `v0.1.0`), verify the CI checks on `main`, and create and push that tag:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The tag workflow packages the platform binaries with exercises, generates `SHA256SUMS`, and attaches the archives to GitHub Releases. See [release instructions](docs/RELEASING.md) for the exact asset names and checksum verification. Check the release workflow and assets before sharing the one-command installers. A tag is a public release action; only push it when the build and course content are ready.
