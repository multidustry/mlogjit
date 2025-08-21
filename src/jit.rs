use core::panic;
use std::collections::HashMap;

use cranelift_codegen::{
    bforest::Set,
    gimli::DW_OP_const1s,
    ir::{AbiParam, InstBuilder, ValueLabelStart},
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;

use crate::ir::{Instr, Operand};

#[repr(C)]
pub struct ProcessorContext {
    pub registers: [f64; 256],
}

pub struct SymbolTable {
    vars: HashMap<String, usize>,
    next: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            next: 0,
        }
    }

    pub fn index_for(&mut self, name: &str) -> usize {
        if let Some(&i) = self.vars.get(name) {
            i
        } else {
            let i = self.next;
            self.vars.insert(name.to_string(), i);
            self.next += 1;
            i
        }
    }
}

pub struct JitCompiler {
    pub module: JITModule,
}

impl JitCompiler {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        let module = JITModule::new(builder);
        Self { module: module }
    }

    pub fn compile(&mut self, ir: &[Instr]) -> *const u8 {
        let mut sig = self.module.make_signature();
        // fn(ctx: *mut ProcessorContext)
        sig.params
            .push(AbiParam::new(self.module.target_config().pointer_type()));
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let entry = fb.create_block();
        fb.append_block_params_for_function_params(entry);
        fb.switch_to_block(entry);
        fb.seal_block(entry);

        let ctx_ptr = fb.block_params(entry)[0];
        let ptr_ty = self.module.target_config().pointer_type();

        let mut symtab = SymbolTable::new();

        for instr in ir {
            match instr {
                Instr::Set(var, oper) => {
                    compile_set(&mut fb, &mut symtab, var, oper);
                }
                _ => panic!("Unsupported instruction"),
            }
        }

        fb.ins().return_(&[]);
        fb.finalize();

        let func_id = self
            .module
            .declare_function(
                "jit_func",
                cranelift_module::Linkage::Export,
                &ctx.func.signature,
            )
            .unwrap();
        self.module.define_function(func_id, &mut ctx);
        self.module.clear_context(&mut ctx);
        self.module.finalize_definitions();

        self.module.get_finalized_function(func_id)
    }
}

fn compile_set(fb: &mut FunctionBuilder, symtab: &mut SymbolTable, var: &String, oper: &Operand) {
    match oper {
        Operand::Const(cons) => {
            let idx = symtab.index_for(var);
        }
        _ => panic!("Unsupported oper"),
    }
}
