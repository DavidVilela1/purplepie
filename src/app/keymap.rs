//! winit → PurplePie input translation (ADR-024): keys, mouse buttons and
//! wheel deltas. The only place that knows both sides, so `input` stays free
//! of winit.

use winit::event::{MouseButton as WinitButton, MouseScrollDelta};
use winit::keyboard::{KeyCode as W, PhysicalKey};

use crate::input::{KeyCode as K, MouseButton, PIXELS_PER_SCROLL_LINE};
use crate::math::Vec2;

/// The PurplePie mouse button, or `None` for extra buttons.
pub(crate) fn translate_button(button: WinitButton) -> Option<MouseButton> {
    Some(match button {
        WinitButton::Left => MouseButton::Left,
        WinitButton::Right => MouseButton::Right,
        WinitButton::Middle => MouseButton::Middle,
        WinitButton::Back => MouseButton::Back,
        WinitButton::Forward => MouseButton::Forward,
        WinitButton::Other(_) => return None,
    })
}

/// A wheel movement in lines (`y > 0` = away from the user). Pixel deltas
/// (touchpads) are physical pixels: divided by the DPI scale, then by
/// `PIXELS_PER_SCROLL_LINE`.
pub(crate) fn scroll_lines(delta: MouseScrollDelta, scale_factor: f64) -> Vec2 {
    match delta {
        MouseScrollDelta::LineDelta(x, y) => Vec2::new(x, y),
        MouseScrollDelta::PixelDelta(p) => {
            let logical = super::state::physical_to_logical(p.x, p.y, scale_factor);
            logical / PIXELS_PER_SCROLL_LINE
        }
    }
}

/// The PurplePie key for a physical key, or `None` for keys PurplePie does
/// not track (media keys, IME keys, unidentified scancodes, …).
pub(crate) fn translate(key: PhysicalKey) -> Option<K> {
    let PhysicalKey::Code(code) = key else {
        return None;
    };
    Some(match code {
        W::KeyA => K::A,
        W::KeyB => K::B,
        W::KeyC => K::C,
        W::KeyD => K::D,
        W::KeyE => K::E,
        W::KeyF => K::F,
        W::KeyG => K::G,
        W::KeyH => K::H,
        W::KeyI => K::I,
        W::KeyJ => K::J,
        W::KeyK => K::K,
        W::KeyL => K::L,
        W::KeyM => K::M,
        W::KeyN => K::N,
        W::KeyO => K::O,
        W::KeyP => K::P,
        W::KeyQ => K::Q,
        W::KeyR => K::R,
        W::KeyS => K::S,
        W::KeyT => K::T,
        W::KeyU => K::U,
        W::KeyV => K::V,
        W::KeyW => K::W,
        W::KeyX => K::X,
        W::KeyY => K::Y,
        W::KeyZ => K::Z,
        W::Digit0 => K::Digit0,
        W::Digit1 => K::Digit1,
        W::Digit2 => K::Digit2,
        W::Digit3 => K::Digit3,
        W::Digit4 => K::Digit4,
        W::Digit5 => K::Digit5,
        W::Digit6 => K::Digit6,
        W::Digit7 => K::Digit7,
        W::Digit8 => K::Digit8,
        W::Digit9 => K::Digit9,
        W::F1 => K::F1,
        W::F2 => K::F2,
        W::F3 => K::F3,
        W::F4 => K::F4,
        W::F5 => K::F5,
        W::F6 => K::F6,
        W::F7 => K::F7,
        W::F8 => K::F8,
        W::F9 => K::F9,
        W::F10 => K::F10,
        W::F11 => K::F11,
        W::F12 => K::F12,
        W::ArrowUp => K::ArrowUp,
        W::ArrowDown => K::ArrowDown,
        W::ArrowLeft => K::ArrowLeft,
        W::ArrowRight => K::ArrowRight,
        W::Space => K::Space,
        W::Enter => K::Enter,
        W::Escape => K::Escape,
        W::Tab => K::Tab,
        W::Backspace => K::Backspace,
        W::Insert => K::Insert,
        W::Delete => K::Delete,
        W::Home => K::Home,
        W::End => K::End,
        W::PageUp => K::PageUp,
        W::PageDown => K::PageDown,
        W::ShiftLeft => K::ShiftLeft,
        W::ShiftRight => K::ShiftRight,
        W::ControlLeft => K::ControlLeft,
        W::ControlRight => K::ControlRight,
        W::AltLeft => K::AltLeft,
        W::AltRight => K::AltRight,
        W::SuperLeft => K::SuperLeft,
        W::SuperRight => K::SuperRight,
        W::Minus => K::Minus,
        W::Equal => K::Equal,
        W::BracketLeft => K::BracketLeft,
        W::BracketRight => K::BracketRight,
        W::Backslash => K::Backslash,
        W::Semicolon => K::Semicolon,
        W::Quote => K::Quote,
        W::Backquote => K::Backquote,
        W::Comma => K::Comma,
        W::Period => K::Period,
        W::Slash => K::Slash,
        W::CapsLock => K::CapsLock,
        W::Numpad0 => K::Numpad0,
        W::Numpad1 => K::Numpad1,
        W::Numpad2 => K::Numpad2,
        W::Numpad3 => K::Numpad3,
        W::Numpad4 => K::Numpad4,
        W::Numpad5 => K::Numpad5,
        W::Numpad6 => K::Numpad6,
        W::Numpad7 => K::Numpad7,
        W::Numpad8 => K::Numpad8,
        W::Numpad9 => K::Numpad9,
        W::NumpadAdd => K::NumpadAdd,
        W::NumpadSubtract => K::NumpadSubtract,
        W::NumpadMultiply => K::NumpadMultiply,
        W::NumpadDivide => K::NumpadDivide,
        W::NumpadEnter => K::NumpadEnter,
        W::NumpadDecimal => K::NumpadDecimal,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::NativeKeyCode;

    #[test]
    fn every_purplepie_key_has_exactly_one_winit_key() {
        let all_winit = [
            W::KeyA,
            W::KeyB,
            W::KeyC,
            W::KeyD,
            W::KeyE,
            W::KeyF,
            W::KeyG,
            W::KeyH,
            W::KeyI,
            W::KeyJ,
            W::KeyK,
            W::KeyL,
            W::KeyM,
            W::KeyN,
            W::KeyO,
            W::KeyP,
            W::KeyQ,
            W::KeyR,
            W::KeyS,
            W::KeyT,
            W::KeyU,
            W::KeyV,
            W::KeyW,
            W::KeyX,
            W::KeyY,
            W::KeyZ,
            W::Digit0,
            W::Digit1,
            W::Digit2,
            W::Digit3,
            W::Digit4,
            W::Digit5,
            W::Digit6,
            W::Digit7,
            W::Digit8,
            W::Digit9,
            W::F1,
            W::F2,
            W::F3,
            W::F4,
            W::F5,
            W::F6,
            W::F7,
            W::F8,
            W::F9,
            W::F10,
            W::F11,
            W::F12,
            W::ArrowUp,
            W::ArrowDown,
            W::ArrowLeft,
            W::ArrowRight,
            W::Space,
            W::Enter,
            W::Escape,
            W::Tab,
            W::Backspace,
            W::Insert,
            W::Delete,
            W::Home,
            W::End,
            W::PageUp,
            W::PageDown,
            W::ShiftLeft,
            W::ShiftRight,
            W::ControlLeft,
            W::ControlRight,
            W::AltLeft,
            W::AltRight,
            W::SuperLeft,
            W::SuperRight,
            W::Minus,
            W::Equal,
            W::BracketLeft,
            W::BracketRight,
            W::Backslash,
            W::Semicolon,
            W::Quote,
            W::Backquote,
            W::Comma,
            W::Period,
            W::Slash,
            W::CapsLock,
            W::Numpad0,
            W::Numpad1,
            W::Numpad2,
            W::Numpad3,
            W::Numpad4,
            W::Numpad5,
            W::Numpad6,
            W::Numpad7,
            W::Numpad8,
            W::Numpad9,
            W::NumpadAdd,
            W::NumpadSubtract,
            W::NumpadMultiply,
            W::NumpadDivide,
            W::NumpadEnter,
            W::NumpadDecimal,
        ];
        let mut mapped: Vec<K> = all_winit
            .iter()
            .filter_map(|&w| translate(PhysicalKey::Code(w)))
            .collect();
        assert_eq!(mapped.len(), all_winit.len());
        mapped.sort();
        mapped.dedup();
        assert_eq!(mapped, K::ALL, "a bijection onto KeyCode::ALL");
    }

    #[test]
    fn names_follow_the_us_layout_positions() {
        assert_eq!(translate(PhysicalKey::Code(W::KeyW)), Some(K::W));
        assert_eq!(translate(PhysicalKey::Code(W::Equal)), Some(K::Equal));
        assert_eq!(
            translate(PhysicalKey::Code(W::ArrowLeft)),
            Some(K::ArrowLeft)
        );
    }

    #[test]
    fn mouse_buttons_map_one_to_one_and_extras_are_ignored() {
        let mapped: Vec<MouseButton> = [
            WinitButton::Left,
            WinitButton::Right,
            WinitButton::Middle,
            WinitButton::Back,
            WinitButton::Forward,
        ]
        .into_iter()
        .filter_map(translate_button)
        .collect();
        assert_eq!(mapped, MouseButton::ALL);
        assert_eq!(translate_button(WinitButton::Other(9)), None);
    }

    #[test]
    fn wheel_lines_pass_through_and_pixels_are_converted() {
        assert_eq!(
            scroll_lines(MouseScrollDelta::LineDelta(0.0, 1.0), 2.0),
            Vec2::new(0.0, 1.0)
        );
        let pixels = winit::dpi::PhysicalPosition::new(0.0, 80.0);
        // 80 physical px at scale 2 = 40 logical px = 2 lines.
        assert_eq!(
            scroll_lines(MouseScrollDelta::PixelDelta(pixels), 2.0),
            Vec2::new(0.0, 2.0)
        );
    }

    #[test]
    fn untracked_and_unidentified_keys_are_ignored() {
        assert_eq!(translate(PhysicalKey::Code(W::MediaPlayPause)), None);
        assert_eq!(
            translate(PhysicalKey::Unidentified(NativeKeyCode::Unidentified)),
            None
        );
    }
}
