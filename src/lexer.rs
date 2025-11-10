use crate::{token, token::{TokenType, Token}};

use log::{info, warn, error};

pub struct Lexer {
    contents:String,
    tokens: Vec<token::Token>,
    index: usize,
    curr_char:char,
    row:usize,
    colmn:usize,
}

impl Lexer {

    pub fn new_token(&self, tokenType:TokenType, plain:String) -> Token {
        Token::new(tokenType, plain, self.row, self.colmn)
    }
    
    pub fn print_line(content: String, target_line:usize) {
        let mut line_count:usize = 0;
        let mut index:usize = 0;
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
        //find next \n
        let start = index - 1;
        let mut end = start;
        let found_newline = false;
        
        while  end < content.len() && !found_newline {
            if let Some(c) = chars.nth(end) {
                if c == '\n' {
                    break;
                }
            }
            end += 1;
        }
        if start < 0 || end >= content.len() {
            return;
        }
        //println!("[{}] {}", target_line, &content[start.. end]);        
    }

    pub fn lex_string(&mut self) -> Token {
        self.next(); //skips the initial "
        let row = self.row;
        let colmn = self.colmn;
        let start = self.index;
        while self.curr_char != '"' {
            if self.curr_char == '\0' {
                return self.new_token(TokenType::ERR(format!("UNABLE TO FIND ENDING \" for string at {} {}", row, colmn)), format!("STRING LIT def ERROR no closng \""));
            }

            if self.curr_char == '\\'{
                self.next(); // skip the '\'
                self.next(); // skip the character after \ ex \0 allowing for \"
                continue;
            }
            self.next();
        }
        
        if start < 0 || self.index >= self.contents.len() {
            return self.new_token(TokenType::ERR(format!("INCORRECT SET BOUNDS FOR STRING DEF at {} {}", row, colmn)), format!("STRING LIT def ERROR incorrect bounds"));
        }

        let string_contents = &self.contents[start.. self.index];
        return self.new_token(TokenType::StringLiteral(format!("{}", string_contents)), format!("{}", string_contents));
        //selfnext from while loop in start will skip the last "
    }
    
    pub fn lex_number(&mut self) -> Token {
        let row = self.row;
        let colmn = self.colmn;
        let start = self.index;
        let mut is_float = false;
        
        while self.curr_char.is_numeric() || (self.curr_char == '.' && !is_float) {
            if self.curr_char == '\0' {
                break;
            }
            let is_dot = self.curr_char == '.';

            if is_dot && is_float {

                return self.new_token(TokenType::ERR(format!("YOU CANNOT HAVE TWO DECIMAL POINTS WHEN DEFINING A FLOAT LITERAL {} {}", row, colmn)), format!("INCORRECT USE OF . FOR FLOAT LITERAL"));
            }

            if is_dot {
                is_float = true;
            }
            self.next();
            info!("[index: {}], {}", self.index, self.curr_char);
        }

        info!("start: {}, index: {}", start, self.index);
        if start < 0 || self.index >= self.contents.len() {
            return self.new_token(TokenType::ERR(format!("INCORRECT SET BOUNDS FOR NUMBER/FLOAT LIT DEF at {} {}", row, colmn)), format!("NUMBER/FLOAT LIT def ERROR incorrect bounds"));
        }

        let index = self.index;
                
        self.back();

        return self.new_token(
            if is_float {
                TokenType::FloatLiteral(format!("{}", &self.contents[start.. index]))
            }else {
                TokenType::NumberLiteral(format!("{}", &self.contents[start.. index]))
            }, 
            format!("{}",&self.contents[start.. index])
        );

    }

    pub fn lex_ID(&mut self) -> Token {
        let row = self.row;
        let colmn = self.colmn;
        let start = self.index;
        let mut end = false; 
        while self.curr_char.is_alphanumeric() || (self.curr_char == '_') {
            if self.curr_char == '\0' {
                break;
            }
            self.next();
        }

        //println!("start: {}, index: {}", start, self.index);
        if start < 0 || self.index  >= self.contents.len() {
            return self.new_token(TokenType::ERR(format!("INCORRECT SET BOUNDS FOR ID at {} {}", row, colmn)), format!("ID LIT def ERROR incorrect bounds from [{} to {}] during substring", start, self.index));
        }

        let string_contents = &self.contents[start.. self.index];
        return self.new_token( 
                TokenType::find_keyword(string_contents),
                format!("{}", string_contents)
        );

    }


    pub fn skip_whitespace(&mut self) {
        while true {
            match self.curr_char {
                ' ' | '\t'  => self.next(),
                _ => return,
            }
        } 
    }

    pub fn get_next(&mut self) -> Option<char> {
        self.skip_whitespace();
        return self.contents.chars().nth(self.index + 1);
    }

    pub fn is_next(&mut self, c:char) -> bool {
        self.skip_whitespace();
        if let Some(ch) = self.contents.chars().nth(self.index + 1) {
            return ch == c;
        }
        return false;
    }


    pub fn collect_toks_between(&mut self, open_tok: TokenType, closed_tok: TokenType) -> Vec<Token> {
        let row = self.row;
        let colmn = self.colmn;
    
        self.next(); //assume first character is the tokenType.
        
        let mut nested = 1;
        let mut curr_tok: Token = Token::empty();
        let mut tokens:Vec<Token> = Vec::new();

        while true {
            curr_tok = self.lex();
            if curr_tok.token_type == closed_tok {
                break;
            }
            tokens.push(curr_tok);
            self.next();
        } 
//        println!("{:#?}", tokens);
        return tokens;
    
    }
    
    
    pub fn lex(&mut self) -> Token {
        match self.curr_char {

            '-' => self.new_token(TokenType::Equals, format!("=")),
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
                let mut toks: Vec<Token> = self.collect_toks_between(TokenType::OpenParen, TokenType::ClosedParen);


                let args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                    .map(|slice| slice.to_vec()) // Convert each slice to Vec<Token>
                    .collect();

                return self.new_token(TokenType::List(args), format!("Tuple"));
            },
            '[' => {
                let mut toks: Vec<Token> = self.collect_toks_between(TokenType::OpenBracket, TokenType::ClosedBracket);


                let args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                    .map(|slice| slice.to_vec()) // Convert each slice to Vec<Token>
                    .collect();

                return self.new_token(TokenType::List(args), format!("List"));
            },
            '{' => {
                let mut toks: Vec<Token> = self.collect_toks_between(TokenType::OpenCurly, TokenType::ClosedCurly);

                let mut args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                    .map(|slice| slice.to_vec()) // Convert each slice to Vec<Token>
                    .collect();                
                args.retain(|vec| !vec.is_empty());
                return self.new_token(TokenType::Block(args), format!("Block"));

                /*
                let args:Vec<String> = self.collect_toks_between('{', '}').split('\n').map(|s| format!("{}", s)).collect();
                println!("ARGS:\n {:#?}", args);
                let mut toks: Vec<Vec<Token>> = Vec::new();
                for arg in args {
                    let tok = Lexer::start(arg.trim().to_string());
                    if tok.len() <= 0 {
                        continue;
                    }
                    toks.push(tok);
                }
                return self.new_token(TokenType::Block(toks), format!("Block"));*/
            },
            '|' => self.new_token(TokenType::Bar, format!("|")),
            '<' => {
                let mut toks: Vec<Token> = self.collect_toks_between(TokenType::LessThan, TokenType::GreaterThan);

                let sides:Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Bar)
                    .map(|slice| slice.to_vec()) // Convert each slice to Vec<Token>
                    .collect();

                if sides.len() > 1 {  
                    let right_side = sides[1].clone();
                    let args: Vec<Vec<Token>> = right_side.split(|tok| tok.token_type == TokenType::Comma)
                        .map(|slice| slice.to_vec()) // Convert each slice to Vec<Token>
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
                    let id = self.lex_ID();
                    self.skip_whitespace();
                    match self.curr_char {
                        '(' => {    
                            let mut toks: Vec<Token> = self.collect_toks_between(TokenType::OpenParen, TokenType::ClosedParen);
                            let args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                                .map(|slice| slice.to_vec()) // Convert each slice to Vec<Token>
                                .collect();
                            self.next();
                            self.skip_whitespace(); 
                            if self.curr_char == '{' {
                                let mut inner_toks: Vec<Token> = self.collect_toks_between(TokenType::OpenCurly, TokenType::ClosedCurly);
                                let mut inner_args: Vec<Vec<Token>> = inner_toks.split(|tok| tok.token_type == TokenType::Comma)
                                    .map(|slice| slice.to_vec()) // Convert each slice to Vec<Token>
                                    .collect();                                           
                                inner_args.retain(|vec| !vec.is_empty());
                                return self.new_token(TokenType::TemplateDef(args, inner_args), format!("{}", id.plain()));                        
                            }
                            self.back();
                            return self.new_token(TokenType::FnCall(args), format!("{}", id.plain())).find_fn_keyword();                        
                        },
                        '[' => {
                            let mut toks: Vec<Token> = self.collect_toks_between(TokenType::OpenBracket, TokenType::ClosedBracket);
                            let args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                                .map(|slice| slice.to_vec()) // Convert each slice to Vec<Token>
                                .collect();                                            
                            return self.new_token(TokenType::Index(args), format!("Index of [{}]", id.plain()));                        
                        },
                        '{' => {
                            let mut toks: Vec<Token> = self.collect_toks_between(TokenType::OpenCurly, TokenType::ClosedCurly);
                            let mut args: Vec<Vec<Token>> = toks.split(|tok| tok.token_type == TokenType::Comma)
                                .map(|slice| slice.to_vec()) // Convert each slice to Vec<Token>
                                .collect();                                           
                            args.retain(|vec| !vec.is_empty());
                            return self.new_token(TokenType::TemplateDef(vec![], args), format!("{}", id.plain()));                        
                        },
                         _ => {}
                    }
                    self.back(); //has to offset the whitespace skip. to move pointer back into
                                 //place for lexxing 
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
        if self.index  <= 0 {
            return;
        }
        self.index -= 1;

        self.curr_char = match self.contents.chars().nth(self.index) {
            Some(c) => c,
            none => '\0'
        };   
    }   
    pub fn next(&mut self) {
        self.index += 1;

        self.curr_char = match self.contents.chars().nth(self.index) {
            Some(c) => c,
            none => '\0'
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

    pub fn start(contents:String) -> Vec<Token>{ 
        if contents.len() <= 0 {
            return Vec::new();
        }

        let mut lexer:Lexer = Lexer {
            contents: contents.clone(),
            tokens: Vec::new(),
            index:0,
            curr_char: contents.chars().nth(0).unwrap(),
            row: 1,
            colmn: 1,
        };

        while lexer.curr_char != '\0' {
            lexer.skip_whitespace();
            let tok = lexer.lex();
            if tok.token_type != TokenType::NewLine {
                lexer.tokens.push(tok);
            }
            lexer.next();
        } 
        
        //lexer.tokens.push(token::Token::new(TokenType::EOT, "EOT".to_string(), lexer.row + 1, lexer.colmn + 1));
        
        return lexer.tokens;
    }
}



