use crate::{AssetQuery, SemVer};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AssetDescriptor {
    pub name: String,
    pub version: SemVer,
    pub content_hash: String,
}

impl AssetDescriptor {
    pub fn new(name: &str, version: SemVer, content_hash: &str) -> Self {
        AssetDescriptor {
            name: name.to_string(),
            version,
            content_hash: content_hash.to_string(),
        }
    }

    pub fn matches_query(&self, query: &AssetQuery) -> bool {
        let name_match = query.name_constraint.matches(&self.name);
        match &query.version_constraint {
            Some(vc) => name_match && vc.matches(&self.version),
            _ => name_match,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{AssetDescriptor, NameConstraint, SemVer, VersionConstraint};
    use std::str::FromStr;

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
}
