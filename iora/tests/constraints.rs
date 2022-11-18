use std::str::FromStr;

use iora::{NameConstraint, SemVer, VersionConstraint, ConstraintParsingError};

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
    assert!(
        VersionConstraint::MatchMajorVersionOnly(34).matches(&SemVer::from_str("34.5.6").unwrap())
    );
    assert!(
        !VersionConstraint::MatchMajorVersionOnly(34).matches(&SemVer::from_str("35.5.6").unwrap())
    );
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
        r => panic!("Unexpected result: {:?}", r)
    }

    match NameConstraint::from_str("b*") {
        Ok(NameConstraint::StartsWith(term)) => assert_eq!(term, "b"),
        r => panic!("Unexpected result: {:?}", r)
    }

    match NameConstraint::from_str("c") {
        Ok(NameConstraint::ExactMatch(term)) => assert_eq!(term, "c"),
        r => panic!("Unexpected result: {:?}", r)
    }

    match NameConstraint::from_str("*b") {
        Err(ConstraintParsingError::UnrecognizedVersionConstraintStructure) => {},
        r => panic!("Unexpected result: {:?}", r)
    }

    match NameConstraint::from_str("**") {
        Err(ConstraintParsingError::UnrecognizedVersionConstraintStructure) => {},
        r => panic!("Unexpected result: {:?}", r)
    }

    match NameConstraint::from_str("") {
        Err(ConstraintParsingError::UnrecognizedVersionConstraintStructure) => {},
        r => panic!("Unexpected result: {:?}", r)
    }
}
