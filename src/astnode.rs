use crate::token::Token;
use crate::token::TokenType;
use crate::token::ArgsToNode;
use serde_json::{Value, Number};

use serde_json::Map;


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
    
    pub fn to_json(&self) -> serde_json::Value {
        match &self.token.token_type {
            TokenType::Block(_) => self.block_to_json(),
            TokenType::NumberLiteral(num) => serde_json::Value::Number(Number::from_f64(num.parse::<f64>().unwrap().into()).unwrap()),
            TokenType::StringLiteral(s) => serde_json::Value::String(s.to_string()), 
            tok => {
                //println!("[ERROR] to_json not implemented for tokentype {:?}", tok);
                serde_json::Value::String(tok.name())
            }
        }
    }

    fn block_to_json(&self) -> serde_json::Value {
        if let TokenType::Block(args) = &self.token.token_type {
            let nodes = args.to_nodes();

            let mut map = Map::new(); // this will become our JSON object

            for node in nodes {
                if node.token.token_type == TokenType::Colon {
                    let key_node = node.left.as_ref().expect("Missing key");
                    let value_node = node.right.as_ref().expect("Missing value");

                    if key_node.token.token_type != TokenType::ID {
                        panic!("Key must be an ID");
                    }

                    let key = key_node.token.plain();
                    let value = value_node.to_json(); // recursive call

                    map.insert(key, value);
                }
            }

            Value::Object(map)
        } else {
            panic!("block_to_json called on non-block token");
        }
    }

}
