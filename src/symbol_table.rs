use std::collections::HashMap;

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
