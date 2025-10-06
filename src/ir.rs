#[derive(PartialEq, PartialOrd, Debug)]
pub enum Operand {
    Null,
    Const(f64),
    Var(String),
    String(String),
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum OpKind {
    Add,
    Sub,
    Mul,
    Div,
    Idiv,
    Mod,
    Pow,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum JumpCondition {
    Equal,
    NotEqual,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    StrictEqual,
    Always,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Instr {
    Set(String, Operand),
    Op(String, OpKind, Operand, Operand),
    Jump(i32, JumpCondition, Operand, Operand),
}
