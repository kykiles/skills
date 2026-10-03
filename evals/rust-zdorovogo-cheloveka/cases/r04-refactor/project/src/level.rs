use std::fmt;

/// Уровень важности события.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    Info,
    Warning,
    Critical,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Level::Info => "Info",
            Level::Warning => "Warning",
            Level::Critical => "Critical",
        })
    }
}
