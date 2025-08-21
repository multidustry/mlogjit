#[derive(PartialEq, PartialOrd, Debug)]
pub enum Operand {
    Null,
    Const(f64),
    Var(String),
    String(String),
}

#[derive(Debug)]
pub enum OpKind {
    Add,
    Sub,
    Mul,
    Div,
    Idiv,
    Mod,
    Pow,
}

#[derive(Debug)]
pub enum Instr {
    Set(String, Operand),
    Op(String, OpKind, Operand, Option<Operand>),
}
