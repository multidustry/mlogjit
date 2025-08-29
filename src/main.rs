use std::time::{Duration, Instant};

use env::DummyProcessorEnv;
use jit::JitCompiler;
use log::info;
use mlogjit::{ctx::ProcessorContext, env, jit, parser};
use parser::parse_code;

static CODE: &str = r#"
set a 10
set b 20
set b 15
set c 30
set d c
op pow result b d
op pow result2 b a
"#;

static CODE2: &str = r#"
set a 10
set b 2
op pow result a b
"#;
fn main() {
    pretty_env_logger::init();

    info!("Code: \n {}", CODE);
    let binding = parse_code(CODE);
    let start_compiling_time = Instant::now();
    let ir: Vec<_> = binding.iter().filter_map(|res| res.as_ref().ok()).collect(); // В проде лучше проверять есть ли ошибки в mlog коде а не просто отбрасывать их
    ir.iter().for_each(|it| info!("{:?}", it));

    let mut compiler = JitCompiler::new();
    let func_ptr = compiler.compile(&ir);

    info!(
        "Time for compiling mlog: {} ns",
        start_compiling_time.elapsed().as_nanos()
    );

    let env = DummyProcessorEnv {};
    let mut ctx = ProcessorContext::new(env);

    let jit_func = unsafe {
        std::mem::transmute::<_, extern "C" fn(*mut ProcessorContext<DummyProcessorEnv>)>(func_ptr)
    };

    info!("{:?}", ctx);

    let start_jit_time = Instant::now();
    jit_func(&mut ctx as *mut _);
    info!(
        "Time for running jit func: {} ns",
        start_jit_time.elapsed().as_nanos()
    );
    info!("{:?}", ctx);

    let binding2 = parse_code(CODE2);
    let start_compiling_time_2 = Instant::now();

    let ir2: Vec<_> = binding2
        .iter()
        .filter_map(|res| res.as_ref().ok())
        .collect(); // В проде лучше проверять есть ли ошибки в mlog коде а не просто отбрасывать их
    let func_ptr2 = compiler.compile(&ir2);

    info!(
        "Time for compiling mlog: {} ns",
        start_compiling_time_2.elapsed().as_nanos()
    );
    let env2 = DummyProcessorEnv {};
    let mut ctx2 = ProcessorContext::new(env2);

    let jit_func2 = unsafe {
        std::mem::transmute::<_, extern "C" fn(*mut ProcessorContext<DummyProcessorEnv>)>(func_ptr2)
    };

    info!("{:?}", ctx2);

    let start_jit_time2 = Instant::now();
    jit_func2(&mut ctx2 as *mut _);
    info!(
        "Time for running jit func: {} ns",
        start_jit_time2.elapsed().as_nanos()
    );
    info!("{:?}", ctx2);
}
