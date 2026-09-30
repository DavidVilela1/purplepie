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
2. Inspect the repository: `Cargo.toml`, `src/`, tests, `docs/`. Do not assume docs are current.
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

Windowed stages in Cowork (Linux, no display) use a virtual display and software Vulkan:

```bash
sudo apt-get install -y xvfb mesa-vulkan-drivers imagemagick   # once per container, if missing
xvfb-run -a -s "-screen 0 1024x768x24" cargo run
```

Automated smoke runs need a timed-exit hook, which is added in PP-003.

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

## 11. Session end report

Summarize: what changed, validation actually executed, docs updated, archive
status, the single next task, and anything unverified. Then stop.
