use regex::Match;

pub(crate) fn match_to_u32(m: Option<Match>) -> Option<u32> {
    match m.map(|inner_match| inner_match.as_str().parse::<u32>()) {
        Some(Ok(i)) => Some(i),
        _ => None,
    }
}

pub(crate) fn match_to_string(m: Option<Match>) -> Option<String> {
    m.map(|inner_match| inner_match.as_str().to_string())
}
