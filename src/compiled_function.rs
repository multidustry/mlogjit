use std::sync::{Arc, RwLock};

use cranelift_jit::JITModule;
use cranelift_module::FuncId;

use crate::{ctx::ProcessorContext, env::ProcessorEnv};

pub struct CompiledFunction {
    pub func: FuncId,
    pub module: Arc<RwLock<JITModule>>,
}

impl CompiledFunction {
    pub fn exec<T: ProcessorEnv>(&self, ctx: &mut ProcessorContext<T>) {
        let module = self.module.read().unwrap();
        let ptr = module.get_finalized_function(self.func);
        let func =
            unsafe { std::mem::transmute::<_, extern "C" fn(*mut ProcessorContext<T>)>(ptr) };
        func(ctx)
    }
}
