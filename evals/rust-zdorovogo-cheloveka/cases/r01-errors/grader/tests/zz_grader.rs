// Скрытые тесты оценщика r01-errors.
use std::error::Error;
use std::num::ParseIntError;
use std::process::Command;
use std::str::ParseBoolError;

// Конфиг, в котором ключ `key` со значением `value` стоит в строке 12.
fn config_with(key: &str, value: &str) -> String {
    let mut text = String::from("# relay\n");
    for _ in 0..9 {
        text.push_str("# padding\n");
    }
    text.push_str("listen = 0.0.0.0\n");
    text.push_str(&format!("{key} = {value}\n"));
    text.push_str("verbose = false\n");
    text
}

fn chain<'a>(error: &'a (dyn Error + 'static)) -> Vec<&'a (dyn Error + 'static)> {
    let mut out = vec![error];
    let mut cur = error.source();
    while let Some(e) = cur {
        out.push(e);
        cur = e.source();
    }
    out
}

fn chain_text(error: &(dyn Error + 'static)) -> String {
    chain(error)
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(": ")
}

fn check_bad_value<T: Error + 'static>(key: &str, value: &str) {
    let text = config_with(key, value);
    let err = match relay::parse(&text) {
        Ok(config) => panic!("{key} = {value}: ожидалась ошибка, получено {config:?}"),
        Err(err) => err,
    };
    let err: &(dyn Error + 'static) = &err;
    let full = chain_text(err);
    assert!(
        chain(err).iter().any(|e| e.downcast_ref::<T>().is_some()),
        "{key} = {value}: исходная ошибка {} потеряна в цепочке source(): {full}",
        std::any::type_name::<T>()
    );
    assert!(
        full.contains(key),
        "{key} = {value}: в сообщении нет ключа: {full}"
    );
    assert!(
        full.contains("12"),
        "{key} = {value}: в сообщении нет номера строки 12: {full}"
    );
}

#[test]
fn grader_bad_port() {
    check_bad_value::<ParseIntError>("port", "80o");
    check_bad_value::<ParseIntError>("port", "70000");
    check_bad_value::<ParseIntError>("port", "");
}

#[test]
fn grader_bad_timeout() {
    check_bad_value::<ParseIntError>("timeout_ms", "-5");
    check_bad_value::<ParseIntError>("timeout_ms", "1.5");
}

#[test]
fn grader_bad_verbose() {
    check_bad_value::<ParseBoolError>("verbose", "yes");
}

#[test]
fn grader_valid_config() {
    let config = relay::parse(&config_with("port", "9000")).unwrap();
    assert_eq!(
        config,
        relay::Config {
            listen: "0.0.0.0".to_string(),
            port: 9000,
            timeout: std::time::Duration::from_secs(30),
            verbose: false,
        }
    );
}

fn run_binary(name: &str, contents: Option<&str>) -> (Option<i32>, String) {
    let path =
        std::env::temp_dir().join(format!("relay-grader-{}-{name}.conf", std::process::id()));
    if let Some(contents) = contents {
        std::fs::write(&path, contents).unwrap();
    }
    let out = Command::new(env!("CARGO_BIN_EXE_relay"))
        .arg(&path)
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&path);
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn grader_binary_reports_bad_value() {
    let (code, stderr) = run_binary("bad", Some(&config_with("port", "80o")));
    assert!(
        !stderr.contains("panicked"),
        "паника вместо ошибки:\n{stderr}"
    );
    assert_eq!(code, Some(2), "код выхода; stderr:\n{stderr}");
    let parse_error = "invalid digit found in string";
    assert_eq!(
        stderr.matches(parse_error).count(),
        1,
        "текст исходной ошибки должен быть в выводе ровно один раз:\n{stderr}"
    );
    assert!(
        stderr.contains("port") && stderr.contains("12"),
        "нет ключа или строки:\n{stderr}"
    );
}

#[test]
fn grader_binary_missing_file() {
    let (code, stderr) = run_binary("missing", None);
    assert_eq!(code, Some(2), "код выхода; stderr:\n{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}
