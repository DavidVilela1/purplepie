//! Keyboard and mouse: which keys are held, which were just pressed, where
//! the cursor is.
//!
//! Read it through [`Context::input`](crate::Context::input) in `fixed_update` or
//! `update`. Keys are named by their position on a US keyboard: `KeyCode::A` …
//! `KeyCode::Z`, `Digit0` … `Digit9`, `ArrowLeft`, `Space`, `Enter`, `ShiftLeft`,
//! `F1` and so on ([`KeyCode::ALL`] lists every one). For the cursor in world
//! coordinates use [`Context::cursor_world`](crate::Context::cursor_world). Guide:
//! section 8 (`docs/GUIDE.md`).
//!
//! ```
//! use purplepie::Context;
//! use purplepie::input::{KeyCode, MouseButton};
//!
//! /// Called from `fixed_update`.
//! fn controls(ctx: &mut Context<'_>) -> (f32, bool, bool) {
//!     let input = ctx.input();
//!     let run = input.axis(KeyCode::ArrowLeft, KeyCode::ArrowRight); // -1, 0 or 1
//!     let jump = input.just_pressed(KeyCode::Space); // once per press
//!     let fire = input.pressed(KeyCode::ControlLeft) || input.mouse_pressed(MouseButton::Left);
//!     (run, jump, fire)
//! }
//! ```
//!
//! # Engine notes
//!
//! [`Input`] answers three questions per [`KeyCode`]: is it held
//! ([`pressed`](Input::pressed)), did it go down ([`just_pressed`](Input::just_pressed)),
//! did it go up ([`just_released`](Input::just_released)). The engine feeds it
//! from window events (the winit translation lives in `app`, so this module is
//! platform-free; ADR-024).
//!
//! **Edges and the fixed timestep.** A frame runs 0, 1 or several
//! `fixed_update`s, then one `update`. Every key edge is reported:
//! - exactly once to `update`: in the first frame after the event;
//! - exactly once to `fixed_update`: in the first fixed step that runs after the
//!   event. If a frame has no fixed step, the edge waits for the next frame
//!   that has one; if it has several, only the first one sees it.
//!
//! So `just_pressed` is safe to use in either callback, and a press is never
//! lost or doubled. Mouse buttons and wheel movement follow the same rules.
//! The cursor position is plain state (the latest position).

use crate::math::Vec2;

/// Declares [`KeyCode`] and its list of all keys from one table.
macro_rules! key_codes {
    ($($(#[$doc:meta])* $name:ident),+ $(,)?) => {
        /// A physical key, named after its position on a US (QWERTY) keyboard.
        ///
        /// Physical keys keep their place across layouts: [`KeyCode::W`] is the
        /// key above `S`, which is `Z` on a French AZERTY keyboard. That is
        /// what movement controls need. Keys not listed here are ignored.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[non_exhaustive]
        #[repr(u8)]
        pub enum KeyCode {
            $(
                #[doc = concat!("The `", stringify!($name), "` key (US-layout position).")]
                $(#[$doc])*
                $name,
            )+
        }

        impl KeyCode {
            /// Every key PurplePie knows, in declaration order.
            pub const ALL: &'static [KeyCode] = &[$(KeyCode::$name,)+];
        }
    };
}

key_codes! {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    Space, Enter, Escape, Tab, Backspace, Insert, Delete, Home, End, PageUp, PageDown,
    ShiftLeft, ShiftRight, ControlLeft, ControlRight, AltLeft, AltRight,
    /// The Windows / Command / Super key.
    SuperLeft,
    /// The Windows / Command / Super key.
    SuperRight,
    /// `-` on a US keyboard.
    Minus,
    /// `=` on a US keyboard.
    Equal,
    BracketLeft, BracketRight, Backslash, Semicolon, Quote,
    /// `` ` `` on a US keyboard.
    Backquote,
    Comma, Period, Slash, CapsLock,
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide, NumpadEnter, NumpadDecimal,
}

// Every key must fit in a 128-bit mask.
const _: () = assert!(KeyCode::ALL.len() <= 128);

/// A mouse button. Other buttons (some mice have more) are ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum MouseButton {
    /// The primary (usually left) button.
    Left,
    /// The secondary (usually right) button.
    Right,
    /// The wheel button.
    Middle,
    /// The "back" side button.
    Back,
    /// The "forward" side button.
    Forward,
}

impl MouseButton {
    /// Every mouse button PurplePie knows.
    pub const ALL: &'static [MouseButton] = &[
        MouseButton::Left,
        MouseButton::Right,
        MouseButton::Middle,
        MouseButton::Back,
        MouseButton::Forward,
    ];
}

/// Logical pixels of a touchpad (pixel-precise) scroll that count as one wheel line.
pub(crate) const PIXELS_PER_SCROLL_LINE: f32 = 20.0;

/// Which callback is reading the input: decides which edge set is visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Fixed,
    Frame,
}

/// Held state plus the two edge sets of ADR-024, for up to 128 buttons
/// (keys or mouse buttons), as bit masks indexed by the enum discriminant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Buttons {
    held: u128,
    /// Edges for `update`: everything since the previous frame.
    frame_pressed: u128,
    frame_released: u128,
    /// Edges for `fixed_update`: kept until a fixed step has seen them.
    fixed_pressed: u128,
    fixed_released: u128,
}

impl Buttons {
    fn held(&self, bit: u8) -> bool {
        self.held & (1 << bit) != 0
    }

    fn just_pressed(&self, bit: u8, phase: Phase) -> bool {
        let set = match phase {
            Phase::Fixed => self.fixed_pressed,
            Phase::Frame => self.frame_pressed,
        };
        set & (1 << bit) != 0
    }

    fn just_released(&self, bit: u8, phase: Phase) -> bool {
        let set = match phase {
            Phase::Fixed => self.fixed_released,
            Phase::Frame => self.frame_released,
        };
        set & (1 << bit) != 0
    }

    /// Ignored if already held (OS key repeat).
    fn down(&mut self, bit: u8) {
        let mask = 1 << bit;
        if self.held & mask != 0 {
            return;
        }
        self.held |= mask;
        self.frame_pressed |= mask;
        self.fixed_pressed |= mask;
    }

    /// Ignored if not held (a release without a press).
    fn up(&mut self, bit: u8) {
        let mask = 1 << bit;
        if self.held & mask == 0 {
            return;
        }
        self.held &= !mask;
        self.frame_released |= mask;
        self.fixed_released |= mask;
    }

    fn release_all(&mut self) {
        self.frame_released |= self.held;
        self.fixed_released |= self.held;
        self.held = 0;
    }

    fn end_fixed_step(&mut self) {
        self.fixed_pressed = 0;
        self.fixed_released = 0;
    }

    fn end_frame(&mut self) {
        self.frame_pressed = 0;
        self.frame_released = 0;
    }
}

/// Keyboard and mouse state for the current callback. See the
/// [module docs](self) for how edges relate to `fixed_update` and `update`;
/// mouse buttons and the wheel follow the same rules as keys.
///
/// ```
/// use purplepie::input::{KeyCode, MouseButton};
/// # fn fixed_update(ctx: &mut purplepie::Context<'_>) {
/// let input = ctx.input();
/// let dx = input.axis(KeyCode::ArrowLeft, KeyCode::ArrowRight); // −1, 0 or 1
/// if input.just_pressed(KeyCode::Space) {
///     // jump: runs once per press, however many fixed steps the frame has
/// }
/// if input.mouse_just_pressed(MouseButton::Left) {
///     if let Some(target) = ctx.cursor_world() {
///         // shoot towards `target` (world coordinates)
///         # let _ = target;
///     }
/// }
/// # let _ = dx;
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Input {
    keys: Buttons,
    mouse: Buttons,
    /// Logical pixels, top-left origin, +Y down; `None` outside the window.
    cursor: Option<Vec2>,
    /// Wheel movement in lines since the previous frame / not yet seen by a fixed step.
    frame_scroll: Vec2,
    fixed_scroll: Vec2,
    phase: Phase,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            keys: Buttons::default(),
            mouse: Buttons::default(),
            cursor: None,
            frame_scroll: Vec2::ZERO,
            fixed_scroll: Vec2::ZERO,
            phase: Phase::Frame,
        }
    }
}

impl Input {
    /// Whether `key` is held down now.
    pub fn pressed(&self, key: KeyCode) -> bool {
        self.keys.held(key as u8)
    }

    /// Whether `key` went down since this callback last looked (see the module docs).
    /// A key pressed and released within one frame is both `just_pressed` and
    /// `just_released`, but not `pressed`.
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.keys.just_pressed(key as u8, self.phase)
    }

    /// Whether `key` went up since this callback last looked (see the module docs).
    pub fn just_released(&self, key: KeyCode) -> bool {
        self.keys.just_released(key as u8, self.phase)
    }

    /// `-1.0` while only `negative` is held, `1.0` while only `positive` is held,
    /// `0.0` for neither or both. Handy for movement: `axis(ArrowLeft, ArrowRight)`.
    pub fn axis(&self, negative: KeyCode, positive: KeyCode) -> f32 {
        f32::from(i8::from(self.pressed(positive)) - i8::from(self.pressed(negative)))
    }

    /// The held keys, in `KeyCode` order.
    pub fn pressed_keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        KeyCode::ALL.iter().copied().filter(|&k| self.pressed(k))
    }

    /// Whether `button` is held down now.
    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse.held(button as u8)
    }

    /// Whether `button` went down since this callback last looked (same rules as keys).
    pub fn mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse.just_pressed(button as u8, self.phase)
    }

    /// Whether `button` went up since this callback last looked (same rules as keys).
    pub fn mouse_just_released(&self, button: MouseButton) -> bool {
        self.mouse.just_released(button as u8, self.phase)
    }

    /// The cursor in **screen** coordinates: logical pixels from the top-left of
    /// the window's drawing area, +Y down (ADR-022). `None` while the cursor is
    /// outside the window. For world coordinates use
    /// [`Context::cursor_world`](crate::Context::cursor_world).
    pub fn cursor_position(&self) -> Option<Vec2> {
        self.cursor
    }

    /// Mouse wheel movement since this callback last looked, in lines (notches):
    /// `y > 0` is away from the user ("scroll up"), `x > 0` is to the right.
    /// Each movement is reported once to `fixed_update` and once to `update`,
    /// like key edges. Touchpad pixel scrolling counts 20 logical pixels per line.
    pub fn scroll(&self) -> Vec2 {
        match self.phase {
            Phase::Fixed => self.fixed_scroll,
            Phase::Frame => self.frame_scroll,
        }
    }

    /// A key went down. Repeats of an already held key are ignored.
    pub(crate) fn key_down(&mut self, key: KeyCode) {
        self.keys.down(key as u8);
    }

    /// A key went up. A release without a matching press (e.g. the key was held
    /// before the window got focus) is ignored.
    pub(crate) fn key_up(&mut self, key: KeyCode) {
        self.keys.up(key as u8);
    }

    pub(crate) fn mouse_down(&mut self, button: MouseButton) {
        self.mouse.down(button as u8);
    }

    pub(crate) fn mouse_up(&mut self, button: MouseButton) {
        self.mouse.up(button as u8);
    }

    /// The cursor moved to `position` (logical pixels), or left the window (`None`).
    pub(crate) fn set_cursor(&mut self, position: Option<Vec2>) {
        self.cursor = position;
    }

    /// The wheel moved by `lines`.
    pub(crate) fn add_scroll(&mut self, lines: Vec2) {
        if lines.is_finite() {
            self.frame_scroll += lines;
            self.fixed_scroll += lines;
        }
    }

    /// Releases every held key and mouse button (the window lost focus, so
    /// releases would be missed).
    pub(crate) fn release_all(&mut self) {
        self.keys.release_all();
        self.mouse.release_all();
    }

    /// Call before each `fixed_update`.
    pub(crate) fn begin_fixed_step(&mut self) {
        self.phase = Phase::Fixed;
    }

    /// Call after each `fixed_update`: the fixed-step edges have now been seen.
    pub(crate) fn end_fixed_step(&mut self) {
        self.keys.end_fixed_step();
        self.mouse.end_fixed_step();
        self.fixed_scroll = Vec2::ZERO;
        self.phase = Phase::Frame;
    }

    /// Call after `update`: the frame edges have now been seen.
    pub(crate) fn end_frame(&mut self) {
        self.keys.end_frame();
        self.mouse.end_frame();
        self.frame_scroll = Vec2::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use KeyCode::*;

    /// What a game sees, frame by frame: `(fixed_update edges, update edges)`.
    #[derive(Debug, Default, PartialEq)]
    struct Seen {
        fixed_presses: usize,
        update_presses: usize,
        fixed_releases: usize,
        update_releases: usize,
    }

    /// Runs one frame like the runner does, counting the edges of `key`.
    fn frame(input: &mut Input, fixed_steps: usize, key: KeyCode, seen: &mut Seen) {
        for _ in 0..fixed_steps {
            input.begin_fixed_step();
            seen.fixed_presses += usize::from(input.just_pressed(key));
            seen.fixed_releases += usize::from(input.just_released(key));
            input.end_fixed_step();
        }
        seen.update_presses += usize::from(input.just_pressed(key));
        seen.update_releases += usize::from(input.just_released(key));
        input.end_frame();
    }

    #[test]
    fn press_hold_release() {
        let mut input = Input::default();
        assert!(!input.pressed(Space));
        input.key_down(Space);
        assert!(input.pressed(Space) && input.just_pressed(Space));
        input.end_frame();
        assert!(
            input.pressed(Space) && !input.just_pressed(Space),
            "held, no new edge"
        );
        input.key_up(Space);
        assert!(!input.pressed(Space) && input.just_released(Space));
        input.end_frame();
        assert!(!input.just_released(Space));
    }

    #[test]
    fn os_key_repeat_does_not_create_new_presses() {
        let mut input = Input::default();
        let mut seen = Seen::default();
        input.key_down(A);
        frame(&mut input, 1, A, &mut seen);
        input.key_down(A); // repeat while held
        input.key_down(A);
        frame(&mut input, 1, A, &mut seen);
        assert_eq!((seen.fixed_presses, seen.update_presses), (1, 1));
    }

    #[test]
    fn edges_are_seen_once_with_one_fixed_step() {
        let mut input = Input::default();
        let mut seen = Seen::default();
        input.key_down(W);
        frame(&mut input, 1, W, &mut seen);
        frame(&mut input, 1, W, &mut seen);
        input.key_up(W);
        frame(&mut input, 1, W, &mut seen);
        frame(&mut input, 1, W, &mut seen);
        assert_eq!(
            seen,
            Seen {
                fixed_presses: 1,
                update_presses: 1,
                fixed_releases: 1,
                update_releases: 1
            }
        );
    }

    #[test]
    fn several_fixed_steps_see_an_edge_only_in_the_first() {
        let mut input = Input::default();
        input.key_down(W);
        let mut per_step = Vec::new();
        for _ in 0..4 {
            input.begin_fixed_step();
            per_step.push(input.just_pressed(W));
            assert!(input.pressed(W), "held state is visible in every step");
            input.end_fixed_step();
        }
        assert_eq!(per_step, [true, false, false, false]);
        assert!(input.just_pressed(W), "update still sees the frame's edge");
    }

    #[test]
    fn a_frame_without_fixed_steps_carries_the_edge_to_the_next_step() {
        let mut input = Input::default();
        let mut seen = Seen::default();
        input.key_down(Space);
        frame(&mut input, 0, Space, &mut seen); // high frame rate: no step this frame
        assert_eq!((seen.fixed_presses, seen.update_presses), (0, 1));
        frame(&mut input, 0, Space, &mut seen);
        assert_eq!(seen.fixed_presses, 0, "still waiting");
        frame(&mut input, 1, Space, &mut seen);
        assert_eq!(
            (seen.fixed_presses, seen.update_presses),
            (1, 1),
            "seen once by each"
        );
        frame(&mut input, 3, Space, &mut seen);
        assert_eq!((seen.fixed_presses, seen.update_presses), (1, 1));
    }

    #[test]
    fn tap_within_one_frame_is_pressed_and_released_but_not_held() {
        let mut input = Input::default();
        let mut seen = Seen::default();
        input.key_down(Enter);
        input.key_up(Enter);
        assert!(!input.pressed(Enter));
        assert!(input.just_pressed(Enter) && input.just_released(Enter));
        frame(&mut input, 0, Enter, &mut seen);
        frame(&mut input, 2, Enter, &mut seen);
        assert_eq!(
            seen,
            Seen {
                fixed_presses: 1,
                update_presses: 1,
                fixed_releases: 1,
                update_releases: 1
            }
        );
    }

    #[test]
    fn release_all_releases_only_held_keys() {
        let mut input = Input::default();
        input.key_down(A);
        input.key_down(ShiftLeft);
        input.end_frame();
        input.release_all();
        assert_eq!(input.pressed_keys().count(), 0);
        assert!(input.just_released(A) && input.just_released(ShiftLeft));
        assert!(!input.just_released(B), "B was never held");
    }

    #[test]
    fn release_without_press_is_ignored() {
        let mut input = Input::default();
        input.key_up(Q);
        assert!(!input.just_released(Q));
    }

    #[test]
    fn axis_combines_two_keys() {
        let mut input = Input::default();
        assert_eq!(input.axis(ArrowLeft, ArrowRight), 0.0);
        input.key_down(ArrowRight);
        assert_eq!(input.axis(ArrowLeft, ArrowRight), 1.0);
        input.key_down(ArrowLeft);
        assert_eq!(input.axis(ArrowLeft, ArrowRight), 0.0, "both cancel");
        input.key_up(ArrowRight);
        assert_eq!(input.axis(ArrowLeft, ArrowRight), -1.0);
    }

    #[test]
    fn mouse_buttons_follow_the_same_edge_rules_as_keys() {
        let mut input = Input::default();
        input.mouse_down(MouseButton::Left);
        input.mouse_down(MouseButton::Left); // duplicate: ignored
        let mut fixed_presses = 0;
        // A frame with no fixed step, then one with three.
        assert!(
            input.mouse_just_pressed(MouseButton::Left),
            "update sees it now"
        );
        input.end_frame();
        for _ in 0..3 {
            input.begin_fixed_step();
            fixed_presses += usize::from(input.mouse_just_pressed(MouseButton::Left));
            assert!(input.mouse_pressed(MouseButton::Left));
            input.end_fixed_step();
        }
        assert!(
            !input.mouse_just_pressed(MouseButton::Left),
            "update saw it last frame"
        );
        assert_eq!(fixed_presses, 1);
        input.mouse_up(MouseButton::Left);
        assert!(input.mouse_just_released(MouseButton::Left));
        assert!(!input.mouse_pressed(MouseButton::Left));
    }

    #[test]
    fn keys_and_mouse_buttons_do_not_share_bits() {
        let mut input = Input::default();
        input.key_down(A); // bit 0, like MouseButton::Left
        assert!(!input.mouse_pressed(MouseButton::Left));
        input.mouse_down(MouseButton::Right);
        assert!(!input.pressed(B));
    }

    #[test]
    fn release_all_includes_mouse_buttons() {
        let mut input = Input::default();
        input.mouse_down(MouseButton::Middle);
        input.key_down(Space);
        input.end_frame();
        input.release_all();
        assert!(!input.mouse_pressed(MouseButton::Middle));
        assert!(input.mouse_just_released(MouseButton::Middle));
        assert!(input.just_released(Space));
    }

    #[test]
    fn cursor_is_plain_state_and_none_outside_the_window() {
        let mut input = Input::default();
        assert_eq!(input.cursor_position(), None);
        input.set_cursor(Some(Vec2::new(10.5, 20.0)));
        input.end_frame();
        assert_eq!(
            input.cursor_position(),
            Some(Vec2::new(10.5, 20.0)),
            "persists across frames"
        );
        input.set_cursor(None);
        assert_eq!(input.cursor_position(), None);
    }

    #[test]
    fn scroll_is_reported_once_to_each_callback() {
        let mut input = Input::default();
        input.add_scroll(Vec2::new(0.0, 1.0));
        input.add_scroll(Vec2::new(0.0, 2.0));
        input.add_scroll(Vec2::new(f32::NAN, 0.0)); // ignored
        assert_eq!(
            input.scroll(),
            Vec2::new(0.0, 3.0),
            "update: summed since last frame"
        );
        input.end_frame(); // a frame without fixed steps
        assert_eq!(input.scroll(), Vec2::ZERO);
        let mut seen = Vec::new();
        for _ in 0..2 {
            input.begin_fixed_step();
            seen.push(input.scroll());
            input.end_fixed_step();
        }
        assert_eq!(
            seen,
            [Vec2::new(0.0, 3.0), Vec2::ZERO],
            "first fixed step only"
        );
    }

    #[test]
    fn every_key_has_its_own_bit() {
        let mut input = Input::default();
        for (i, &key) in KeyCode::ALL.iter().enumerate() {
            assert_eq!(key as usize, i);
            input.key_down(key);
        }
        assert_eq!(input.pressed_keys().count(), KeyCode::ALL.len());
        assert!(KeyCode::ALL.len() >= 99);
    }
}
