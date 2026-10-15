//! Which screen the game is on, and what a click, a button or a key does
//! there (docs/GAME3.md, "Screens"). Free of engine types, so the whole flow
//! is unit-tested.

/// The screens of GAME3.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// "Click to start" over the first level, frozen.
    Title,
    /// The game is running.
    Playing,
    /// Frozen; a click resumes, Escape or Q quits.
    Paused,
    /// A short banner after an exit, over the next level, frozen; it ends by
    /// itself (the game sends [`Action::Continue`]) or on a click.
    LevelComplete,
    /// The last exit was reached: the time and the caught count, and a
    /// "Play again" button.
    Won,
}

/// What the player did this frame (edges, not held keys).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Presses {
    /// The left mouse button went down, anywhere.
    pub click: bool,
    /// The "Play again" button was clicked.
    pub play_again: bool,
    /// Escape went down.
    pub escape: bool,
    /// Q went down.
    pub quit: bool,
}

/// What the game should do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Nothing,
    /// Leave the title and play.
    Start,
    Pause,
    Resume,
    /// Leave the level-complete banner and play the next level.
    Continue,
    /// Play again from the first level, with the time and the caught count
    /// back to zero.
    Restart,
    Quit,
}

impl Screen {
    /// The action for `presses` on this screen. A click wins over a key
    /// pressed in the same frame, so a click never quits by accident. On the
    /// win screen only the button plays again (a stray click does not).
    pub fn action(self, presses: Presses) -> Action {
        let leave = presses.escape || presses.quit;
        match self {
            Screen::Title if presses.click => Action::Start,
            Screen::Paused if presses.click => Action::Resume,
            Screen::LevelComplete if presses.click => Action::Continue,
            Screen::Won if presses.play_again => Action::Restart,
            Screen::Title | Screen::Paused | Screen::Won if leave => Action::Quit,
            Screen::Playing if presses.escape => Action::Pause,
            _ => Action::Nothing,
        }
    }

    /// The screen after `action`.
    pub fn after(self, action: Action) -> Screen {
        match action {
            Action::Start | Action::Resume | Action::Continue | Action::Restart => Screen::Playing,
            Action::Pause => Screen::Paused,
            Action::Nothing | Action::Quit => self,
        }
    }

    /// Whether the world moves (the player, guards and the clock): only while
    /// playing.
    pub fn runs(self) -> bool {
        self == Screen::Playing
    }
}

/// "0:07", "1:05", "61:40": a time in whole seconds as minutes and seconds.
pub fn clock(seconds: f64) -> String {
    let whole = if seconds.is_finite() && seconds > 0.0 {
        seconds.floor() as u64
    } else {
        0
    };
    format!("{}:{:02}", whole / 60, whole % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: Presses = Presses {
        click: false,
        play_again: false,
        escape: false,
        quit: false,
    };
    const CLICK: Presses = Presses {
        click: true,
        ..NONE
    };
    const BUTTON: Presses = Presses {
        click: true,
        play_again: true,
        ..NONE
    };
    const ESCAPE: Presses = Presses {
        escape: true,
        ..NONE
    };
    const Q: Presses = Presses { quit: true, ..NONE };

    #[test]
    fn a_whole_game_title_play_pause_level_complete_win_and_again() {
        let mut screen = Screen::Title;
        let mut seen = Vec::new();
        for presses in [NONE, Q.with_click(), CLICK, Q, ESCAPE, CLICK] {
            let action = screen.action(presses);
            screen = screen.after(action);
            seen.push((action, screen));
        }
        assert_eq!(
            seen,
            [
                (Action::Nothing, Screen::Title),
                (Action::Start, Screen::Playing), // the click wins over Q
                (Action::Nothing, Screen::Playing), // clicks do nothing while playing
                (Action::Nothing, Screen::Playing), // nor does Q
                (Action::Pause, Screen::Paused),
                (Action::Resume, Screen::Playing),
            ]
        );
        // The game shows the banner itself after an exit, and ends it.
        let banner = Screen::LevelComplete;
        assert_eq!(banner.action(NONE), Action::Nothing);
        assert_eq!(banner.action(ESCAPE), Action::Nothing);
        assert_eq!(banner.action(CLICK), Action::Continue, "a click skips it");
        assert_eq!(banner.after(Action::Continue), Screen::Playing);
        // The game shows the win screen itself after the last exit.
        let won = Screen::Won;
        assert_eq!(won.action(CLICK), Action::Nothing, "only the button");
        assert_eq!(won.action(BUTTON), Action::Restart);
        assert_eq!(won.after(Action::Restart), Screen::Playing);
    }

    #[test]
    fn escape_or_q_quits_from_the_title_pause_and_win_screens() {
        for screen in [Screen::Title, Screen::Paused, Screen::Won] {
            assert_eq!(screen.action(ESCAPE), Action::Quit, "{screen:?}");
            assert_eq!(screen.action(Q), Action::Quit, "{screen:?}");
            assert_eq!(screen.after(Action::Quit), screen);
        }
        assert_eq!(Screen::Playing.action(ESCAPE), Action::Pause);
        assert_eq!(Screen::Playing.action(Q), Action::Nothing);
        assert_eq!(
            Screen::Won.action(Presses {
                escape: true,
                ..BUTTON
            }),
            Action::Restart,
            "the button wins over a key"
        );
    }

    #[test]
    fn only_playing_runs_the_world() {
        assert!(Screen::Playing.runs());
        for screen in [
            Screen::Title,
            Screen::Paused,
            Screen::LevelComplete,
            Screen::Won,
        ] {
            assert!(!screen.runs(), "{screen:?}");
        }
    }

    #[test]
    fn times_read_as_minutes_and_seconds() {
        assert_eq!(clock(0.0), "0:00");
        assert_eq!(clock(7.9), "0:07");
        assert_eq!(clock(65.0), "1:05");
        assert_eq!(clock(3700.2), "61:40");
        assert_eq!(clock(-3.0), "0:00");
        assert_eq!(clock(f64::NAN), "0:00");
    }

    impl Presses {
        const fn with_click(self) -> Self {
            Self {
                click: true,
                ..self
            }
        }
    }
}
