
use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // stores each enum value as a u8
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)] // Ensures the ColorCode type has the same data layout as a u8
struct ColorCode(u8); // this will contain the full color byte, foreground and background

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8)) // shift background left 4 to make a byte's worth of data out of 2 Color values
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)] // Guarantees the correct field ordering by specifying that the struct must be laid out like a C struct
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

use volatile::Volatile;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    column_position: usize, // keeps track of current position in last row
    color_code: ColorCode, // holds the foreground and background color
    buffer: &'static mut Buffer, // reference to VGA buffer ('static specifies that this reference is valid for the duration of the programs run time
}

/// Only link something if it is called, allowing us to compute the static's value at runtime
use lazy_static::lazy_static;
/// Can be used as an interface from other modules without carrying a Writer instance around
lazy_static! { // Declare this function as lazily linked
    /* since it's a static reference it must provide asynchronous access to it
    by all member functions so as not to create a race for the data, we can do this with a "spinlock"
    which basically means instead of blocking, a thread may attempt to acquire a lock on the data over and over again until the
    Mutex is freed from the last thread that had a lock on it. We use this version of synchronized
    interior mutability because we have no underlying OS that handles Mutexes or threads*/
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
      column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe {&mut *(0xb8000 as *mut Buffer)},
    });
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), // If byte is new line, call new_line()
            byte => {
                if self.column_position >= BUFFER_WIDTH { // Is line full? If so, call new_line()
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar { // Write new ScreenChar to buffer
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1; // Current column position increments
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Not part of printable ASCII range, print a ■ character
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT { // Omit first row as it is the row that is shifted off screen
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read(); // grabbing the char at that position
                self.buffer.chars[row - 1][col].write(character); // moving that char up a row
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1); // clearing the duplicates from the previous row
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ', // Space
            color_code: self.color_code, // Get vga_buffer's current color_code
        };

        for col in 0..BUFFER_WIDTH { // iterate through columns in the row and write the space character (which is blank)
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

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}