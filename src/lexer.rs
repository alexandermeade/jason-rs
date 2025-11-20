use crate::{token, token::{TokenType, Token}};
use log::{info};

pub struct Lexer {
    contents: String,
    tokens: Vec<token::Token>,
    char_index: usize,      // Character position (for chars().nth()) :<
    byte_index: usize,      // Byte position (for slicing) :3
    curr_char: char,
    row: usize,
    colmn: usize,
}

impl Lexer {
    pub fn new_token(&self, token_type: TokenType, plain: String) -> Token {
        Token::new(token_type, plain, self.row, self.colmn)
    }
    
    #[allow(dead_code)]
    pub fn print_line(content: String, target_line: usize) {
        let mut line_count: usize = 0;
        let mut index: usize = 0;
        let mut chars = content.chars();
        while index < content.len() && line_count != target_line {
            if let Some(c) = chars.nth(index) {
                if c == '\n' {
                    line_count += 1;
                }
                index += 1;
                continue;
            }
            return;
        }
        let start = index.saturating_sub(1);
        let mut end = start;
        let found_newline = false;
        
        while end < content.len() && !found_newline {
            if let Some(c) = chars.nth(end) {
                if c == '\n' {
                    break;
                }
            }
            end += 1;
        }
        if end >= content.len() {
            return;
        }
    }
       
    fn get_substring_by_bytes(&self, start_byte: usize, end_byte: usize) -> &str {
        &self.contents[start_byte..end_byte]
    }
    
    pub fn lex_string(&mut self) -> Token {
        self.next(); // skips the initial "
        let row = self.row;
        let colmn = self.colmn;
        
        let mut string_contents = String::new();
        
        while self.curr_char != '"' {
            if self.curr_char == '\0' {
                return self.new_token(
                    TokenType::ERR(format!("UNABLE TO FIND ENDING \" for string at {} {}", row, colmn)), 
                    format!("STRING LIT def ERROR no closing \"")
                );
            }
            
            if self.curr_char == '\\' {
                self.next(); // skip the '\'
                if self.curr_char == '\0' {
                    return self.new_token(
                        TokenType::ERR(format!("UNEXPECTED END OF FILE after escape in string at {} {}", row, colmn)), 
                        format!("STRING LIT def ERROR incomplete escape")
                    );
                }
                // Handle escape sequences
                string_contents.push(self.curr_char);
                self.next();
                continue;
            }
            
            // Push the Unicode character directly
            string_contents.push(self.curr_char);
            self.next();
        }
        
        return self.new_token(
            TokenType::StringLiteral(string_contents.clone()), 
            string_contents
        );
    }
    
    pub fn lex_number(&mut self) -> Token {
        let row = self.row;
        let colmn = self.colmn;
        let start_byte = self.byte_index;
        let mut is_float = false;
        
        while self.curr_char.is_numeric() || (self.curr_char == '.' && !is_float) {
            if self.curr_char == '\0' {
                break;
            }
            let is_dot = self.curr_char == '.';
            if is_dot && is_float {
                return self.new_token(
                    TokenType::ERR(format!("YOU CANNOT HAVE TWO DECIMAL POINTS WHEN DEFINING A FLOAT LITERAL {} {}", row, colmn)), 
                    format!("INCORRECT USE OF . FOR FLOAT LITERAL")
                );
            }
            if is_dot {
                is_float = true;
            }
            self.next();
            info!("[index: {}], {}", self.char_index, self.curr_char);
        }
        info!("start: {}, index: {}", start_byte, self.byte_index);
        
        if self.byte_index >= self.contents.len() {
            return self.new_token(
                TokenType::ERR(format!("INCORRECT SET BOUNDS FOR NUMBER/FLOAT LIT DEF at {} {}", row, colmn)), 
                format!("NUMBER/FLOAT LIT def ERROR incorrect bounds")
            );
        }
        
        let end_byte = self.byte_index;
        self.back();
        
        let num_str = self.get_substring_by_bytes(start_byte, end_byte);
        
        return self.new_token(
            if is_float {
                TokenType::FloatLiteral(format!("{}", num_str))
            } else {
                TokenType::NumberLiteral(format!("{}", num_str))
            }, 
            format!("{}", num_str)
        );
    }
    
    pub fn lex_id(&mut self) -> Token {
        let row = self.row;
        let colmn = self.colmn;
        let start_byte = self.byte_index;
        
        while self.curr_char.is_alphanumeric() || (self.curr_char == '_') {
            if self.curr_char == '\0' {
                break;
            }
            // Check if character is ASCII
            if !self.curr_char.is_ascii() {
                return self.new_token(
                    TokenType::ERR(format!("IDENTIFIERS MUST BE ASCII ONLY at {} {}", row, colmn)), 
                    format!("ID contains non-ASCII character: '{}'", self.curr_char)
                );
            }
            self.next();
        }
        
        if self.byte_index >= self.contents.len() {
            return self.new_token(
                TokenType::ERR(format!("INCORRECT SET BOUNDS FOR ID at {} {}", row, colmn)), 
                format!("ID LIT def ERROR incorrect bounds from [{} to {}] during substring", start_byte, self.byte_index)
            );
        }
        
        let string_contents = self.get_substring_by_bytes(start_byte, self.byte_index);
        return self.new_token( 
            TokenType::find_keyword(string_contents),
            format!("{}", string_contents)
        );
    }
    
    pub fn skip_whitespace(&mut self) {
        loop {
            match self.curr_char {
                ' ' | '\t' => self.next(),
                _ => return,
            }
        } 
    }
    
    pub fn get_next(&mut self) -> Option<char> {
        self.skip_whitespace();
        return self.contents.chars().nth(self.char_index + 1);
    }
    
    #[allow(dead_code)]
    pub fn is_next(&mut self, c: char) -> bool {
        self.skip_whitespace();
        if let Some(ch) = self.contents.chars().nth(self.char_index + 1) {
            return ch == c;
        }
        return false;
    }
    
    pub fn collect_toks_between(&mut self, _open_tok: TokenType, closed_tok: TokenType) -> Vec<Token> {
        self.next();
        
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let curr_tok = self.lex();
            if curr_tok.token_type == closed_tok {
                break;
            }
            tokens.push(curr_tok);
            self.next();
        } 
        return tokens;
    }
    
    pub fn lex(&mut self) -> Token {
        match self.curr_char {
            '+' => self.new_token(TokenType::Plus, format!("+")),
            '-' => self.new_token(TokenType::Minus, format!("-")),
            '*' => {
                if let Some(c) = self.get_next() {
                    if c == '/' {
                        self.next();
                        self.next();
                        return self.new_token(TokenType::EndComment, format!("*/"))        
                    }
                }
                return self.new_token(TokenType::Mult, format!("*"))
            },
            '/' => {
                if let Some(c) = self.get_next() {
                    if c == '*' {
                        self.next();
                        self.next();
                        while self.lex().token_type != TokenType::EndComment {
                            self.next();
                        }
                        return self.lex();
                    }
                    if c == '/' {
                        self.next();
                        self.next();
                        while self.lex().token_type != TokenType::NewLine {
                            self.next();
                        }
                        return self.lex();
                    }
                }
                self.new_token(TokenType::Divide, format!("/"))
            },
            '%' => self.new_token(TokenType::Mod, format!("%")),
            '$' => self.new_token(TokenType::DollarSign, format!("$")),
            '"' => self.lex_string(),
            '.' => self.new_token(TokenType::Dot, format!(".")),
            ',' => self.new_token(TokenType::Comma, format!(",")),
            ':' => self.new_token(TokenType::Colon, format!(":")),
            '>' => self.new_token(TokenType::GreaterThan, format!(">")),
            '\n' => return self.new_token(TokenType::NewLine, format!("\\n")),
            '\t' | ' ' => {
                self.next();
                return self.lex();
            },

            '(' => {
                /*
                let toks: Vec<Token> = self.collect_toks_between(TokenType::OpenParen, TokenType::ClosedParen);
                let args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                    .map(|slice| slice.to_vec())
                    .collect();
                return self.new_token(TokenType::Tuple(args), format!("Tuple"));*/
                return self.new_token(TokenType::OpenParen, format!("("));
            },
            '[' => {
                let toks: Vec<Token> = self.collect_toks_between(TokenType::OpenBracket, TokenType::ClosedBracket);
                let args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                    .map(|slice| slice.to_vec())
                    .collect();
                return self.new_token(TokenType::List(args), format!("List"));
            },
            '{' => {
                let toks: Vec<Token> = self.collect_toks_between(TokenType::OpenCurly, TokenType::ClosedCurly);
                let mut args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                    .map(|slice| slice.to_vec())
                    .collect();                
                args.retain(|vec| !vec.is_empty());
                return self.new_token(TokenType::Block(args), format!("Block"));
            },
            '|' => self.new_token(TokenType::Bar, format!("|")),
            '<' => {
                let toks: Vec<Token> = self.collect_toks_between(TokenType::LessThan, TokenType::GreaterThan);
                let sides: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Bar)
                    .map(|slice| slice.to_vec())
                    .collect();
                if sides.len() > 1 {  
                    let right_side = sides[1].clone();
                    let args: Vec<Vec<Token>> = right_side.split(|tok| tok.token_type == TokenType::Comma)
                        .map(|slice| slice.to_vec())
                        .collect();
                    return self.new_token(TokenType::Template(sides[0].clone(), args), format!("Template"));
                }
                return self.new_token(TokenType::Template(sides[0].clone(), vec![]), format!("Template"));
            },
            ')' => return self.new_token(TokenType::ClosedParen, format!(")")),
            ']' => return self.new_token(TokenType::ClosedBracket, format!("]")),
            '}' => return self.new_token(TokenType::ClosedCurly, format!("}}")),
            '=' => return self.new_token(TokenType::Equals, format!("=")),
            '\0' => self.new_token(TokenType::EOF, format!("\\0")),
            c => { 
                if c.is_alphabetic() || c == '_' {
                    let id = self.lex_id();
                    self.skip_whitespace();
                    match self.curr_char {
                        '(' => {    
                            let toks: Vec<Token> = self.collect_toks_between(TokenType::OpenParen, TokenType::ClosedParen);
                            let args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                                .map(|slice| slice.to_vec())
                                .collect();
                            self.next();
                            self.skip_whitespace(); 
                            if self.curr_char == '!' {
                                return self.new_token(TokenType::LuaFnCall(args), format!("{}", id.plain()));                        
                            }
                            if self.curr_char == '{' {
                                let inner_toks: Vec<Token> = self.collect_toks_between(TokenType::OpenCurly, TokenType::ClosedCurly);
                                let mut inner_args: Vec<Vec<Token>> = inner_toks.split(|tok| tok.token_type == TokenType::Comma)
                                    .map(|slice| slice.to_vec())
                                    .collect();                                           
                                inner_args.retain(|vec| !vec.is_empty());
                                return self.new_token(TokenType::TemplateDef(args, inner_args), format!("{}", id.plain()));                        
                            }
                            self.back();
                            return self.new_token(TokenType::FnCall(args), format!("{}", id.plain())).find_fn_keyword();                        
                        },
                        _ => {}
                    }
                    self.back();
                    return id;
                }
                if c.is_numeric() || c == '.' {
                    return self.lex_number();
                }
                
                self.new_token(TokenType::Unknown(c), format!("?Unknown Character?"))
            }
        }
    }
    
    pub fn back(&mut self) {
        if self.char_index <= 0 {
            return;
        }
        self.char_index -= 1;
        self.curr_char = match self.contents.chars().nth(self.char_index) {
            Some(c) => c,
            None => '\0'
        };
        
        // Recalculate byte_index
        self.byte_index = self.contents.chars()
            .take(self.char_index)
            .map(|c| c.len_utf8())
            .sum();
    }   
    
    pub fn next(&mut self) {
        // Move byte index forward by the byte length of current char
        if self.curr_char != '\0' {
            self.byte_index += self.curr_char.len_utf8();
        }
        
        self.char_index += 1;
        self.curr_char = match self.contents.chars().nth(self.char_index) {
            Some(c) => c,
            None => '\0'
        };
        
        match self.curr_char {
            '\n' => {
                self.row += 1;
                self.colmn = 1;
            },
            '\t' => {
                self.colmn += 4;
            },
            ' ' => {
                self.colmn += 1;
            },
            '\0' => {},
            _ => {
                self.colmn += 1;
            },
        };     
    }
    
    pub fn start(contents: String) -> Vec<Token> { 
        if contents.is_empty() {
            return Vec::new();
        }
        let mut lexer: Lexer = Lexer {
            contents: contents.clone(),
            tokens: Vec::new(),
            char_index: 0,
            byte_index: 0,
            curr_char: contents.chars().nth(0).unwrap(),
            row: 1,
            colmn: 1,
        };
        
        while lexer.curr_char != '\0' {
            lexer.skip_whitespace();
            let tok = lexer.lex();
            if tok.token_type != TokenType::NewLine && tok.token_type != TokenType::EOF {
                lexer.tokens.push(tok);
            } else if tok.token_type == TokenType::EOF {
                break;
            }
            lexer.next();
        } 
        
        return lexer.tokens;
    }
}
