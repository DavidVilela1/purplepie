//! Which screen the game is on, and what a click or a key does there.
//!
//! Kept free of engine types so the whole flow is unit-tested.

/// The four screens of GAME2.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// "Click to start".
    Title,
    /// The game is running.
    Playing,
    /// Frozen; click resumes, Escape or Q quits.
    Paused,
    /// The player has no hit points left; click plays again.
    Over,
}

/// What the player did this frame (edges, not held keys).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Presses {
    /// Left mouse button went down.
    pub click: bool,
    /// Escape went down.
    pub escape: bool,
    /// Q went down.
    pub quit: bool,
}

/// What the game should do in response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Nothing,
    Start,
    Pause,
    Resume,
    Restart,
    Quit,
}

impl Screen {
    /// The action for `presses` on this screen. A click wins over a key
    /// pressed in the same frame, so a click never quits by accident.
    pub fn action(self, presses: Presses) -> Action {
        let leave = presses.escape || presses.quit;
        match self {
            Screen::Title if presses.click => Action::Start,
            Screen::Paused if presses.click => Action::Resume,
            Screen::Over if presses.click => Action::Restart,
            Screen::Title | Screen::Paused | Screen::Over if leave => Action::Quit,
            Screen::Playing if presses.escape => Action::Pause,
            _ => Action::Nothing,
        }
    }

    /// The screen after `action`.
    pub fn after(self, action: Action) -> Screen {
        match action {
            Action::Start | Action::Resume | Action::Restart => Screen::Playing,
            Action::Pause => Screen::Paused,
            Action::Nothing | Action::Quit => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLICK: Presses = Presses {
        click: true,
        escape: false,
        quit: false,
    };
    const ESCAPE: Presses = Presses {
        click: false,
        escape: true,
        quit: false,
    };
    const Q: Presses = Presses {
        click: false,
        escape: false,
        quit: true,
    };
    const NONE: Presses = Presses {
        click: false,
        escape: false,
        quit: false,
    };

    #[test]
    fn a_whole_session_title_play_pause_resume_over_restart_quit() {
        let mut screen = Screen::Title;
        let mut actions = Vec::new();
        for (presses, expected) in [
            (NONE, Screen::Title),
            (CLICK, Screen::Playing),
            (CLICK, Screen::Playing), // shooting, not a screen change
            (Q, Screen::Playing),     // Q does nothing while playing
            (ESCAPE, Screen::Paused),
            (CLICK, Screen::Playing),
        ] {
            let action = screen.action(presses);
            actions.push(action);
            screen = screen.after(action);
            assert_eq!(screen, expected, "after {presses:?}");
        }
        assert_eq!(
            actions,
            [
                Action::Nothing,
                Action::Start,
                Action::Nothing,
                Action::Nothing,
                Action::Pause,
                Action::Resume
            ]
        );
        // The game sets `Over` itself when the hit points run out.
        let over = Screen::Over;
        assert_eq!(over.action(CLICK), Action::Restart);
        assert_eq!(over.after(Action::Restart), Screen::Playing);
        assert_eq!(over.action(ESCAPE), Action::Quit);
        assert_eq!(over.action(NONE), Action::Nothing);
    }

    #[test]
    fn escape_or_q_quits_from_every_screen_except_play() {
        for screen in [Screen::Title, Screen::Paused, Screen::Over] {
            assert_eq!(screen.action(ESCAPE), Action::Quit, "{screen:?}");
            assert_eq!(screen.action(Q), Action::Quit, "{screen:?}");
            assert_eq!(screen.after(Action::Quit), screen);
        }
        assert_eq!(Screen::Playing.action(ESCAPE), Action::Pause);
    }

    #[test]
    fn a_click_wins_over_a_key_in_the_same_frame() {
        let both = Presses {
            click: true,
            escape: true,
            quit: true,
        };
        assert_eq!(Screen::Title.action(both), Action::Start);
        assert_eq!(Screen::Paused.action(both), Action::Resume);
        assert_eq!(Screen::Over.action(both), Action::Restart);
    }
}
