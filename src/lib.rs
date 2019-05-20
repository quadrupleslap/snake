#![no_std]
#![feature(abi_x86_interrupt)]

use lazy_static::lazy_static;
use core::ffi::c_void;
use core::mem;
use spin::Mutex;
use pic8259_simple::ChainedPics;

#[macro_use]
mod print;

lazy_static! {
    static ref IDT: [IDTEntry; 256] = {
        let mut idt = [IDTEntry::unused(); 256];
        idt[0x08] = IDTEntry::new(double_fault as _);
        idt[0x20] = IDTEntry::new(timer_interrupt as _);
        idt[0x21] = IDTEntry::new(keyboard_interrupt as _);
        idt
    };
}

static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(0x20, 0x28)
});

#[no_mangle]
pub extern fn start() {
    //TODO: Initialize the game.

    // Set up the IDT and enable interrupts.

    unsafe {
        load_idt();
    }
}

unsafe fn load_idt() {
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
    // print!("TODO: Timer interrupt.\n");

    unsafe {
        PICS.lock().notify_end_of_interrupt(0x20);
    }
}

extern "x86-interrupt" fn keyboard_interrupt(_: *const c_void) {
    let code = unsafe {
        x86::io::inb(0x60)
    };

    print!("TODO: Keyboard interrupt: {}.\n", code);

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
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("{}\n", info);
    loop {}
}
