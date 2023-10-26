#![no_std]

mod panic;

#[repr(C)]
struct FourValues {
    a: i32,
    b: i32,
    c: i32,
    d: i32,
}

#[link(wasm_import_module = "debug")]
extern "C" {
    fn test_multivalue() -> FourValues;
}

#[no_mangle]
pub extern "C" fn run() -> i32 {
    let FourValues { a, b, c, d } = unsafe { test_multivalue() };

    a * 2 + b * 3 + c * 7 + d * 11
}
