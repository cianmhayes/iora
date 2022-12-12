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

impl SemVer {
    pub fn new(
        major: u32,
        minor: u32,
        patch: u32,
        prerelease: Option<String>,
        buildmetadata: Option<String>,
    ) -> Self {
        SemVer {
            major,
            minor,
            patch,
            prerelease,
            buildmetadata,
        }
    }
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
}

impl FromStr for SemVer {
    type Err = SemVerParseEror;
    fn from_str(string_version: &str) -> Result<Self, Self::Err> {
        if let (Some(major), Some(minor), Some(patch), prerelease, buildmetadata) =
            regexes::parse_semver(string_version)
        {
            return Ok(SemVer {
                major,
                minor,
                patch,
                prerelease,
                buildmetadata,
            });
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
            (Ordering::Equal, Ordering::Equal, o) => o,
            (Ordering::Equal, o, _) => o,
            (o, _, _) => o,
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

#[cfg(test)]
mod tests {
    use super::{SemVer, SemVerParseEror};
    use std::str::FromStr;

    #[test]
    fn from_str() {
        assert_eq!(
            SemVer::from_str("3.45.6").unwrap(),
            SemVer::new(3, 45, 6, None, None)
        );

        assert_eq!(
            SemVer::from_str("3.45.6-beta").unwrap(),
            SemVer::new(3, 45, 6, Some("beta".to_string()), None)
        );

        assert_eq!(
            SemVer::from_str("3.45.6+build123").unwrap(),
            SemVer::new(3, 45, 6, None, Some("build123".to_string()))
        );

        assert_eq!(
            SemVer::from_str("3.45.6-beta+build123").unwrap(),
            SemVer::new(
                3,
                45,
                6,
                Some("beta".to_string()),
                Some("build123".to_string())
            )
        );

        assert!(match SemVer::from_str("1.0") {
            Err(SemVerParseEror::UnparsableSemVer) => true,
            _ => false,
        });
    }

    #[test]
    fn comparison() {
        assert!(SemVer::from_str("3.45.6").unwrap() > SemVer::from_str("3.5.6").unwrap());
        assert!(
            SemVer::from_str("3.45.6").unwrap() > SemVer::from_str("3.5.6-prerelease").unwrap()
        );
        assert!(SemVer::from_str("3.45.7").unwrap() > SemVer::from_str("3.5.6").unwrap());
        assert!(SemVer::from_str("3.45.7").unwrap() > SemVer::from_str("3.45.6").unwrap());
        assert!(SemVer::from_str("3.45.7").unwrap() < SemVer::from_str("4.5.6").unwrap());
        assert!(SemVer::from_str("3.45.7").unwrap() <= SemVer::from_str("4.5.6").unwrap());
        assert!(SemVer::from_str("3.45.7").unwrap() != SemVer::from_str("4.5.6").unwrap());
    }

    #[test]
    fn from_json() {
        let json_parsed: SemVer = serde_json::from_str(
            r#"
        {
            "major" : 78,
            "minor" : 123,
            "patch" : 1,
            "prerelease" : "beta",
            "buildmetadata" : "build123"
        }
        "#,
        )
        .unwrap();
        assert_eq!(
            json_parsed,
            SemVer::from_str("78.123.1-beta+build123").unwrap()
        );
    }
}
