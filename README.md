# Mlogjit

JIT compiler for MLOG (Mindustry Logic) using Cranelift.

Compiles MLOG instructions to native machine code for high-performance execution.

## Features

- Parse MLOG text into intermediate representation
- JIT compilation to native code via Cranelift
- ~100x faster than interpreted execution
- Support for variables, constants, and arithmetic operations

## Quick Start

See [examples](examples/main.rs) 

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

## TODO:
1. CompiledFunction struct с Arc<JITModule>
2. Метод execute() вместо сырого указателя
3. Top-level compile() функция для простоты
4. Nightly feature для Fn traits
5. Обновить README с новыми примерами
6. Тест на dangling pointer (должен падать сейчас)
