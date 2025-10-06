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

fn main() {
    pretty_env_logger::init();

    info!("Code: \n {}", CODE);
    let binding = parse_code(CODE);
    let start_compiling_time = Instant::now();
    let ir: Vec<_> = binding.iter().filter_map(|res| res.as_ref().ok()).collect(); // В проде лучше проверять есть ли ошибки в mlog коде а не просто отбрасывать их
    ir.iter().for_each(|it| info!("{:?}", it));

    let mut compiler = JitCompiler::new();
    let func = compiler.compile(&ir);

    info!(
        "Time for compiling mlog: {} ns",
        start_compiling_time.elapsed().as_nanos()
    );

    let env = DummyProcessorEnv {};
    let mut ctx = ProcessorContext::new(env);

    info!("{:?}", ctx);

    let start_jit_time = Instant::now();
    func.exec(&mut ctx);
    info!(
        "Time for running jit func: {} ns",
        start_jit_time.elapsed().as_nanos()
    );
    info!("{:?}", ctx);
}
