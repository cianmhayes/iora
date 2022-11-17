use std::str::FromStr;

use crate::{SemVer, regexes};

use lazy_static::lazy_static;
use regex::Regex;


#[derive(Clone, Debug, Hash, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum NameConstraint {
    ExactMatch(String),
    StartsWith(String),
    Contains(String),
}

impl NameConstraint {
    pub fn matches(&self, name: &str) -> bool {
        match self {
            NameConstraint::ExactMatch(target) => name == *target,
            NameConstraint::StartsWith(prefix) => name.starts_with(prefix),
            NameConstraint::Contains(substring) => name.contains(substring),
        }
    }
}

#[derive(Debug)]
pub enum ConstraintParsingError {
    FailedToParseNameConstraint
}

impl FromStr for NameConstraint {
    type Err = ConstraintParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref NAME_CONSTRAINT_PARSE: Regex = Regex::new(r"^(?P<start>\*)?(?P<term>[^\*]+)(?P<end>\*)?$").unwrap();
        }
        if let Some(captures) = NAME_CONSTRAINT_PARSE.captures(&s) {
            return match (regexes::match_to_string(captures.name("start")), regexes::match_to_string(captures.name("term")), regexes::match_to_string(captures.name("end"))) {
                (Some(_), Some(term), Some(_)) => Ok(NameConstraint::Contains(term)),
                (None, Some(term), Some(_)) => Ok(NameConstraint::StartsWith(term)),
                (None, Some(term), None) => Ok(NameConstraint::ExactMatch(term)),
                _ => Err(ConstraintParsingError::FailedToParseNameConstraint)
            };
        }
        return Err(ConstraintParsingError::FailedToParseNameConstraint);
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum VersionConstraint {
    ExactMatch(SemVer),
    MatchMajorVersionOnly(u32),
    MatchMajorAndMinorVersionOnly((u32, u32)),
    MinVersion(SemVer),
    Between((SemVer,SemVer)),
}

impl FromStr for VersionConstraint {
    type Err = ConstraintParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(",") {
            let parts:Vec<_> = s.split(',').collect();
            if parts.len() > 1 {
                if let (Ok(min_ver), Ok(max_ver)) = (SemVer::from_str(parts[0]), SemVer::from_str(parts[1])) {
                    return Ok(VersionConstraint::Between((min_ver, max_ver)));
                }
            } else {
                if let Ok(min_ver) = SemVer::from_str(parts[0]) {
                    return Ok(VersionConstraint::MinVersion(min_ver));
                }
            }
        }
        if let Ok(full_sem_ver) = SemVer::from_str(s) {
            return Ok(VersionConstraint::ExactMatch(full_sem_ver));
        }
        lazy_static! {
            static ref PARTIAL_VERSION_MATCH_PARSE: Regex = Regex::new(r"^(?P<major>[0-9]+)(\.(?P<minor>[0-9]+))?$").unwrap();
        }
        if let Some(captures) = PARTIAL_VERSION_MATCH_PARSE.captures(&s) {
            match (regexes::match_to_u32(captures.name("major")), regexes::match_to_u32(captures.name("minor"))) {
                (Some(major), Some(minor)) => {return Ok(VersionConstraint::MatchMajorAndMinorVersionOnly((major, minor)));},
                (Some(major), None) => {return Ok(VersionConstraint::MatchMajorVersionOnly(major));},
                _ => {}
            }
        }
        return Err(ConstraintParsingError::FailedToParseNameConstraint);
    }
}

impl VersionConstraint {
    pub fn matches(&self, version: &SemVer) -> bool {
        match self {
            VersionConstraint::ExactMatch(target) => version == target,
            VersionConstraint::MinVersion(target) => version >= target,
            VersionConstraint::MatchMajorVersionOnly(target) => version.major == *target,
            VersionConstraint::MatchMajorAndMinorVersionOnly(target) => {
                version.major == target.0 && version.minor == target.1
            }
            VersionConstraint::Between((min, max)) => version >= min && version < max,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AssetQuery {
    pub name_constraint: NameConstraint,
    pub version_constraint: Option<VersionConstraint>,
}

impl AssetQuery {
    pub fn new(
        name_constraint: &NameConstraint,
        version_constraint: &Option<VersionConstraint>,
    ) -> Self {
        AssetQuery {
            name_constraint: name_constraint.clone(),
            version_constraint: version_constraint.clone(),
        }
    }
}

impl From<NameConstraint> for AssetQuery {
    fn from(nc: NameConstraint) -> Self {
        AssetQuery { name_constraint:nc, version_constraint: None }
    }
}

impl From<&NameConstraint> for AssetQuery {
    fn from(nc: &NameConstraint) -> Self {
        AssetQuery { name_constraint:nc.clone(), version_constraint: None }
    }
}

impl From<(NameConstraint, Option<VersionConstraint>)> for AssetQuery {
    fn from(tuple: (NameConstraint, Option<VersionConstraint>)) -> Self {
        AssetQuery { name_constraint: tuple.0, version_constraint: tuple.1}
    }
}

impl From<(&NameConstraint, &Option<VersionConstraint>)> for AssetQuery {
    fn from(tuple: (&NameConstraint, &Option<VersionConstraint>)) -> Self {
        AssetQuery { name_constraint: tuple.0.clone(), version_constraint: tuple.1.clone()}
    }
}

impl From<(NameConstraint, VersionConstraint)> for AssetQuery {
    fn from(tuple: (NameConstraint, VersionConstraint)) -> Self {
        AssetQuery { name_constraint: tuple.0, version_constraint: Some(tuple.1)}
    }
}

impl From<(&NameConstraint, &VersionConstraint)> for AssetQuery {
    fn from(tuple: (&NameConstraint, &VersionConstraint)) -> Self {
        AssetQuery { name_constraint: tuple.0.clone(), version_constraint: Some(tuple.1.clone())}
    }
}
