use core::panic;
use std::{collections::HashMap, env::VarError};

use cranelift_codegen::ir::{AbiParam, InstBuilder, MemFlags, MemoryType, Type, Value, types};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;

use crate::{
    ctx::ProcessorContext,
    env::DummyProcessorEnv,
    ir::{Instr, OpKind, Operand},
};

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

    /// Compiles mlog ir into jit function.
    pub fn compile(&mut self, ir: &[&Instr]) -> *const u8 {
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

        let mut symtab = SymbolTable::new();

        for instr in ir {
            match instr {
                Instr::Set(var, oper) => {
                    self.compile_set(&mut fb, &ctx_ptr, &mut symtab, var, oper);
                }
                Instr::Op(var, opkind, left, Some(right)) => {
                    self.compile_op(&mut fb, &ctx_ptr, &mut symtab, var, opkind, left, right);
                }
                _ => panic!("Unsupported instruction"),
            };
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
        let _ = self.module.define_function(func_id, &mut ctx);
        self.module.clear_context(&mut ctx);
        let _ = self.module.finalize_definitions();

        self.module.get_finalized_function(func_id)
    }

    fn compile_set(
        &mut self,
        fb: &mut FunctionBuilder,
        ctx_ptr: &Value,
        symtab: &mut SymbolTable,
        var: &str,
        oper: &Operand,
    ) {
        // match oper {
        //     Operand::Const(val) => {
        //         let idx = symtab.index_for(var);
        //         let offset = (idx * 8) as i32;
        //         let val_ir = fb.ins().f64const(*val);

        //         fb.ins().store(MemFlags::new(), val_ir, *ctx_ptr, offset)
        //     }
        //     _ => panic!("Unsupported oper"),
        // };
        let offset = (symtab.index_for(var) * 8) as i32;
        let val = oper_helper(fb, ctx_ptr, symtab, oper);
        fb.ins().store(MemFlags::new(), val, *ctx_ptr, offset);
    }

    fn compile_op(
        &mut self,
        fb: &mut FunctionBuilder,
        ctx_ptr: &Value,
        symtab: &mut SymbolTable,
        var: &str,
        opkind: &OpKind,
        left: &Operand,
        right: &Operand,
    ) {
        match opkind {
            OpKind::Add => {
                let left_ssa = oper_helper(fb, ctx_ptr, symtab, left);
                let right_ssa = oper_helper(fb, ctx_ptr, symtab, right);
                let var_offset = (symtab.index_for(var) * 8) as i32;

                let result = fb.ins().fadd(left_ssa, right_ssa);
                fb.ins()
                    .store(MemFlags::new(), result, *ctx_ptr, var_offset);
            }
            _ => todo!("Only addition supported now"),
        }
    }
}

fn oper_helper(
    fb: &mut FunctionBuilder,
    ctx_ptr: &Value,
    symtab: &mut SymbolTable,
    oper: &Operand,
) -> Value {
    match oper {
        Operand::Const(val) => fb.ins().f64const(*val),
        Operand::Var(var) => {
            let offset = (symtab.index_for(var) * 8) as i32;
            fb.ins().load(types::F64, MemFlags::new(), *ctx_ptr, offset)
        }
        _ => todo!("String and Null not supported for now"),
    }
}

#[test]
fn test_set_op() {
    let binding = Instr::Set("a".into(), Operand::Const(10.0));
    let ir = vec![&binding];
    let mut compiler = JitCompiler::new();
    let func_ptr = compiler.compile(&ir);

    let env = DummyProcessorEnv {};
    let mut context = ProcessorContext::new(env);

    let jit_func = unsafe {
        std::mem::transmute::<_, extern "C" fn(*mut ProcessorContext<DummyProcessorEnv>)>(func_ptr)
    };

    jit_func(&mut context as *mut _);

    assert_eq!(10.0, context.registers[0]);
}
