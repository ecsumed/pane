pub mod direction_serialization {
    use ratatui::layout::Direction;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(direction: &Direction, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match direction {
            Direction::Vertical => "Vertical",
            Direction::Horizontal => "Horizontal",
        };
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Direction, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Vertical" => Ok(Direction::Vertical),
            "Horizontal" => Ok(Direction::Horizontal),
            _ => Err(serde::de::Error::custom("Invalid Direction string")),
        }
    }
}