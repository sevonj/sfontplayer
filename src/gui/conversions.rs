use std::time::Duration;

pub(crate) fn format_duration(dur: Duration) -> String {
    let sec = dur.as_secs() % 60;
    let min = dur.as_secs() / 60;
    let h = dur.as_secs() / 60 / 60;

    if h == 0 {
        return format!("{:02}:{:02}", min, sec);
    } else {
        return format!("{:0}:{:02}:{:02}", h, min, sec);
    }
}
