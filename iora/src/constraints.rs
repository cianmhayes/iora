use std::str::FromStr;

use thiserror::Error;

use crate::{regexes, SemVer};

#[derive(Error, Debug)]
pub enum ConstraintParsingError {
    #[error("A name constraint is required but none was provided.")]
    EmptyNameConstraint,
    #[error("A name constraint is required but the provided constraint was malformed: '{0}'.")]
    UnrecognizedNameConstraintStructure(String),
    #[error("The version constraint was set but the value was empty.")]
    EmptyVersionConstraint,
    #[error("The version constraint was set but the value was malformed: '{0}'.")]
    UnrecognizedVersionConstraintStructure(String),
}

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

impl FromStr for NameConstraint {
    type Err = ConstraintParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ConstraintParsingError::EmptyNameConstraint);
        }
        match regexes::parse_name_constraint(s) {
            (Some(_), Some(term), Some(_)) => Ok(NameConstraint::Contains(term)),
            (None, Some(term), Some(_)) => Ok(NameConstraint::StartsWith(term)),
            (None, Some(term), None) => Ok(NameConstraint::ExactMatch(term)),
            _ => Err(ConstraintParsingError::UnrecognizedNameConstraintStructure(
                s.to_owned(),
            )),
        }
    }
}

impl ToString for NameConstraint {
    fn to_string(&self) -> String {
        match self {
            Self::ExactMatch(term) => term.to_owned(),
            Self::Contains(term) => "*".to_owned() + term + "*",
            Self::StartsWith(term) => term.to_owned() + "*",
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum VersionConstraint {
    ExactMatch(SemVer),
    MatchMajorVersionOnly(u32),
    MatchMajorAndMinorVersionOnly((u32, u32)),
    MinVersion(SemVer),
    Between((SemVer, SemVer)),
}

impl FromStr for VersionConstraint {
    type Err = ConstraintParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ConstraintParsingError::EmptyVersionConstraint);
        }
        if s.contains(',') {
            let parts: Vec<_> = s.split(',').collect();
            if parts.len() > 1 {
                if let (Ok(min_ver), Ok(max_ver)) =
                    (SemVer::from_str(parts[0]), SemVer::from_str(parts[1]))
                {
                    return Ok(VersionConstraint::Between((min_ver, max_ver)));
                }
            } else if let Ok(min_ver) = SemVer::from_str(parts[0]) {
                return Ok(VersionConstraint::MinVersion(min_ver));
            }
        }
        if let Ok(full_sem_ver) = SemVer::from_str(s) {
            return Ok(VersionConstraint::ExactMatch(full_sem_ver));
        }
        match regexes::parse_partial_version_constraint(s) {
            (Some(major), Some(minor)) => Ok(VersionConstraint::MatchMajorAndMinorVersionOnly((
                major, minor,
            ))),
            (Some(major), None) => Ok(VersionConstraint::MatchMajorVersionOnly(major)),
            _ => Err(ConstraintParsingError::UnrecognizedVersionConstraintStructure(s.to_owned())),
        }
    }
}

impl ToString for VersionConstraint {
    fn to_string(&self) -> String {
        match self {
            Self::Between((min, max)) => min.to_string() + "," + &max.to_string(),
            Self::ExactMatch(target) => target.to_string(),
            Self::MinVersion(target) => target.to_string(),
            Self::MatchMajorAndMinorVersionOnly((major, minor)) => format!("{major}.{minor}"),
            Self::MatchMajorVersionOnly(major) => format!("{major}"),
        }
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

    pub fn new_from_strings(
        name_constraint: &str,
        version_constraint: &Option<String>,
    ) -> Result<Self, ConstraintParsingError> {
        let nc = NameConstraint::from_str(name_constraint);
        let vc = version_constraint
            .as_ref()
            .map(|s| VersionConstraint::from_str(s));
        match (nc, vc) {
            (Ok(nc), Some(Ok(vc))) => Ok((nc, Some(vc)).into()),
            (Ok(nc), None) => Ok((nc, None).into()),
            (Ok(_), Some(Err(e))) => Err(e),
            (Err(e), _) => Err(e),
        }
    }
}

impl From<NameConstraint> for AssetQuery {
    fn from(nc: NameConstraint) -> Self {
        AssetQuery {
            name_constraint: nc,
            version_constraint: None,
        }
    }
}

impl From<(NameConstraint, Option<VersionConstraint>)> for AssetQuery {
    fn from(tuple: (NameConstraint, Option<VersionConstraint>)) -> Self {
        AssetQuery {
            name_constraint: tuple.0,
            version_constraint: tuple.1,
        }
    }
}

impl From<(&NameConstraint, &Option<VersionConstraint>)> for AssetQuery {
    fn from(tuple: (&NameConstraint, &Option<VersionConstraint>)) -> Self {
        AssetQuery {
            name_constraint: tuple.0.clone(),
            version_constraint: tuple.1.clone(),
        }
    }
}

impl From<(&NameConstraint, &VersionConstraint)> for AssetQuery {
    fn from(tuple: (&NameConstraint, &VersionConstraint)) -> Self {
        AssetQuery {
            name_constraint: tuple.0.clone(),
            version_constraint: Some(tuple.1.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ConstraintParsingError, NameConstraint, SemVer, VersionConstraint};
    use std::str::FromStr;

    #[test]
    fn name_exact_match() {
        assert!(NameConstraint::ExactMatch("asset.name".to_string()).matches("asset.name"));
        assert!(!NameConstraint::ExactMatch("asset.name".to_string()).matches("asset.nam"));
        assert!(!NameConstraint::ExactMatch("".to_string()).matches("asset.name"));
        assert!(NameConstraint::ExactMatch("".to_string()).matches(""));
    }

    #[test]
    fn name_starts_with() {
        assert!(NameConstraint::StartsWith("asset".to_string()).matches("asset.name"));
        assert!(NameConstraint::StartsWith("asset.".to_string()).matches("asset.name"));
        assert!(NameConstraint::StartsWith("asset.name".to_string()).matches("asset.name"));
        assert!(!NameConstraint::StartsWith("asset.name".to_string()).matches("asset"));
    }

    #[test]
    fn name_contains() {
        assert!(NameConstraint::Contains("asset".to_string()).matches("asset.name"));
        assert!(NameConstraint::Contains("name".to_string()).matches("asset.name"));
        assert!(NameConstraint::Contains("asset.nam".to_string()).matches("asset.name"));
        assert!(NameConstraint::Contains("asset.name".to_string()).matches("asset.name"));
        assert!(!NameConstraint::Contains("asset.name".to_string()).matches("asset"));
        assert!(!NameConstraint::Contains("assert".to_string()).matches("asset.name"));
    }

    #[test]
    fn version_exact_match() {
        assert!(
            VersionConstraint::ExactMatch(SemVer::from_str("34.5.6-prerelease").unwrap())
                .matches(&SemVer::from_str("34.5.6-prerelease").unwrap())
        );
        assert!(
            !VersionConstraint::ExactMatch(SemVer::from_str("34.5.6-prerelease").unwrap())
                .matches(&SemVer::from_str("34.5.6").unwrap())
        );
        assert!(
            !VersionConstraint::ExactMatch(SemVer::from_str("34.5.6").unwrap())
                .matches(&SemVer::from_str("34.5.6-prerelease").unwrap())
        );
    }

    #[test]
    fn version_min_version() {
        assert!(
            VersionConstraint::MinVersion(SemVer::from_str("34.5.6").unwrap())
                .matches(&SemVer::from_str("34.5.6").unwrap())
        );
        assert!(
            VersionConstraint::MinVersion(SemVer::from_str("34.5.6").unwrap())
                .matches(&SemVer::from_str("34.5.7").unwrap())
        );
        assert!(
            VersionConstraint::MinVersion(SemVer::from_str("34.5.6").unwrap())
                .matches(&SemVer::from_str("34.5.6-prerelease").unwrap())
        );
        assert!(
            !VersionConstraint::MinVersion(SemVer::from_str("34.5.7").unwrap())
                .matches(&SemVer::from_str("34.5.6").unwrap())
        );
    }

    #[test]
    fn version_major_version_only() {
        assert!(VersionConstraint::MatchMajorVersionOnly(34)
            .matches(&SemVer::from_str("34.5.6").unwrap()));
        assert!(!VersionConstraint::MatchMajorVersionOnly(34)
            .matches(&SemVer::from_str("35.5.6").unwrap()));
    }

    #[test]
    fn version_major_and_minor() {
        assert!(VersionConstraint::MatchMajorAndMinorVersionOnly((34, 5))
            .matches(&SemVer::from_str("34.5.6").unwrap()));
        assert!(!VersionConstraint::MatchMajorAndMinorVersionOnly((34, 5))
            .matches(&SemVer::from_str("34.6.6").unwrap()));
        assert!(!VersionConstraint::MatchMajorAndMinorVersionOnly((34, 5))
            .matches(&SemVer::from_str("35.5.6").unwrap()));
    }

    #[test]
    fn version_range() {
        let r = VersionConstraint::Between((
            SemVer::from_str("23.56.1").unwrap(),
            SemVer::from_str("24.0.0").unwrap(),
        ));
        assert!(r.matches(&SemVer::from_str("23.56.1").unwrap()));
        assert!(r.matches(&SemVer::from_str("23.56.2").unwrap()));
        assert!(!r.matches(&SemVer::from_str("23.56.0").unwrap()));
        assert!(!r.matches(&SemVer::from_str("23.6.1").unwrap()));
        assert!(!r.matches(&SemVer::from_str("24.0.0").unwrap()));
    }

    #[test]
    fn name_constraint_from_str() {
        match NameConstraint::from_str("*a*") {
            Ok(NameConstraint::Contains(term)) => assert_eq!(term, "a"),
            r => panic!("Unexpected result: {:?}", r),
        }

        match NameConstraint::from_str("b*") {
            Ok(NameConstraint::StartsWith(term)) => assert_eq!(term, "b"),
            r => panic!("Unexpected result: {:?}", r),
        }

        match NameConstraint::from_str("c") {
            Ok(NameConstraint::ExactMatch(term)) => assert_eq!(term, "c"),
            r => panic!("Unexpected result: {:?}", r),
        }

        match NameConstraint::from_str("*b") {
            Err(ConstraintParsingError::UnrecognizedNameConstraintStructure(_)) => {}
            r => panic!("Unexpected result: {:?}", r),
        }

        match NameConstraint::from_str("**") {
            Err(ConstraintParsingError::UnrecognizedNameConstraintStructure(_)) => {}
            r => panic!("Unexpected result: {:?}", r),
        }

        match NameConstraint::from_str("") {
            Err(ConstraintParsingError::EmptyNameConstraint) => {}
            r => panic!("Unexpected result: {:?}", r),
        }
    }
}
