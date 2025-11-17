//this token defintion is borrowed from my nyx language project

use crate::parser::Parser;
use crate::astnode::ASTNode;

pub type Args = Vec<Vec<Token>>;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    ERR(String),
    Unknown(char),
    StringLiteral(String),
    NumberLiteral(String),
    BoolLiteral(bool),
    FloatLiteral(String),
    Import(Args),
    Export(Args),
    FnCall(Args),
    LuaFnCall(Args),
    Index(Args),
    // input args, block args
    TemplateDef(Args, Args),
    //StructDef(Box<Token>, Args),
    Block(Args),
    List(Args),
    Tuple(Args),
    Path(String),
    ID,
    Template(Vec<Token>, Args),
    InnerType(String, Vec<Token>),
    FunctionCall(String, Vec<Token>),
    Return,
    //special chars
    Bar,
    Colon,
    Dot,
    //whitespace
    NewLine,
    Comma,
    //enclosing symbols
    OpenParen,
    OpenBracket,
    OpenCurly,
    ClosedParen,
    ClosedBracket,
    ClosedCurly,
    // Math Symbols
    Equals,
    Plus,
    Minus,
    Mult,
    Divide,
    Mod, 
    StartComment,
    EndComment,
    EOF,
    EOT,
    //Logic Symbol
    LessThan,
    GreaterThan,
    // keywords
    From,
    Repeat,
    Append, 
    Unpack,
    FN,
    Let,
    AS,
    Out,
    Const,
    Type,
    StringType,
    NumberType,
    FloatType,
    CharType,
    Embed,
    Use(Args),
    Empty,
    DollarSign,
}

impl TokenType {
    
    pub fn find_keyword(content:&str) -> TokenType {
        match content {
            "from" => TokenType::From,
            "fn" => TokenType::FN, 
            "let" => TokenType::Let,
            "as" => TokenType::AS,
            "string" => TokenType::StringType,
            "int" => TokenType::NumberType,
            "true" => TokenType::BoolLiteral(true),
            "false" => TokenType::BoolLiteral(false),
            "char" => TokenType::CharType,
            "float" => TokenType::FloatType,
            "const" => TokenType::Const,
            "type" => TokenType::Type,
            "embed" => TokenType::Embed,
            "return" => TokenType::Return,
            "out" => TokenType::Out,
            "repeat" => TokenType::Repeat,
            "append" => TokenType::Append,
            "unpack" => TokenType::Unpack,
            _ => TokenType::ID
        }
    }

    //rust thinks this function isn't being called but is :/
    #[allow(dead_code)]
    pub fn is_err(&self) -> bool {
        return match self {
            TokenType::ERR(_) => true,
            _ => false,
        }
    }
      
    //rust thinks this function isn't being called but is :/
    #[allow(dead_code)]
    pub fn is_bound(&self) -> bool { 
        return match *self {
            TokenType::EOT |
            TokenType::EOF => true,
            _ => false,
        }
    }
    //rust thinks this function isn't being called but is :/
    #[allow(dead_code)]
    pub fn is_type(&self) -> bool {
        return *self == TokenType::StringType || 
               *self == TokenType::NumberType || 
               *self == TokenType::FloatType  ||
               *self == TokenType::CharType   ||
               matches!(*self, TokenType::InnerType(_, _)); 
    }

}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    plain: String,
    pub token_type: TokenType,
    pub row: usize,
    pub colmn: usize,
}

impl Token {
    pub fn new(token_type: TokenType, plain:String, row:usize, colmn:usize) -> Token {
        Token {
            token_type,
            plain,
            row,
            colmn
        }
    }

    pub fn find_fn_keyword(self) -> Self {
        let row = self.row;
        let colmn = self.colmn;
        let name:&str = &(self.plain().clone());
        let token_type = self.token_type.clone();

        if let TokenType::FnCall(args) = token_type {
            return match name {
                "import" => Token::new(TokenType::Import(args), name.to_string(), row, colmn),
                "export" => Token::new(TokenType::Export(args), name.to_string(), row, colmn),
                "use" => Token::new(TokenType::Use(args), name.to_string(), row, colmn),
                _ => self,
            } 
        }

        return self;
    }   

    pub fn plain(&self) -> String {
        self.plain.clone()
    }
    
    #[allow(dead_code)]
    pub fn is_err(&self) -> bool {
        self.token_type.is_err()
    }

    #[allow(dead_code)]
    pub fn is_bound(&self) -> bool {
        self.token_type.is_bound()
    }

    #[allow(dead_code)]
    pub fn empty() -> Token {
        Token {
            token_type: TokenType::Empty,
            plain: String::from(""),
            row: 0,
            colmn: 0
        }
    }
}

pub trait ArgsToNode {
    fn to_nodes(&self) -> Vec<ASTNode>;
}

impl ArgsToNode for Args {
    fn to_nodes(&self) -> Vec<ASTNode> {
        self.iter()
            .flat_map(|tokens| {
                let filtered: Vec<Token> = tokens
                    .iter()
                    .filter(|token| token.token_type != TokenType::NewLine)
                    .cloned()
                    .collect();

                Parser::start(filtered)
            })
            .collect::<Vec<ASTNode>>()
    }
}

