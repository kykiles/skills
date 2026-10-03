//! Загрузка конфигурации сервиса relay.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

/// Настройки сервиса.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub listen: String,
    pub port: u16,
    pub timeout: Duration,
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1".to_string(),
            port: 8080,
            timeout: Duration::from_secs(30),
            verbose: false,
        }
    }
}

/// Ошибка загрузки конфигурации.
#[derive(Debug)]
pub enum ConfigError {
    /// Файл не прочитан.
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Строка не имеет вида `ключ = значение`.
    Syntax { line: usize },
    /// Ключ не поддерживается.
    UnknownKey { line: usize, key: String },
    /// Значение ключа не разбирается.
    Value {
        line: usize,
        key: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, .. } => write!(f, "не удалось прочитать {}", path.display()),
            Self::Syntax { line } => write!(f, "строка {line}: ожидается `ключ = значение`"),
            Self::UnknownKey { line, key } => write!(f, "строка {line}: неизвестный ключ `{key}`"),
            Self::Value { line, key, .. } => {
                write!(f, "строка {line}: некорректное значение ключа `{key}`")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Value { source, .. } => Some(source.as_ref()),
            Self::Syntax { .. } | Self::UnknownKey { .. } => None,
        }
    }
}

/// Читает конфигурацию из файла.
///
/// # Errors
///
/// Возвращает [`ConfigError::Read`], если файл не прочитан, и ошибки [`parse`].
pub fn load(path: &Path) -> Result<Config, ConfigError> {
    let text = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    parse(&text)
}

/// Разбирает текст конфигурации. Пустые строки и строки с `#` пропускаются,
/// не заданные ключи берутся из [`Config::default`].
///
/// # Errors
///
/// Возвращает [`ConfigError::Syntax`] для строки без `=`,
/// [`ConfigError::UnknownKey`] для неизвестного ключа и
/// [`ConfigError::Value`] для значения, которое не разбирается.
pub fn parse(text: &str) -> Result<Config, ConfigError> {
    let mut config = Config::default();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }
        let Some((key, value)) = raw.split_once('=') else {
            return Err(ConfigError::Syntax { line });
        };
        let (key, value) = (key.trim(), value.trim());
        match key {
            "listen" => config.listen = value.to_string(),
            "port" => config.port = parse_value(line, key, value)?,
            "timeout_ms" => config.timeout = Duration::from_millis(parse_value(line, key, value)?),
            "verbose" => config.verbose = parse_value(line, key, value)?,
            _ => {
                return Err(ConfigError::UnknownKey {
                    line,
                    key: key.to_string(),
                })
            }
        }
    }
    Ok(config)
}

fn parse_value<T>(line: usize, key: &str, value: &str) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    value.parse().map_err(|source: T::Err| ConfigError::Value {
        line,
        key: key.to_string(),
        source: Box::new(source),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_keys() {
        let text = "# test\nlisten = 0.0.0.0\nport = 9000\n\ntimeout_ms = 1500\nverbose = true\n";
        assert_eq!(
            parse(text).unwrap(),
            Config {
                listen: "0.0.0.0".to_string(),
                port: 9000,
                timeout: Duration::from_millis(1500),
                verbose: true,
            }
        );
    }

    #[test]
    fn rejects_unknown_key() {
        assert!(matches!(
            parse("port = 1\nhost = x\n"),
            Err(ConfigError::UnknownKey { line: 2, .. })
        ));
    }
}
