//! "Save / Don't save / Cancel": what the editor asks before something would
//! lose unsaved changes (PP-044a1, ADR-046).

/// Something that leaves the open scene, waiting for the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Leave {
    /// Closing the window.
    Close,
}

impl Leave {
    /// The question's last words, e.g. "before closing".
    pub(crate) fn before(self) -> &'static str {
        match self {
            Leave::Close => "before closing",
        }
    }
}

/// The user's answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Answer {
    Save,
    DontSave,
    Cancel,
}

/// Whether to go on with what was asked after `answer`. `save` is called only
/// for [`Answer::Save`] and says whether the scene was saved; a failed save
/// stays, so nothing is lost.
pub(crate) fn proceeds(answer: Answer, save: impl FnOnce() -> bool) -> bool {
    match answer {
        Answer::Save => save(),
        Answer::DontSave => true,
        Answer::Cancel => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_successful_save_or_dont_save_goes_on() {
        let mut saves = 0;
        assert!(proceeds(Answer::Save, || {
            saves += 1;
            true
        }));
        assert!(!proceeds(Answer::Save, || {
            saves += 1;
            false
        }));
        assert_eq!(saves, 2);
        assert!(proceeds(Answer::DontSave, || panic!("not saved")));
        assert!(!proceeds(Answer::Cancel, || panic!("not saved")));
        assert_eq!(Leave::Close.before(), "before closing");
    }
}
