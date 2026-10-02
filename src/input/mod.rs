//! Keyboard input state (ADR-024).
//!
//! [`Input`] answers three questions per [`KeyCode`]: is it held
//! ([`pressed`](Input::pressed)), did it go down ([`just_pressed`](Input::just_pressed)),
//! did it go up ([`just_released`](Input::just_released)). The engine feeds it
//! from window events (the winit translation lives in `app`, so this module is
//! platform-free) and lends it to the game through
//! [`Context::input`](crate::Context::input).
//!
//! **Edges and the fixed timestep.** A frame runs 0, 1 or several
//! `fixed_update`s, then one `update`. Every key edge is reported:
//! - exactly once to `update`: in the first frame after the event;
//! - exactly once to `fixed_update`: in the first fixed step that runs after the
//!   event. If a frame has no fixed step, the edge waits for the next frame
//!   that has one; if it has several, only the first one sees it.
//!
//! So `just_pressed` is safe to use in either callback, and a press is never
//! lost or doubled.

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
            $($(#[$doc])* $name,)+
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

// Every key must fit in the 128-bit `KeySet`.
const _: () = assert!(KeyCode::ALL.len() <= 128);

/// A set of keys as a 128-bit mask (one bit per `KeyCode`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct KeySet(u128);

impl KeySet {
    fn bit(key: KeyCode) -> u128 {
        1 << (key as u8)
    }
    fn contains(self, key: KeyCode) -> bool {
        self.0 & Self::bit(key) != 0
    }
    fn insert(&mut self, key: KeyCode) {
        self.0 |= Self::bit(key);
    }
    fn remove(&mut self, key: KeyCode) {
        self.0 &= !Self::bit(key);
    }
}

/// Which callback is reading the input: decides which edge set is visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Fixed,
    Frame,
}

/// Keyboard state for the current callback. See the [module docs](self) for
/// how edges relate to `fixed_update` and `update`.
///
/// ```
/// use purplepie::input::KeyCode;
/// # fn fixed_update(ctx: &mut purplepie::Context<'_>) {
/// let input = ctx.input();
/// let dx = input.axis(KeyCode::ArrowLeft, KeyCode::ArrowRight); // −1, 0 or 1
/// if input.just_pressed(KeyCode::Space) {
///     // jump: runs once per press, however many fixed steps the frame has
/// }
/// # let _ = dx;
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Input {
    held: KeySet,
    /// Edges for `update`: everything since the previous frame.
    frame_pressed: KeySet,
    frame_released: KeySet,
    /// Edges for `fixed_update`: kept until a fixed step has seen them.
    fixed_pressed: KeySet,
    fixed_released: KeySet,
    phase: Phase,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            held: KeySet::default(),
            frame_pressed: KeySet::default(),
            frame_released: KeySet::default(),
            fixed_pressed: KeySet::default(),
            fixed_released: KeySet::default(),
            phase: Phase::Frame,
        }
    }
}

impl Input {
    /// Whether `key` is held down now.
    pub fn pressed(&self, key: KeyCode) -> bool {
        self.held.contains(key)
    }

    /// Whether `key` went down since this callback last looked (see the module docs).
    /// A key pressed and released within one frame is both `just_pressed` and
    /// `just_released`, but not `pressed`.
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        match self.phase {
            Phase::Fixed => self.fixed_pressed.contains(key),
            Phase::Frame => self.frame_pressed.contains(key),
        }
    }

    /// Whether `key` went up since this callback last looked (see the module docs).
    pub fn just_released(&self, key: KeyCode) -> bool {
        match self.phase {
            Phase::Fixed => self.fixed_released.contains(key),
            Phase::Frame => self.frame_released.contains(key),
        }
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

    /// A key went down. Repeats of an already held key are ignored.
    pub(crate) fn key_down(&mut self, key: KeyCode) {
        if self.held.contains(key) {
            return;
        }
        self.held.insert(key);
        self.frame_pressed.insert(key);
        self.fixed_pressed.insert(key);
    }

    /// A key went up. A release without a matching press (e.g. the key was held
    /// before the window got focus) is ignored.
    pub(crate) fn key_up(&mut self, key: KeyCode) {
        if !self.held.contains(key) {
            return;
        }
        self.held.remove(key);
        self.frame_released.insert(key);
        self.fixed_released.insert(key);
    }

    /// Releases every held key (the window lost focus, so releases would be missed).
    pub(crate) fn release_all(&mut self) {
        for &key in KeyCode::ALL {
            self.key_up(key);
        }
    }

    /// Call before each `fixed_update`.
    pub(crate) fn begin_fixed_step(&mut self) {
        self.phase = Phase::Fixed;
    }

    /// Call after each `fixed_update`: the fixed-step edges have now been seen.
    pub(crate) fn end_fixed_step(&mut self) {
        self.fixed_pressed = KeySet::default();
        self.fixed_released = KeySet::default();
        self.phase = Phase::Frame;
    }

    /// Call after `update`: the frame edges have now been seen.
    pub(crate) fn end_frame(&mut self) {
        self.frame_pressed = KeySet::default();
        self.frame_released = KeySet::default();
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
