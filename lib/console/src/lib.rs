//! A text console drawn into a framebuffer with JetBrains Mono. It keeps
//! the text in a grid of cells and redraws the screen from that when it
//! scrolls, so it never reads the framebuffer, which firmware often maps
//! uncached.

#![no_std]
#![forbid(unsafe_code)]

mod font;

use core::fmt;

use thisos_hal::{Framebuffer, PixelFormat};

/// The most columns and rows the grid holds. A larger screen keeps an
/// empty margin on the right or at the bottom.
const MAX_COLS: usize = 256;
const MAX_ROWS: usize = 128;

/// The console uses the largest font that gives at least this many rows,
/// or the smallest when none does.
const MIN_ROWS: usize = 40;

const FIRST_GLYPH: u8 = b' ';
const LAST_GLYPH: u8 = b'~';

/// One size of the font, as `xtask font` rasterized it.
struct Font {
    width: usize,
    height: usize,
    /// `width * height` bytes of coverage per glyph, from `' '` to `'~'`.
    glyphs: &'static [u8],
}

impl Font {
    fn pick(screen_height: usize) -> &'static Font {
        font::FONTS
            .iter()
            .rev()
            .find(|font| screen_height / font.height >= MIN_ROWS)
            .unwrap_or(&font::FONTS[0])
    }

    fn glyph(&self, byte: u8) -> &'static [u8] {
        let size = self.width * self.height;
        let start = usize::from(byte - FIRST_GLYPH) * size;
        &self.glyphs[start..start + size]
    }
}

/// A text color. The background is always black.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    /// The color of ordinary text.
    Gray,
    /// Dimmer than [`Color::Gray`].
    DarkGray,
    /// Red.
    Red,
    /// Yellow.
    Yellow,
    /// Green.
    Green,
    /// Blue.
    Blue,
}

impl Color {
    fn rgb(self) -> [u8; 3] {
        match self {
            Self::Gray => [0xC0, 0xC0, 0xC0],
            Self::DarkGray => [0x80, 0x80, 0x80],
            Self::Red => [0xF0, 0x50, 0x50],
            Self::Yellow => [0xF0, 0xD0, 0x40],
            Self::Green => [0x60, 0xD0, 0x60],
            Self::Blue => [0x60, 0x90, 0xF0],
        }
    }
}

#[derive(Clone, Copy)]
struct Cell {
    byte: u8,
    color: Color,
}

const BLANK: Cell = Cell {
    byte: b' ',
    color: Color::Gray,
};

/// A text console. Writing `\n` starts a new line, a line longer than the
/// screen wraps, and characters outside printable ASCII show as `?`.
pub struct Console<'a> {
    screen: Option<Screen<'a>>,
    cells: [Cell; MAX_COLS * MAX_ROWS],
    col: usize,
    row: usize,
    color: Color,
}

struct Screen<'a> {
    fb: Framebuffer<'a>,
    font: &'static Font,
    cols: usize,
    rows: usize,
}

impl<'a> Console<'a> {
    /// A console with no screen, which drops what it is given until
    /// [`Console::attach`]. It is `const` so that a static can hold it:
    /// the grid is too large to build on the kernel's stack.
    #[expect(
        clippy::large_stack_arrays,
        reason = "the kernel only calls this in a static's initializer"
    )]
    pub const fn new() -> Self {
        Self {
            screen: None,
            cells: [BLANK; MAX_COLS * MAX_ROWS],
            col: 0,
            row: 0,
            color: Color::Gray,
        }
    }

    /// Starts drawing to `fb`, and clears it to black.
    pub fn attach(&mut self, fb: Framebuffer<'a>) {
        let font = Font::pick(fb.height);
        let cols = (fb.width / font.width).min(MAX_COLS);
        let rows = (fb.height / font.height).min(MAX_ROWS);
        fb.buffer.fill(0);
        self.cells.fill(BLANK);
        self.col = 0;
        self.row = 0;
        self.screen = Some(Screen {
            fb,
            font,
            cols,
            rows,
        });
    }

    /// Sets the color of the text written from now on.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    fn put(&mut self, byte: u8) {
        let Some(cols) = self.screen.as_ref().map(|screen| screen.cols) else {
            return;
        };
        // Wrap only when the next character arrives, so that a line exactly
        // as wide as the screen doesn't leave an empty row after it.
        if self.col == cols {
            self.newline();
        }
        let cell = Cell {
            byte,
            color: self.color,
        };
        self.cells[self.row * MAX_COLS + self.col] = cell;
        if let Some(screen) = &mut self.screen {
            screen.draw(self.col, self.row, cell);
        }
        self.col += 1;
    }

    fn newline(&mut self) {
        let Some(screen) = &mut self.screen else {
            return;
        };
        self.col = 0;
        if self.row + 1 < screen.rows {
            self.row += 1;
            return;
        }
        let last = (screen.rows - 1) * MAX_COLS;
        self.cells.copy_within(MAX_COLS..last + MAX_COLS, 0);
        self.cells[last..last + MAX_COLS].fill(BLANK);
        for row in 0..screen.rows {
            for col in 0..screen.cols {
                screen.draw(col, row, self.cells[row * MAX_COLS + col]);
            }
        }
    }
}

impl Default for Console<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Write for Console<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            match u8::try_from(c) {
                Ok(b'\n') => self.newline(),
                Ok(byte @ FIRST_GLYPH..=LAST_GLYPH) => self.put(byte),
                _ => self.put(b'?'),
            }
        }
        Ok(())
    }
}

impl Screen<'_> {
    fn draw(&mut self, col: usize, row: usize, cell: Cell) {
        let Font { width, height, .. } = *self.font;
        let fg = cell.color.rgb();
        let glyph = self.font.glyph(cell.byte);
        for (y, coverage) in glyph.chunks_exact(width).enumerate() {
            let start = ((row * height + y) * self.fb.stride + col * width) * 4;
            let (pixels, _) = self.fb.buffer[start..start + width * 4].as_chunks_mut::<4>();
            for (pixel, &alpha) in pixels.iter_mut().zip(coverage) {
                let [r, g, b] = fg.map(|channel| scale(channel, alpha));
                *pixel = match self.fb.format {
                    PixelFormat::Rgb => [r, g, b, 0],
                    PixelFormat::Bgr => [b, g, r, 0],
                };
            }
        }
    }
}

/// Blends `channel` over the black background by `alpha` out of 255.
fn scale(channel: u8, alpha: u8) -> u8 {
    let blended = u16::from(channel) * u16::from(alpha) / 255;
    u8::try_from(blended).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::fmt::Write;
    use std::boxed::Box;
    use std::string::String;
    use std::vec;
    use std::vec::Vec;

    use super::*;

    // Holds 10 columns and 2 rows of the 12 px font.
    const WIDTH: usize = 70;
    const HEIGHT: usize = 32;

    fn buffer() -> Vec<u8> {
        vec![0xAA; WIDTH * HEIGHT * 4]
    }

    fn console(buffer: &mut [u8], format: PixelFormat) -> Box<Console<'_>> {
        let mut console = Box::new(Console::new());
        console.attach(Framebuffer {
            buffer,
            width: WIDTH,
            height: HEIGHT,
            stride: WIDTH,
            format,
        });
        console
    }

    fn text(console: &Console<'_>, row: usize) -> String {
        let cols = console.screen.as_ref().unwrap().cols;
        let cells = &console.cells[row * MAX_COLS..row * MAX_COLS + cols];
        let line: String = cells.iter().map(|cell| char::from(cell.byte)).collect();
        String::from(line.trim_end())
    }

    fn pixels<'a>(console: &'a Console<'_>) -> &'a [u8] {
        console.screen.as_ref().unwrap().fb.buffer
    }

    #[test]
    fn picks_the_largest_font_with_40_rows() {
        // QEMU's 1280x800, 1080p, the laptop's 1920x1200, 4K, and a screen
        // too small for 40 rows of any size.
        for (height, cell) in [(800, 16), (1080, 22), (1200, 22), (2160, 43), (100, 16)] {
            assert_eq!(Font::pick(height).height, cell, "screen height {height}");
        }
    }

    #[test]
    fn clears_the_screen_and_draws_in_the_pixel_format() {
        for (format, red) in [(PixelFormat::Rgb, 0), (PixelFormat::Bgr, 2)] {
            let mut buffer = buffer();
            let mut console = console(&mut buffer, format);
            console.set_color(Color::Red);
            write!(console, "@").unwrap();
            let pixels = pixels(&console);
            assert!(!pixels.contains(&0xAA), "{format:?}: not cleared");
            // Red is 0xF0, 0x50, 0x50, so a drawn pixel has more red than blue.
            assert!(
                pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|pixel| pixel[red] > pixel[2 - red] && pixel[3] == 0),
                "{format:?}: no red pixel"
            );
        }
    }

    #[test]
    fn wraps_long_lines_without_an_empty_row() {
        let mut buffer = buffer();
        let mut console = console(&mut buffer, PixelFormat::Bgr);
        write!(console, "0123456789\nab").unwrap();
        assert_eq!(text(&console, 0), "0123456789");
        assert_eq!(text(&console, 1), "ab");

        let mut buffer = self::buffer();
        let mut console = self::console(&mut buffer, PixelFormat::Bgr);
        write!(console, "0123456789x").unwrap();
        assert_eq!(text(&console, 1), "x");
    }

    #[test]
    fn scrolls_the_text_and_the_pixels() {
        let (mut a, mut b) = (buffer(), buffer());
        let mut scrolled = console(&mut a, PixelFormat::Bgr);
        write!(scrolled, "a\nb\nc").unwrap();
        assert_eq!(text(&scrolled, 0), "b");
        assert_eq!(text(&scrolled, 1), "c");

        let mut fresh = console(&mut b, PixelFormat::Bgr);
        write!(fresh, "b\nc").unwrap();
        assert_eq!(pixels(&scrolled), pixels(&fresh));
    }

    #[test]
    fn shows_unprintable_characters_as_question_marks() {
        let mut buffer = buffer();
        let mut console = console(&mut buffer, PixelFormat::Bgr);
        write!(console, "é\tx").unwrap();
        assert_eq!(text(&console, 0), "??x");
    }
}
