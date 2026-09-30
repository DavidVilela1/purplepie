# PurplePie Development Protocol

How Claude Cowork (and any human contributor) works on PurplePie. This file is
persistent. Follow it in every session.

## 1. Source of truth

```text
Actual validated repository state      (code + commands actually executed)
        ↓ outranks
Project documentation                  (ARCHITECTURE, DECISIONS, PROJECT_STATUS, TASKS, RISKS)
        ↓ outranks
Roadmap                                (ROADMAP: intent, not fact)
        ↓ outranks
Conversation history
```

Conversation history never overrides evidence in the repository. If a
document contradicts the code, the code plus validation is the baseline, and the
document is corrected.

## 2. Session start checklist

1. Read [PROJECT_STATUS.md](PROJECT_STATUS.md), then [TASKS.md](TASKS.md), then the relevant parts of [DECISIONS.md](DECISIONS.md).
2. Get the current project: Claude works only in its own session workspace (§3,
   owner rule). If the workspace has no `PurplePie/` copy, or the owner may have
   changed files since the last delivered ZIP, ask the owner to attach a ZIP of the
   repo (without `target/`). Then inspect `Cargo.toml`, `src/`, tests and `docs/`.
   Do not assume docs are current.
3. Run the validation suite (§8) **before** changing anything, so pre-existing failures are known.
4. Confirm which single task the session is for. If none was requested, propose the task marked "next" in TASKS.md and wait.

## 3. Working rules

- **One verified increment at a time.** Work on one task, and a stage never starts automatically.
- **Inspect before modifying.** Read files before editing them.
- **Preserve working architecture.** Change it only deliberately (§7).
- **Implement only the requested scope.** No silent work on future stages.
- **Keep the sandbox runnable** after every task.
- **No new dependency** without ADR-013's checks (current version, MSRV, API verified against source) and a TECH_STACK entry.
- **Never claim a command passed unless it was executed in this session.** If
  something could not be executed, say: *"This code has been reviewed for
  consistency but has not been executed in this environment,"* and list what remains unverified.
- **Label owner-reported results** as owner-reported, never as executed.
- **Owner rule: never touch the owner's computer.** Claude does not read, write,
  stage or commit files in the owner's repo or any folder on the owner's computer,
  even when a folder is connected. All work happens in Claude's own workspace.
- **Owner rule: every delivery is a ZIP + an expand command, always.** At the end
  of every task, send the complete project as a ZIP (§10) together with the exact
  PowerShell `Expand-Archive` command that updates the owner's repo in place. The
  owner extracts it and commits it themselves.
- **Owner rule: deliver to the chat only.** Never save outputs on the owner's
  computer unless the owner specifically asks for it. While a folder from the
  owner's computer is connected to the session, the app copies every file sent
  in chat into `Claude outputs/` inside that folder. So before sending a file,
  confirm that no folder is connected. If one is, ask the owner to remove it
  from the session first.
- **Commit messages (owner rule):** never add AI attribution lines such as
  `Co-Authored-By: Claude …` or `Claude-Session: …`. Use conventional style
  (`feat:`, `fix:`, `docs:` …): a subject line, then a body listing what changed and why.
- **Stop after the requested task** and report.

## 4. Definition of Done

A task is `DONE` only when all of these hold:

- [ ] The implementation exists and matches the task's acceptance criteria.
- [ ] It builds: `cargo check --all-targets` and `cargo build` pass.
- [ ] Style and lints: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass.
- [ ] Relevant tests exist for new logic and pass (`cargo test`).
- [ ] Behavior is checked where applicable: smoke run (Xvfb in Cowork) and owner run on Windows for windowed/GPU stages.
- [ ] Architecture stays coherent: the dependency rules in ARCHITECTURE §4 hold, and there are no upward imports.
- [ ] Docs are updated: ARCHITECTURE (if structure changed), DECISIONS (if a decision was made), TECH_STACK (if dependencies changed), RISKS (if risk changed), README (if usage changed).
- [ ] `Cargo.toml`/configuration is updated if needed. `Cargo.lock` is committed.
- [ ] No obvious regression: the sandbox still runs.
- [ ] Resumable: PROJECT_STATUS and TASKS reflect the new state.

If any item cannot be verified, the task stays `IN_PROGRESS` (or is marked
`DONE` with an explicit **Unverified** note agreed with the owner). The gap is
written in PROJECT_STATUS.

## 5. Tasks

- IDs: `PP-NNN`, sequential, never reused. Fields: ID, title, stage, priority, status, dependencies, acceptance criteria.
- Size: one meaningful increment that can be completed and validated in one session. Future stages keep one coarse task each. Split a task when its stage becomes current.
- Status transitions: `TODO → IN_PROGRESS → DONE`, or `→ BLOCKED` (record the blocker and what unblocks it) or `→ CANCELLED` (record why).
- Exactly one task is marked **next** in TASKS.md.

## 6. Architecture decisions

**Write an ADR** (in [DECISIONS.md](DECISIONS.md), next free number) when a
decision materially affects architecture, dependencies, the public API,
ownership, the ECS, the rendering architecture, threading, resource or asset
management, the engine/game split, module boundaries, persistence, or a
significant performance trade-off.

**Do not write an ADR** for naming, trivial helpers, formatting, ordinary bug
fixes, or implementation details with no architectural consequence.

Questions that are not yet decided go in **Pending Decisions** (PD-xx) with the
task that will decide them. They are not ADRs. When decided, a pending decision
becomes an ADR and its PD row is removed. Superseding an ADR means a new ADR
plus a `Superseded by ADR-xxx` status on the old one. Never delete an ADR.

## 7. Architectural drift

If the implementation diverges from ARCHITECTURE or ROADMAP:

1. Do not hide the divergence. State it in the session report.
2. Decide whether it is intentional.
3. If it is beneficial and significant, update ARCHITECTURE to match.
4. Record an ADR when §6 applies.
5. Update ROADMAP if sequencing or scope changed.
6. Update PROJECT_STATUS (Recent Changes) and TASKS.

A bad plan is not preserved merely because it was written first. An accidental
divergence is fixed in the code instead.

## 8. Validation commands

Run from the project root:

```bash
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
cargo run            # sandbox
```

Windowed stages in Cowork (Linux, no display) use a virtual X server:

```bash
sudo apt-get install -y xvfb mesa-vulkan-drivers xdotool x11-utils imagemagick   # once per container if missing
pip install --break-system-packages python-xlib                     # to simulate the close button

# timed run (sandbox requests exit after N frames); expect exit 0 and ~N/60 s
PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES=120 xvfb-run -a cargo run
```

Interactive checks inside `xvfb-run` (there is no window manager):
- **Find the window:** `xdotool search --name "PurplePie Sandbox"`. Check attributes with `xwininfo -id <id>`.
- **Keys:** `xdotool windowfocus --sync <id>; xdotool key Escape`. XTEST input is
  required, because `xdotool key --window` sends synthetic events that winit (XInput2) ignores.
- **Close button:** send a `WM_DELETE_WINDOW` ClientMessage with python-xlib, which is what a WM does.
- **Busy-loop check:** read `utime+stime` from `/proc/<pid>/stat` after a few idle seconds.
- **Resize:** `xdotool windowsize <id> 640 360`.
- **Pixels (Stage 4+):** `import -window root shot.png; convert shot.png -format "%c" histogram:info:- | sort -rn | head`.
  The window area must be exactly the clear color, e.g. 921,600 px `#6A0DAD` for 1280×720.
  Needs `imagemagick` and the lavapipe Vulkan driver (`mesa-vulkan-drivers`).
- **No-GPU error path:** run with `VK_ICD_FILENAMES=/nonexistent.json VK_DRIVER_FILES=/nonexistent.json __EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent.json`. Expect `Error::Surface` and exit 1, no panic.

The owner confirms every windowed stage on Windows with `cargo test` and `cargo run`.

## 9. Environment setup

**All platforms:** Rust stable ≥ 1.90 (1.95 in use), plus the `rustfmt` and `clippy` components.
`rustup update stable && rustup component add rustfmt clippy`.

**Windows (owner's machine):**
- The MSVC linker is required. In Visual Studio Installer, add **"Desktop
  development with C++"** (MSVC x64/x86 build tools + a Windows SDK). Without
  it, `check`/`clippy` pass but `test`/`build`/`run` fail with `link.exe not found` (R-14).
- Keep the project, or at least `target/`, **outside OneDrive** (R-15). For example,
  move it to `C:\Users\<you>\dev\PurplePie`, or run `setx CARGO_TARGET_DIR C:\Users\<you>\cargo-target`.

**Linux:** development packages for X11/Wayland are loaded at runtime. A Vulkan
driver or Mesa is needed to run from Stage 4.

## 10. Stage archives

Every completed stage ends with `PurplePie-stage-N.zip`. A documentation
update inside a stage re-issues that stage's archive.

```bash
cd <parent of PurplePie>
zip -r -X PurplePie-stage-N.zip PurplePie \
    -x 'PurplePie/target/*' '*.DS_Store' '*/Thumbs.db' '*.swp' '*/.idea/*' '*/.vscode/*'
unzip -tq PurplePie-stage-N.zip                      # integrity
unzip -Z1 PurplePie-stage-N.zip | grep -c target/    # must print 0
```

Before delivery, verify:
- the ZIP has a single `PurplePie/` root;
- its contents are identical to the workspace (`diff -r`, excluding `target/`);
- the extracted copy builds and tests.

Deliver the ZIP as a downloadable file, and report Created/Verified/Downloadable/Location factually.

**Delivery (owner rule, always):** send the ZIP as a downloadable file, and
give the exact commands to apply it to the owner's repo:

```powershell
# ALWAYS first (owner rule): delete the copy the app may drop into the repo
Remove-Item -Recurse -Force "C:\Users\35193\OneDrive\Ambiente de Trabalho\Programing\3-major-software-projects\PurplePie\PurplePie-stage-0\PurplePie\Claude outputs" -ErrorAction SilentlyContinue

# ZIP root is PurplePie/, so the destination is the folder that CONTAINS the repo
Expand-Archive -Path "$HOME\Downloads\<zip name>" -DestinationPath "C:\Users\35193\OneDrive\Ambiente de Trabalho\Programing\3-major-software-projects\PurplePie\PurplePie-stage-0" -Force
```

Extracting never deletes files. If the task removed or renamed files, also give
the exact `Remove-Item` commands. A ZIP never contains `.git/` or `target/`, so
extracting leaves the owner's Git history and build cache untouched.

## 11. Session end report

Summarize: what changed, validation actually executed, docs updated, archive
status, the single next task, and anything unverified. Then stop.
