#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// A representation of a VGA color byte
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Creates a new `ColorCode` from two colors
    const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

/// A representation of a character to be printed on the VGA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// The VGA buffer
#[repr(transparent)]
struct Buffer {
    chars: [[volatile::Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer to actually write to the screen
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// A method that writes a single byte to the screen
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            // If the byte is newline, continue printing to the new line
            b'\n' => self.new_line(),
            byte => {
                // If the line is full, continue printing to the new line
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// A method that writes the whole string to the VGA
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Print the byte if it is part of the ASCII table
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // If not, just print ■
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Moves every character one line up
    fn new_line(&mut self) {
        // We ommit the first row, because it is shifted off screen
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clean_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// A method to write the row with spaces (so it appears empty)
    fn clean_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

use core::fmt;
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use spin::Mutex;
lazy_static::lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Red, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// Prints to the VGA buffer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Prints to the VGA buffer, adding a newline at the end
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        // FIX: Don't use unwrap
        WRITER.lock().write_fmt(args).unwrap();
    });
}

// Tests

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_printn_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{s}").expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    })
}

#[test_case]
fn test_println_long_output() {
    let s =
        "Some test string that can't possibly fit on a single line of screen space in VGA buffer";
    println!("{s}");
    for (i, c) in s.chars().enumerate() {
        let screen_char = if i < BUFFER_WIDTH {
            WRITER.lock().buffer.chars[BUFFER_HEIGHT - 3][i].read()
        } else {
            WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i - BUFFER_WIDTH].read()
        };
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}
