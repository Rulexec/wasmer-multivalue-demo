use std::sync::{Arc, Mutex};

use wasmer::{AsStoreRef, Imports, Memory, MemoryView, Store};

use crate::wasm::{Allocation, WasmError};

pub mod debug;
pub mod memory;

pub struct WasmEnv {
    error: Arc<Mutex<Option<WasmError>>>,
    memory: Arc<Mutex<Option<Memory>>>,
    allocation: Arc<Mutex<Option<Allocation>>>,
}

impl WasmEnv {
    pub fn new() -> Self {
        Self {
            error: Arc::new(Mutex::new(None)),
            memory: Arc::new(Mutex::new(None)),
            allocation: Arc::new(Mutex::new(None)),
        }
    }

    pub fn register_imports(&self, store: &mut Store, imports: &mut Imports) {
        self.register_debug_wasm_imports(store, imports);
    }

    pub fn memory_view<'a>(
        memory: &Arc<Mutex<Option<Memory>>>,
        store: &'a (impl AsStoreRef + ?Sized),
    ) -> Result<MemoryView<'a>, WasmError> {
        let lock = memory.lock().map_err(|_| WasmError::MutexPoisoned)?;
        let memory = lock.as_ref().ok_or_else(|| WasmError::NoMemory)?;
        let view = memory.view(store);

        Ok(view)
    }

    pub fn handle_error<T>(
        error: &Arc<Mutex<Option<WasmError>>>,
        result: Result<T, WasmError>,
    ) -> Option<T> {
        let wasm_err = match result {
            Ok(x) => {
                return Some(x);
            }
            Err(x) => x,
        };

        let Ok(mut lock) = error.try_lock() else {
            // If cannot take mutex, then someone took it to set error
            return None;
        };

        if lock.is_some() {
            return None;
        }

        lock.replace(wasm_err);

        None
    }
}