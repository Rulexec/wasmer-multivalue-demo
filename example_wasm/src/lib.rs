#![no_std]

extern crate alloc;

use crate::debug_print::debug_print_string;
use alloc::format;

mod debug_print;
mod global_allocator;
mod panic;

#[link(wasm_import_module = "debug")]
extern "C" {
    fn test_multivalue_dynamic() -> (i32, i32, i32, i32);
    fn test_multivalue_static() -> (i32, i32, i32, i32);
}

#[no_mangle]
pub extern "C" fn run() {
    let result = unsafe { test_multivalue_dynamic() };

    debug_print_string(format!("dynamic {result:?}"));

    let result = unsafe { test_multivalue_static() };

    debug_print_string(format!("static {result:?}"));
}
