pub use self::dispatcher::handle_event;

pub mod actions;
mod dispatcher;
mod display_select_mode;
mod edit_mode;
mod normal_mode;
mod session_load_mode;
mod session_save_mode;
