#![no_std]
#![feature(abi_x86_interrupt)]

use lazy_static::lazy_static;
use core::ffi::c_void;
use core::mem;
use game::{Direction, Game};
use spin::Mutex;
use pic8259_simple::ChainedPics;

mod game;
mod screens;

const WHITE: u16 = 0x0F00;
const BLUE: u16 = 0x0100;
const RED: u16 = 0x0400;

lazy_static! {
    static ref IDT: [IDTEntry; 256] = {
        let mut idt = [IDTEntry::unused(); 256];
        idt[0x08] = IDTEntry::new(double_fault as _);
        idt[0x20] = IDTEntry::new(timer_interrupt as _);
        idt[0x21] = IDTEntry::new(keyboard_interrupt as _);
        idt
    };

    static ref GAME: Mutex<Game> = Mutex::new(Game::new());

    static ref SCREEN: Mutex<&'static mut [[u16; 80]; 25]> = Mutex::new(unsafe {
        &mut *(0xB8000 as *mut [[u16; 80]; 25])
    });
}

static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(0x20, 0x28)
});

fn print(color: u16, y0: usize, x0: usize, s: &str) {
    let mut screen = SCREEN.lock();
    let mut x = x0;
    let mut y = y0;
    for &c in s.as_bytes() {
        if c == b'\n' {
            y += 1;
            x = x0;
        } else {
            screen[y][x] = color | c as u16;
            x += 1;
        }
    }
}

fn print_score(y: usize, mut x: usize, mut score: u32) {
    let mut screen = SCREEN.lock();

    let start = x - 10;

    if score == 0 {
        screen[y][x] = WHITE | 0x30;
        x -= 1;
    } else {
        while score != 0 {
            screen[y][x] = WHITE | 0x30 | (score % 10) as u16;
            x -= 1;
            score /= 10;
        }
    }

    while x != start {
        screen[y][x] = WHITE | 0x20;
        x -= 1;
    }
}

#[no_mangle]
pub extern fn start() {
    // Disable the cursor.

    unsafe {
        x86::io::outb(0x3D4, 0x0A);
        x86::io::outb(0x3D5, 0x20);
    }

    // Print the instructions.

    print(WHITE, 0, 50, screens::INSTRUCTIONS);

    // Set things up so that we start receiving interrupts.

    unsafe {
        enable_interrupts();
    }
}

unsafe fn enable_interrupts() {
    // Tell the computer where to find the interrupt table.

    x86::dtables::lidt(&x86::dtables::DescriptorTablePointer {
        limit: (256 * mem::size_of::<IDTEntry>() - 1) as u16,
        base: IDT.as_ptr(),
    });

    // Initialize the PICs.

    PICS.lock().initialize();

    // Enable interrupts.

    x86::irq::enable();
}

extern "x86-interrupt" fn double_fault(_: *const c_void) {}

extern "x86-interrupt" fn timer_interrupt(_: *const c_void) {
    let mut game = GAME.lock();

    game.tick();

    match game.state {
        game::State::Title => {
            print(WHITE, 0, 0, screens::TITLE);
        }
        game::State::Countdown(n) => {
            let screen = match n {
                0 => screens::ZERO,
                1 => screens::ONE,
                2 => screens::TWO,
                _ => screens::THREE,
            };

            print(WHITE, 0, 0, screen);
        }
        game::State::Main => {
            let mut screen = SCREEN.lock();

            for y in 0..25 {
                for x in 0..50 {
                    screen[y][x] = match game.board[[y, x]] {
                        game::Cell::Blue => 0x1020,
                        game::Cell::Red => 0x4020,
                        game::Cell::Empty => 0x0220,
                    };
                }
            }

            screen[game.blue.pos[0]][game.blue.pos[1]] = 0x1020;
            screen[game.red.pos[0]][game.red.pos[1]] = 0x4020;
        },
        game::State::Death => {
            let (screen, color) = match (game.blue.alive, game.red.alive) {
                (false, false) => (screens::TIE, WHITE),
                (false, true) => (screens::WIN, RED),
                (true, false) => (screens::WIN, BLUE),
                (true, true) => unreachable!(),
            };

            print(color, 0, 0, screen);
        }
        game::State::Paused => {
            print(WHITE, 0, 0, screens::PAUSED);
        }
    }

    print_score(7, 78, game.blue.score);
    print_score(8, 78, game.red.score);

    unsafe {
        PICS.lock().notify_end_of_interrupt(0x20);
    }
}

extern "x86-interrupt" fn keyboard_interrupt(_: *const c_void) {
    let code = unsafe { x86::io::inb(0x60) };
    let mut game = GAME.lock();

    match code {
        57 => game.start(), // Space
        25 => game.pause(), // P
        16 => game.quit(), // Quit

        17 => game.input(false, Direction::Up), // W
        31 => game.input(false, Direction::Down), // S
        30 => game.input(false, Direction::Left), // A
        32 => game.input(false, Direction::Right), // D

        72 => game.input(true, Direction::Up), // Up
        80 => game.input(true, Direction::Down), // Down
        75 => game.input(true, Direction::Left), // Left
        77 => game.input(true, Direction::Right), // Right

        _ => {}
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(0x21);
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct IDTEntry {
    /// The address of the handler's entrypoint.
    offset_lo: u16,
    /// The segment (always 0x08).
    selector: u16,
    /// Flags.
    flags: u16,
    /// The rest of the offset.
    offset_hi: u16,
}

impl IDTEntry {
    const fn new(offset: u32) -> Self {
        Self {
            offset_lo: offset as _,
            selector: 0x08,
            flags: 0b10001110_00000000,
            offset_hi: (offset >> 16) as _,
        }
    }

    const fn unused() -> Self {
        Self {
            offset_lo: 0,
            selector: 0,
            flags: 0,
            offset_hi: 0,
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
