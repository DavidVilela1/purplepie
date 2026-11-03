//! "Save / Don't save / Cancel": what the editor asks before something would
//! lose unsaved changes (PP-044a1, PP-044a2, ADR-046).

use std::path::PathBuf;

/// Something that leaves the open scene, waiting for the answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Leave {
    /// Closing the window.
    Close,
    /// Opening this scene file instead.
    Open(PathBuf),
    /// Starting an empty scene.
    New,
}

impl Leave {
    /// The question's last words, e.g. "before closing".
    pub(crate) fn before(&self) -> String {
        match self {
            Leave::Close => "before closing".to_owned(),
            Leave::Open(file) => format!(
                "before opening {}",
                file.file_name().map_or_else(
                    || file.display().to_string(),
                    |n| n.to_string_lossy().into_owned()
                )
            ),
            Leave::New => "before starting a new scene".to_owned(),
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
        assert_eq!(
            Leave::Open(PathBuf::from("assets/levels/level2.ron")).before(),
            "before opening level2.ron"
        );
        assert_eq!(Leave::New.before(), "before starting a new scene");
    }
}
