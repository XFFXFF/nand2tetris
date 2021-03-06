use std::collections::HashMap;

pub struct SymbolTalbe {
    table: HashMap<String, i16>,
    next_alloc: i16,
}

impl SymbolTalbe {
    pub fn new() -> Self {
        let table = Self::predefined_table();
        SymbolTalbe {
            table,
            next_alloc: 16,
        }
    }

    pub fn add_entry(&mut self, symbol: &str, address: i16) {
        self.table.insert(symbol.to_string(), address);
    }

    pub fn add_alloc(&mut self, symbol: &str) -> i16 {
        let alloc = self.next_alloc;
        self.table.insert(symbol.to_string(), self.next_alloc);
        self.next_alloc += 1;
        alloc
    }

    pub fn get_address(&self, symbol: &str) -> Option<&i16> {
        self.table.get(symbol)
    }

    fn predefined_table() -> HashMap<String, i16> {
        let mut table = HashMap::new();
        table.insert("SP".to_string(), 0);
        table.insert("LCL".to_string(), 1);
        table.insert("ARG".to_string(), 2);
        table.insert("THIS".to_string(), 3);
        table.insert("THAT".to_string(), 4);
        for i in 0..16 {
            table.insert(format!("R{}", i), i);
        }
        table.insert("SCREEN".to_string(), 16384);
        table.insert("KBD".to_string(), 24576);
        table
    }
}

impl Default for SymbolTalbe {
    fn default() -> Self {
        Self::new()
    }
}
