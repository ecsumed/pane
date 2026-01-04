use std::fmt::Display;
pub trait ToSettingsString {
    fn to_settings_string(&self) -> String;
}

impl<T: Display> ToSettingsString for T {
    fn to_settings_string(&self) -> String {
        self.to_string()
    }
}

#[macro_export]
macro_rules! settings_line {
    ($p:expr, $width:expr, $($label:expr => $value:expr),* $(,)?) => {
        vec![
            $(
                Line::from(vec![
                    Span::styled(format!("{:<width$} -> ", $label, width = $width), $p.meta_secondary),
                    Span::styled($value.to_settings_string(), $p.meta_value),
                ]),
            )*
        ]
    };

    ($p:expr, $width:expr, @list $items:expr) => {
        $items.into_iter().map(|(key, action)| {
            Line::from(vec![
                Span::styled(format!("{:<width$} -> ", key.to_settings_string(), width = $width), $p.meta_secondary),
                Span::styled(format!("{:?}", action), $p.meta_value),
            ])
        }).collect::<Vec<Line>>()
    };
}
