use crate::token::Token;
use crate::token::TokenType::*;
use crate::token::ArgsToNode;
type ChildNode = Option<Box<ASTNode>>;

#[derive(Debug, Clone)]
pub struct ASTNode {
    pub left: ChildNode,
    pub right: ChildNode,
    pub token: Token
}
#[allow(dead_code)]
impl ASTNode {
    pub fn new(token: Token) -> Self {
        ASTNode { left: None, right: None, token }
    }

    pub fn empty() -> Self {
        ASTNode {left: None, right:None, token:Token::empty()}
    }

    pub fn children(mut self, left: ChildNode, right: ChildNode) -> Self{
        self.left = left;
        self.right = right;
        self
    }
    pub fn to_code(&self) -> String {
        match &self.token.token_type {
            StringLiteral(_) | NumberLiteral(_) | BoolLiteral(_) |
            FloatLiteral(_) | ID | Path(_) => self.token.plain(),

            // For binary ops, just show left + token + right if they exist
            Plus | Minus | Mult | Divide | Mod | Equals => {
                let left = self.left.as_ref().map(|n| n.token.plain()).unwrap_or_default();
                let right = self.right.as_ref().map(|n| n.token.plain()).unwrap_or_default();
                format!("{} {} {}", left, self.token.plain(), right)
            }

            // For function calls, show only the function name
            FnCall(_) | Import(_) | Export(_) | Use(_) | LuaFnCall(_) => {
                self.token.plain()
            }

            // Blocks and lists just print a placeholder
            Block(_) => "{...}".to_string(),
            List(_) => "[...]".to_string(),
            Tuple(_) => "(...)".to_string(),

            // Default fallback
            _ => self.token.plain()
        }
    }

}
