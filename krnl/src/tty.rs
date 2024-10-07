use core::fmt::{self, Write};

use alloc::boxed::Box;
use spin::Mutex;

use crate::mm;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

struct Tty {
    buffer: &'static mut [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT],
    column: u8,
}

impl Write for Tty {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        match c {
            '\u{0020}'..='\u{007E}' => {
                if self.column >= BUFFER_WIDTH as u8 {
                    self.write_char('\n')?;
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column as usize;
                self.buffer[row][col] = (c as u16) | 0x0F << 8;

                self.column += 1;
            }
            '\n' => {
                for row in 1..BUFFER_HEIGHT {
                    for col in 0..BUFFER_WIDTH {
                        self.buffer[row - 1][col] = self.buffer[row][col];
                    }
                }

                let row = BUFFER_HEIGHT - 1;
                for col in 0..BUFFER_WIDTH {
                    self.buffer[row][col] = b' ' as u16 | 0x0F << 8;
                }

                self.column = 0;
            }
            _ => {}
        }

        Ok(())
    }
}

struct TtySubscriber(Mutex<Tty>);

impl Default for TtySubscriber {
    fn default() -> Self {
        Self(Mutex::new(Tty {
            buffer: unsafe {
                &mut *((0xB8000 + (&mm::KERNEL_VMA as *const u8 as usize))
                    as *mut [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT])
            },
            column: 0,
        }))
    }
}

impl log::Log for TtySubscriber {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        unsafe {
            self.0.force_unlock();
        }
        let _ = writeln!(self.0.lock(), "{}", record.args());
    }

    fn flush(&self) {}
}

pub fn init_logging() {
    log::set_max_level(log::LevelFilter::Debug);
    let _ = log::set_logger(Box::leak(Box::new(TtySubscriber::default())));
}
