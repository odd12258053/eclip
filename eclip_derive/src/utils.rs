pub fn help_message(padding: usize) -> String {
    format!(
        "  {:<padding$} {}\n  {:<padding$} {}",
        "--help", "Show this message.", "--version", "Show this version.",
    )
}
