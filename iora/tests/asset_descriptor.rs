use std::str::FromStr;

use iora::{AssetDescriptor, NameConstraint, SemVer, VersionConstraint};

#[test]
fn asset_descriptor_match() {
    let ad = AssetDescriptor {
        name: "asset.name".to_string(),
        version: SemVer::from_str("23.45.678").unwrap(),
        content_hash: "content_hash".to_string(),
    };
    assert!(ad.matches_query(&NameConstraint::StartsWith("asset".to_string()).into()));
    assert!(!ad.matches_query(&NameConstraint::StartsWith("assert".to_string()).into()));
    assert!(ad.matches_query(
        &(
            NameConstraint::StartsWith("asset".to_string()),
            Some(VersionConstraint::MatchMajorVersionOnly(23))
        )
            .into()
    ));
    assert!(!ad.matches_query(
        &(
            NameConstraint::StartsWith("asset".to_string()),
            Some(VersionConstraint::MatchMajorVersionOnly(24))
        )
            .into()
    ));
    assert!(!ad.matches_query(
        &(
            NameConstraint::StartsWith("assert".to_string()),
            Some(VersionConstraint::MatchMajorVersionOnly(23))
        )
            .into()
    ));
}
