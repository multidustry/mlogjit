use core::{fmt, panic};
use cranelift_codegen::{
    bforest::Set,
    ir::{AbiParam, FuncRef, InstBuilder, MemFlags, MemoryType, Type, Value, types},
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Module};
use std::sync::Arc;
use std::{cmp::Ordering, collections::HashMap, env::VarError};

use crate::{
    compiled_function::CompiledFunction,
    ctx::ProcessorContext,
    env::DummyProcessorEnv,
    ir::{Instr, OpKind, Operand},
    oper_functions::pow::host_pow,
    symbol_table::SymbolTable,
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

pub struct HostFunctions {
    pub pow_func_id: FuncId,
}

pub struct FuncRefs {
    pub pow_func_ref: FuncRef,
}

impl FuncRefs {
    pub fn new(compiler: &mut JitCompiler, fb: &mut FunctionBuilder) -> Self {
        Self {
            pow_func_ref: compiler
                .module
                .declare_func_in_func(compiler.host_functions.pow_func_id, fb.func),
        }
    }
}

pub struct HostFunctions {
    pub pow_func_id: FuncId,
}

impl HostFunctions {
    pub fn register_symbols(jit_builder: &mut JITBuilder) {
        jit_builder.symbol("host_pow", host_pow as *const u8);
    }

    pub fn new(module: &mut JITModule) -> Self {
        let pow_func_id = Self::declare_binary(module, "host_pow");
        Self {
            pow_func_id: pow_func_id,
        }
    }

    fn declare_binary(module: &mut JITModule, name: &str) -> cranelift_module::FuncId {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(types::F64));
        sig.params.push(AbiParam::new(types::F64));
        sig.returns.push(AbiParam::new(types::F64));

        module
            .declare_function(name, cranelift_module::Linkage::Import, &sig)
            .unwrap()
    }
    fn declare_unary(module: &mut JITModule, name: &str) -> cranelift_module::FuncId {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(types::F64));
        sig.returns.push(AbiParam::new(types::F64));

        module
            .declare_function(name, cranelift_module::Linkage::Import, &sig)
            .unwrap()
    }
}

pub struct JitCompiler {
    pub module: Arc<JITModule>,
    pub host_functions: HostFunctions,
}

impl JitCompiler {
    pub fn new() -> Self {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        HostFunctions::register_symbols(&mut builder);
        let mut module = JITModule::new(builder);
        let host_functions = HostFunctions::new(&mut module);
        Self {
            module: module,
            host_functions: host_functions,
        }
    }

    /// Compiles mlog ir into jit function.
    pub fn compile(&mut self, ir: &[&Instr]) -> CompiledFunction {
        let mut sig = self.module.make_signature();
        // fn(ctx: *mut ProcessorContext)
        sig.params
            .push(AbiParam::new(self.module.target_config().pointer_type()));
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let mut func_refs = FuncRefs::new(self, &mut fb);
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
                Instr::Op(var, opkind, left, right) => {
                    self.compile_op(
                        &mut fb,
                        &ctx_ptr,
                        &mut symtab,
                        &mut func_refs,
                        var,
                        opkind,
                        left,
                        right,
                    );
                }
                _ => panic!("Unsupported instruction"),
            };
        }

        fb.ins().return_(&[]);
        fb.finalize();

        let func_id = self
            .module
            .declare_function(
                &format!("jit_func_{:?}", ir),
                cranelift_module::Linkage::Export,
                &ctx.func.signature,
            )
            .unwrap();
        let _ = self.module.define_function(func_id, &mut ctx).unwrap();
        self.module.clear_context(&mut ctx);
        let _ = self.module.finalize_definitions();

        CompiledFunction {
            func: func_id,
            module: Arc::clone(&self.module),
        }
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
        func_refs: &mut FuncRefs,
        var: &str,
        opkind: &OpKind,
        left: &Operand,
        right: &Operand,
    ) {
        let left_ssa = oper_helper(fb, ctx_ptr, symtab, left);
        let right_ssa = oper_helper(fb, ctx_ptr, symtab, right);
        let var_offset = (symtab.index_for(var) * 8) as i32;
        let result = match opkind {
            OpKind::Add => fb.ins().fadd(left_ssa, right_ssa),
            OpKind::Sub => fb.ins().fsub(left_ssa, right_ssa),
            OpKind::Mul => fb.ins().fmul(left_ssa, right_ssa),
            OpKind::Div => fb.ins().fdiv(left_ssa, right_ssa),
            OpKind::Idiv => {
                let t = fb.ins().fdiv(left_ssa, right_ssa);
                fb.ins().floor(t)
            }
            OpKind::Mod => {
                let div = fb.ins().fdiv(left_ssa, right_ssa);
                let floor_div = fb.ins().floor(div);
                let mult = fb.ins().fmul(floor_div, right_ssa);
                fb.ins().fsub(left_ssa, mult)
            }
            OpKind::Pow => {
                let call = fb
                    .ins()
                    .call(func_refs.pow_func_ref, &[left_ssa, right_ssa]);
                fb.inst_results(call)[0]
            }
            _ => todo!("This opkind don't supported now: {:?}", opkind),
        };
        fb.ins()
            .store(MemFlags::new(), result, *ctx_ptr, var_offset);
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
