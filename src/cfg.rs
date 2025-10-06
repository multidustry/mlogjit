use std::collections::{BTreeMap, BTreeSet};

use cranelift_frontend::FunctionBuilder;

use crate::ir::Instr;

pub fn generate_cfg(fb: FunctionBuilder, instrs: &[&Instr]) {
    let mut jumps_lines = BTreeSet::new();
    for instr in instrs {
        if let Instr::Jump(line, _, _, _) = instr {
            jumps_lines.insert(line);
        }
    }
}
