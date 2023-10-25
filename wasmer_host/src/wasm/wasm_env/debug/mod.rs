use crate::wasm::wasm_env::WasmEnv;
use crate::wasm::WasmError;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use wasmer::{
    Function, FunctionEnv, FunctionEnvMut, Imports, Memory, RuntimeError, Store, Value, WasmPtr,
};
use wasmer_types::{FunctionType, Type};

impl WasmEnv {
    pub fn register_debug_wasm_imports(&self, store: &mut Store, imports: &mut Imports) {
        struct DebugEnv {
            error: Arc<Mutex<Option<WasmError>>>,
            memory: Arc<Mutex<Option<Memory>>>,
        }

        let env = FunctionEnv::new(
            store,
            DebugEnv {
                error: self.error.clone(),
                memory: self.memory.clone(),
            },
        );

        fn print(mut env_mut: FunctionEnvMut<DebugEnv>, s: WasmPtr<u8>, s_size: i32) -> () {
            let (env, store) = env_mut.data_and_store_mut();
            let DebugEnv { error, memory } = env;

            let result = (|| {
                let view = WasmEnv::memory_view(memory, &store)?;

                let s = s.slice(&view, s_size as u32)?;
                let s = s.access()?;
                let s = from_utf8(s.as_ref()).map_err(WasmError::Utf8)?;

                println!("WASM: {s}");

                Ok::<_, WasmError>(())
            })();

            () = WasmEnv::handle_error(error, result).unwrap_or(());
        }

        imports.define(
            "debug",
            "print",
            Function::new_typed_with_env(store, &env, print),
        );

        fn test_multivalue_dynamic(
            mut env_mut: FunctionEnvMut<DebugEnv>,
            args: &[Value],
        ) -> Result<Vec<Value>, RuntimeError> {
            Ok(vec![
                Value::I32(1),
                Value::I32(2),
                Value::I32(3),
                Value::I32(4),
            ])
        }

        imports.define(
            "debug",
            "test_multivalue_dynamic",
            Function::new_with_env(
                store,
                &env,
                FunctionType::new([], [Type::I32, Type::I32, Type::I32, Type::I32]),
                test_multivalue_dynamic,
            ),
        );

        fn test_multivalue_static(mut env_mut: FunctionEnvMut<DebugEnv>) -> (i32, i32, i32, i32) {
            (1, 2, 3, 4)
        }

        imports.define(
            "debug",
            "test_multivalue_static",
            Function::new_typed_with_env(store, &env, test_multivalue_static),
        );
    }
}
