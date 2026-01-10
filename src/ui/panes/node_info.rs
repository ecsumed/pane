use std::borrow::Cow;

use humantime::format_duration;

use crate::{
    command::Command, config::AppConfig, ui::panes::history_meter::generate_history_meter_string,
};

pub struct NodeInfo<'a> {
    pub is_active: bool,
    pub exec_str: Cow<'a, str>,
    pub interval_secs_str: Cow<'a, str>,
    pub last_exec_time: Cow<'a, str>,
    pub duration: Cow<'a, str>,
    pub state_str: Cow<'a, str>,
    pub display_type_str: Cow<'a, str>,
    pub history_limit: Cow<'a, str>,
}

impl<'a> NodeInfo<'a> {
    pub fn with_command(config: &AppConfig, is_active: bool, c: &'a Command) -> Self {
        let last_exec_time = c
            .last_output()
            .map(|out| out.time.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let duration = c
            .last_output()
            .map(|out| format_duration(out.duration).to_string()) // Assuming this returns String
            .unwrap_or_else(|| "N/A".to_string());

        let meter = generate_history_meter_string(c.output_history.len(), config.max_history);

        NodeInfo {
            is_active,
            exec_str: Cow::Borrowed(&c.exec),
            interval_secs_str: Cow::Owned(format!("{:?}", c.interval)),
            last_exec_time: Cow::Owned(last_exec_time),
            duration: Cow::Owned(duration),
            state_str: Cow::Owned(c.state.to_string()),
            display_type_str: Cow::Owned(format!("{:?}", c.display_type)),
            history_limit: Cow::Owned(meter),
        }
    }

    pub fn no_command(is_active: bool) -> Self {
        Self {
            is_active: is_active,
            exec_str: Cow::Borrowed("N/A"),
            interval_secs_str: Cow::Borrowed("0s"),
            last_exec_time: Cow::Borrowed("N/A"),
            duration: Cow::Borrowed("0s"),
            state_str: Cow::Borrowed("N/A"),
            display_type_str: Cow::Borrowed("N/A"),
            history_limit: Cow::Borrowed(""),
        }
    }
}
