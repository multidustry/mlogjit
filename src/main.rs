use ir::Operand;
use log::info;
use parser::parse_line;

pub mod ir;
pub mod jit;
pub mod parser;

fn main() {
    pretty_env_logger::init();

    let code = r#"set a 10
        set b 10
        op add result a b"#;
    let mut ir = Vec::new();
    for i in code.lines() {
        let oper = parse_line(i);
        info!("{:?}", oper);
        if let Ok(ir_instr) = oper {
            ir.push(ir_instr);
        }
    }
}
