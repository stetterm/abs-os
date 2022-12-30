use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

// Global serial port interface for
// redirecting output of abs_os to
// stdout of host machine
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

// Print function to write the
// specified arguments to the
// serial port
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // Disable interrupts to prevent
    // deadlocks from printing to
    // serial port.
    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

// Useful macro for printing
// formatted text to the serial port
#[macro_export]
macro_rules! serial_print {
  ($($arg:tt)*) => {
    $crate::serial::_print(format_args!($($arg)*));
  }
}

// Println to serial port
#[macro_export]
macro_rules! serial_println {
  () => ($crate::serial_print!("\n"));
  ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
  ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
          concat!($fmt, "\n"), $($arg)*));
}
