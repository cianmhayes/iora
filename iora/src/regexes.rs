use once_cell::sync::OnceCell;
use regex::{Captures, Match, Regex};

pub(crate) fn semver_regex(s: &str) -> Option<Captures> {
    static SEMVER_REGEX: OnceCell<Regex> = OnceCell::new();
    SEMVER_REGEX.get_or_init(|| regex::Regex::new(r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$").unwrap()).captures(s)
}

pub(crate) fn partial_version_constraint_regex(s: &str) -> Option<Captures> {
    static PARTIAL_VERSION_CONSTRAINT_REGEX: OnceCell<Regex> = OnceCell::new();
    PARTIAL_VERSION_CONSTRAINT_REGEX.get_or_init(|| regex::Regex::new(r"^(?P<major>[0-9]+)(\.(?P<minor>[0-9]+))?$").unwrap()).captures(s)
}

pub(crate) fn name_constraint_regex(s: &str) -> Option<Captures> {
    static PARTIAL_VERSION_CONSTRAINT_REGEX: OnceCell<Regex> = OnceCell::new();
    PARTIAL_VERSION_CONSTRAINT_REGEX.get_or_init(|| regex::Regex::new(r"^(?P<start>\*)?(?P<term>[^\*]+)(?P<end>\*)?$").unwrap()).captures(s)
}


pub(crate) fn parse_u32(m: Option<Match>) -> Option<u32> {
    match m.map(|inner_match| inner_match.as_str().parse::<u32>()) {
        Some(Ok(i)) => Some(i),
        _ => None,
    }
}

pub(crate) fn match_to_string(m: Option<Match>) -> Option<String> {
    m.map(|inner_match| inner_match.as_str().to_string())
}
