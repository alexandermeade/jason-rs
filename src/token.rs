//this token defintion is borrowed from my nyx language project

use crate::parser::Parser;
use crate::astnode::ASTNode;

pub type Args = Vec<Vec<Token>>;

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
    StructDef(Box<Token>, Args),
    Block(Args),
    List(Args),
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
    Use,
    Empty,
    DollarSign
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
            "use" => TokenType::Use,
            _ => TokenType::ID
        }
    }

    pub fn is_err(&self) -> bool {
        return match self {
            TokenType::ERR(_) => true,
            _ => false,
        }
    }

    pub fn is_bound(&self) -> bool { 
        return match *self {
            TokenType::EOT |
            TokenType::EOF => true,
            _ => false,
        }
    }

    pub fn is_type(&self) -> bool {
        return *self == TokenType::StringType || 
               *self == TokenType::NumberType || 
               *self == TokenType::FloatType  ||
               *self == TokenType::CharType   ||
               matches!(*self, TokenType::InnerType(_, _)); 
    }
    //NOTE: this function is terrible and remove it on release
    pub fn name(&self) -> String {
        let s = format!("{:?}", self);
        s.split('(').next().unwrap().to_string()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    plain: String,
    pub token_type: TokenType,
    row: usize,
    colmn: usize,
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
                _ => self,
            } 
        }

        return self;
    }   
    pub fn into_path(toks: Vec<Token>) -> Token {
        let mut result:String = "".to_string();
        let row = toks[0].row;
        let colmn = toks[0].colmn;
        for tok in toks {
            result.push_str(&tok.plain());
        }
        return Token::new(TokenType::Path(result.clone()), result, row, colmn);
    }

    pub fn to_json(&self) -> String {
        match self.token_type {
            _ => self.plain()
        }
    }

    pub fn plain(&self) -> String {
        self.plain.clone()
    }

    pub fn is_err(&self) -> bool {
        self.token_type.is_err()
    }

    pub fn is_bound(&self) -> bool {
        self.token_type.is_bound()
    }

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

