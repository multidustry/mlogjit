# Mlogjit

JIT compiler for MLOG (Mindustry Logic) using Cranelift.

Compiles MLOG instructions to native machine code for high-performance execution.

## Features

- Parse MLOG text into intermediate representation
- JIT compilation to native code via Cranelift
- ~100x faster than interpreted execution
- Support for variables, constants, and arithmetic operations

## Quick Start

```rust
use mlogjit::{JitCompiler, ProcessorContext, DummyProcessorEnv, parse_code};

// MLOG code
let code = r#"
set a 10
set b 2
op pow result a b
"#;

// Parse to IR
let parsed = parse_code(code);
let ir: Vec<_> = parsed.iter().filter_map(|r| r.as_ref().ok()).collect();

// Compile to native code
let mut compiler = JitCompiler::new();
let func_ptr = compiler.compile(&ir);

// Execute
let mut ctx = ProcessorContext::new(DummyProcessorEnv {});
let jit_func = unsafe {
    std::mem::transmute::<_, extern "C" fn(*mut ProcessorContext<DummyProcessorEnv>)>(func_ptr)
};

jit_func(&mut ctx);

// Access results in ctx.registers
println!("Result: {}", ctx.registers); // result variable
```

## API

### `JitCompiler`
Main compiler instance. Call `compile(&[&Instr])` to generate native code.

### `ProcessorContext<T>`
Execution context with 256 f64 registers and custom environment.

### `parse_code(code: &str)`
Parses MLOG text into `Vec<Result<Instr, ParseError>>`.

### Supported Operations
- `set` - assign value to variable
- `op add/sub/mul/div/idiv/mod/pow` - arithmetic operations

## Performance

Typical compilation: ~1-5 microseconds  
Typical execution: ~100-500 nanoseconds (depends on code complexity)

## Status

🚧 Early development - basic operations work, more instructions coming soon.

## License

Apache-2.0
