use chrono::NaiveDateTime;
use serde::{self, Deserialize, Deserializer, Serializer};

use crate::logging::debug;

const FORMAT: &str = "%a %e %b %Y %H:%M:%S";

pub mod naivedatetime_format {
    use super::*;

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let s = date.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(s.trim(), FORMAT).map_err(serde::de::Error::custom)
    }
}