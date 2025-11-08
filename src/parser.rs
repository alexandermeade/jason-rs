
use crate::token::Token;
use crate::token::TokenType;

type ChildNode = Option<Box<ASTNode>>;

#[derive(Debug, Clone)]
pub struct ASTNode {
    left: ChildNode,
    right: ChildNode,
    token: Token
}

impl ASTNode {
    pub fn new(token: Token) -> Self {
        ASTNode { left: None, right: None, token }
    }

    pub fn children(mut self, left: ChildNode, right: ChildNode) -> Self{
        self.left = left;
        self.right = right;
        self
    }

    pub fn EOT() -> Self {
        Self { left: None, right: None, token: Token::EOT() }
    }
}

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
            TokenType::StringLiteral(_) => {
                self.next();
                ASTNode::new(token) 
            },
            TokenType::FnCall(_) => {
                self.next();
                ASTNode::new(token).children(None, Some(Box::new(self.expr())))             
            },
            _ => { self.next(); ASTNode::new(token)},
        }
    }

    fn term(&mut self) -> ASTNode {
        self.factor()
    }

    fn expr(&mut self) -> ASTNode {
        // Start with the leftmost term
        let mut node = self.term();

        while let Some(token) = self.current().cloned() {
            match token.token_type {
                TokenType::Colon | TokenType::From => {
                    self.next(); // consume ':'
                    let right = self.term(); // parse right-hand side of ':'

                    // build new AST node where ':' is parent
                    node = ASTNode::new(token)
                        .children(Some(Box::new(node)), Some(Box::new(right)));
                }
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
