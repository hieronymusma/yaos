#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(panic_info_message)]
#![feature(pointer_byte_offsets)]
#![feature(strict_provenance)]
#![feature(nonzero_ops)]

mod asm;
mod heap;
mod mmio;
mod page_allocator;
mod println;
mod uart;
mod util;

use core::{cmp::Ordering, panic::PanicInfo};

// extern crate alloc;

extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

#[no_mangle]
extern "C" fn kernel_init() {
    uart::QEMU_UART.init();
    println!("Hello World from YaROS!");
    unsafe {
        page_allocator::init(HEAP_START, HEAP_SIZE);
    }

    loop {
        let page = page_allocator::zalloc();
        if page.is_none() {
            break;
        }
        println!("Allocated {:?}", page);
        unsafe {
            assert!(page.unwrap().addr().as_ref().cmp(&[0; 4096]) == Ordering::Equal);
        }
    }

    // heap::init();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic Occured!");
    if let Some(message) = info.message() {
        println!("Message: {}", message);
    }
    if let Some(location) = info.location() {
        println!("Location: {}", location);
    }
    loop {}
}
