use std::sync::Arc;

use cranelift_jit::JITModule;
use cranelift_module::FuncId;

use crate::{ctx::ProcessorContext, env::ProcessorEnv};

pub struct CompiledFunction {
    pub func: FuncId,
    pub module: Arc<JITModule>,
}

impl CompiledFunction {
    pub fn exec<T: ProcessorEnv>(&self, ctx: &mut ProcessorContext<T>) {
        let ptr = self.module.get_finalized_function(self.func);
        let func =
            unsafe { std::mem::transmute::<_, extern "C" fn(*mut ProcessorContext<T>)>(ptr) };
        func(ctx)
    }
}
