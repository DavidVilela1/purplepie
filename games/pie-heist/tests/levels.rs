//! Every level shipped in `assets/levels/` loads strictly, exactly as the
//! game loads it, and is playable as far as the rules can tell
//! (`pie_heist::check`). Runs without a window (`purplepie::Headless`,
//! ADR-045), so it is part of `cargo test` and CI.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use pie_heist::{Exit, FIRST_LEVEL, check, next_level};
use purplepie::Headless;

/// `levels/<name>.ron` for every `.ron` file in `assets/levels/`, sorted.
fn shipped_levels(assets: &Path) -> Vec<String> {
    let mut levels: Vec<String> = std::fs::read_dir(assets.join("levels"))
        .expect("assets/levels/")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name.ends_with(".ron"))
        .map(|name| format!("levels/{name}"))
        .collect();
    levels.sort();
    levels
}

#[test]
fn every_shipped_level_loads_and_is_playable() {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let levels = shipped_levels(&assets);
    assert!(levels.contains(&FIRST_LEVEL.to_owned()), "no {FIRST_LEVEL}");

    let mut engine = Headless::new(&assets);
    pie_heist::register(&mut engine.context()).expect("register");
    let mut problems = Vec::new();
    let mut next = BTreeMap::new();
    for level in &levels {
        let mut ctx = engine.context();
        ctx.world_mut().clear();
        if let Err(error) = ctx.load_scene(level) {
            problems.push(format!("{level} does not load: {error:?}"));
            continue;
        }
        let world = ctx.world();
        problems.extend(
            check::problems(world)
                .into_iter()
                .map(|problem| format!("{level}: {problem}")),
        );
        let exits: Vec<Exit> = world.query::<&Exit>().iter().cloned().collect();
        if let [exit] = exits.as_slice() {
            next.insert(level.clone(), next_level(exit).map(str::to_owned));
        }
    }
    if problems.is_empty() {
        match check::order(FIRST_LEVEL, &next) {
            Ok(order) => println!("levels in play order: {}", order.join(" → ")),
            Err(problem) => problems.push(problem),
        }
    }
    assert!(
        problems.is_empty(),
        "shipped levels with problems:\n{}",
        problems.join("\n")
    );
}
