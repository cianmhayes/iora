use std::str::FromStr;

use iora::{SemVer, SemVerParseEror};

#[test]
fn from_str() {
    assert_eq!(
        SemVer::from_str("3.45.6").unwrap(),
        SemVer {
            major: 3,
            minor: 45,
            patch: 6,
            prerelease: None,
            buildmetadata: None
        }
    );

    assert_eq!(
        SemVer::from_str("3.45.6-beta").unwrap(),
        SemVer {
            major: 3,
            minor: 45,
            patch: 6,
            prerelease: Some("beta".to_string()),
            buildmetadata: None
        }
    );

    assert_eq!(
        SemVer::from_str("3.45.6+build123").unwrap(),
        SemVer {
            major: 3,
            minor: 45,
            patch: 6,
            prerelease: None,
            buildmetadata: Some("build123".to_string())
        }
    );

    assert_eq!(
        SemVer::from_str("3.45.6-beta+build123").unwrap(),
        SemVer {
            major: 3,
            minor: 45,
            patch: 6,
            prerelease: Some("beta".to_string()),
            buildmetadata: Some("build123".to_string())
        }
    );

    assert!(match SemVer::from_str("1.0") {
        Err(SemVerParseEror::UnparsableSemVer) => true,
        _ => false,
    });
}

#[test]
fn comparison() {
    assert!(SemVer::from_str("3.45.6").unwrap() > SemVer::from_str("3.5.6").unwrap());
    assert!(SemVer::from_str("3.45.6").unwrap() > SemVer::from_str("3.5.6-prerelease").unwrap());
    assert!(SemVer::from_str("3.45.7").unwrap() > SemVer::from_str("3.5.6").unwrap());
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
