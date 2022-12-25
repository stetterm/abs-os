
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
// The list of 16 supported
// text colors by the os
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

// ColorCode is a wrapper for u8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

// Returns the correct u8 color value
// with the specified background and
// foreground color
impl ColorCode {
  fn new (foreground: Color, background: Color) -> ColorCode {
    ColorCode((background as u8) << 4 | (foreground as u8))
  }
}

// Stores the ascii character
// type and the color values
// associated with the second
// byte of the char
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
  ascii_character: u8,
  color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

use volatile::Volatile;

// Write buffer of ScreenChar
// is created for writing characters
#[repr(transparent)]
struct Buffer {
  chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}


// The writer keeps track of the
// position, the current color
// value and a buffer to write
// to that is static for each
// execution
pub struct Writer {
  column_position: usize,
  color_code: ColorCode,
  buffer: &'static mut Buffer,
}

use lazy_static::lazy_static;
use spin::Mutex;

// One static writer is initialized
// each time the program runs. This
// feature is not stable yet, so this
// requires lazy_static. Mutable static
// variables are generally a bad idea,
// so a mutex is requires here
lazy_static! {
  pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::White, Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
  });
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
  use x86_64::instructions::interrupts;

  // disable interrupts to prevent
  // deadlocks from happening from
  // printing text.
  interrupts::without_interrupts(|| {
    WRITER.lock().write_fmt(args).unwrap();
  });
}

impl Writer {

  /// Function to write a byte to the
  /// screen. This will put the character
  /// at the position of the cursor, and
  /// increment the cursor.
  /// byte:     character to write
  pub fn write_byte(&mut self, byte: u8) {
    match byte {

      // If the byte is a new line,
      // skip a line
      b'\n' => self.new_line(),
      
      // For all other bytes, write
      // the character into the buffer
      // in the writer, and increment
      // the column position
      byte => {

        // If the cursor is at the end
        // of the line, skip a line
        if self.column_position >= BUFFER_WIDTH {
          self.new_line();
        }

        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let color_code = self.color_code;

        // Write the character into
        // the buffer using the current
        // color, and increment the column number
        self.buffer.chars[row][col].write(ScreenChar {
          ascii_character: byte,
          color_code,
        });
        self.column_position += 1;
      }
    }
  }

  /// Write a string of bytes
  /// into the vga buffer.
  /// s:    string to print
  pub fn write_string(&mut self, s: &str) {
    for byte in s.bytes() {
      match byte {

        // Print all printable characters
        0x20..=0x7e | b'\n' => self.write_byte(byte),

        // Print 0x7e if not printable
        _ => self.write_byte(0xfe),
      }
    }
  }

  /// Skips a line on the VGA
  /// buffer. This requires copying
  /// the rows to the row above
  /// and clearing the last row.
  fn new_line(&mut self) {
    for row in 1..BUFFER_HEIGHT {
      for col in 0..BUFFER_WIDTH {
        let character = self.buffer.chars[row][col].read();
        self.buffer.chars[row - 1][col].write(character);
      }
    }
    self.clear_row(BUFFER_HEIGHT - 1);
    self.column_position = 0;
  }

  /// Clear the provided row
  /// of the VGA buffer by filling
  /// it with space characters.
  /// row:      row number to clear
  fn clear_row(&mut self, row: usize) {
    for col in 0..BUFFER_WIDTH {
      self.buffer.chars[row][col].write(
        ScreenChar {
          ascii_character: b' ',
          color_code: self.color_code,
        }
      );
    }
  }
}

// Implements usage of the vga 
// buffer using write macro like
// the following:
//      write!(writer, "I have {} apples", 12);
use core::fmt;
impl fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.write_string(s);
    Ok(())
  }
}

// Test ensures that a single print
// macro call will not panic.
#[test_case]
fn test_println_simple() {
  println!("test_println_simple output");
}

// Test ensures that many print
// statements will not cause a 
// panic.
#[test_case]
fn test_println_many() {
  for _ in 0..200 {
    println!("test_println_simple output");
  }
}

// Prints out a single-line string
// and ensures that the characters
// in the VGA buffer match the characters
// passed to the println! macro.
#[test_case]
fn test_println_output() {
  use core::fmt::Write;
  use x86_64::instructions::interrupts;

  let s = "Here are some words in a string for testing";
  interrupts::without_interrupts(|| {
    let mut writer = WRITER.lock();
    writeln!(writer, "\n{}", s).expect("writeln failed");
    for (i, c) in s.chars().enumerate() {
      let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
      assert_eq!(char::from(screen_char.ascii_character), c);
    }
  });
}














