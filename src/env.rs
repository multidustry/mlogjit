pub trait ProcessorEnv {}

#[derive(Debug)]
pub struct DummyProcessorEnv {}

impl ProcessorEnv for DummyProcessorEnv {}
