use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;
use crate::regexes;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Deserialize, Serialize, Hash)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub buildmetadata: Option<String>,
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
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
        lazy_static! {
            static ref SEMVER_PARSE: Regex = Regex::new(r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$").unwrap();
        }
        if let Some(captures) = SEMVER_PARSE.captures(&string_version) {
            let major = regexes::match_to_u32(captures.name("major"));
            let minor = regexes::match_to_u32(captures.name("minor"));
            let patch = regexes::match_to_u32(captures.name("patch"));
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
        return Err(SemVerParseEror::UnparsableSemVer);
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.major < other.major {
            return Ordering::Less;
        } else if self.major > other.major {
            return Ordering::Greater;
        }

        if self.minor < other.minor {
            return Ordering::Less;
        } else if self.minor > other.minor {
            return Ordering::Greater;
        }

        if self.patch < other.patch {
            return Ordering::Less;
        } else if self.patch > other.patch {
            return Ordering::Greater;
        }

        if self.prerelease.is_none() && other.prerelease.is_none() {
            return Ordering::Equal;
        } else if self.prerelease.is_some() && other.prerelease.is_some() {
            return self.prerelease.cmp(&other.prerelease);
        } else {
            if self.prerelease.is_some() {
                return Ordering::Less;
            } else {
                return Ordering::Greater;
            }
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
