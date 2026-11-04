//! A text console drawn into a framebuffer with JetBrains Mono. It keeps
//! the text in a grid of cells and, when it scrolls, redraws from that the
//! cells that change, so it never reads the framebuffer. Firmware often maps
//! the framebuffer uncached, where every store is a bus transaction, so it
//! also writes two pixels per store.

#![no_std]
#![forbid(unsafe_code)]

mod font;

use core::fmt;
use core::iter;
use core::ops::Range;

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

#[derive(Clone, Copy, PartialEq, Eq)]
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
    /// Rows whose pixels don't match the grid, because large text covers
    /// them.
    stale: Range<usize>,
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
            stale: 0..0,
        });
    }

    /// Sets the color of the text written from now on.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Writes `text` on lines of its own, each glyph `scale` times its size
    /// in both directions, in the current color. Characters past the
    /// screen's width are dropped. The large text isn't kept in the grid, so
    /// it disappears when the console next scrolls.
    pub fn write_large(&mut self, text: &str, scale: usize) {
        if self.col != 0 {
            self.newline();
        }
        for _ in 0..scale {
            self.newline();
        }
        let Some(screen) = &mut self.screen else {
            return;
        };
        let top = self.row.saturating_sub(scale);
        screen.stale = top..top + scale;
        for (i, c) in text.chars().take(screen.cols / scale).enumerate() {
            let byte = u8::try_from(c)
                .ok()
                .filter(|byte| (FIRST_GLYPH..=LAST_GLYPH).contains(byte))
                .unwrap_or(b'?');
            screen.draw_large(i * scale, top, byte, self.color, scale);
        }
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
        // Each screen row gets the row below it, and the last row goes
        // blank. Only cells that differ from what the row shows need
        // drawing, except on stale rows, which need every cell.
        for row in 0..screen.rows {
            let stale = screen.stale.contains(&row);
            for col in 0..screen.cols {
                let shown = self.cells[row * MAX_COLS + col];
                let next = if row + 1 < screen.rows {
                    self.cells[(row + 1) * MAX_COLS + col]
                } else {
                    BLANK
                };
                if stale || next != shown {
                    screen.draw(col, row, next);
                }
            }
        }
        screen.stale = 0..0;
        let last = (screen.rows - 1) * MAX_COLS;
        self.cells.copy_within(MAX_COLS..last + MAX_COLS, 0);
        self.cells[last..last + MAX_COLS].fill(BLANK);
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
        let format = self.fb.format;
        let glyph = self.font.glyph(cell.byte);
        for (y, coverage) in glyph.chunks_exact(width).enumerate() {
            let start = ((row * height + y) * self.fb.stride + col * width) * 4;
            write_pixels(
                &mut self.fb.buffer[start..start + width * 4],
                coverage.iter().map(|&alpha| pixel(fg, alpha, format)),
            );
        }
    }

    /// Draws `byte` `factor` times its size, from the cell at `col`, `row`.
    fn draw_large(&mut self, col: usize, row: usize, byte: u8, color: Color, factor: usize) {
        let Font { width, height, .. } = *self.font;
        let fg = color.rgb();
        let format = self.fb.format;
        let glyph = self.font.glyph(byte);
        for (y, coverage) in glyph.chunks_exact(width).enumerate() {
            for dy in 0..factor {
                let start = ((row * height + y * factor + dy) * self.fb.stride + col * width) * 4;
                write_pixels(
                    &mut self.fb.buffer[start..start + width * factor * 4],
                    coverage
                        .iter()
                        .flat_map(|&alpha| iter::repeat_n(pixel(fg, alpha, format), factor)),
                );
            }
        }
    }
}

/// Writes `pixels` along `row`, two per 8-byte store and the last one alone
/// when there is an odd number: on an uncached framebuffer each store is a
/// bus transaction, whatever its width.
fn write_pixels(row: &mut [u8], mut pixels: impl Iterator<Item = u32>) {
    let (pairs, last) = row.as_chunks_mut::<8>();
    for pair in pairs {
        let (Some(low), Some(high)) = (pixels.next(), pixels.next()) else {
            return;
        };
        *pair = (u64::from(low) | u64::from(high) << 32).to_le_bytes();
    }
    if let (Ok(last), Some(pixel)) = (<&mut [u8; 4]>::try_from(last), pixels.next()) {
        *last = pixel.to_le_bytes();
    }
}

/// `fg` blended over black by `alpha`, as the four bytes of a pixel in
/// `format` read as a little-endian word.
fn pixel(fg: [u8; 3], alpha: u8, format: PixelFormat) -> u32 {
    let [r, g, b] = fg.map(|channel| scale(channel, alpha));
    u32::from_le_bytes(match format {
        PixelFormat::Rgb => [r, g, b, 0],
        PixelFormat::Bgr => [b, g, r, 0],
    })
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
        sized_console(buffer, HEIGHT, format)
    }

    fn sized_console(buffer: &mut [u8], height: usize, format: PixelFormat) -> Box<Console<'_>> {
        let mut console = Box::new(Console::new());
        console.attach(Framebuffer {
            buffer,
            width: WIDTH,
            height,
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
    fn scrolls_cells_that_change_only_in_color() {
        let (mut a, mut b) = (buffer(), buffer());
        let mut scrolled = console(&mut a, PixelFormat::Bgr);
        scrolled.set_color(Color::Red);
        writeln!(scrolled, "ab").unwrap();
        scrolled.set_color(Color::Gray);
        write!(scrolled, "ab\na").unwrap();

        let mut fresh = console(&mut b, PixelFormat::Bgr);
        write!(fresh, "ab\na").unwrap();
        assert_eq!(pixels(&scrolled), pixels(&fresh));
    }

    #[test]
    fn large_text_goes_when_the_console_scrolls() {
        // Four rows of the 12 px font.
        let (mut a, mut b) = (vec![0; WIDTH * 64 * 4], vec![0; WIDTH * 64 * 4]);
        let mut scrolled = sized_console(&mut a, 64, PixelFormat::Bgr);
        scrolled.write_large("@", 2);
        write!(scrolled, "x\ny\nz").unwrap();

        let mut fresh = sized_console(&mut b, 64, PixelFormat::Bgr);
        write!(fresh, "\nx\ny\nz").unwrap();
        assert_eq!(pixels(&scrolled), pixels(&fresh));
    }

    #[test]
    fn shows_unprintable_characters_as_question_marks() {
        let mut buffer = buffer();
        let mut console = console(&mut buffer, PixelFormat::Bgr);
        write!(console, "é\tx").unwrap();
        assert_eq!(text(&console, 0), "??x");
    }

    /// Checks that the 7x16 glyph at the top left of `small` appears twice
    /// its size in `large`, from pixel row `top`.
    fn assert_doubled(small: &[u8], large: &[u8], top: usize) {
        let (small, large) = (small.as_chunks::<4>().0, large.as_chunks::<4>().0);
        for y in 0..16 {
            for x in 0..7 {
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let big = large[(top + 2 * y + dy) * WIDTH + 2 * x + dx];
                    assert_eq!(big, small[y * WIDTH + x], "glyph pixel {x}, {y}");
                }
            }
        }
    }

    #[test]
    fn draws_large_text_as_scaled_glyphs() {
        // Four rows of the 12 px font.
        let (mut a, mut b) = (vec![0; WIDTH * 64 * 4], vec![0; WIDTH * 64 * 4]);
        let mut small = sized_console(&mut a, 64, PixelFormat::Bgr);
        write!(small, "@").unwrap();
        let mut large = sized_console(&mut b, 64, PixelFormat::Bgr);
        large.write_large("@", 2);
        write!(large, "x").unwrap();
        assert_doubled(pixels(&small), pixels(&large), 0);
        assert_eq!(text(&large, 2), "x");
    }

    #[test]
    fn scrolls_to_make_room_for_large_text() {
        let (mut a, mut b) = (vec![0; WIDTH * 64 * 4], vec![0; WIDTH * 64 * 4]);
        let mut small = sized_console(&mut a, 64, PixelFormat::Bgr);
        write!(small, "@").unwrap();
        let mut large = sized_console(&mut b, 64, PixelFormat::Bgr);
        write!(large, "a\nb\nc\nd").unwrap();
        large.write_large("@", 2);
        assert_eq!(text(&large, 0), "d");
        assert_doubled(pixels(&small), pixels(&large), 16);
    }
}
