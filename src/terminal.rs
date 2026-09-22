use std::{
    io::{self, IsTerminal, Write},
    panic,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

use crate::canvas::{Canvas, Style};

const SYNC_START: &str = "\x1b[?2026h";
const SYNC_END: &str = "\x1b[?2026l";
const DISABLE_WRAP: &str = "\x1b[?7l";
const ENABLE_WRAP: &str = "\x1b[?7h";

pub struct TerminalSession {
    raw: bool,
    alternate_screen: bool,
}

impl TerminalSession {
    pub fn enter() -> io::Result<Self> {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return Err(io::Error::other(
                "interactive mode requires terminal stdin and stdout",
            ));
        }
        if std::env::var_os("TERM").is_some_and(|term| term == "dumb") {
            return Err(io::Error::other("TERM=dumb does not support animation"));
        }

        let mut session = Self {
            raw: false,
            alternate_screen: false,
        };
        install_panic_cleanup();
        enable_raw_mode()?;
        session.raw = true;

        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            Hide,
            Clear(ClearType::All),
            MoveTo(0, 0)
        )?;
        stdout.write_all(DISABLE_WRAP.as_bytes())?;
        stdout.flush()?;
        session.alternate_screen = true;
        Ok(session)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        restore_terminal(self.alternate_screen, self.raw);
    }
}

fn install_panic_cleanup() {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        restore_terminal(true, true);
        previous(info);
    }));
}

fn restore_terminal(alternate_screen: bool, raw: bool) {
    let mut stdout = io::stdout();
    let _ = stdout.write_all(b"\x1b[0m");
    let _ = stdout.write_all(ENABLE_WRAP.as_bytes());
    let _ = execute!(stdout, Show);
    if alternate_screen {
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
    let _ = stdout.flush();
    if raw {
        let _ = disable_raw_mode();
    }
}

#[derive(Default)]
pub struct Renderer {
    previous: Option<Canvas>,
    previous_colored: bool,
    output: Vec<u8>,
}

impl Renderer {
    pub fn invalidate(&mut self) {
        self.previous = None;
    }

    pub fn render(
        &mut self,
        writer: &mut impl Write,
        canvas: &Canvas,
        colored: bool,
    ) -> io::Result<()> {
        let full = self.previous.as_ref().is_none_or(|previous| {
            previous.width() != canvas.width() || previous.height() != canvas.height()
        }) || colored != self.previous_colored;
        let previous = self.previous.as_ref();
        self.output.clear();
        self.output.extend_from_slice(SYNC_START.as_bytes());
        self.output.extend_from_slice(b"\x1b[?25l\x1b[0m");
        if full {
            self.output.extend_from_slice(b"\x1b[2J");
        }

        let mut cursor = None;
        let mut active_style = Style::default();
        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                let index = usize::from(y) * usize::from(canvas.width()) + usize::from(x);
                let cell = canvas.cells()[index];
                let changed = full || previous.is_none_or(|old| old.cells()[index] != cell);
                if !changed {
                    continue;
                }

                if cursor != Some((x, y)) {
                    write!(self.output, "\x1b[{};{}H", y + 1, x + 1)?;
                }
                let effective = Style {
                    foreground: colored.then_some(cell.style.foreground).flatten(),
                    ..cell.style
                };
                if effective != active_style {
                    write_style(&mut self.output, effective)?;
                    active_style = effective;
                }
                write!(self.output, "{}", cell.glyph)?;
                cursor = Some((x.saturating_add(1), y));
            }
        }

        self.output.extend_from_slice(b"\x1b[0m");
        self.output.extend_from_slice(SYNC_END.as_bytes());
        writer.write_all(&self.output)?;
        writer.flush()?;
        self.previous = Some(canvas.clone());
        self.previous_colored = colored;
        Ok(())
    }
}

fn write_style(writer: &mut impl Write, style: Style) -> io::Result<()> {
    writer.write_all(b"\x1b[0m")?;
    if style.bold {
        writer.write_all(b"\x1b[1m")?;
    }
    if style.dim {
        writer.write_all(b"\x1b[2m")?;
    }
    if let Some(color) = style.foreground {
        write!(writer, "\x1b[38;2;{};{};{}m", color.r, color.g, color.b)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_synchronized_full_then_small_diff() {
        let mut renderer = Renderer::default();
        let mut canvas = Canvas::new(5, 2);
        canvas.put(1, 0, 'A', Style::default());
        let mut first = Vec::new();
        renderer.render(&mut first, &canvas, false).unwrap();

        canvas.put(2, 0, 'B', Style::default());
        let mut second = Vec::new();
        renderer.render(&mut second, &canvas, false).unwrap();

        assert!(first.starts_with(SYNC_START.as_bytes()));
        assert!(first.ends_with(SYNC_END.as_bytes()));
        assert!(second.len() < first.len());
        assert!(String::from_utf8(second).unwrap().contains('B'));
    }
}
