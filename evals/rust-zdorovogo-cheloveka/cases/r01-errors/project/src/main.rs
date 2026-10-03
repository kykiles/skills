use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let path = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("relay.conf"), PathBuf::from);
    match relay::load(&path) {
        Ok(config) => {
            println!("{config:?}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprint!("ошибка: {error}");
            let mut cause = error.source();
            while let Some(current) = cause {
                eprint!(": {current}");
                cause = current.source();
            }
            eprintln!();
            ExitCode::from(2)
        }
    }
}
