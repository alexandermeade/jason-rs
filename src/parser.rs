use crate::jason_errors::JasonError;
use crate::jason_errors::JasonErrorKind;
use crate::token::Token;
use crate::token::TokenType;
use crate::astnode::ASTNode;
use std::rc::Rc;

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }
    
    fn next(&mut self) {
        self.index += 1;
    }
    
    fn next_insure(&mut self, token_type: TokenType) -> bool {
        let tok_type = &self.current().unwrap().token_type;
        if *tok_type != token_type {
            //panic!("{:?} doesn't match {:?}", tok_type, token_type); 
            return false;
        }
        self.index += 1;
        return true;
    }
    
    fn factor(&mut self) -> ASTNode {
        let token = self.current().cloned().unwrap_or(Token::new(TokenType::EOT, "EOT".to_string(), 1, 1));
        
        return match token.token_type {
            TokenType::ID               | 
            TokenType::NumberType       | 
            TokenType::List(_)          | 
            TokenType::FnCall(_)        |
            TokenType::LuaFnCall(_)     |
            TokenType::Import(_)        |
            TokenType::StringLiteral(_) | 
            TokenType::BoolLiteral(_)   |
            TokenType::DollarSign       |
            TokenType::Use(_)           |
            TokenType::Mult => {
                self.next();
                ASTNode::new(token) 
            },
            TokenType::OpenParen => {
                self.next_insure(TokenType::OpenParen);
                let node = self.expr();
                if !self.next_insure(TokenType::ClosedParen) {
                    // Ensure we have a closing ')'
                    return ASTNode::new(Token::new(
                        TokenType::ERR(format!("Expected ')' but not found: {}", self.current().unwrap().plain())),
                        "Parse error".to_string(),
                        0,
                        0,
                    ));
                }
                node
            }
            _ => { self.next(); ASTNode::new(token)},
        }
    }
    
    fn term(&mut self) -> ASTNode {
        let mut node = self.factor();
        // Handle Repeat/Mult at term level (higher precedence)
        // REMOVED Plus from here!
        while let Some(token) = self.current().cloned() {
            match token.token_type {
                TokenType::Repeat | TokenType::Mult => {
                    self.next();
                    let right = self.factor();
                    node = ASTNode::new(token)
                        .children(Some(Box::new(node)), Some(Box::new(right)));
                },
                _ => break,
            }
        }
        node
    }
    
    fn expr(&mut self) -> ASTNode {
        // Check for Out FIRST, BEFORE parsing any terms
        if let Some(token) = self.current() {
            if token.token_type == TokenType::Out {
                let out_token = token.clone();
                self.next(); // consume 'out'
                let right = self.addition(); // parse what comes after 'out'
                return ASTNode::new(out_token)
                    .children(None, Some(Box::new(right)));
            }
        }
        
        // NOW parse the leftmost term
        let mut node = self.addition(); // Changed from term() to addition()
        
        while let Some(token) = self.current().cloned() {
            match token.token_type {
                TokenType::Colon  | 
                TokenType::From   |
                TokenType::AS     |
                TokenType::Append |
                TokenType::Equals => {
                    self.next(); // consume the operator
                    let right = self.addition(); // Changed from term() to addition()
                    // build new AST node where operator is parent
                    node = ASTNode::new(token)
                        .children(Some(Box::new(node)), Some(Box::new(right)));
                },
                _ => break,
            }
        }
        node
    }
    
    fn addition(&mut self) -> ASTNode {
        let mut node = self.term();
        
        while let Some(token) = self.current().cloned() {
            match token.token_type {
                TokenType::Plus | TokenType::Minus => {
                    self.next();
                    let right = self.term(); // Right side parses at higher precedence
                    node = ASTNode::new(token)
                        .children(Some(Box::new(node)), Some(Box::new(right)));
                },
                _ => break,
            }
        }
        node
    }
    
    pub fn start(file_path: Rc<String>, tokens: Vec<Token>) -> Result<Vec<ASTNode>, JasonError> {
        let mut parser = Parser {
            tokens,
            index: 0
        };
        let mut nodes: Vec<ASTNode> = Vec::new();
        let mut errs: Vec<ASTNode> = Vec::new();
        loop {
            let node = parser.expr();
            if node.token.token_type == TokenType::EOT {
                break;
            }   
            
            if matches!(node.token.token_type, TokenType::ERR(_)) {
                errs.push(node);
                continue;
            }
            nodes.push(node);
        }  
        let errs:Vec<JasonError> = errs.iter().map(|err| 
            JasonError::new(JasonErrorKind::ParseError(err.token.plain()), file_path.clone(), None, "Parser Error")
        ).collect();

        if errs.len() > 0 {
            return Err(JasonError::new(
                JasonErrorKind::Bundle(
                    errs
                ), 
                file_path.clone(),
                None, 
                "Parser Error".to_string()
            ));
        }
        return Ok(nodes);
    }
}
