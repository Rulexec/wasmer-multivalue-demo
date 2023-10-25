use crate::wasm::{NewWasmInstanceOptions, Wasm};

mod wasm;

fn main() {
    let wasm = Wasm {
        bytes: include_bytes!(
            "../../example_wasm/target/wasm32-unknown-unknown/release/example_wasm.wasm"
        ),
    }
    .new_wasm_instance(NewWasmInstanceOptions { config_path: "" })
    .unwrap();

    wasm.run("run").unwrap();
}
