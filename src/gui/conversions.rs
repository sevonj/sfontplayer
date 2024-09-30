use std::time::Duration;

/// Formatted to represent playback times. "07:32"
pub(crate) fn format_duration(dur: Duration) -> String {
    let sec = dur.as_secs() % 60;
    let min = dur.as_secs() / 60;
    let h = dur.as_secs() / 60 / 60;

    if h == 0 {
        format!("{min:02}:{sec:02}")
    } else {
        format!("{h:0}:{min:02}:{sec:02}")
    }
}
