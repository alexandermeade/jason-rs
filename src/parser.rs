use crate::token::Token;
use crate::token::TokenType;
use crate::astnode::ASTNode;

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
            TokenType::Mult => {
                self.next();
                ASTNode::new(token) 
            },
            _ => { self.next(); ASTNode::new(token)},
        }
    }

    fn term(&mut self) -> ASTNode {
        self.factor()
    }
    fn expr(&mut self) -> ASTNode {
        // Check for Out FIRST, BEFORE parsing any terms
        if let Some(token) = self.current() {
            if token.token_type == TokenType::Out {
                let out_token = token.clone();
                self.next(); // consume 'out'
                let right = self.term(); // parse what comes after 'out'
                return ASTNode::new(out_token)
                    .children(None, Some(Box::new(right)));
            }
        }

        // NOW parse the leftmost term
        let mut node = self.term();

        while let Some(token) = self.current().cloned() {
            match token.token_type {
                TokenType::Colon  | 
                TokenType::From   |
                TokenType::AS     |
                TokenType::Equals => {
                    self.next(); // consume the operator
                    let right = self.term(); // parse right-hand side
                    // build new AST node where operator is parent
                    node = ASTNode::new(token)
                        .children(Some(Box::new(node)), Some(Box::new(right)));
                },
                // REMOVE THIS ENTIRE CASE - Out is now handled at the top
                // TokenType::Out => { ... },
                _ => break,
            }
        }
        node
    }
    pub fn start(tokens: Vec<Token>) -> Vec<ASTNode> {
        let mut parser = Parser {
            tokens,
            index: 0
        };
        let mut nodes:Vec<ASTNode> = Vec::new();
        loop {
            let node = parser.expr();
            //println!("{:#?}", node);
            if node.token.token_type == TokenType::EOT {
                break;
            }   
            nodes.push(node);
        }         
        return nodes
    }
}
