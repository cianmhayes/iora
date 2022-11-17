use regex::Match;

pub(crate) fn match_to_u32(m: Option<Match>) -> Option<u32> {
    if let Some(v) = m {
        if let Ok(i) = u32::from_str_radix(v.as_str(), 10) {
            return Some(i);
        }
    }
    return None;
}

pub(crate) fn match_to_string(m: Option<Match>) -> Option<String> {
    if let Some(v) = m {
        return Some(v.as_str().to_string());
    }
    return None;
}