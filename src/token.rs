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

     pub fn pretty(&self) -> String {
        match &self.token_type {
            // ===== Literals =====
            TokenType::StringLiteral(s) => format!("{:?}", s), // adds quotes
            TokenType::NumberLiteral(n) => n.clone(),
            TokenType::FloatLiteral(f) => f.clone(),
            TokenType::BoolLiteral(b) => b.to_string(),

            // ===== Identifiers & Paths =====
            TokenType::ID => self.plain.clone(),
            TokenType::Path(p) => p.clone(),

            // ===== Function calls =====
            TokenType::FnCall(args)
            | TokenType::Import(args)
            | TokenType::Export(args)
            | TokenType::Use(args) => {
                let args_str = args.as_string_tuple();
                format!("{}{}", self.plain, args_str)
            }

            TokenType::LuaFnCall(args) => {
                let args_str = args.as_string_tuple();
                format!("{}{}!", self.plain, args_str)
            }

            // ===== Index (x[...]) =====
            TokenType::Index(args) => {
                let inside = args.as_string_list();
                format!("{}{}", self.plain, inside)
            }

            // ===== Template definitions =====
            TokenType::TemplateDef(input_args, block_args) => {
                let inputs = input_args.as_string_tuple();
                let blocks = block_args.as_string_tuple();
                format!("template {} {}", inputs, blocks)
            }

            // ===== Templates =====
            TokenType::Template(tokens, args) => {
                let name = tokens.iter()
                    .map(|t| t.pretty())
                    .collect::<Vec<_>>()
                    .join("");
                let args_str = args.as_string_tuple();
                format!("{}{}", name, args_str)
            }

            // ===== Composite structures =====
            TokenType::Block(args) => args.as_string_list().replace("[", "{ ").replace("]", " }"),
            TokenType::List(args) => args.as_string_list(),
            TokenType::Tuple(args) => args.as_string_tuple(),

            // ===== Inner generic type (Type<...>) =====
            TokenType::InnerType(name, toks) => {
                let inner = toks.iter()
                    .map(|t| t.pretty())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, inner)
            }

            // ===== Simple punctuation =====
            TokenType::Bar           => "|".to_string(),
            TokenType::Colon         => ":".to_string(),
            TokenType::Dot           => ".".to_string(),
            TokenType::Comma         => ",".to_string(),
            TokenType::OpenParen     => "(".to_string(),
            TokenType::ClosedParen   => ")".to_string(),
            TokenType::OpenBracket   => "[".to_string(),
            TokenType::ClosedBracket => "]".to_string(),
            TokenType::OpenCurly     => "{".to_string(),
            TokenType::ClosedCurly   => "}".to_string(),

            // ===== Operators =====
            TokenType::Equals => "=".to_string(),
            TokenType::Plus   => "+".to_string(),
            TokenType::Minus  => "-".to_string(),
            TokenType::Mult   => "*".to_string(),
            TokenType::Divide => "/".to_string(),
            TokenType::Mod    => "%".to_string(),

            // ===== Comparisons =====
            TokenType::LessThan    => "<".to_string(),
            TokenType::GreaterThan => ">".to_string(),

            // ===== Misc =====
            TokenType::Return     => "return".to_string(),
            TokenType::From       => "from".to_string(),
            TokenType::Repeat     => "repeat".to_string(),
            TokenType::Append     => "append".to_string(),
            TokenType::Unpack     => "unpack".to_string(),
            TokenType::FN         => "fn".to_string(),
            TokenType::Let        => "let".to_string(),
            TokenType::AS         => "as".to_string(),
            TokenType::Out        => "out".to_string(),
            TokenType::Const      => "const".to_string(),
            TokenType::Type       => "type".to_string(),
            TokenType::StringType => "string".to_string(),
            TokenType::NumberType => "int".to_string(),
            TokenType::FloatType  => "float".to_string(),
            TokenType::CharType   => "char".to_string(),
            TokenType::Embed      => "embed".to_string(),

            TokenType::DollarSign => "$".to_string(),

            // ===== Error / unknown =====
            TokenType::ERR(s) => format!("<err: {}>", s),
            TokenType::Unknown(c) => c.to_string(),

            // ===== Bounds / EOF =====
            TokenType::NewLine => "\n".to_string(),
            TokenType::StartComment => "/*".to_string(),
            TokenType::EndComment => "*/".to_string(),
            TokenType::EOF => "<EOF>".to_string(),
            TokenType::EOT => "<EOT>".to_string(),
            TokenType::Empty => "".to_string(),
        }
    }
}

pub trait ArgsToNode {
    fn to_nodes(&self) -> Vec<ASTNode>;
    fn as_string_list(&self) -> String;
    fn as_string_tuple(&self) -> String;
}
impl ArgsToNode for Args {
    fn to_nodes(&self) -> Vec<ASTNode> {
        // keep for actual parsing use
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

    fn as_string_list(&self) -> String {
        "[".to_string()
            + &self.iter()
                .map(|tokens| {
                    tokens
                        .iter()
                        .map(|t| t.pretty()) // 🚀 use pretty of raw token
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect::<Vec<_>>()
                .join(", ")
            + "]"
    }

    fn as_string_tuple(&self) -> String {
        "(".to_string()
            + &self
                .iter()
                .map(|tokens| {
                    tokens
                        .iter()
                        .map(|t| t.pretty()) // 🚀 preserve literal text
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect::<Vec<_>>()
                .join(", ")
            + ")"
    }
}

