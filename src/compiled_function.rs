use std::sync::{Arc, RwLock};

use cranelift_jit::JITModule;
use cranelift_module::FuncId;

use crate::{ctx::ProcessorContext, env::ProcessorEnv};

pub struct CompiledFunction {
    pub func_ptr: *const u8,
    pub module: Arc<RwLock<JITModule>>,
}

impl CompiledFunction {
    pub fn new(module: Arc<RwLock<JITModule>>, func: FuncId) -> CompiledFunction {
        let func_ptr = module.read().unwrap().get_finalized_function(func);
        Self { func_ptr, module }
    }

    pub fn exec<T: ProcessorEnv>(&self, ctx: &mut ProcessorContext<T>) {
        let func = unsafe {
            std::mem::transmute::<_, extern "C" fn(*mut ProcessorContext<T>)>(self.func_ptr)
        };
        func(ctx)
    }
}
