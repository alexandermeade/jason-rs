use crate::jason_errors::JasonResult;

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
    
    fn factor(&mut self) -> JasonResult<ASTNode> {
        let token = self.current().cloned().unwrap_or(Token::new(TokenType::EOT, "EOT".to_string(), 1, 1));
        
        return match token.token_type {
            TokenType::ID                   | 
            TokenType::FloatLiteral(_)      |
            TokenType::IntLiteral(_)        |
            TokenType::NumberType           | 
            TokenType::IntType              |
            TokenType::FloatType            |
            TokenType::StringType           |
            TokenType::BoolType             | 
            TokenType::AnyType              | 
            TokenType::NullType             | 
            TokenType::List(_)              | 
            TokenType::FnCall(_)            |
            TokenType::LuaFnCall(_)         |
            TokenType::Import(_)            |
            TokenType::StringLiteral(_)     | 
            TokenType::CompositeString(_,_) | 
            TokenType::BoolLiteral(_)       |
            TokenType::DollarSign           |
            TokenType::Use(_)               |
            TokenType::StringConverion(_)   | 
            TokenType::IntConverion(_)      | 
            TokenType::FloatConverion(_)    | 
            TokenType::Mult => {
                self.next();
                Ok(ASTNode::new(token)) 
            },
            TokenType::Out | TokenType::Include | TokenType::Info | TokenType::InfoT => {
                self.next(); // consume the keyword
                let rhs = self.expr()?; // Parse what comes after
                Ok(ASTNode::new(token).children(None, Some(Box::new(rhs))))
                },

            TokenType::Minus => {
                self.next();
                let num_token = self.current().cloned().unwrap_or(Token::new(TokenType::EOT, "EOT".to_string(), 1, 1));
                match num_token.token_type {
                    TokenType::IntLiteral(n) => {
                        let tok = Token::new(TokenType::IntLiteral(-n), format!("{}", -n), token.row, token.colmn);
                        self.next(); 
                        return Ok(ASTNode::new(tok));
                    },
                    TokenType::FloatLiteral(n) => {
                        let tok = Token::new(TokenType::FloatLiteral(-n), format!("{}", -n), token.row, token.colmn);
                        self.next(); 
                        return Ok(ASTNode::new(tok));                       
                    },
                    _ => {
                        return Err(JasonError::new(JasonErrorKind::ParseError(format!("{:?}", token.token_type)), Rc::new("".to_string()), None, format!("the unary - expects a number literal after wards")))
                    }
                }
            },
            TokenType::OpenParen => {
                self.next_insure(TokenType::OpenParen);
                let node = self.expr()?;
                if !self.next_insure(TokenType::ClosedParen) {
                    // Ensure we have a closing ')'
                    return Ok(ASTNode::new(Token::new(
                        TokenType::ERR(format!("Expected ')' but not found: {}", token.plain())),
                        "Parse error".to_string(),
                        token.row,
                        token.colmn,
                    )));
                }
                Ok(node)
            },
            TokenType::Plus  => {
                 return Ok(ASTNode::new(Token::new(
                    TokenType::ERR(format!("unexpected '+' found {}", token.plain())),
                    "Parse error".to_string(),
                    token.row,
                    token.colmn,
                )));               
            },
            _ => { self.next(); Ok(ASTNode::new(token))},
        }
    }
    
    fn term(&mut self) -> JasonResult<ASTNode> {

        if let Some(token) = self.current().cloned() {
            match token.token_type {
                TokenType::GreaterThan |
                TokenType::LessThan |
                TokenType::GreaterThanEqualTo |
                TokenType::LessThanEqualTo => {
                    self.next();
                    let right = self.factor()?;
                    return Ok(ASTNode::new(token).children(None, Some(Box::new(right))));
                },
                _ => {}
            }
        }


        let mut node = self.factor()?;


        while let Some(token) = self.current().cloned() {
            match token.token_type {
                TokenType::VarianceOperator => {
                    self.next(); // consume the operator
                    node = ASTNode::new(token)
                        .children(Some(Box::new(node)), None);

                },

                TokenType::Repeat | TokenType::Mult | TokenType::Divide | TokenType::Mod => {
                    self.next();
                    let right = self.factor()?;
                    node = ASTNode::new(token)
                        .children(Some(Box::new(node)), Some(Box::new(right)));
                },
                TokenType::At          | 
                TokenType::Pick        | 
                TokenType::UPick       |
                TokenType::With        |
                TokenType::Map(_)   => {
                    self.next();
                    let right = self.addition()?;
                    node = ASTNode::new(token)
                        .children(Some(Box::new(node)), Some(Box::new(right)));
                },
                TokenType::GreaterThan |
                TokenType::LessThan    |
                TokenType::GreaterThanEqualTo|
                TokenType::LessThanEqualTo   => {
                    self.next(); // consume 'out'
                    node = ASTNode::new(token)
                        .children(None, Some(Box::new(self.factor()?)));
                },
                _ => break,
            }
        }
        Ok(node)
    }
    
    fn expr(&mut self) -> JasonResult<ASTNode> {
        // Fallback: normal expressions
        let mut node = self.addition()?;
        while let Some(token) = self.current().cloned() {
            match token.token_type {
                TokenType::Colon | TokenType::From | TokenType::AS | TokenType::Append |
                TokenType::Equals | TokenType::DoubleColon | TokenType::Narwhal |
                TokenType::SpiderWalrus => {
                    self.next();
                    let right = self.addition()?;
                    node = ASTNode::new(token)
                        .children(Some(Box::new(node)), Some(Box::new(right)));
                },
                _ => break,
            }
        }
        Ok(node)
    }

    
    fn addition(&mut self) -> JasonResult<ASTNode> {
        let mut node = self.term()?;
        
        while let Some(token) = self.current().cloned() {
            match token.token_type {
                TokenType::Plus | TokenType::Minus | TokenType::Bar | TokenType::Merge | TokenType::While => {
                    self.next();
                    let right = self.term()?; // Right side parses at higher precedence
                    node = ASTNode::new(token)
                        .children(Some(Box::new(node)), Some(Box::new(right)));
                },
                _ => break,
            }
        }
        Ok(node)
    }
    
    pub fn start(file_path: Rc<String>, tokens: Vec<Token>) -> JasonResult<Vec<ASTNode>> {
        let mut parser = Parser {
            tokens,
            index: 0
        };
        let mut nodes: Vec<ASTNode> = Vec::new();
        let mut errs: Vec<ASTNode> = Vec::new();
        loop {
            let node = parser.expr()?;
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
