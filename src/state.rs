use std::{borrow::Cow, f64::consts::PI};

use xi_rope::{interval::IntervalBounds, Cursor, LinesMetric, Rope, RopeInfo};

pub(crate) enum Command {
    Up,
    Down,
    Left,
    Right,
    JumpToBottom,
    InsertCharacter(char),
}

pub(crate) struct User {
    byte_offset_to_top_left: usize,
    /// x, y. 0-indexed from the top-left
    cursor_pos: (u16, u16),
    screen_size: (u16, u16),
    rope: Rope,
}

impl User {
    pub(crate) fn new(screen_size: (u16, u16), rope: Rope) -> Self {
        Self {
            byte_offset_to_top_left: 0,
            cursor_pos: (0, 0),
            screen_size,
            rope,
        }
    }

    pub(crate) fn slice_to_cow<T: IntervalBounds>(&self, range: T) -> Cow<str> {
        self.rope.slice_to_cow(range)
    }

    pub(crate) fn cursor_x(&self) -> u16 {
        self.cursor_pos.0
    }

    pub(crate) fn cursor_y(&self) -> u16 {
        self.cursor_pos.1
    }

    pub(crate) fn height(&self) -> u16 {
        self.screen_size.1
    }

    pub(crate) fn width(&self) -> u16 {
        self.screen_size.0
    }

    pub(crate) fn start_of_screen(&self) -> Cursor<RopeInfo> {
        Cursor::new(&self.rope, self.byte_offset_to_top_left)
    }

    fn start_of_current_line(&self) -> Option<Cursor<RopeInfo>> {
        let mut cursor = self.start_of_screen();

        // Move the cursor to the current line
        for _i in 0..self.cursor_pos.1 {
            let _pos: usize = cursor.next::<LinesMetric>()?;
        }

        Some(cursor)
    }

    pub(crate) fn current_line_length(&self) -> Option<u16> {
        let mut cursor = self.start_of_current_line()?;
        let pos = cursor.pos();

        Some((cursor.next::<LinesMetric>().unwrap_or(self.rope.len()) - pos - 1) as u16)
    }

    pub(crate) fn handle_event(&mut self, command: &Command) {
        match command {
            Command::Up => {
                if let Some(_) = self.start_of_current_line() {
                    if self.cursor_pos.1 == 0 {
                        let mut cursor = self.start_of_screen();
                        if let Some(new_start_pos) = cursor.prev::<LinesMetric>() {
                            self.byte_offset_to_top_left = new_start_pos;
                        }
                    }
                    self.cursor_pos.1 = self.cursor_pos.1.saturating_sub(1);
                }
            }
            Command::Down => {
                if let Some(mut cursor) = self.start_of_current_line() {
                    if let Some(_) = cursor.next::<LinesMetric>() {
                        if self.cursor_pos.1 == self.screen_size.1 - 1 {
                            let mut cursor = self.start_of_screen();
                            if let Some(new_start_pos) = cursor.next::<LinesMetric>() {
                                self.byte_offset_to_top_left = new_start_pos;
                            }
                        } else {
                            self.cursor_pos.1 = (self.cursor_pos.1 + 1).min(self.screen_size.1 - 1);
                        }
                    }
                }
            }
            Command::Left => {
                self.cursor_pos.0 = self
                    .cursor_pos
                    .0
                    .min(self.current_line_length().unwrap_or(0));
                self.cursor_pos.0 = self.cursor_pos.0.saturating_sub(1)
            }
            Command::Right => {
                self.cursor_pos.0 =
                    (self.cursor_pos.0 + 1).min(self.current_line_length().unwrap_or(0))
            }
            Command::JumpToBottom => {
                // TODO: This isn't really jumping to the bottom of the file.
                self.cursor_pos.1 = self.screen_size.1 - 1;
            }

            Command::InsertCharacter(c) => {
                if let Some(cursor) = self.start_of_current_line() {
                    let pos = cursor.pos();
                    let pos = pos
                        + (self
                            .cursor_pos
                            .0
                            .min(self.current_line_length().unwrap_or(0))
                            as usize);

                    self.rope.edit(pos..pos, format!("{c}"));
                    self.cursor_pos.0 += 1;
                }
            }
        }
    }
}
