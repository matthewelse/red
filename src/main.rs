use std::{
    fs::File,
    io::{stdin, stdout, Read, Write},
    path::PathBuf,
};

use clap::Parser;
use termion::{
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
    screen::IntoAlternateScreen,
};
use tracing::{debug, info};
use tracing_appender::rolling::daily;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use xi_rope::{Cursor, LinesMetric, Rope};

mod state;

#[derive(Parser)]
struct CommandArgs {
    file_path: PathBuf,
}

fn render_screen(stdout: &mut impl Write, state: &state::User) {
    write!(stdout, "{}", termion::clear::All).unwrap();

    let mut cursor = state.start_of_screen();
    for i in 0u16..state.height() {
        let pos = cursor.pos();
        match cursor.next::<LinesMetric>() {
            Some(next_pos) => {
                let line = state.slice_to_cow(pos..(next_pos - 1));
                write!(
                    stdout,
                    "{}{}{}{}",
                    termion::cursor::Goto(1, i + 1),
                    i + 1,
                    termion::cursor::Goto(4, i + 1),
                    &line[..(state.width() as usize).saturating_sub(4).min(line.len())]
                )
                .unwrap();
            }
            None => break,
        }
    }

    let line_length = state.current_line_length();
    let x_pos = state.cursor_x().min(line_length.unwrap_or(0));
    debug!("line length {line_length:?} --> y_pos = {x_pos:?}");

    write!(
        stdout,
        "{}",
        termion::cursor::Goto(4 + x_pos, state.cursor_y() + 1)
    )
    .unwrap();

    stdout.flush().unwrap();
}

#[derive(Clone, Copy, Debug)]
enum Mode {
    Normal,
    Insert,
}

fn main() {
    let file_appender = daily("", "red.log");

    let subscriber = tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false) // Disable ANSI escape codes in log files
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let stdin = stdin();
    let mut stdout = MouseTerminal::from(
        stdout()
            .into_raw_mode()
            .unwrap()
            .into_alternate_screen()
            .unwrap(),
    );

    let args = CommandArgs::parse();

    let mut file = File::open(&args.file_path).unwrap();
    let mut mode = Mode::Normal;

    // TODO: only load part of the file at a time
    let mut file_buf = String::new();
    file.read_to_string(&mut file_buf).unwrap();

    let mut state = state::User::new(termion::terminal_size().unwrap(), Rope::from(&file_buf));

    render_screen(&mut stdout, &state);

    for event in stdin.events() {
        use termion::event::{Event, Key};

        match (mode, event.unwrap()) {
            (Mode::Normal, Event::Key(Key::Char('q'))) => break,
            (Mode::Normal, Event::Key(Key::Char('i'))) => {
                info!("Entering insert mode.");
                mode = Mode::Insert;
            }
            (Mode::Normal, Event::Key(Key::Char('G'))) => {
                info!("Jumping to bottom.");
                state.handle_event(&state::Command::JumpToBottom);
            }

            (Mode::Insert, Event::Key(Key::Esc)) => {
                info!("Entering normal mode.");
                mode = Mode::Normal
            }
            (Mode::Insert, Event::Key(Key::Char(x))) => {
                // rope.edit(cursor_byte_pos..cursor_byte_pos, format!("{x}"));
                // cursor_byte_pos += 1;
                // cursor_x += 1;
                debug!("Inserting character: {:?}", x);
                state.handle_event(&state::Command::InsertCharacter(x))
            }

            (Mode::Insert, Event::Key(Key::Backspace)) => {
                // if cursor_byte_pos > 0 {
                //     rope.edit((cursor_byte_pos - 1)..cursor_byte_pos, "".to_string());
                //     cursor_byte_pos -= 1;
                //     if cursor_x == 0 {
                //         cursor_y = cursor_y.saturating_sub(1);
                //     } else {
                //         cursor_x = cursor_x.saturating_sub(1);
                //     }
                //     debug!("Backspace");
                // }
            }

            // Arrow keys are always available. hjkl are available in normal mode.
            (_, Event::Key(Key::Up)) | (Mode::Normal, Event::Key(Key::Char('k'))) => {
                state.handle_event(&state::Command::Up)
            }
            (_, Event::Key(Key::Down)) | (Mode::Normal, Event::Key(Key::Char('j'))) => {
                state.handle_event(&state::Command::Down)
            }
            (_, Event::Key(Key::Left)) | (Mode::Normal, Event::Key(Key::Char('h'))) => {
                state.handle_event(&state::Command::Left)
            }
            (_, Event::Key(Key::Right)) | (Mode::Normal, Event::Key(Key::Char('l'))) => {
                state.handle_event(&state::Command::Right)
            }
            _ => (),
        }

        render_screen(&mut stdout, &state);
    }
}
