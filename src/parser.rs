use crate::ir::{Instr, OpKind, Operand};

#[derive(Debug)]
pub enum ParseError {
    WrongArgumentCount,
    UnknownInstruction(String),
}

pub fn parse_code(code: &str) -> Vec<Result<Instr, ParseError>> {
    code.lines()
        .into_iter()
        .filter(|x| !x.is_empty())
        .map(|line| parse_line(line))
        .collect()
}

pub fn parse_line(line: &str) -> Result<Instr, ParseError> {
    let tokens: Vec<String> = tokenizer(line);
    if tokens.is_empty() {
        return Err(ParseError::WrongArgumentCount);
    }

    match tokens[0].as_str() {
        "set" => {
            if tokens.len() != 3 {
                return Err(ParseError::WrongArgumentCount);
            }
            let var = &tokens[1];
            let oper = parse_operand(&tokens[2]);
            Ok(Instr::Set(var.to_string(), oper))
        }
        "op" => {
            if tokens.len() != 5 {
                return Err(ParseError::WrongArgumentCount);
            }
            let operation = match tokens[1].as_str() {
                "add" => OpKind::Add,
                "sub" => OpKind::Sub,
                "mul" => OpKind::Mul,
                "div" => OpKind::Div,
                "idiv" => OpKind::Idiv,
                "mod" => OpKind::Mod,
                "pow" => OpKind::Pow,
                _ => {
                    return Err(ParseError::UnknownInstruction(tokens.join(" ")));
                }
            };
            let var = &tokens[2];
            let op1 = parse_operand(&tokens[3]);
            let op2 = parse_operand(&tokens[4]);
            Ok(Instr::Op(var.to_string(), operation, op1, op2))
        }
        _ => Err(ParseError::UnknownInstruction(tokens[0].to_string())),
    }
}

fn parse_operand(token: &str) -> Operand {
    if token == "null" {
        Operand::Null
    } else if let Ok(val) = token.parse::<f64>() {
        Operand::Const(val)
    } else if token.starts_with("\"") {
        Operand::String(token.to_string().replace("\"", ""))
    } else {
        Operand::Var(token.to_string().replace(" ", "_"))
    }
}

fn tokenizer(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            // пробелы просто пропускаем
            ' ' | '\t' => {
                chars.next();
            }

            // строковый литерал
            '"' => {
                chars.next(); // снять кавычку
                let mut buf = String::new();
                while let Some(&d) = chars.peek() {
                    if d == '"' {
                        chars.next(); // закрывающая кавычка
                        break;
                    } else {
                        buf.push(d);
                        chars.next();
                    }
                }
                tokens.push(format!("\"{}\"", buf));
            }

            // иначе: слово или число
            _ => {
                let mut buf = String::new();
                while let Some(&d) = chars.peek() {
                    if d == ' ' || d == '\t' {
                        break;
                    }
                    buf.push(d);
                    chars.next();
                }
                tokens.push(buf);
            }
        }
    }

    tokens
}

#[test]
fn spliting() {
    assert_eq!(vec!["a", "\"a\""], tokenizer("a \"a\""));
}

#[test]
fn spliting_empty() {
    assert_eq!(Vec::<String>::new(), tokenizer(""));
}

#[test]
fn set_to_ir() {
    assert_eq!(
        Instr::Set("a".into(), Operand::Const(10.0)),
        parse_line("set a 10").unwrap()
    )
}

#[test]
fn bad_insr() {
    parse_line("asfdasdfsadfjkj").unwrap_err();
}
