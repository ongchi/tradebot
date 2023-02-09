use serde::{
    de::{Error, Unexpected, Visitor},
    Deserializer,
};
use std::fmt;

struct BoolVisitor;

impl<'de> Visitor<'de> for BoolVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("bool from value, string of true/false, or integer of 0/1")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            0 => Ok(false),
            1 => Ok(true),
            o => Err(Error::invalid_value(Unexpected::Signed(o), &"0 or 1")),
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            "false" => Ok(false),
            "true" => Ok(true),
            o => Err(Error::invalid_value(Unexpected::Other(o), &"false or true")),
        }
    }
}

pub fn bool_from_val<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(BoolVisitor)
}

pub fn bool_from_val_option<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer
        .deserialize_any(BoolVisitor)
        .map(Some)
        .or(Ok(None))
}

#[cfg(test)]
mod tests {
    use super::{bool_from_val, bool_from_val_option};
    use serde::{Deserialize, Serialize};
    use serde_json::error::Category;

    #[derive(Serialize, Deserialize, Debug)]
    struct S {
        #[serde(deserialize_with = "bool_from_val")]
        b: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct OptS {
        #[serde(deserialize_with = "bool_from_val_option")]
        b: Option<bool>,
    }

    #[test]
    fn de_bool() {
        let s1: S = serde_json::from_str("{\"b\":true}").unwrap();
        let s2: S = serde_json::from_str("{\"b\":1}").unwrap();
        let s3: S = serde_json::from_str("{\"b\":false}").unwrap();
        let s4: S = serde_json::from_str("{\"b\":0}").unwrap();

        assert_eq!(s1.b, true);
        assert_eq!(s2.b, true);
        assert_eq!(s3.b, false);
        assert_eq!(s4.b, false);
    }

    #[test]
    fn de_invalid_val() {
        let e1 = serde_json::from_str::<S>("{b:foo}").unwrap_err();
        let e2 = serde_json::from_str::<S>("{b:123}").unwrap_err();
        let e3 = serde_json::from_str::<OptS>("{b:foo}").unwrap_err();
        let e4 = serde_json::from_str::<OptS>("{b:123}").unwrap_err();

        assert_eq!(e1.classify(), Category::Syntax);
        assert_eq!(e2.classify(), Category::Syntax);
        assert_eq!(e3.classify(), Category::Syntax);
        assert_eq!(e4.classify(), Category::Syntax);
    }

    #[test]
    fn de_bool_opt_with_val() {
        let s1: OptS = serde_json::from_str("{\"b\":true}").unwrap();
        let s2: OptS = serde_json::from_str("{\"b\":1}").unwrap();
        let s3: OptS = serde_json::from_str("{\"b\":false}").unwrap();
        let s4: OptS = serde_json::from_str("{\"b\":0}").unwrap();

        assert_eq!(s1.b, Some(true));
        assert_eq!(s2.b, Some(true));
        assert_eq!(s3.b, Some(false));
        assert_eq!(s4.b, Some(false));
    }

    #[test]
    fn de_bool_opt_without_val() {
        let s1: OptS = serde_json::from_str("{\"b\":}").unwrap();

        assert_eq!(s1.b, None);
    }
}
