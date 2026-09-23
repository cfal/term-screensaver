use std::{
    io::{self, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};
use signal_hook::consts::signal::{SIGINT, SIGTERM};

use crate::{
    canvas::{Canvas, Rgb, Style},
    config::{ColorChoice, Config},
    effects::{Animation, EffectKind, FrameContext, random_seed},
    terminal::{Renderer, TerminalSession},
};

pub fn run(config: Config) -> io::Result<()> {
    let shutdown = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGINT, Arc::clone(&shutdown))?;
    signal_hook::flag::register(SIGTERM, Arc::clone(&shutdown))?;
    let _terminal = TerminalSession::enter()?;

    let mut app = App::new(config);
    let mut renderer = Renderer::default();
    let mut canvas = Canvas::new(0, 0);
    let stdout = io::stdout();
    let mut stdout = io::BufWriter::new(stdout.lock());
    let frame_period = Duration::from_secs_f64(1.0 / f64::from(app.config.fps));
    let mut next_frame = Instant::now();

    while !app.should_quit && !shutdown.load(Ordering::Relaxed) {
        let now = Instant::now();
        let timeout = next_frame.saturating_duration_since(now);
        if event::poll(timeout)? {
            for _ in 0..32 {
                let event = event::read()?;
                if app.handle_event(event) {
                    renderer.invalidate();
                }
                if !event::poll(Duration::ZERO)? {
                    break;
                }
            }
        }

        let now = Instant::now();
        if now < next_frame {
            continue;
        }
        app.advance(now);
        let (terminal_width, terminal_height) = crossterm::terminal::size()?;
        let width = terminal_width.saturating_sub(1);
        canvas.resize(width, terminal_height);
        canvas.clear();
        app.render(&mut canvas);
        renderer.render(&mut stdout, &canvas, app.is_colored())?;

        next_frame += frame_period;
        while next_frame <= now {
            next_frame += frame_period;
        }
    }
    stdout.flush()
}

pub struct App {
    config: Config,
    rng: StdRng,
    bag: Vec<EffectKind>,
    current_kind: EffectKind,
    animation: Box<dyn Animation>,
    scene_seed: u64,
    scene_elapsed: Duration,
    last_advance: Instant,
    paused: bool,
    force_monochrome: bool,
    scene_colored: bool,
    show_help: bool,
    should_quit: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        let seed = config.seed.unwrap_or_else(random_seed);
        let mut rng = StdRng::seed_from_u64(seed);
        let mut bag = config.profile().effects().to_vec();
        bag.shuffle(&mut rng);
        let current_kind = config
            .effect
            .unwrap_or_else(|| bag.pop().unwrap_or(EffectKind::Plasma));
        let scene_seed = rng.r#gen();
        let scene_colored = choose_color(config.color, &mut rng);
        Self {
            config,
            rng,
            bag,
            current_kind,
            animation: current_kind.create(scene_seed),
            scene_seed,
            scene_elapsed: Duration::ZERO,
            last_advance: Instant::now(),
            paused: false,
            force_monochrome: false,
            scene_colored,
            show_help: false,
            should_quit: false,
        }
    }

    fn advance(&mut self, now: Instant) {
        let delta = now.saturating_duration_since(self.last_advance);
        self.last_advance = now;
        if self.paused {
            return;
        }
        self.scene_elapsed += delta;
        if self.scene_elapsed >= self.config.interval {
            self.next_scene();
        }
    }

    fn next_scene(&mut self) {
        let kind = self.next_kind();
        self.replace_scene(kind);
    }

    fn regenerate(&mut self) {
        self.replace_scene(self.current_kind);
    }

    fn replace_scene(&mut self, kind: EffectKind) {
        self.current_kind = kind;
        self.scene_seed = self.rng.r#gen();
        self.scene_colored = choose_color(self.config.color, &mut self.rng);
        self.animation = kind.create(self.scene_seed);
        self.scene_elapsed = Duration::ZERO;
        self.last_advance = Instant::now();
    }

    fn next_kind(&mut self) -> EffectKind {
        if let Some(kind) = self.config.effect {
            return kind;
        }
        if self.bag.is_empty() {
            self.bag.extend_from_slice(self.config.profile().effects());
            self.bag.shuffle(&mut self.rng);
            if self.bag.len() > 1 && self.bag.last() == Some(&self.current_kind) {
                let last = self.bag.len() - 1;
                self.bag.swap(0, last);
            }
        }
        self.bag.pop().unwrap_or(EffectKind::Plasma)
    }

    fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::Resize(_, _) => true,
            Event::Key(key) if should_handle_key(key) => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.should_quit = true;
                    }
                    KeyCode::Char(' ') | KeyCode::Right => self.next_scene(),
                    KeyCode::Char('r') => self.regenerate(),
                    KeyCode::Char('p') => {
                        self.paused = !self.paused;
                        self.last_advance = Instant::now();
                    }
                    KeyCode::Char('m') => self.force_monochrome = !self.force_monochrome,
                    KeyCode::Char('?') => self.show_help = !self.show_help,
                    _ => return false,
                }
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, canvas: &mut Canvas) {
        let frame = FrameContext {
            scene_seconds: self.scene_elapsed.as_secs_f64(),
            wall_time: Local::now(),
            ascii: self.config.ascii,
            colored: self.is_colored(),
        };
        self.animation.render(&frame, canvas);
        if self.show_help {
            draw_help(canvas, self.is_colored());
        }
    }

    fn is_colored(&self) -> bool {
        self.scene_colored && !self.force_monochrome
    }
}

fn choose_color(choice: ColorChoice, rng: &mut StdRng) -> bool {
    match choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => std::env::var_os("NO_COLOR").is_none() && rng.gen_bool(0.72),
    }
}

fn should_handle_key(key: KeyEvent) -> bool {
    matches!(key.kind, KeyEventKind::Press)
}

fn draw_help(canvas: &mut Canvas, colored: bool) {
    const LINES: &[&str] = &[
        "TERM SCREENSAVER",
        "",
        "Space / Right  next scene",
        "r              regenerate",
        "p              pause",
        "m              toggle monochrome",
        "?              close help",
        "q / Esc        quit",
    ];
    let width = LINES.iter().map(|line| line.len()).max().unwrap_or(0) as u16 + 4;
    let height = LINES.len() as u16 + 2;
    let left = i32::from(canvas.width().saturating_sub(width) / 2);
    let top = i32::from(canvas.height().saturating_sub(height) / 2);
    let color = colored.then_some(Rgb::new(120, 235, 196));
    let style = Style {
        foreground: color,
        bold: false,
        dim: false,
    };
    canvas.fill_rect(left, top, width, height, Style::default());
    for (row, line) in LINES.iter().enumerate() {
        let x = left + (i32::from(width) - line.len() as i32) / 2;
        canvas.text(x, top + row as i32 + 1, line, style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;

    #[test]
    fn pinned_effect_regenerates_without_changing_kind() {
        let app_config = Config {
            seed: Some(5),
            effect: Some(EffectKind::Plasma),
            ..Config::default()
        };
        let mut app = App::new(app_config);
        let first_seed = app.scene_seed;
        app.next_scene();
        assert_eq!(app.current_kind, EffectKind::Plasma);
        assert_ne!(app.scene_seed, first_seed);
    }

    #[test]
    fn profile_rotation_stays_within_group() {
        let mut app = App::new(Config {
            seed: Some(23),
            profile: Some(Profile::ThreeD),
            ..Config::default()
        });
        let allowed = Profile::ThreeD.effects();
        assert!(allowed.contains(&app.current_kind));
        let mut previous = app.current_kind;
        for _ in 0..allowed.len() * 3 {
            app.next_scene();
            assert!(allowed.contains(&app.current_kind));
            assert_ne!(app.current_kind, previous);
            previous = app.current_kind;
        }
    }

    #[test]
    fn single_effect_profile_repeats_same_kind() {
        let mut app = App::new(Config {
            seed: Some(7),
            profile: Some(Profile::Time),
            ..Config::default()
        });
        for _ in 0..4 {
            app.next_scene();
            assert_eq!(app.current_kind, EffectKind::Clock);
        }
    }

    #[test]
    fn shuffled_rotation_avoids_immediate_repeats() {
        let mut app = App::new(Config {
            seed: Some(17),
            ..Config::default()
        });
        let mut previous = app.current_kind;
        for _ in 0..EffectKind::ALL.len() * 3 {
            app.next_scene();
            assert_ne!(app.current_kind, previous);
            previous = app.current_kind;
        }
    }
}
