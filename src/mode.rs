use tui_input::Input;

#[derive(Debug, Default)]
pub struct InputState {
    pub input: Input,
    pub mode: InputMode,
}

#[derive(Debug, PartialEq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}
