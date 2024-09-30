use core::fmt::{self, Write};

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

impl tracing::Subscriber for TtySubscriber {
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }

    fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        event.record(&mut TtyFieldVisitor(&mut self.0.lock()));
    }

    fn enter(&self, _span: &tracing::span::Id) {}

    fn exit(&self, _span: &tracing::span::Id) {}
}

struct TtyFieldVisitor<'tty>(&'tty mut Tty);

impl<'tty> tracing::field::Visit for TtyFieldVisitor<'tty> {
    fn record_debug(&mut self, _field: &tracing::field::Field, value: &dyn fmt::Debug) {
        let _ = writeln!(self.0, "{:#?}", value);
    }
}

pub fn init_logging() {
    let _ = tracing::subscriber::set_global_default(TtySubscriber::default());
}
