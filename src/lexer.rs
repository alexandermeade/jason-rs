use crate::{jason_errors::{JasonError, JasonErrorKind}, token::{self, TokensToNode, Token, TokenType}};
use crate::jason::CompilerResult;
use std::rc::Rc;

pub struct Lexer {
    contents: String,
    file: Rc<String>,
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
    
    pub fn lex_composite_string(&mut self) -> Token {
        self.next(); // skip opening "
        
        let row = self.row;
        let colmn = self.colmn;
        
        let mut literals = Vec::new();
        let mut args = Vec::new();
        let mut current = String::new();
        
        while self.curr_char != '"' {
            if self.curr_char == '\0' {
                return self.new_token(
                    TokenType::ERR(format!("UNABLE TO FIND ENDING \" for composite string at {} {}", row, colmn)),
                    "COMPOSITE STRING LIT def ERROR no closing \"".to_string(),
                );
            }
            
            match self.curr_char {
                '\\' => {
                    self.next();
                    if self.curr_char == '\0' {
                        return self.new_token(
                            TokenType::ERR(format!("UNEXPECTED END OF FILE after escape in composite string at {} {}", row, colmn)),
                            "COMPOSITE STRING LIT def ERROR incomplete escape".to_string(),
                        );
                    }
                    current.push(self.curr_char);
                    self.next();
                }
                '{' => {
                    literals.push(std::mem::take(&mut current));
                    self.next(); // skip '{'
                    
                    let mut expr = Vec::new();
                    let mut brace_depth = 1;
                    
                    while brace_depth > 0 {
                        if self.curr_char == '\0' {
                            return self.new_token(
                                TokenType::ERR(format!("Unclosed '{{' in composite string at {} {}", row, colmn)),
                                "COMPOSITE STRING LIT def ERROR unclosed brace".to_string(),
                            );
                        }
                        
                        let tok = self.lex();
                        if tok.token_type == TokenType::ClosedCurly {
                            brace_depth -= 1;
                            if brace_depth == 0 {
                                self.next(); // consume the closing }
                                break;
                            }
                            let tok = self.lex();
                            expr.push(tok);
                            self.next();
                            continue;
                        }
                    
                        match &tok.token_type {
                            TokenType::OpenCurly => {
                                brace_depth += 1;
                            },
                            TokenType::ClosedCurly => {
                                brace_depth -= 1;
                            }
                            _ => {}
                        }
                        
                        expr.push(tok);
                        self.next();
                    }
                    
                    args.push(expr);
                    continue;
                }
                _ => {
                    current.push(self.curr_char);
                    self.next();
                }
            }
        }
        
        literals.push(current);
        
        let nodes = if args.is_empty() {
            Vec::new()
        } else {
            match args.to_nodes() {
                Ok(nodes) => nodes,
                Err(err) => {
                    return self.new_token(
                        TokenType::ERR(format!("Failed to build composite string: {}", err)),
                        format!("{}", err),
                    )
                }
            }
        };
        
        let result = TokenType::CompositeString(literals, nodes);
        let mut result: Token = self.new_token(result, "".into());
        result.plain = result.pretty();
        result
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
        }
    
        let end_byte = self.byte_index;
        
        if start_byte > self.contents.len() || end_byte > self.contents.len() {
            return self.new_token(
                TokenType::ERR(format!("INCORRECT SET BOUNDS FOR ID at {} {}", row, colmn)), 
                format!("ID LIT def ERROR incorrect bounds from [{} to {}] during substring", start_byte, end_byte)
            );
        }
        let end_byte = self.byte_index;
        self.back();
        
        let num_str = self.get_substring_by_bytes(start_byte, end_byte);
    
        if is_float {
            match num_str.parse::<f64>() {
                Ok(res) => return self.new_token(TokenType::FloatLiteral(res), num_str.to_string()),
                Err(_) => { 
                    let msg = format!("couldn't convert {} into f64", num_str);
                    return self.new_token(TokenType::ERR(msg.clone()), msg);
                }
            };
        }
        match num_str.parse::<i64>() {
            Ok(res) => return self.new_token(TokenType::IntLiteral(res), num_str.to_string()),
            Err(_) => { 
                let msg = format!("couldn't convert {} into f64", num_str);
                return self.new_token(TokenType::ERR(msg.clone()), msg);
            }
        };            
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
        
        let end_byte = self.byte_index;
        
        if start_byte > self.contents.len() || end_byte > self.contents.len() {
            return self.new_token(
                TokenType::ERR(format!("INCORRECT SET BOUNDS FOR ID at {} {}", row, colmn)), 
                format!("ID LIT def ERROR incorrect bounds from [{} to {}] during substring", start_byte, end_byte)
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

    pub fn get_direct_next(&mut self) -> Option<char> {
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
    /*
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
    }*/
    #[allow(dead_code, unused_variables)]
    pub fn collect_toks_between(&mut self, open_tok: TokenType, closed_tok: TokenType) -> Result<Vec<Token>, Token> {
        let start_index = self.char_index;
        let start_row = self.row;
        let start_col = self.colmn;
        let open_char = open_tok.as_open_delim().unwrap_or('?');
        
        // Capture the line where the delimiter was opened
        let line_content = self.get_line(start_row);
        
        self.next();
        
        let mut tokens: Vec<Token> = Vec::new();
        let mut depth = 1;
        
        loop {
            if self.curr_char == '\0' {
                while self.char_index != start_index {
                    self.back();
                }
                return Err(self.new_token(
                    TokenType::ERR(format!(
                        "Unclosed '{}' opened at {}:{}\n |{}",
                        open_char, start_row, start_col,
                        line_content.trim_end(), 
                    )),
                    "Unclosed delimiter".to_string()
                ));
            }
            
            let curr_tok = self.lex();
            
            if curr_tok.token_type == TokenType::EOF {
                return Err(self.new_token(
                    TokenType::ERR(format!(
                        "Unclosed '{}' opened at {}:{}\n  --> {}",
                        open_char, start_row, start_col,
                        line_content.trim_end(),
                    )),
                    "Unclosed delimiter".to_string()
                ));
            }
            
            if curr_tok.token_type.same_delim_type(&open_tok) {
                depth += 1;
            } else if curr_tok.token_type.matches_open(&open_tok) {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            
            tokens.push(curr_tok);
            self.next();
        }
        
        Ok(tokens)
    }

    /// Get the content of a specific line (1-indexed)
    fn get_line(&self, line_num: usize) -> String {
        self.contents
            .lines()
            .nth(line_num.saturating_sub(1))
            .unwrap_or("")
            .to_string()
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
            '$' => {
                match self.get_direct_next() {
                    Some('"') => {
                        self.next();
                        let tok = self.lex_composite_string();
                        return tok; 
                    },
                    _ => return self.new_token(TokenType::DollarSign, format!("$")),
                }
            },
            '&' => self.new_token(TokenType::Merge, format!("&")),
            '"' => self.lex_string(),
            '.' => self.new_token(TokenType::Dot, format!(".")),
            ',' => self.new_token(TokenType::Comma, format!(",")),
            ':' => {
                if let Some(next) = self.get_direct_next() {
                    match next {
                        ':' => {
                            self.next();
                            if let Some(next) = self.get_direct_next() {
                                if next == '=' {
                                    self.next();
                                    return self.new_token(TokenType::SpiderWalrus, format!("::="))
                                }
                            }
                            return self.new_token(TokenType::DoubleColon, format!("::"))
                        },
                        '=' => {
                            self.next();
                            return self.new_token(TokenType::Narwhal, format!(":="))
                        }
                        _ => {}
                    }
                }

                self.new_token(TokenType::Colon, format!(":"))
            },
            '>' => {
                if let Some(c) = self.get_direct_next() {
                    if c == '=' {
                        self.next();
                        return self.new_token(TokenType::GreaterThanEqualTo, format!(">="));
                    }
                }

                self.new_token(TokenType::GreaterThan, format!(">"))
            },
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
                let toks = match self.collect_toks_between(TokenType::OpenBracket, TokenType::ClosedBracket) {
                    Ok(toks) => toks,
                    Err(e) => return e
                };
                let args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                    .map(|slice| slice.to_vec())
                    .collect();

                match args.to_nodes() {
                    Ok(nodes) => return self.new_token(TokenType::List(nodes), format!("List")),
                    Err(err) => return self.new_token(TokenType::ERR(err.message.clone()), err.message),
                }

            },

            '\'' => return self.new_token(TokenType::VarianceOperator, format!("'")),
            '{' => {
                let toks: Vec<Token> = match self.collect_toks_between(TokenType::OpenCurly, TokenType::ClosedCurly) {
                    Ok(toks) => toks,
                    Err(e) => return e
                };

                let mut args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                    .map(|slice| slice.to_vec())
                    .collect();                

                args.retain(|vec| !vec.is_empty());
                match args.to_nodes() {
                    Ok(nodes) => return self.new_token(TokenType::Block(nodes), format!("Block")), 
                    Err(err) => return self.new_token(TokenType::ERR(err.message.clone()), err.message),
                }
            },
            '|' => self.new_token(TokenType::Bar, format!("|")),
            '<' => {
                if let Some(c) = self.get_direct_next() {
                    if c == '=' {
                        self.next();
                        return self.new_token(TokenType::LessThanEqualTo, format!("<="));
                    }
                }

                self.new_token(TokenType::LessThan, format!("<"))
            },


                /*{
                let toks: Vec<Token> = match self.collect_toks_between(TokenType::LessThan, TokenType::GreaterThan) {
                    Ok(toks) => toks,
                    Err(e) => return e
                };
                let sides: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Bar)
                    .map(|slice| slice.to_vec())
                    .collect();
                if sides.len() > 1 {  
                    let right_side = sides[1].clone();
                    let args: Vec<Vec<Token>> = right_side.split(|tok| tok.token_type == TokenType::Comma)
                        .map(|slice| slice.to_vec())
                        .collect();

                    match args.to_nodes() {
                        Ok(nodes) => return self.new_token(TokenType::Template(sides[0].clone(), nodes), format!("Template")), 
                        Err(err) => return self.new_token(TokenType::ERR(err.message.clone()), err.message),
                    }

                }
                return self.new_token(TokenType::Template(sides[0].clone(), vec![]), format!("Template"));
            },*/
            ')' => return self.new_token(TokenType::ClosedParen, format!(")")),
            ']' => return self.new_token(TokenType::ClosedBracket, format!("]")),
            '}' => return self.new_token(TokenType::ClosedCurly, format!("}}")),
            '=' => return self.new_token(TokenType::Equals, format!("=")),
            '\0' => self.new_token(TokenType::EOF, format!("\\0")),
            c => { 
                if c.is_alphabetic() || c == '_' {
                    let id = self.lex_id();
                    self.skip_whitespace();
                    
                    if TokenType::is_keyword(&id.plain()) {
                        self.back();
                        return id;
                    }

                    match self.curr_char {
                        '(' => {    
                            let toks: Vec<Token> = match self.collect_toks_between(TokenType::OpenParen, TokenType::ClosedParen){
                                Ok(toks) => toks,
                                Err(e) => return e
                            };
                            let args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                                .map(|slice| slice.to_vec())
                                .collect();
                            self.next();
                            self.skip_whitespace();
                            
                            if Token::is_fn_keyword(&id.plain) {
                                self.back();
                                match args.to_nodes() {
                                    Ok(nodes) => return self.new_token(TokenType::FnCall(nodes), format!("{}", id.plain())).find_fn_keyword(), 
                                    Err(err) => return self.new_token(TokenType::ERR(err.message.clone()), err.message),
                                }
                            }

                            if self.curr_char == '!' {
                                match args.to_nodes() {
                                    Ok(nodes) => return self.new_token(TokenType::LuaFnCall(nodes), format!("{}", id.plain())), 
                                    Err(err) => return self.new_token(TokenType::ERR(err.message.clone()), err.message),
                                }

                            }
                            if self.curr_char == '{' {
                                let inner_toks: Vec<Token> = match self.collect_toks_between(TokenType::OpenCurly, TokenType::ClosedCurly) {
                                    Ok(toks) => toks,
                                    Err(e) => return e
                                };
                                let mut inner_args: Vec<Vec<Token>> = inner_toks.split(|tok| tok.token_type == TokenType::Comma)
                                    .map(|slice| slice.to_vec())
                                    .collect();                                           
                                inner_args.retain(|vec| !vec.is_empty());

                                match (args.to_nodes(), inner_args.to_nodes()) {
                                    (Ok(nodes), Ok(inner_nodes)) => return self.new_token(TokenType::TemplateDef(nodes, inner_nodes), format!("{}", id.plain())),
                                    (Ok(_), Err(err)) |
                                    (Err(err), Ok(_)) => return self.new_token(TokenType::ERR(err.message.clone()), err.message),

                                    (Err(err1), Err(err2)) => return self.new_token(TokenType::ERR(format!("{}\n{}", err1.message, err2.message)), format!("{}\n{}", err1.message, err2.message)),
                                }
                            }
                            self.back();
                            match args.to_nodes() {
                                Ok(nodes) => return self.new_token(TokenType::FnCall(nodes), format!("{}", id.plain())).find_fn_keyword(), 
                                Err(err) => return self.new_token(TokenType::ERR(err.message.clone()), err.message),
                            }


                                                   
                        },
                        _ => {}
                    }
                    self.back();
                    return id;
                }
                if c.is_numeric() || c == '.' {
                    return self.lex_number();
                }
                
                self.new_token(TokenType::Unknown(c), format!("{}", c))
            }
        }
    }
    
    pub fn next(&mut self) {
        match self.curr_char {
            '\n' => {
                self.row += 1;
                self.colmn = 1;
            },
            '\t' => {
                self.colmn += 4;
            },
            '\0' => {},
            _ => {
                self.colmn += 1;
            },
        }
        
        if self.curr_char != '\0' {
            self.byte_index += self.curr_char.len_utf8();
        }
        
        self.char_index += 1;
        self.curr_char = self.contents.chars().nth(self.char_index).unwrap_or('\0');
    }

    pub fn back(&mut self) {
        if self.char_index == 0 {
            return;
        }
        
        self.char_index -= 1;
        self.curr_char = self.contents.chars().nth(self.char_index).unwrap_or('\0');
        
        // Recalculate byte_index
        self.byte_index = self.contents.chars()
            .take(self.char_index)
            .map(|c| c.len_utf8())
            .sum();
        
        // Recalculate row and column by scanning from the start
        // (This is expensive but back() is rarely called)
        self.row = 1;
        self.colmn = 1;
        for (i, ch) in self.contents.chars().enumerate() {
            if i >= self.char_index {
                break;
            }
            match ch {
                '\n' => {
                    self.row += 1;
                    self.colmn = 1;
                },
                '\t' => {
                    self.colmn += 4;
                },
                _ => {
                    self.colmn += 1;
                },
            }
        }
    }
    
    pub fn start(file: Rc<String>, contents: String) -> CompilerResult<Vec<Token>> { 
        if contents.is_empty() {
            return Ok(Vec::new());
        }
        let mut lexer: Lexer = Lexer {
            contents: contents.clone(),
            tokens: Vec::new(),
            file: file.clone(),
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

        let errs: Vec<JasonError> = lexer
            .tokens
            .iter()
            .filter_map(|tok| {
                if let TokenType::ERR(err_msg) = &tok.token_type {
                    Some(JasonError::new(
                        JasonErrorKind::LexerError(err_msg.clone()),  // Use the actual error message
                        lexer.file.clone(),
                        None,
                        format!("Lexer error at {}:{}", tok.row, tok.colmn)
                    ))
                } else {
                    None
                }
            })
            .collect();

        if !errs.is_empty() {
            return Err(JasonError::new(
                JasonErrorKind::Bundle(errs),
                lexer.file.clone(),
                None,
                "Lexer errors"
            ));
        }


        Ok(lexer.tokens)
    }
}
