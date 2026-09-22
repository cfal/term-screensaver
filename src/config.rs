use std::{fmt, time::Duration};

use crate::effects::EffectKind;

pub const HELP: &str = "\
ascii-screensaver - procedural terminal animation\n\n\
Usage: ascii-screensaver [OPTIONS]\n\n\
Options:\n\
  --fps <1-60>                 Frame rate [default: 30]\n\
  --interval <SECONDS>         Seconds between scenes [default: 20]\n\
  --seed <INTEGER>             Reproducible random seed\n\
  --effect <NAME>              Stay within one animation family\n\
  --color <auto|always|never>  Color policy [default: auto]\n\
  --ascii                      Use printable ASCII glyphs only\n\
  --list                       List animation families\n\
  -h, --help                   Show this help\n\
  -V, --version                Show version\n\n\
Controls:\n\
  Space/Right  next scene      r  regenerate scene\n\
  p            pause           m  toggle monochrome\n\
  ?            toggle help     q/Esc/Ctrl-C  quit\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub fps: u32,
    pub interval: Duration,
    pub seed: Option<u64>,
    pub effect: Option<EffectKind>,
    pub color: ColorChoice,
    pub ascii: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fps: 30,
            interval: Duration::from_secs(20),
            seed: None,
            effect: None,
            color: ColorChoice::Auto,
            ascii: false,
        }
    }
}

pub enum Command {
    Run(Config),
    Help,
    Version,
    List,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Command, ParseError> {
    let mut config = Config::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "-V" | "--version" => return Ok(Command::Version),
            "--list" => return Ok(Command::List),
            "--ascii" => config.ascii = true,
            "--fps" => {
                let value = value_for(&arg, &mut args)?;
                config.fps = value
                    .parse::<u32>()
                    .ok()
                    .filter(|value| (1..=60).contains(value))
                    .ok_or_else(|| ParseError("--fps must be an integer from 1 to 60".into()))?;
            }
            "--interval" => {
                let value = value_for(&arg, &mut args)?;
                let seconds = value
                    .parse::<f64>()
                    .map_err(|_| ParseError("--interval must be a number from 1 to 3600".into()))?;
                if !seconds.is_finite() || !(1.0..=3600.0).contains(&seconds) {
                    return Err(ParseError(
                        "--interval must be a number from 1 to 3600".into(),
                    ));
                }
                config.interval = Duration::from_secs_f64(seconds);
            }
            "--seed" => {
                let value = value_for(&arg, &mut args)?;
                config.seed = Some(
                    value
                        .parse()
                        .map_err(|_| ParseError("--seed must be an unsigned integer".into()))?,
                );
            }
            "--effect" => {
                let value = value_for(&arg, &mut args)?;
                config.effect = Some(EffectKind::from_name(&value).ok_or_else(|| {
                    ParseError(format!(
                        "unknown effect '{value}'; use --list to see available effects"
                    ))
                })?);
            }
            "--color" => {
                let value = value_for(&arg, &mut args)?;
                config.color = match value.as_str() {
                    "auto" => ColorChoice::Auto,
                    "always" => ColorChoice::Always,
                    "never" => ColorChoice::Never,
                    _ => {
                        return Err(ParseError(
                            "--color must be one of: auto, always, never".into(),
                        ));
                    }
                };
            }
            _ if arg.starts_with('-') => {
                return Err(ParseError(format!("unknown option '{arg}'")));
            }
            _ => return Err(ParseError(format!("unexpected argument '{arg}'"))),
        }
    }

    Ok(Command::Run(config))
}

fn value_for(option: &str, args: &mut impl Iterator<Item = String>) -> Result<String, ParseError> {
    args.next()
        .ok_or_else(|| ParseError(format!("{option} requires a value")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings<'a>(args: &'a [&'a str]) -> impl Iterator<Item = String> + 'a {
        args.iter().map(|value| (*value).to_owned())
    }

    #[test]
    fn parses_runtime_options() {
        let Command::Run(config) = parse(strings(&[
            "--fps",
            "24",
            "--interval",
            "7.5",
            "--seed",
            "42",
            "--effect",
            "plasma",
            "--color",
            "never",
            "--ascii",
        ]))
        .unwrap() else {
            panic!("expected run command");
        };

        assert_eq!(config.fps, 24);
        assert_eq!(config.interval, Duration::from_secs_f64(7.5));
        assert_eq!(config.seed, Some(42));
        assert_eq!(config.effect, Some(EffectKind::Plasma));
        assert_eq!(config.color, ColorChoice::Never);
        assert!(config.ascii);
    }

    #[test]
    fn rejects_invalid_ranges() {
        assert_eq!(
            parse(strings(&["--fps", "0"])).err().unwrap().to_string(),
            "--fps must be an integer from 1 to 60"
        );
        assert!(parse(strings(&["--interval", "NaN"])).is_err());
    }
}
