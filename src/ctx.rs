use crate::env::ProcessorEnv;

#[repr(C)]
#[derive(Debug)]
pub struct ProcessorContext<T: ProcessorEnv> {
    pub registers: [f64; 256],
    pub env: T,
}

impl<T: ProcessorEnv> ProcessorContext<T> {
    pub fn new(env: T) -> Self {
        Self {
            registers: [0.0; 256],
            env: env,
        }
    }
}
