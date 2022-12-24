use crate::{AssetQuery, SemVer};
use serde::{
    de::{Deserializer, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AssetLocator {
    #[serde(serialize_with = "serialize_uri_to_string")]
    #[serde(deserialize_with = "deserialize_uri_from_string")]
    pub url: reqwest::Url,
}

pub fn serialize_uri_to_string<S>(uri: &reqwest::Url, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(uri.as_str())
}

pub fn deserialize_uri_from_string<'de, D>(d: D) -> Result<reqwest::Url, D::Error>
where
    D: Deserializer<'de>,
{
    struct UriVisitor {}
    impl<'de> Visitor<'de> for UriVisitor {
        type Value = reqwest::Url;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("String containing a well formatted Uri.")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match reqwest::Url::from_str(v) {
                Ok(u) => Ok(u),
                Err(e) => Err(E::custom(e.to_string())),
            }
        }
    }
    d.deserialize_str(UriVisitor {})
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AssetDescriptor {
    pub name: String,
    pub version: SemVer,
    pub content_hash: String,
    pub size: usize,
    pub locators: Vec<AssetLocator>,
}

impl AssetDescriptor {
    pub fn new(
        name: &str,
        version: SemVer,
        content_hash: &str,
        size: usize,
        locators: Vec<AssetLocator>,
    ) -> Self {
        AssetDescriptor {
            name: name.to_string(),
            version,
            content_hash: content_hash.to_string(),
            size,
            locators,
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
        let ad = AssetDescriptor::new(
            "asset.name",
            SemVer::from_str("23.45.678").unwrap(),
            "content_hash",
            0,
            vec![],
        );
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
