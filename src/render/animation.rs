//! Sprite frame animation (ADR-028).
//!
//! `SpriteAnimation` is a component holding which frames of a [`SpriteGrid`]
//! to play and how far playback has got. The game advances every animation
//! with [`advance_animations`] from `fixed_update`, like
//! [`ecs::integrate_velocity`](crate::ecs::integrate_velocity); the renderer
//! draws the current frame. Playback depends only on the summed `dt`, so it
//! is as deterministic as the fixed timestep.

use super::region::{SpriteGrid, TextureRegion};
use crate::ecs::World;

/// Time added to every frame-boundary check, so that summing a step like
/// 1/60 s in `f32` does not land a hair short of a boundary and show a frame
/// one step late.
const BOUNDARY_SLACK: f64 = 1e-6;

/// What happens after the last frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AnimationMode {
    /// Start again from the first frame, forever. The default.
    #[default]
    Loop,
    /// Stop on the last frame; [`SpriteAnimation::is_finished`] becomes `true`.
    Once,
}

/// Plays frames `first..=last` of a [`SpriteGrid`] on the entity's
/// [`Sprite`](super::Sprite) at `fps` frames per second.
///
/// While an entity has a `SpriteAnimation`, its sprite shows the animation's
/// current frame, and [`Sprite::region`](super::Sprite::region) is ignored.
/// `last` may be smaller than `first` to play the frames backwards. Advance
/// all animations with [`advance_animations`] (normally from
/// [`Game::fixed_update`](crate::Game::fixed_update)), or one with
/// [`advance`](Self::advance).
///
/// ```no_run
/// use purplepie::math::{Transform2D, Vec2};
/// use purplepie::render::{Sprite, SpriteAnimation, SpriteGrid};
/// # fn init(ctx: &mut purplepie::Context<'_>) -> purplepie::Result<()> {
/// let sheet = ctx.load_texture("textures/hero.png")?;
/// let grid = SpriteGrid::new(16, 16, 8, 4); // 8 frames per row, 4 rows
/// ctx.world_mut().spawn((
///     Transform2D::default(),
///     Sprite::new(sheet, Vec2::splat(64.0)),
///     SpriteAnimation::new(grid, 8, 15, 12.0), // the second row, 12 frames per second, looping
/// ));
/// # Ok(())
/// # }
/// // In fixed_update: `let dt = ctx.dt(); purplepie::render::advance_animations(ctx.world_mut(), dt);`
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct SpriteAnimation {
    /// The sheet's layout; frame numbers index into it.
    pub grid: SpriteGrid,
    /// Frame shown first.
    pub first: u32,
    /// Frame shown last (inclusive). Smaller than `first`: plays backwards.
    pub last: u32,
    /// Frames per second. Zero, negative or not finite: the animation holds
    /// its current frame.
    pub fps: f32,
    /// Loop forever, or play once and stop on `last`.
    pub mode: AnimationMode,
    /// Frames already played since `first`, `0..len`.
    step: u32,
    /// Seconds spent on the current frame, `0..1/fps`.
    time_in_frame: f64,
    finished: bool,
}

impl SpriteAnimation {
    /// A looping animation of frames `first..=last` of `grid` at `fps`
    /// frames per second, starting on `first`.
    pub const fn new(grid: SpriteGrid, first: u32, last: u32, fps: f32) -> Self {
        Self {
            grid,
            first,
            last,
            fps,
            mode: AnimationMode::Loop,
            step: 0,
            time_in_frame: 0.0,
            finished: false,
        }
    }

    /// The same animation, played once (it stops on the last frame).
    pub const fn once(mut self) -> Self {
        self.mode = AnimationMode::Once;
        self
    }

    /// Number of frames in `first..=last`.
    pub const fn len(&self) -> u32 {
        self.first.abs_diff(self.last).saturating_add(1)
    }

    /// Always `false`: an animation has at least one frame.
    pub const fn is_empty(&self) -> bool {
        false
    }

    /// The grid frame shown now.
    pub fn frame(&self) -> u32 {
        let step = self.step.min(self.len() - 1);
        if self.last >= self.first {
            self.first + step
        } else {
            self.first - step
        }
    }

    /// The texture region shown now, or `None` if the frame is outside the grid.
    pub fn region(&self) -> Option<TextureRegion> {
        self.grid.frame(self.frame())
    }

    /// `true` once a [`Once`](AnimationMode::Once) animation has reached its
    /// last frame. Looping animations never finish.
    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    /// Back to the first frame, not finished.
    pub fn restart(&mut self) {
        self.step = 0;
        self.time_in_frame = 0.0;
        self.finished = false;
    }

    /// The playback position: frames played since `first`, seconds spent on
    /// the current frame, and whether a `Once` animation has finished. Scene
    /// files save it (ADR-035).
    pub(crate) fn playback(&self) -> (u32, f64, bool) {
        (self.step, self.time_in_frame, self.finished)
    }

    /// The same animation at a saved playback position, made valid: the step
    /// is kept inside the range, a negative or non-finite time becomes 0, and
    /// only a `Once` animation can be finished (on its last frame).
    pub(crate) fn with_playback(mut self, step: u32, time_in_frame: f64, finished: bool) -> Self {
        self.step = step.min(self.len() - 1);
        self.time_in_frame = if time_in_frame.is_finite() && time_in_frame >= 0.0 {
            time_in_frame
        } else {
            0.0
        };
        self.finished = finished && self.mode == AnimationMode::Once;
        if self.finished {
            self.step = self.len() - 1;
            self.time_in_frame = 0.0;
        }
        self
    }

    /// Moves playback forward by `dt` seconds (several frames if `dt` is
    /// long). Negative or non-finite `dt` is ignored.
    pub fn advance(&mut self, dt: f32) {
        if self.finished || !(dt.is_finite() && dt > 0.0) {
            return;
        }
        if !(self.fps.is_finite() && self.fps > 0.0) {
            return;
        }
        let frame_time = 1.0 / f64::from(self.fps);
        self.time_in_frame += f64::from(dt);
        let frames = ((self.time_in_frame + BOUNDARY_SLACK) / frame_time).floor();
        if frames < 1.0 {
            return;
        }
        self.time_in_frame = (self.time_in_frame - frames * frame_time).max(0.0);
        let len = u64::from(self.len());
        // `frames` is ≥ 1 and finite here; saturate absurdly long steps.
        let frames = if frames >= u64::MAX as f64 {
            u64::MAX
        } else {
            frames as u64
        };
        let reached = u64::from(self.step).saturating_add(frames);
        match self.mode {
            AnimationMode::Loop => self.step = (reached % len) as u32,
            AnimationMode::Once => {
                if reached >= len - 1 {
                    self.step = (len - 1) as u32;
                    self.time_in_frame = 0.0;
                    self.finished = true;
                } else {
                    self.step = reached as u32;
                }
            }
        }
    }
}

/// Advances every [`SpriteAnimation`] in `world` by `dt` seconds. Call it
/// once per fixed step:
///
/// ```
/// use purplepie::{render, Context, Game};
///
/// struct MyGame;
///
/// impl Game for MyGame {
///     fn fixed_update(&mut self, ctx: &mut Context<'_>) {
///         let dt = ctx.dt(); // read first: `world_mut` borrows the context exclusively
///         render::advance_animations(ctx.world_mut(), dt);
///     }
/// }
/// ```
pub fn advance_animations(world: &mut World, dt: f32) {
    for animation in world.query_mut::<&mut SpriteAnimation>() {
        animation.advance(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: f32 = 1.0 / 60.0;

    fn grid() -> SpriteGrid {
        SpriteGrid::new(8, 8, 4, 2)
    }

    /// The frame after each of `steps` fixed steps.
    fn frames(mut animation: SpriteAnimation, steps: usize) -> Vec<u32> {
        (0..steps)
            .map(|_| {
                animation.advance(STEP);
                animation.frame()
            })
            .collect()
    }

    #[test]
    fn frames_change_exactly_on_time_at_the_fixed_step() {
        // 10 fps at 60 Hz: 6 steps per frame, with no drift over 8 loops.
        let seen = frames(SpriteAnimation::new(grid(), 0, 7, 10.0), 6 * 8 * 8);
        for (i, frame) in seen.iter().enumerate() {
            let step = i as u32 + 1;
            assert_eq!(*frame, (step / 6) % 8, "after {step} steps");
        }
    }

    #[test]
    fn frame_boundaries_hold_for_common_step_rates() {
        // (fixed steps per second, frames per second) with a whole number of steps per frame.
        for (rate, fps) in [
            (60, 10),
            (60, 12),
            (60, 15),
            (60, 30),
            (120, 24),
            (144, 24),
            (50, 25),
            (30, 6),
        ] {
            let dt = 1.0 / rate as f32;
            let per_frame = rate / fps;
            let mut animation = SpriteAnimation::new(grid(), 0, 7, fps as f32);
            for step in 1..=(per_frame * 8 * 20) {
                animation.advance(dt);
                assert_eq!(
                    animation.frame(),
                    (step / per_frame) % 8,
                    "{rate} Hz, {fps} fps, step {step}"
                );
            }
        }
    }

    #[test]
    fn a_subrange_loops_and_maps_to_grid_regions() {
        let mut animation = SpriteAnimation::new(grid(), 4, 6, 60.0);
        assert_eq!(animation.len(), 3);
        assert_eq!(animation.region(), grid().frame(4));
        let seen: Vec<u32> = (0..7)
            .map(|_| {
                animation.advance(STEP);
                animation.frame()
            })
            .collect();
        assert_eq!(seen, [5, 6, 4, 5, 6, 4, 5]);
        assert_eq!(animation.region(), Some(TextureRegion::new(8, 8, 8, 8)));
        assert!(!animation.is_finished(), "loops never finish");
    }

    #[test]
    fn reversed_ranges_play_backwards() {
        let seen = frames(SpriteAnimation::new(grid(), 3, 0, 60.0), 5);
        assert_eq!(seen, [2, 1, 0, 3, 2]);
    }

    #[test]
    fn once_stops_on_the_last_frame_and_restart_rewinds() {
        let mut animation = SpriteAnimation::new(grid(), 0, 2, 30.0).once();
        let seen: Vec<(u32, bool)> = (0..8)
            .map(|_| {
                animation.advance(STEP);
                (animation.frame(), animation.is_finished())
            })
            .collect();
        assert_eq!(
            seen,
            [
                (0, false),
                (1, false),
                (1, false),
                (2, true),
                (2, true),
                (2, true),
                (2, true),
                (2, true)
            ]
        );
        animation.restart();
        assert_eq!((animation.frame(), animation.is_finished()), (0, false));
    }

    #[test]
    fn one_long_step_skips_frames_like_many_short_ones() {
        let mut long = SpriteAnimation::new(grid(), 0, 7, 12.0);
        long.advance(STEP * 25.0);
        let short = *frames(SpriteAnimation::new(grid(), 0, 7, 12.0), 25)
            .last()
            .expect("25 steps");
        assert_eq!(long.frame(), short);
        assert_eq!(long.frame(), 5, "25/60 s at 12 fps = 5 frames");
        // Hours in one step: still a valid frame, no overflow.
        long.advance(3600.0 * 10.0);
        assert!(long.frame() < 8);
        let mut once = SpriteAnimation::new(grid(), 0, 7, 12.0).once();
        once.advance(f32::MAX);
        assert!(once.is_finished());
        assert_eq!(once.frame(), 7);
    }

    #[test]
    fn invalid_fps_or_dt_hold_the_current_frame() {
        for fps in [0.0, -5.0, f32::NAN, f32::INFINITY] {
            let mut animation = SpriteAnimation::new(grid(), 0, 7, fps);
            animation.advance(1.0);
            assert_eq!(animation.frame(), 0, "fps {fps}");
        }
        let mut animation = SpriteAnimation::new(grid(), 0, 7, 10.0);
        for dt in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            animation.advance(dt);
        }
        assert_eq!(animation.frame(), 0);
    }

    #[test]
    fn a_single_frame_animation_never_moves() {
        let mut animation = SpriteAnimation::new(grid(), 2, 2, 60.0);
        animation.advance(1.0);
        assert_eq!((animation.len(), animation.frame()), (1, 2));
        let mut once = SpriteAnimation::new(grid(), 2, 2, 60.0).once();
        once.advance(STEP);
        assert!(once.is_finished());
    }

    #[test]
    fn saved_playback_positions_are_restored_and_made_valid() {
        let mut playing = SpriteAnimation::new(grid(), 0, 7, 10.0);
        playing.advance(0.35);
        let (step, time, finished) = playing.playback();
        assert_eq!((step, finished), (3, false));
        let restored = SpriteAnimation::new(grid(), 0, 7, 10.0).with_playback(step, time, finished);
        assert_eq!(restored, playing);
        // Out-of-range or nonsense values become valid ones.
        let fixed = SpriteAnimation::new(grid(), 2, 4, 10.0).with_playback(99, f64::NAN, true);
        assert_eq!(
            fixed.playback(),
            (2, 0.0, false),
            "a loop is never finished"
        );
        assert_eq!(fixed.frame(), 4);
        let done = SpriteAnimation::new(grid(), 0, 3, 10.0)
            .once()
            .with_playback(1, 0.05, true);
        assert_eq!((done.frame(), done.is_finished()), (3, true));
        assert_eq!(
            SpriteAnimation::new(grid(), 0, 3, 10.0)
                .with_playback(0, -1.0, false)
                .playback(),
            (0, 0.0, false)
        );
    }

    #[test]
    fn advance_animations_steps_every_animation_in_the_world() {
        let mut world = World::new();
        let a = world.spawn((SpriteAnimation::new(grid(), 0, 7, 60.0),));
        let b = world.spawn((SpriteAnimation::new(grid(), 7, 0, 30.0),));
        for _ in 0..2 {
            advance_animations(&mut world, STEP);
        }
        let frame = |e| world.get::<&SpriteAnimation>(e).expect("animation").frame();
        assert_eq!((frame(a), frame(b)), (2, 6));
    }
}
