use core::fmt;
use spin::Mutex;

const SCREEN: *mut u8 = 0xB8000 as _;

static WRITER: Mutex<Writer> = Mutex::new(Writer { index: 0 });

struct Writer {
    index: usize,
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let i = &mut self.index;

        for &c in s.as_bytes() {
            if c == b'\n' {
                *i = (*i + 79) / 80 * 80;
            } else {
                unsafe {
                    *SCREEN.add(2 * *i + 0) = if c.is_ascii() { c } else { b'?' };
                    *SCREEN.add(2 * *i + 1) = 0x02;
                }

                *i += 1;
            }

            *i = *i % (80 * 25);
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn print(f: fmt::Arguments) {
    let _ = fmt::Write::write_fmt(&mut *WRITER.lock(), f);
}

#[macro_export]
macro_rules! print {
    ($($x:tt)*) => {{
        $crate::print::print(format_args!($($x)*));
    }}
}
