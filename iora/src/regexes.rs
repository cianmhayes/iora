use once_cell::sync::OnceCell;
use regex::{Match, Regex};

#[allow(clippy::type_complexity)]
pub(crate) fn parse_semver(
    s: &str,
) -> (
    Option<u32>,
    Option<u32>,
    Option<u32>,
    Option<String>,
    Option<String>,
) {
    static SEMVER_REGEX: OnceCell<Regex> = OnceCell::new();
    if let Some(captures) = SEMVER_REGEX.get_or_init(|| regex::Regex::new(r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$").unwrap()).captures(s) {
        (
            match_to_u32(captures.name("major")),
            match_to_u32(captures.name("minor")),
            match_to_u32(captures.name("patch")),
            match_to_string(captures.name("prerelease")),
            match_to_string(captures.name("buildmetadata")))
    } else {
        (None, None, None, None, None)
    }
}

pub(crate) fn parse_partial_version_constraint(s: &str) -> (Option<u32>, Option<u32>) {
    static PARTIAL_VERSION_CONSTRAINT_REGEX: OnceCell<Regex> = OnceCell::new();
    if let Some(captures) = PARTIAL_VERSION_CONSTRAINT_REGEX
        .get_or_init(|| regex::Regex::new(r"^(?P<major>[0-9]+)(\.(?P<minor>[0-9]+))?$").unwrap())
        .captures(s)
    {
        (
            match_to_u32(captures.name("major")),
            match_to_u32(captures.name("minor")),
        )
    } else {
        (None, None)
    }
}

pub(crate) fn parse_name_constraint(s: &str) -> (Option<String>, Option<String>, Option<String>) {
    static NAME_CONSTRAINT_REGEX: OnceCell<Regex> = OnceCell::new();
    if let Some(captures) = NAME_CONSTRAINT_REGEX
        .get_or_init(|| regex::Regex::new(r"^(?P<start>\*)?(?P<term>[^\*]+)(?P<end>\*)?$").unwrap())
        .captures(s)
    {
        (
            match_to_string(captures.name("start")),
            match_to_string(captures.name("term")),
            match_to_string(captures.name("end")),
        )
    } else {
        (None, None, None)
    }
}

fn match_to_u32(m: Option<Match>) -> Option<u32> {
    match m.map(|inner_match| inner_match.as_str().parse::<u32>()) {
        Some(Ok(i)) => Some(i),
        _ => None,
    }
}

fn match_to_string(m: Option<Match>) -> Option<String> {
    m.map(|inner_match| inner_match.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_semver_test() {
        assert_eq!(parse_semver("12.34.56"), (Some(12), Some(34), Some(56), None, None));
        assert_eq!(parse_semver("12.34.56-beta+nightly"), (Some(12), Some(34), Some(56), Some("beta".to_owned()), Some("nightly".to_owned())));
        assert_eq!(parse_semver("12.34.56-beta"), (Some(12), Some(34), Some(56), Some("beta".to_owned()), None));
        assert_eq!(parse_semver("12.34.56+nightly"), (Some(12), Some(34), Some(56), None, Some("nightly".to_owned())));
        
        assert_eq!(parse_semver(""), (None, None, None, None, None));
        assert_eq!(parse_semver("v"), (None, None, None, None, None));
        assert_eq!(parse_semver("12.34.56-+"), (None, None, None, None, None));
        assert_eq!(parse_semver("12.34"), (None, None, None, None, None));
        assert_eq!(parse_semver("12"), (None, None, None, None, None));
    }

    #[test]
    fn parse_partial_version_constraint_test() {
        assert_eq!(parse_partial_version_constraint("12.34"), (Some(12), Some(34)));
        assert_eq!(parse_partial_version_constraint("12"), (Some(12), None));

        assert_eq!(parse_partial_version_constraint("-12"), (None, None));
        assert_eq!(parse_partial_version_constraint(""), (None, None));
        assert_eq!(parse_partial_version_constraint("12.34.56"), (None, None));
        assert_eq!(parse_partial_version_constraint("v"), (None, None));
    }

    #[test]
    fn parse_name_constraint_test() {
        assert_eq!(parse_name_constraint("thing"), (None, Some("thing".to_owned()), None));
        assert_eq!(parse_name_constraint("thing*"), (None, Some("thing".to_owned()), Some("*".to_owned())));
        assert_eq!(parse_name_constraint("*thing"), (Some("*".to_owned()), Some("thing".to_owned()), None));
        assert_eq!(parse_name_constraint("*thing*"), (Some("*".to_owned()), Some("thing".to_owned()), Some("*".to_owned())));

        assert_eq!(parse_name_constraint(""), (None, None, None));
        assert_eq!(parse_name_constraint("*"), (None, None, None));
        assert_eq!(parse_name_constraint("**"), (None, None, None));
    }
}