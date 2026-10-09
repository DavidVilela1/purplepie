# Releasing PurplePie

What a PurplePie version promises, and how one is made. Written in PP-035a; first release 0.1.0 (PP-035b,
2026-10-09).

## 1. How PurplePie is distributed

- **By git tag, not crates.io.** A release is a tag `vX.Y.Z` on the repository. Games depend on it with
  `purplepie = { git = "https://github.com/DavidVilela1/purplepie", tag = "v0.1.0" }`, or by path to a checkout. `publish = false` stays in
  `Cargo.toml`.
- **Why not crates.io yet:**
  - the name has not been reserved;
  - the package would ship the engine's `assets/` and `docs/`, which need an `include` list;
  - the engine's API has had only one outside user (the PP-031 trial).

  Revisit after 0.1.0 has been used by the second outside game (PP-036).
- The `Cargo.lock` in the repository is the tested set of dependency versions. A game resolves its own lock file;
  the "Latest dependencies" CI workflow tests those newer patch versions (section 4).

## 2. Compatibility policy (0.x)

PurplePie follows [semantic versioning](https://semver.org/) with the usual 0.x rule:

- **0.x.0 (minor)** may break compatibility. Every break is listed in `CHANGELOG.md` with what to change.
- **0.x.y (patch)** never breaks code or files that worked with 0.x.0: only fixes and additions.

What a patch release keeps working:

| Area | Promise within 0.x.y |
|---|---|
| **Public API** | Everything reachable from `purplepie::` that is documented: names, signatures, behaviour described in the docs. Types marked `#[non_exhaustive]` (ADR-040) may gain fields or variants, so build them with their constructors and builders and keep a wildcard arm when matching their enums (`Error`, `KeyCode`, `MouseButton`, `AnimationMode`, `TextureFilter`). |
| **Re-exported dependencies** | `ecs::hecs` / `ecs::World` / `ecs::Entity` (hecs 0.11), `math::Vec2` / `math::Mat4` (glam 0.33), and the `serde` 1 bounds of `register_scene_component` are part of the API. Moving them to an incompatible version needs a minor release. |
| **Scene files** | Files of format version 1 keep loading. A patch release never writes a file that the same 0.x.0 cannot read. |
| **Assets** | PNG textures, TrueType/OpenType fonts, WAV and OGG Vorbis sounds, and the asset-folder lookup rules (ADR-025). |
| **Minimum Rust version** | `rust-version` in `Cargo.toml` (1.90 for 0.1). Raising it needs a minor release. CI builds with it (`msrv` job). |
| **Platforms** | Windows, Linux (X11; Wayland untested) and macOS are built and tested in CI on every push. Rendering is verified by hand: on Linux by Cowork (software GPU), on Windows by the owner's checklist (`docs/CHECKLIST.md`). macOS rendering is not yet verified. |

Not covered by the promise:

- crate-private items;
- the sandbox, the examples and their environment variables (`PURPLEPIE_SANDBOX_*`, `PURPLEPIE_BREAKOUT_AUTOPLAY`,
  `PURPLEPIE_SCENE_*`);
- the exact text of error and log messages;
- exact pixels, beyond what the docs describe.

## 3. Release procedure

Done by Claude up to the archive, and by the owner from the commit on.

1. **Preconditions:**
   - CI is green, including `msrv` and "Latest dependencies";
   - the owner's latest checklist run (`docs/CHECKLIST.md`) shows no unexplained failure;
   - no open task is marked as blocking the release.
2. **Version:** set `version` in `Cargo.toml` and refresh `Cargo.lock` (`cargo check`).
3. **Changelog:** rename `[Unreleased]` to `[X.Y.Z] - YYYY-MM-DD` and open a new empty `[Unreleased]`.
4. **Docs:** update the dependency lines in the README and the guide (git URL and tag), PROJECT_STATUS and TASKS.
5. **Validation:** the full DEVELOPMENT §8 set, the outside-crate check (a fresh crate depending on the new version),
   and the archive checks.
6. **Owner:** commit, push, wait for green CI, then tag and push the tag:

   ```powershell
   git tag -a vX.Y.Z -m "PurplePie X.Y.Z"
   git push origin vX.Y.Z
   ```

## 4. CI jobs that guard the promise

- **`ci.yml`:**
  - `lint`: fmt and clippy with the latest stable toolchain;
  - `test`: check and test on Windows, Linux and macOS with the committed lock file;
  - `msrv`: `cargo +1.90 check` with the committed lock file.
- **`latest-deps.yml`:** runs `cargo update`, then check and test on Linux. It runs on every push, weekly and on
  demand. If it is red while CI is green, an upstream patch release broke something. Update `Cargo.lock` (and fix the
  code if needed) in a normal task.

Before delivering, Cowork can approximate the `msrv` job: clippy's `incompatible_msrv` lint reads `rust-version`
and reports standard-library items newer than it. Cowork cannot download other toolchains (R-19).
