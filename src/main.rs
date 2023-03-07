
#![no_std] // Don't link the Rust standard library
#![no_main] // Disable rust entry points
use core::panic::PanicInfo;
mod vga_buffer;
/// Because there's no std library, we must handle errors if they occur
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

/// No mangle ensures that rust does not output the function with a cryptic name to differentiate it
/// from other functions, instead it will remain _start to make it easier to reference the entry point
/// to the linker
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    loop {}
}


