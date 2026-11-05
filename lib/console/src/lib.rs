//! A text console drawn into a framebuffer with JetBrains Mono. It keeps
//! the text in a grid of cells and, when it scrolls, redraws from that the
//! cells that change, so it never reads the framebuffer. Firmware often maps
//! the framebuffer uncached, where every store is a bus transaction, so it
//! also writes two pixels per store where it can.

#![no_std]
#![forbid(unsafe_code)]

mod font;

use core::fmt;
use core::iter;
use core::ops::Range;

use thisos_hal::{Framebuffer, PixelFormat, Pixels};

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
pub struct Console<M> {
    screen: Option<Screen<M>>,
    cells: [Cell; MAX_COLS * MAX_ROWS],
    col: usize,
    row: usize,
    color: Color,
}

struct Screen<M> {
    fb: Framebuffer<M>,
    font: &'static Font,
    cols: usize,
    rows: usize,
    /// Rows whose pixels don't match the grid, because large text covers
    /// them.
    stale: Range<usize>,
}

impl<M: Pixels> Console<M> {
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
    pub fn attach(&mut self, mut fb: Framebuffer<M>) {
        let font = Font::pick(fb.height);
        let cols = (fb.width / font.width).min(MAX_COLS);
        let rows = (fb.height / font.height).min(MAX_ROWS);
        let len = fb.pixels.len();
        write_pixels(&mut fb.pixels, 0..len, iter::repeat(0));
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

impl<M: Pixels> Default for Console<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Pixels> fmt::Write for Console<M> {
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

impl<M: Pixels> Screen<M> {
    fn draw(&mut self, col: usize, row: usize, cell: Cell) {
        let Font { width, height, .. } = *self.font;
        let fg = cell.color.rgb();
        let format = self.fb.format;
        let glyph = self.font.glyph(cell.byte);
        for (y, coverage) in glyph.chunks_exact(width).enumerate() {
            let start = (row * height + y) * self.fb.stride + col * width;
            write_pixels(
                &mut self.fb.pixels,
                start..start + width,
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
                let start = (row * height + y * factor + dy) * self.fb.stride + col * width;
                write_pixels(
                    &mut self.fb.pixels,
                    start..start + width * factor,
                    coverage
                        .iter()
                        .flat_map(|&alpha| iter::repeat_n(pixel(fg, alpha, format), factor)),
                );
            }
        }
    }
}

/// Writes `pixels` to the indices in `run`, two per 8-byte store, which
/// [`Pixels::store_pair`] only takes at an even index: a run that starts
/// at an odd index writes its first pixel alone, and one with an odd
/// number of pixels after that writes its last alone. On an uncached framebuffer each store is a bus
/// transaction, whatever its width.
fn write_pixels(fb: &mut impl Pixels, run: Range<usize>, mut pixels: impl Iterator<Item = u32>) {
    let mut index = run.start;
    if index % 2 == 1
        && index < run.end
        && let Some(pixel) = pixels.next()
    {
        fb.store(index, pixel);
        index += 1;
    }
    while index + 2 <= run.end {
        let (Some(low), Some(high)) = (pixels.next(), pixels.next()) else {
            return;
        };
        fb.store_pair(index, u64::from(low) | u64::from(high) << 32);
        index += 2;
    }
    if index < run.end
        && let Some(pixel) = pixels.next()
    {
        fb.store(index, pixel);
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

    /// Framebuffer memory that panics on the stores the real one can't
    /// take, so that the tests catch them.
    #[derive(Debug)]
    struct Memory(Vec<u32>);

    impl Pixels for Memory {
        fn len(&self) -> usize {
            self.0.len()
        }

        fn store(&mut self, index: usize, pixel: u32) {
            self.0[index] = pixel;
        }

        fn store_pair(&mut self, index: usize, pixels: u64) {
            assert!(index % 2 == 0, "pair stored at odd index {index}");
            let low = u32::try_from(pixels & 0xFFFF_FFFF).unwrap();
            let high = u32::try_from(pixels >> 32).unwrap();
            self.0[index..index + 2].copy_from_slice(&[low, high]);
        }
    }

    // Set in every byte before the console clears the screen.
    const UNCLEARED: u32 = 0xAAAA_AAAA;

    fn console(format: PixelFormat) -> Box<Console<Memory>> {
        sized_console(HEIGHT, format)
    }

    fn sized_console(height: usize, format: PixelFormat) -> Box<Console<Memory>> {
        let mut console = Box::new(Console::new());
        console.attach(Framebuffer {
            pixels: Memory(vec![UNCLEARED; WIDTH * height]),
            width: WIDTH,
            height,
            stride: WIDTH,
            format,
        });
        console
    }

    fn text(console: &Console<Memory>, row: usize) -> String {
        let cols = console.screen.as_ref().unwrap().cols;
        let cells = &console.cells[row * MAX_COLS..row * MAX_COLS + cols];
        let line: String = cells.iter().map(|cell| char::from(cell.byte)).collect();
        String::from(line.trim_end())
    }

    fn pixels(console: &Console<Memory>) -> &[u32] {
        &console.screen.as_ref().unwrap().fb.pixels.0
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
            let mut console = console(format);
            console.set_color(Color::Red);
            write!(console, "@").unwrap();
            let pixels = pixels(&console);
            assert!(!pixels.contains(&UNCLEARED), "{format:?}: not cleared");
            // Red is 0xF0, 0x50, 0x50, so a drawn pixel has more red than blue.
            assert!(
                pixels
                    .iter()
                    .map(|pixel| pixel.to_le_bytes())
                    .any(|pixel| pixel[red] > pixel[2 - red] && pixel[3] == 0),
                "{format:?}: no red pixel"
            );
        }
    }

    // The 12 px font's cells are 7 pixels wide, so the second column starts
    // at an odd index and its rows need a single store before the pairs.
    #[test]
    fn draws_a_glyph_the_same_at_an_odd_index() {
        let mut even = console(PixelFormat::Bgr);
        write!(even, "@").unwrap();
        let mut odd = console(PixelFormat::Bgr);
        write!(odd, " @").unwrap();
        let (even, odd) = (pixels(&even), pixels(&odd));
        for y in 0..16 {
            let row = y * WIDTH;
            assert_eq!(odd[row + 7..row + 14], even[row..row + 7], "glyph row {y}");
        }
    }

    #[test]
    fn wraps_long_lines_without_an_empty_row() {
        let mut console = console(PixelFormat::Bgr);
        write!(console, "0123456789\nab").unwrap();
        assert_eq!(text(&console, 0), "0123456789");
        assert_eq!(text(&console, 1), "ab");

        let mut console = self::console(PixelFormat::Bgr);
        write!(console, "0123456789x").unwrap();
        assert_eq!(text(&console, 1), "x");
    }

    #[test]
    fn scrolls_the_text_and_the_pixels() {
        let mut scrolled = console(PixelFormat::Bgr);
        write!(scrolled, "a\nb\nc").unwrap();
        assert_eq!(text(&scrolled, 0), "b");
        assert_eq!(text(&scrolled, 1), "c");

        let mut fresh = console(PixelFormat::Bgr);
        write!(fresh, "b\nc").unwrap();
        assert_eq!(pixels(&scrolled), pixels(&fresh));
    }

    #[test]
    fn scrolls_cells_that_change_only_in_color() {
        let mut scrolled = console(PixelFormat::Bgr);
        scrolled.set_color(Color::Red);
        writeln!(scrolled, "ab").unwrap();
        scrolled.set_color(Color::Gray);
        write!(scrolled, "ab\na").unwrap();

        let mut fresh = console(PixelFormat::Bgr);
        write!(fresh, "ab\na").unwrap();
        assert_eq!(pixels(&scrolled), pixels(&fresh));
    }

    #[test]
    fn large_text_goes_when_the_console_scrolls() {
        // Four rows of the 12 px font.
        let mut scrolled = sized_console(64, PixelFormat::Bgr);
        scrolled.write_large("@", 2);
        write!(scrolled, "x\ny\nz").unwrap();

        let mut fresh = sized_console(64, PixelFormat::Bgr);
        write!(fresh, "\nx\ny\nz").unwrap();
        assert_eq!(pixels(&scrolled), pixels(&fresh));
    }

    #[test]
    fn shows_unprintable_characters_as_question_marks() {
        let mut console = console(PixelFormat::Bgr);
        write!(console, "é\tx").unwrap();
        assert_eq!(text(&console, 0), "??x");
    }

    /// Checks that the 7x16 glyph at the top left of `small` appears twice
    /// its size in `large`, from pixel row `top`.
    fn assert_doubled(small: &[u32], large: &[u32], top: usize) {
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
        let mut small = sized_console(64, PixelFormat::Bgr);
        write!(small, "@").unwrap();
        let mut large = sized_console(64, PixelFormat::Bgr);
        large.write_large("@", 2);
        write!(large, "x").unwrap();
        assert_doubled(pixels(&small), pixels(&large), 0);
        assert_eq!(text(&large, 2), "x");
    }

    #[test]
    fn scrolls_to_make_room_for_large_text() {
        let mut small = sized_console(64, PixelFormat::Bgr);
        write!(small, "@").unwrap();
        let mut large = sized_console(64, PixelFormat::Bgr);
        write!(large, "a\nb\nc\nd").unwrap();
        large.write_large("@", 2);
        assert_eq!(text(&large, 0), "d");
        assert_doubled(pixels(&small), pixels(&large), 16);
    }
}
