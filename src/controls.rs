pub use self::dispatcher::handle_event;

mod dispatcher;
mod normal_mode;
mod edit_mode;
mod session_load_mode;
mod session_save_mode;