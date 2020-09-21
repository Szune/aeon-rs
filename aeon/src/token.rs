#[derive(Debug)]
pub enum Token {
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    LeftParenthesis,
    RightParenthesis,
    Colon,
    Comma,
    Identifier(String),
    String(String),
    Integer(i64),
    Double(f64),
    At,
    Nil,
}
