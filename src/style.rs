#[cfg(feature = "atty")]
pub fn warning_style(message: String, is_term: bool) -> String {
    if is_term {
        format!("\x1b[31;1m{}\x1b[0m", message)
    } else {
        message
    }
}


#[cfg(any(not(feature = "atty"), feature = "no_color"))]
pub fn warning_style(message: String, _: bool) -> String { message }
