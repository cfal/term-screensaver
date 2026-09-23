# term-screensaver

A procedural terminal screensaver written in Rust. It renders into an in-memory
cell buffer, emits only changed cells, and wraps frames in synchronized ANSI
output for smooth, low-flicker animation.

## Animations

- A luminous, shaded energy orb with rotating surface filaments and sparks
- Animated Mandelbrot and Julia fractals
- A shaded spinning donut and rotating wireframe solids
- Tumbling 5x7 glyphs cycling through `A-Z` and `0-9`
- The current `HH:MM` time as a rotating panel, orbiting clocks, or a twisted ribbon
- A warp-speed starfield
- A full-screen interference plasma

Every scene receives newly randomized parameters. The default shuffled rotation
avoids immediate repeats and deliberately mixes color and monochrome scenes.

## Run

```sh
cargo run --release
```

Useful options:

```text
--fps 30
--interval 20
--seed 42
--effect orb
--profile 3d
--color auto|always|never
--ascii
--list
```

`--color auto` honors the `NO_COLOR` environment variable. `--ascii` avoids
Unicode density glyphs for terminals with uncertain character-width support.

Controls:

```text
Space / Right  next scene      r  regenerate scene
p              pause           m  toggle monochrome
?              toggle help     q/Esc/Ctrl-C  quit
```

Pausing freezes animation and scene rotation. Clock scenes continue to display
the real current time.

## Verify

```sh
CARGO_BUILD_JOBS=1 cargo test --offline --locked
CARGO_BUILD_JOBS=1 cargo clippy --offline --locked --all-targets -- -D warnings
cargo fmt --check
CARGO_BUILD_JOBS=1 cargo build --release --offline --locked
python3 tests/pty_smoke.py target/release/term-screensaver
```

The PTY smoke test checks resize handling and terminal restoration after normal
exit and `SIGTERM`. Like other terminal programs, recovery is impossible after
`SIGKILL`.
