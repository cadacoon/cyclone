use core::{
    fmt::{self, Write},
    hint,
};

use alloc::boxed::Box;
use pio::{Port, ReadOnly};
use spin::Mutex;

const VGA_VRAM_WIDTH: usize = 80;
const VGA_VRAM_HEIGHT: usize = 25;
static VGA: Mutex<Vga> = Mutex::new(Vga {
    vram: 0xC00B_8000 as *mut u16,
    col: 0,
});

struct Vga {
    vram: *mut u16,
    col: u8,
}

unsafe impl Send for Vga {}

impl Write for Vga {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        match c {
            '\u{0020}'..='\u{007E}' => {
                if self.col >= VGA_VRAM_WIDTH as u8 {
                    self.write_char('\n')?;
                }

                unsafe {
                    self.vram
                        .add((VGA_VRAM_HEIGHT - 1) * VGA_VRAM_WIDTH + self.col as usize)
                        .write((c as u16) | 0x0F << 8);
                }

                self.col += 1;
            }
            '\n' => {
                for row in 1..VGA_VRAM_HEIGHT {
                    for col in 0..VGA_VRAM_WIDTH {
                        unsafe {
                            self.vram
                                .add((row - 1) * VGA_VRAM_WIDTH + col)
                                .write(self.vram.add(row * VGA_VRAM_WIDTH + col).read());
                        }
                    }
                }

                for col in 0..VGA_VRAM_WIDTH {
                    unsafe {
                        self.vram
                            .add((VGA_VRAM_HEIGHT - 1) * VGA_VRAM_WIDTH + col)
                            .write(b' ' as u16 | 0x0F << 8)
                    }
                }

                self.col = 0;
            }
            _ => {}
        }

        Ok(())
    }
}

static COM1: Mutex<Com> = Mutex::new(unsafe { Com::new(0x3F8) });

struct Com {
    data: Port<u8>,
    int_control: Port<u8>,
    fifo_control: Port<u8>,
    line_control: Port<u8>,
    modem_control: Port<u8>,
    line_status: Port<u8, ReadOnly>,
    modem_status: Port<u8, ReadOnly>,
}

impl Com {
    const unsafe fn new(base: u16) -> Self {
        Self {
            data: Port::new(base),
            int_control: Port::new(base + 1),
            fifo_control: Port::new(base + 2),
            line_control: Port::new(base + 3),
            modem_control: Port::new(base + 4),
            line_status: Port::new(base + 5),
            modem_status: Port::new(base + 6),
        }
    }

    fn init(&mut self) {
        self.int_control.write(0);
        self.line_control.write(0b1000_0000); // DLAB
        self.data.write(3); // 38400
        self.int_control.write(0);
        self.line_control.write(0b0000_0011); // 8N1
        self.fifo_control.write(0b1100_0111); // enable and clear FIFO, 14B trigger
        self.modem_control.write(0b0000_1011); // DTR, RTS, enable IRQ
    }
}

impl Write for Com {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        while self.line_status.read() & 1 << 6 == 0 {
            hint::spin_loop();
        }
        self.data.write(c as u8);

        Ok(())
    }
}

#[derive(Default)]
struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let _ = writeln!(COM1.lock(), "{}", record.args());
    }

    fn flush(&self) {}
}

pub fn init() {
    COM1.lock().init();

    log::set_max_level(log::LevelFilter::Debug);
    let _ = log::set_logger(Box::leak(Box::new(Logger::default())));
}
