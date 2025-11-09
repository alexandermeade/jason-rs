use crate::token::Token;
use crate::token::TokenType;


type ChildNode = Option<Box<ASTNode>>;

#[derive(Debug, Clone)]
pub struct ASTNode {
    pub left: ChildNode,
    pub right: ChildNode,
    pub token: Token
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

    pub fn into_json(&self) -> String {
        "".to_string() 
    }
    

}
