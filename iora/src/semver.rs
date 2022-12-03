use crate::regexes;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub buildmetadata: Option<String>,
}

impl ToString for SemVer {
    fn to_string(&self) -> String {
        format!(
            "{}.{}.{}{}{}",
            self.major,
            self.minor,
            self.patch,
            if let Some(prerelease) = &self.prerelease {
                "-".to_string() + prerelease
            } else {
                "".to_string()
            },
            if let Some(buildmetadata) = &self.buildmetadata {
                "+".to_string() + buildmetadata
            } else {
                "".to_string()
            }
        )
    }
}

#[derive(Debug)]
pub enum SemVerParseEror {
    UnparsableSemVer,
    MissingRequiredVersionPiece,
}

impl FromStr for SemVer {
    type Err = SemVerParseEror;
    fn from_str(string_version: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = regexes::semver_regex(string_version) {
            let major = regexes::parse_u32(captures.name("major"));
            let minor = regexes::parse_u32(captures.name("minor"));
            let patch = regexes::parse_u32(captures.name("patch"));
            let prerelease = regexes::match_to_string(captures.name("prerelease"));
            let buildmetadata = regexes::match_to_string(captures.name("buildmetadata"));
            if let (Some(major), Some(minor), Some(patch), prerelease, buildmetadata) =
                (major, minor, patch, prerelease, buildmetadata)
            {
                return Ok(SemVer {
                    major,
                    minor,
                    patch,
                    prerelease,
                    buildmetadata,
                });
            }
        }
        Err(SemVerParseEror::UnparsableSemVer)
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    fn ge(&self, other: &Self) -> bool {
        self.cmp(other) != Ordering::Less
    }

    fn gt(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Greater
    }

    fn le(&self, other: &Self) -> bool {
        self.cmp(other) != Ordering::Greater
    }

    fn lt(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Less
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        match (
            self.major.cmp(&other.major),
            self.minor.cmp(&other.minor),
            self.patch.cmp(&other.patch),
        ) {
            (Ordering::Equal, Ordering::Equal, Ordering::Equal) => {
                match (&self.prerelease, &other.prerelease) {
                    (None, None) => Ordering::Equal,
                    (Some(_), Some(_)) => self.prerelease.cmp(&other.prerelease),
                    (Some(_), None) => Ordering::Greater,
                    (None, Some(_)) => Ordering::Less,
                }
            }
            (Ordering::Equal, Ordering::Equal, Ordering::Greater) => Ordering::Greater,
            (Ordering::Equal, Ordering::Equal, Ordering::Less) => Ordering::Less,
            (Ordering::Equal, Ordering::Greater, _) => Ordering::Greater,
            (Ordering::Equal, Ordering::Less, _) => Ordering::Less,
            (Ordering::Greater, _, _) => Ordering::Greater,
            (Ordering::Less, _, _) => Ordering::Less,
        }
    }

    fn max(self, other: Self) -> Self {
        if self.cmp(&other) != Ordering::Less {
            self
        } else {
            other
        }
    }

    fn min(self, other: Self) -> Self {
        if self.cmp(&other) != Ordering::Greater {
            self
        } else {
            other
        }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        if self.cmp(&min) == Ordering::Less {
            min
        } else if self.cmp(&max) == Ordering::Greater {
            max
        } else {
            self
        }
    }
}
