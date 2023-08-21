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
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
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

    fn new_line(&mut self) {
        // TODO:
    }
}

/// Just a test function
pub fn print_something() {
    // Setting up the writer
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        // Creating the buffer at the address of VGA
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
}
