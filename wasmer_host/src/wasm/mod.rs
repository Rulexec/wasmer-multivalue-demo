use std::cell::RefCell;
use std::io::ErrorKind;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::rc::Rc;
use std::str::Utf8Error;

use thiserror::Error;
use wasmer::{
    CompileError, Cranelift, ExportError, FromToNativeWasmType, Imports, Instance,
    InstantiationError, Memory, MemoryAccessError, MemoryError, Module, RuntimeError, Store,
    TypedFunction, WasmPtr, WasmTypeList,
};
use wasmer_types::Features;

use crate::wasm::wasm_env::WasmEnv;

mod wasm_env;

pub struct Wasm {
    pub bytes: &'static [u8],
}

#[derive(Error, Debug)]
pub enum WasmError {
    #[error("AlreadyErrored")]
    AlreadyErrored,
    #[error("{0:?}")]
    Io(std::io::Error),
    #[error("{0:?}")]
    Compile(CompileError),
    #[error("{0:?}")]
    Instantiation(InstantiationError),
    #[error("{0:?}")]
    Memory(MemoryError),
    #[error("{original:?}: {context}")]
    Export {
        original: ExportError,
        context: String,
    },
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error("{0:?}")]
    MemoryAccess(MemoryAccessError),
    #[error("{0:?}")]
    Utf8(Utf8Error),
    #[error("MutexPoisoned")]
    MutexPoisoned,
    #[error("NoMemory")]
    NoMemory,
    #[error("NoAllocation")]
    NoAllocation,
    #[error("{0:?}")]
    Unspecified(String),
}

pub fn export_error_context<F: FnOnce() -> String>(
    context: F,
) -> impl FnOnce(ExportError) -> WasmError {
    |original| WasmError::Export {
        original,
        context: context(),
    }
}

impl From<MemoryAccessError> for WasmError {
    fn from(value: MemoryAccessError) -> Self {
        Self::MemoryAccess(value)
    }
}

pub struct NewWasmInstanceOptions<'a> {
    pub config_path: &'a str,
}

pub struct WasmModuleInstance {
    store: RefCell<Store>,
    instance: Instance,
}

#[derive(Clone)]
pub struct Allocation {
    alloc: TypedFunction<i32, WasmPtr<u8>>,
    dealloc: TypedFunction<(WasmPtr<u8>, i32), ()>,
    memory: Memory,
}

impl Wasm {
    pub fn new_wasm_instance(
        &self,
        options: NewWasmInstanceOptions<'_>,
    ) -> Result<WasmModuleInstance, WasmError> {
        let NewWasmInstanceOptions { config_path } = options;

        let mut store = Store::default();
        let wasm_mod = Module::new(&store, self.bytes).map_err(WasmError::Compile)?;

        let mut memories = wasm_mod.exports().memories();
        let Some(memory) = memories.next() else {
            return Err(WasmError::Unspecified(
                "Module does not exports memory".to_string(),
            ));
        };

        if memories.next().is_some() {
            return Err(WasmError::Unspecified(
                "Module exports multiple memories".to_string(),
            ));
        }

        let mut import_object = Imports::new();

        let env = WasmEnv::new();

        env.register_imports(&mut store, &mut import_object);

        let instance = Instance::new(&mut store, &wasm_mod, &import_object)
            .map_err(WasmError::Instantiation)?;

        let memory = instance
            .exports
            .get_memory(memory.name())
            .map_err(export_error_context(|| "memory".to_string()))?;

        env.set_memory(memory.clone());

        Ok(WasmModuleInstance {
            store: RefCell::new(store),
            instance,
        })
    }
}

impl WasmModuleInstance {
    pub fn run(&self, name: &str) -> Result<(), WasmError> {
        let fun: TypedFunction<(), ()> = {
            let store = self.store.borrow();

            self.instance
                .exports
                .get_typed_function(&store, name)
                .map_err(|original| WasmError::Export {
                    original,
                    context: "run".to_string(),
                })?
        };

        let mut store = self.store.borrow_mut();

        () = fun.call(&mut store)?;

        Ok(())
    }
}
