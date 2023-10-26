use wasmer::{Function, FunctionType, Imports, Instance, Module, RuntimeError, Store, Type, TypedFunction, Value, wat2wasm};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_bytes = wat2wasm(
        br#"
(module
  (type $t0 (func (result i32 i32 i32 i32)))
  (type $t1 (func (result i32)))
  (import "debug" "test_multivalue" (func $debug.test_multivalue (type $t0)))
  (func $run (export "run") (type $t1) (result i32)
    (local $l0 i32) (local $l1 i32) (local $l2 i32)
    (call $debug.test_multivalue)
    (local.set $l2)
    (local.set $l1)
    (local.set $l0)
    (i32.const 1)
    (i32.add
      (i32.add
        (i32.add
          (i32.shl)
          (i32.mul
            (local.get $l0)
            (i32.const 3)))
        (i32.mul
          (local.get $l1)
          (i32.const 7)))
      (i32.mul
        (local.get $l2)
        (i32.const 11))))
  (memory $memory (export "memory") 16)
  (global $g0 (mut i32) (i32.const 1048576))
  (global $__data_end (export "__data_end") i32 (i32.const 1048576))
  (global $__heap_base (export "__heap_base") i32 (i32.const 1048576)))
"#,
    )?;

    let mut store = Store::default();
    let module = Module::new(&store, wasm_bytes)?;

    let mut imports_dynamic = Imports::new();

    fn test_multivalue_dynamic(
        _args: &[Value],
    ) -> Result<Vec<Value>, RuntimeError> {
        Ok(vec![
            Value::I32(1),
            Value::I32(2),
            Value::I32(3),
            Value::I32(4),
        ])
    }

    imports_dynamic.define(
        "debug",
        "test_multivalue",
        Function::new(
            &mut store,
            FunctionType::new([], [Type::I32, Type::I32, Type::I32, Type::I32]),
            test_multivalue_dynamic,
        ),
    );

    let mut imports_static = Imports::new();

    fn test_multivalue_static() -> (i32, i32, i32, i32) {
        (1, 2, 3, 4)
    }

    imports_static.define(
        "debug",
        "test_multivalue",
        Function::new_typed(&mut store, test_multivalue_static),
    );

    let instance_dynamic = Instance::new(&mut store, &module, &imports_dynamic)?;
    let instance_static = Instance::new(&mut store, &module, &imports_static)?;

    let run_dynamic: TypedFunction<(), i32> = instance_dynamic.exports.get_typed_function(&store, "run")?;
    let run_static: TypedFunction<(), i32> = instance_static.exports.get_typed_function(&store, "run")?;

    let dynamic_result = run_dynamic.call(&mut store)?;
    let static_result = run_static.call(&mut store)?;

    println!("dynamic multivalue: {}", dynamic_result);
    println!("static multivalue: {}", static_result);

    Ok(())
}