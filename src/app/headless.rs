//! The engine's state without a window, for tests and tools (ADR-045).

use std::path::PathBuf;

use super::config::EngineConfig;
use super::game::Context;
use super::state::EngineState;
use crate::assets::AssetRoot;

/// The engine's state without a window, a GPU or an audio device: lends the
/// same [`Context`] games get in their callbacks, for tests and tools that
/// load and check game data (ADR-045).
///
/// Everything that does not need a window works as in a running game: the
/// world, [registering](Context::register_scene_component) and
/// [loading scenes](Context::load_scene) (strictly, with every texture and
/// font they name read and decoded), loading textures, fonts and sounds, the
/// camera. Nothing is drawn or heard: sounds play silently, no keys or mouse
/// buttons are ever pressed, the viewport is zero by zero, and time does not
/// advance (the context's [`dt`](Context::dt) is the default fixed step, 1/60 s).
///
/// A game's test that every shipped level loads, with the game's own
/// component registration:
///
/// ```
/// use purplepie::Headless;
/// # use serde::{Deserialize, Serialize};
/// # #[derive(Serialize, Deserialize)] struct Spin { speed: f32, direction: Turn }
/// # #[derive(Serialize, Deserialize)] enum Turn { Clockwise, CounterClockwise }
/// # #[derive(Serialize, Deserialize)] struct Visits { label: String, count: u32 }
/// # fn register(ctx: &mut purplepie::Context<'_>) -> purplepie::Result<()> {
/// #     ctx.register_scene_component::<Spin>("scene_demo::Spin")?;
/// #     ctx.register_scene_component::<Visits>("scene_demo::Visits")
/// # }
/// let mut engine = Headless::new(concat!(env!("CARGO_MANIFEST_DIR"), "/assets"));
/// let mut ctx = engine.context();
/// register(&mut ctx)?; // the game's own `register_scene_component` calls
/// let entities = ctx.load_scene("scenes/demo.ron")?;
/// assert!(!entities.is_empty());
/// # Ok::<(), purplepie::Error>(())
/// ```
pub struct Headless {
    state: EngineState,
}

impl Headless {
    /// An empty engine state whose relative asset paths are resolved against
    /// `asset_root` (a relative `asset_root` is taken relative to the working
    /// directory now, as [`EngineConfig::with_asset_root`] does at startup).
    pub fn new(asset_root: impl Into<PathBuf>) -> Self {
        let root: PathBuf = asset_root.into();
        let cwd = std::env::current_dir().ok();
        let assets = AssetRoot::resolve(Some(&root), None, cwd.as_deref());
        Self {
            state: EngineState::new(EngineConfig::DEFAULT_FIXED_DT, assets),
        }
    }

    /// The context for this state. Everything done through it (entities,
    /// loaded assets, registered components) stays in this state for the next
    /// call.
    pub fn context(&mut self) -> Context<'_> {
        Context::new(&mut self.state, EngineConfig::DEFAULT_FIXED_DT)
    }
}

impl std::fmt::Debug for Headless {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Headless")
            .field("entities", &self.state.world.len())
            .field("asset_root", &self.state.assets.path())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Transform2D;
    use crate::render::{Quad, Sprite};
    use serde::{Deserialize, Serialize};

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Spin {
        speed: f32,
        direction: Turn,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Turn {
        Clockwise,
        CounterClockwise,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Visits {
        label: String,
        count: u32,
    }

    #[test]
    fn scenes_load_strictly_and_keep_their_state_between_contexts() {
        let mut engine = Headless::new(root());
        // Strict: the demo scene's game components are not registered yet.
        let error = engine
            .context()
            .load_scene("scenes/demo.ron")
            .expect_err("unregistered components");
        assert!(format!("{error:?}").contains("scene_demo::"), "{error:?}");
        assert_eq!(engine.context().world().len(), 0, "nothing spawned");

        {
            let mut ctx = engine.context();
            ctx.register_scene_component::<Spin>("scene_demo::Spin")
                .expect("register");
            ctx.register_scene_component::<Visits>("scene_demo::Visits")
                .expect("register");
        }
        let entities = engine
            .context()
            .load_scene("scenes/demo.ron")
            .expect("loads once registered");
        assert!(!entities.is_empty());
        let mut ctx = engine.context();
        assert_eq!(ctx.world().len() as usize, entities.len());
        let world = ctx.world_mut();
        assert!(world.query_mut::<&Spin>().into_iter().count() > 0);
        assert!(
            world
                .query_mut::<(&Transform2D, &Quad)>()
                .into_iter()
                .count()
                > 0
        );
        assert!(
            world.query_mut::<&Sprite>().into_iter().count() > 0,
            "textures decoded"
        );
    }

    #[test]
    fn nothing_is_pressed_time_stands_still_and_missing_assets_are_errors() {
        let mut engine = Headless::new(root());
        let ctx = engine.context();
        assert_eq!(ctx.dt(), (1.0 / 60.0) as f32);
        assert_eq!(ctx.time().elapsed(), 0.0);
        assert_eq!(ctx.viewport_size(), crate::math::Vec2::ZERO);
        assert!(ctx.input().pressed_keys().next().is_none());
        let mut ctx = engine.context();
        assert!(ctx.load_texture("textures/missing.png").is_err());
        assert!(ctx.load_scene("scenes/missing.ron").is_err());
        let sound = ctx.load_sound("sounds/blip.wav").expect("decoded");
        ctx.play_sound(sound, 1.0); // silent, not an error
        assert!(format!("{engine:?}").contains("Headless"));
    }

    #[test]
    fn a_relative_root_is_taken_from_the_working_directory() {
        let engine = Headless::new("assets");
        let cwd = std::env::current_dir().expect("cwd");
        assert_eq!(
            engine.state.assets.path(),
            Some(cwd.join("assets").as_path())
        );
    }
}
