use crate::command::CommandSerializableState;
use crate::pane::{PaneKey, PaneManager};
use serde::{de, Deserialize, Serialize};
use serde_with::{serde_as, DeserializeAs, SerializeAs};
use slotmap::{Key, KeyData};
use std::collections::HashMap;

pub struct PaneKeyAsString;

impl SerializeAs<PaneKey> for PaneKeyAsString {
    fn serialize_as<S>(source: &PaneKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&source.data().as_ffi().to_string())
    }
}

impl<'de> DeserializeAs<'de, PaneKey> for PaneKeyAsString {
    fn deserialize_as<D>(deserializer: D) -> Result<PaneKey, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let key_data = s.parse::<u64>().map_err(de::Error::custom)?;
        Ok(PaneKey::from(KeyData::from_ffi(key_data)))
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct SessionState {
    pub pane_manager: PaneManager,
    #[serde_as(as = "HashMap<PaneKeyAsString, _>")]
    pub tasks: HashMap<PaneKey, CommandSerializableState>,
}
