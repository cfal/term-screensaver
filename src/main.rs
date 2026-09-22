use std::process::ExitCode;

use ascii_screensaver::{app, config::Command, effects::EffectKind};

fn main() -> ExitCode {
    match ascii_screensaver::config::parse(std::env::args().skip(1)) {
        Ok(Command::Help) => {
            print!("{}", ascii_screensaver::config::HELP);
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("ascii-screensaver {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(Command::List) => {
            for effect in EffectKind::ALL {
                println!("{}", effect.name());
            }
            ExitCode::SUCCESS
        }
        Ok(Command::Run(config)) => match app::run(config) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("ascii-screensaver: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("ascii-screensaver: {error}\n\nTry --help for usage.");
            ExitCode::from(2)
        }
    }
}
