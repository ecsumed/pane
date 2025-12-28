use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum KeyMode {
    Global,
    Normal,
    CmdEdit,
    SessionLoad,
    SessionSave,
    Observe,
    DisplayTypeSelect,
    Help,
}

impl fmt::Display for KeyMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KeyMode::Global => write!(f, "Global"),
            KeyMode::Normal => write!(f, "Normal"),
            KeyMode::CmdEdit => write!(f, "CmdEdit"),
            KeyMode::SessionLoad => write!(f, "SessionLoad"),
            KeyMode::SessionSave => write!(f, "SessionSave"),
            KeyMode::Observe => write!(f, "Observe"),
            KeyMode::DisplayTypeSelect => write!(f, "DisplayTypeSelect"),
            KeyMode::Help => write!(f, "Help"),
        }
    }
}
