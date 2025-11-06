//this token defintion is borrowed from my nyx language project

pub type Args = Vec<Vec<Token>>;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    ERR(String),
    Unknown(char),
    StringLiteral(String),
    NumberLiteral(String),
    FloatLiteral(String),
    FnCall(Args),
    Index(Args),
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
    FN,
    Let,
    AS,
    Const,
    Type,
    StringType,
    NumberType,
    FloatType,
    CharType,
    Embed,
    Use,
    Empty,
}

impl TokenType {
    pub fn find_keyword(content:&str) -> TokenType {
        match content {
            "fn" => TokenType::FN, 
            "let" => TokenType::Let,
            "as" => TokenType::AS,
            "string" => TokenType::StringType,
            "int" => TokenType::NumberType,
            "char" => TokenType::CharType,
            "float" => TokenType::FloatType,
            "const" => TokenType::Const,
            "type" => TokenType::Type,
            "embed" => TokenType::Embed,
            "return" => TokenType::Return,
            "use" => TokenType::Use,
            _ => TokenType::ID
        }
    }
    
    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            // Special characters
            TokenType::Colon => Some(":"),
            TokenType::Dot => Some("."),
            TokenType::Comma => Some(","),
            TokenType::OpenParen => Some("("),
            TokenType::ClosedParen => Some(")"),
            TokenType::OpenBracket => Some("["),
            TokenType::ClosedBracket => Some("]"),
            TokenType::OpenCurly => Some("{"),
            TokenType::ClosedCurly => Some("}"),

            // Math symbols
            TokenType::Equals => Some("="),
            TokenType::Plus => Some("+"),
            TokenType::Minus => Some("-"),
            TokenType::Mult => Some("*"),
            TokenType::Divide => Some("/"),
            TokenType::Mod => Some("%"),

            // Logic symbols
            TokenType::LessThan => Some("<"),
            TokenType::GreaterThan => Some(">"),

            // Keywords
            TokenType::FN => Some("fn"),
            TokenType::Let => Some("let"),
            TokenType::AS => Some("as"),
            TokenType::Const => Some("const"),
            TokenType::Type => Some("type"),
            TokenType::StringType => Some("string"),
            TokenType::NumberType => Some("number"),
            TokenType::FloatType => Some("float"),
            TokenType::CharType => Some("char"),
            TokenType::Embed => Some("embed"),
            TokenType::Use => Some("use"),
            TokenType::Empty => Some("empty"),

            // Special / control tokens
            TokenType::StartComment => Some("//"),
            TokenType::EndComment => Some("*/"),
            TokenType::NewLine => Some("\n"),
            TokenType::EOF => Some("<EOF>"),
            TokenType::EOT => Some("<EOT>"),
            _ => None
    }
}
    pub fn is_err(&self) -> bool {
        return match self {
            TokenType::ERR(content) => true,
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
    
    pub fn into_path(toks: Vec<Token>) -> Token {
        let mut result:String = "".to_string();
        let row = toks[0].row;
        let colmn = toks[0].colmn;
        for tok in toks {
            result.push_str(&tok.plain());
        }
        return Token::new(TokenType::Path(result.clone()), result, row, colmn);
    }
    pub fn EOT() -> Self {
        Self::new(TokenType::EOT, "EOT".to_string(), 1, 1)
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

