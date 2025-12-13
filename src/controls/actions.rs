use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Confirm,
    CyclePanes,
    EnterCmdMode,
    EnterDisplaySelectMode,
    EnterSessionLoadMode,
    EnterSessionSaveMode,
    Escape,
    IntervalDecrease,
    IntervalIncrease,
    KillPane,
    LoadLatestSession,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveUp,
    PaneDecreaseHorizontal,
    PaneDecreaseVertical,
    PaneIncreaseHorizontal,
    PaneIncreaseVertical,
    Pause,
    Quit,
    Resume,
    SaveSession,
    SplitHorizontal,
    SplitVertical,
    TabComplete,
}