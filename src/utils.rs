pub fn format_time(t: u64) -> String {
    format!("{}:{:02}", (t - (t % 60)) / 60, t % 60)
}
