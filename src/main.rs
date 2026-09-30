//! PurplePie sandbox.
//!
//! This binary plays the role of a *game*: it may only use the public
//! `purplepie` API, exactly like an external game crate would. That keeps the
//! engine/game boundary enforced by the compiler from day one.
//!
//! Stage 0: there is no window yet. Stage 1 replaces this with
//! `Engine::new(config)?.run(Sandbox::default())`.

fn main() {
    println!(
        "PurplePie sandbox v{} (Stage 0 scaffold: no window yet, see docs/ROADMAP.md)",
        purplepie::VERSION
    );
}
