use crate::token::Token;
use crate::token::TokenType::*;
type ChildNode = Option<Box<ASTNode>>;

#[derive(Debug, Clone)]
pub struct ASTNode {
    pub left: ChildNode,
    pub right: ChildNode,
    pub plain_sum: String,
    pub parent: ChildNode,
    pub token: Token
}

#[allow(dead_code)]
impl ASTNode {
    pub fn new(token: Token) -> Self {
        let plain = token.pretty();
        ASTNode { left: None, right: None, token, plain_sum: plain, parent: None }
    }

    pub fn empty() -> Self {
        ASTNode {left: None, right:None, token:Token::empty(), plain_sum: String::new(), parent: None}
    }

    pub fn parent(&self) -> Option<&ASTNode> {
        self.parent.as_deref()
    }

    pub fn root(&self) -> &ASTNode {
        let mut node = self;
        while let Some(parent) = node.parent() {
            node = parent;
        }
        node
    }


    pub fn children(mut self, left: ChildNode, right: ChildNode) -> Self {
        // If left is missing, create a placeholder ASTNode
        let left_node = left.or_else(|| Some(Box::new(ASTNode::empty())));
        // If right is missing, create a placeholder ASTNode

        let right_node = right.or_else(|| Some(Box::new(ASTNode::empty())));


        self.left = left_node;
        self.right = right_node;

        // Build plain_sum using guaranteed children
        let left_text = self.left.as_ref().unwrap().plain_sum.clone();
        let right_text = self.right.as_ref().unwrap().plain_sum.clone();

        self.plain_sum = format!("{} {} {}", left_text, self.token.plain(), right_text);

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
