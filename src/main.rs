use jit::{JitCompiler, ProcessorContext};
use log::info;
use parser::parse_line;

pub mod ir;
pub mod jit;
pub mod parser;

static CODE: &str = r#"
set a 10
set b 10
"#;
fn main() {
    pretty_env_logger::init();

    info!("Code: \n {}", CODE);
    let mut ir = Vec::new();
    for i in CODE.lines() {
        let oper = parse_line(i);
        info!("{:?}", oper);
        if let Ok(ir_instr) = oper {
            ir.push(ir_instr);
        }
    }

    let mut compiler = JitCompiler::new();
    let func_ptr = compiler.compile(&ir);

    let mut context = ProcessorContext {
        registers: [0.0; 256],
    };

    let jit_func =
        unsafe { std::mem::transmute::<_, extern "C" fn(*mut ProcessorContext)>(func_ptr) };

    info!("{:?}", context);

    jit_func(&mut context as *mut _);

    info!("{:?}", context);
}
