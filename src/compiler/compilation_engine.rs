use super::tkzr::{KeyWord, TokenType, Tokenizer};
use std::fs::File;
use std::path::Path;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

pub struct CompilationEngine {
    tkzr: Tokenizer,
    writer: EventWriter<File>,
}

impl CompilationEngine {
    pub fn new(path: &Path) -> Self {
        let tkzr = Tokenizer::new(path);
        let mut output_path = path.to_path_buf();
        output_path.set_extension("xml");
        let file = File::create(&output_path).unwrap();
        let mut config = EmitterConfig::new();
        config.perform_escaping = false;
        let writer = config
            .write_document_declaration(false)
            .perform_indent(true)
            .create_writer(file);
        CompilationEngine { tkzr, writer }
    }

    // 'class' className '{' classVarDec* subroutineDec* '}'
    pub fn compile_class(&mut self) {
        self.tkzr.advance();
        self.write_start_event("class");
        self.compile_key_word();
        self.compile_identifier();
        self.compile_symbol();
        while let TokenType::KeyWord(key_word) = self.tkzr.token_type() {
            match key_word {
                KeyWord::Static | KeyWord::Field => {
                    self.compile_class_var_dec();
                }
                KeyWord::Function | KeyWord::Method | KeyWord::Constructor => {
                    self.compile_subroutine_dec();
                }
                _ => break,
            }
        }
        self.compile_symbol();
        self.write_end_event();
    }

    fn write_start_event(&mut self, name: &str) {
        let event: XmlEvent = XmlEvent::start_element(name).into();
        self.writer.write(event).unwrap();
    }

    fn write_characters(&mut self, s: &str) {
        let event: XmlEvent = XmlEvent::characters(s);
        self.writer.write(event).unwrap();
    }

    fn write_end_event(&mut self) {
        let event: XmlEvent = XmlEvent::end_element().into();
        self.writer.write(event).unwrap();
    }

    // ('static' | 'field') type varName (',' varName)* ';'
    fn compile_class_var_dec(&mut self) {
        self.write_start_event("classVarDec");
        self.compile_key_word();
        self.compile_type();
        loop {
            self.compile_identifier();
            if let TokenType::Symbol(s) = self.tkzr.token_type() {
                if s == ";" {
                    break;
                }
            }
            self.compile_symbol();
        }
        self.compile_symbol();
        self.write_end_event();
    }

    // type: 'int'|'char'|'boolean'|className
    fn compile_type(&mut self) {
        match self.tkzr.token_type() {
            TokenType::KeyWord(_) => self.compile_key_word(),
            TokenType::Identifier(_) => self.compile_identifier(),
            _ => panic!(
                "{:?} is invalid, KeyWord or Identifier is required",
                self.tkzr.token_type()
            ),
        }
    }

    // ('constructor'|'function'|'method')('void'|type) subroutineName
    fn compile_subroutine_dec(&mut self) {
        self.write_start_event("subroutineDec");
        self.compile_key_word();
        self.compile_type();
        self.compile_identifier();
        self.compile_symbol(); // (
        self.compile_parameter_list();
        self.compile_symbol(); // )
        self.compile_subroutine_body();
        self.write_end_event();
    }

    // ((type varName)(',' type varName)*)?
    fn compile_parameter_list(&mut self) {
        self.write_start_event("parameterList");
        loop {
            if let TokenType::Symbol(symbol) = self.tkzr.token_type() {
                if symbol == ")" {
                    break;
                }
                self.compile_symbol();
            }
            self.compile_type();
            self.compile_identifier();
        }
        self.write_end_event();
    }

    // '{' varDec* statements '}'
    fn compile_subroutine_body(&mut self) {
        self.write_start_event("subroutineBody");
        self.compile_symbol();
        while let TokenType::KeyWord(key_word) = self.tkzr.token_type() {
            match key_word {
                KeyWord::Var => self.compile_var_dec(),
                KeyWord::Let | KeyWord::Do | KeyWord::Return | KeyWord::If => {
                    self.compile_statements()
                }
                _ => {}
            }
        }
        self.compile_symbol();
        self.write_end_event();
    }

    // 'var' type varName (',' varName)* ';'
    fn compile_var_dec(&mut self) {
        self.write_start_event("varDec");
        self.compile_key_word();
        loop {
            if let TokenType::Symbol(s) = self.tkzr.token_type() {
                self.compile_symbol();
                if s == ";" {
                    break;
                }
            }
            self.compile_type();
        }
        self.write_end_event();
    }

    // letStatement | ifStatement | whileStatement | doStatement | returnStatement
    fn compile_statements(&mut self) {
        self.write_start_event("statements");
        while let TokenType::KeyWord(key_word) = self.tkzr.token_type() {
            match key_word {
                KeyWord::If => self.compile_if(),
                KeyWord::Let => self.compile_let(),
                KeyWord::Do => self.compile_do(),
                KeyWord::Return => self.compile_return(),
                KeyWord::While => self.compile_while(),
                _ => break,
            }
        }
        self.write_end_event();
    }

    // 'let' varName('[' expression ']')? '=' expression ';'
    fn compile_let(&mut self) {
        self.write_start_event("letStatement");
        self.compile_key_word();
        // Todo(zhoufan): 2d array?
        if self.tkzr.next_token() == Some("[".to_string()) {
            self.compile_array();
        } else {
            self.compile_identifier();
        }
        self.compile_symbol();
        self.compile_expression();
        self.compile_symbol();
        self.write_end_event();
    }

    // 'do' sobroutineCall ';'
    fn compile_do(&mut self) {
        self.write_start_event("doStatement");
        self.compile_key_word();
        self.compile_subroutine_call();
        self.compile_symbol();
        self.write_end_event();
    }

    // 'return' expression? ';'
    fn compile_return(&mut self) {
        self.write_start_event("returnStatement");
        self.compile_key_word();
        match self.tkzr.token_type() {
            TokenType::KeyWord(_) => self.compile_expression(),
            TokenType::Identifier(_) => self.compile_expression(),
            _ => {}
        }
        self.compile_symbol();
        self.write_end_event();
    }

    // 'while' '(' expression ')' '{' statements '}'
    fn compile_while(&mut self) {
        self.write_start_event("whileStatement");
        self.compile_key_word();
        self.compile_symbol();
        self.compile_expression();
        self.compile_symbol();
        self.compile_symbol();
        self.compile_statements();
        self.compile_symbol();
        self.write_end_event();
    }

    // 'if' '(' expression ')' '{' statements '}' ('else' '{' statements '}')?
    fn compile_if(&mut self) {
        self.write_start_event("ifStatement");
        self.compile_key_word();
        self.compile_symbol();
        self.compile_expression();
        self.compile_symbol();
        self.compile_symbol();
        self.compile_statements();
        self.compile_symbol();
        if let TokenType::KeyWord(key_word) = self.tkzr.token_type() {
            if key_word == KeyWord::Else {
                self.compile_key_word();
                self.compile_symbol();
                self.compile_statements();
                self.compile_symbol();
            }
        }
        self.write_end_event();
    }

    // term(op term)*
    fn compile_expression(&mut self) {
        self.write_start_event("expression");
        self.compile_term();
        while let TokenType::Symbol(symbol) = self.tkzr.token_type() {
            if symbol == "*"
                || symbol == "/"
                || symbol == "|"
                || symbol == "+"
                || symbol == "&lt;"
                || symbol == "&amp;"
                || symbol == "&gt;"
                || symbol == "-"
                || symbol == "="
            {
                self.compile_symbol();
                self.compile_term();
            } else {
                break;
            }
        }
        self.write_end_event();
    }

    // integerConstant | stringConstant | keywordConstant | varName |
    // varName '[' expression ']' | subroutineCall | '(' expression ')' | unaryOp term
    fn compile_term(&mut self) {
        self.write_start_event("term");
        match self.tkzr.token_type() {
            TokenType::IntConst(_) => self.compile_int(),
            TokenType::StringConst(_) => self.compile_string(),
            TokenType::KeyWord(key_word) => match key_word {
                KeyWord::This | KeyWord::True | KeyWord::False | KeyWord::Null => {
                    self.compile_key_word();
                }
                _ => panic!("{:?} is invalid in term", self.tkzr.token_type()),
            },
            TokenType::Identifier(_) => {
                if self.tkzr.next_token() == Some(".".to_string()) {
                    self.compile_subroutine_call();
                } else if self.tkzr.next_token() == Some("[".to_string()) {
                    self.compile_array();
                } else {
                    self.compile_identifier();
                }
            }
            TokenType::Symbol(symbol) => {
                if symbol == "(" {
                    self.compile_symbol();
                    self.compile_expression();
                    self.compile_symbol();
                } else if symbol == "-" || symbol == "~" {
                    self.compile_symbol();
                    self.compile_term();
                }
            }
        }
        self.write_end_event();
    }

    // (expression( ',' expression)*)?
    fn compile_expression_list(&mut self) {
        self.write_start_event("expressionList");
        loop {
            if let TokenType::Symbol(symbol) = self.tkzr.token_type() {
                if symbol == ")" {
                    break;
                }
                if symbol == "," {
                    // Todo(zhoufan): why if
                    self.compile_symbol();
                }
            }
            self.compile_expression();
        }
        self.write_end_event();
    }

    // subroutineName '(' expressionList ')' | (className | varName) '.' subroutineName '(' expressionList ')'
    fn compile_subroutine_call(&mut self) {
        self.compile_identifier();
        if self.tkzr.current_token == "." {
            self.compile_symbol();
            self.compile_identifier();
        }
        self.compile_symbol();
        self.compile_expression_list();
        self.compile_symbol();
    }

    fn compile_array(&mut self) {
        self.compile_identifier();
        self.compile_symbol();
        self.compile_expression();
        self.compile_symbol();
    }

    fn compile_key_word(&mut self) {
        if let TokenType::KeyWord(key_word) = self.tkzr.token_type() {
            self.write_start_event("keyword");
            let key_word = format!("{}", key_word);
            self.write_characters(&key_word);
            self.write_end_event();
            if self.tkzr.has_more_commands() {
                self.tkzr.advance();
            }
        } else {
            panic!("{:?} is not a KeyWord", self.tkzr.token_type());
        }
    }

    fn compile_symbol(&mut self) {
        if let TokenType::Symbol(symbol) = self.tkzr.token_type() {
            self.write_start_event("symbol");
            self.write_characters(&symbol);
            self.write_end_event();
            if self.tkzr.has_more_commands() {
                self.tkzr.advance();
            }
        } else {
            panic!("{:?} is not a Symbol", self.tkzr.token_type());
        }
    }

    fn compile_int(&mut self) {
        if let TokenType::IntConst(val) = self.tkzr.token_type() {
            self.write_start_event("integerConstant");
            self.write_characters(&val.to_string());
            self.write_end_event();
            if self.tkzr.has_more_commands() {
                self.tkzr.advance();
            }
        } else {
            panic!("{:?} is not a IntConst", self.tkzr.token_type());
        }
    }

    fn compile_string(&mut self) {
        if let TokenType::StringConst(val) = self.tkzr.token_type() {
            self.write_start_event("stringConstant");
            self.write_characters(&val);
            self.write_end_event();
            if self.tkzr.has_more_commands() {
                self.tkzr.advance();
            }
        } else {
            panic!("{:?} is not a StringConst", self.tkzr.token_type());
        }
    }

    fn compile_identifier(&mut self) {
        if let TokenType::Identifier(identifier) = self.tkzr.token_type() {
            self.write_start_event("identifier");
            self.write_characters(&identifier);
            self.write_end_event();
            if self.tkzr.has_more_commands() {
                self.tkzr.advance();
            }
        } else {
            panic!("{:?} is not a Identifier", self.tkzr.token_type());
        }
    }
}
