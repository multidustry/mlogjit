use crate::ctx::ProcessorContext;
use env::DummyProcessorEnv;
use jit::JitCompiler;
use log::info;
use parser::{parse_code, parse_line};

pub mod ctx;
pub mod env;
pub mod ir;
pub mod jit;
pub mod parser;

static CODE: &str = r#"
set a 10
set b 20
set b 15
set c 30
set d c
op add result b d
"#;
fn main() {
    pretty_env_logger::init();

    info!("Code: \n {}", CODE);
    let binding = parse_code(CODE);
    let ir: Vec<_> = binding.iter().filter_map(|res| res.as_ref().ok()).collect(); // В проде лучше проверять есть ли ошибки в mlog коде а не просто отбрасывать их
    let mut compiler = JitCompiler::new();
    let func_ptr = compiler.compile(&ir);

    let env = DummyProcessorEnv {};
    let mut ctx = ProcessorContext::new(env);

    let jit_func = unsafe {
        std::mem::transmute::<_, extern "C" fn(*mut ProcessorContext<DummyProcessorEnv>)>(func_ptr)
    };

    info!("{:?}", ctx);

    jit_func(&mut ctx as *mut _);

    info!("{:?}", ctx);
}
