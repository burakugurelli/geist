use geist_core::{Cue, UnknownCue};

#[test]
fn semantic_names_round_trip() {
    for (name, cue) in [
        ("press", Cue::Press),
        ("success", Cue::Success),
        ("error", Cue::Error),
    ] {
        assert_eq!(Cue::try_from(name), Ok(cue));
        assert_eq!(cue.as_str(), name);
    }
}

#[test]
fn unsupported_name_returns_a_typed_error() {
    assert_eq!(Cue::try_from("chime"), Err(UnknownCue));
}

#[test]
fn all_contains_each_public_cue_once() {
    assert_eq!(Cue::ALL, [Cue::Press, Cue::Success, Cue::Error]);
}
