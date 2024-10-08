use std::{collections::HashSet, iter::Peekable, str::Chars, sync::LazyLock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
    pub value: TokenValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenValue {
    None,
    Integer(u32),
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Eof,
    Name,
    UnsignedInteger,
    String,
    // 以下キーワード
    Program,
    Var,
    Array,
    Of,
    Begin,
    End,
    If,
    Then,
    Else,
    Procedure,
    Return,
    Call,
    While,
    DO,
    Not,
    Or,
    Div,
    And,
    Char,
    Integer,
    Boolean,
    Read,
    Write,
    Readln,
    Writeln,
    True,
    False,
    Break,
    // 以下記号
    Plus,
    Minus,
    Star,
    Equal,
    NotEq,
    Less,
    LessEq,
    Great,
    GreatEq,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Assign,
    Dot,
    Comma,
    Colon,
    Semicolon,
    // どれでもない
    Unknown,
}

// 記号のトークンについて1文字のみの記号か，2文字以上の可能性がある記号かを保持する
// つまり，最初の文字を読んだ段階で確定できるものを集めた配列
static SYMBOLS_LEN_1: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    vec!["+", "-", "*", "=", "(", ")", "[", "]", ".", ",", ";"]
        .into_iter()
        .collect::<HashSet<&str>>()
});

fn match_keyword(ident: &str) -> Kind {
    if ident.len() == 1 || ident.len() > 10 {
        return Kind::Name;
    }
    match ident {
        "program" => Kind::Program,
        "var" => Kind::Var,
        "array" => Kind::Array,
        "of" => Kind::Of,
        "begin" => Kind::Begin,
        "end" => Kind::End,
        "if" => Kind::If,
        "then" => Kind::Then,
        "else" => Kind::Else,
        "procedure" => Kind::Procedure,
        "return" => Kind::Return,
        "call" => Kind::Call,
        "while" => Kind::While,
        "do" => Kind::DO,
        "not" => Kind::Not,
        "or" => Kind::Or,
        "div" => Kind::Div,
        "and" => Kind::And,
        "char" => Kind::Char,
        "integer" => Kind::Integer,
        "boolean" => Kind::Boolean,
        "read" => Kind::Read,
        "write" => Kind::Write,
        "readln" => Kind::Readln,
        "writeln" => Kind::Writeln,
        "true" => Kind::True,
        "false" => Kind::False,
        "break" => Kind::Break,
        _ => Kind::Name,
    }
}

fn match_symbol(symbol: &str) -> Kind {
    match symbol {
        "+" => Kind::Plus,
        "-" => Kind::Minus,
        "*" => Kind::Star,
        "=" => Kind::Equal,
        "<>" => Kind::NotEq,
        "<" => Kind::Less,
        "<=" => Kind::LessEq,
        ">" => Kind::Great,
        ">=" => Kind::GreatEq,
        "(" => Kind::LParen,
        ")" => Kind::RParen,
        "[" => Kind::LBracket,
        "]" => Kind::RBracket,
        ":=" => Kind::Assign,
        "." => Kind::Dot,
        "," => Kind::Comma,
        ":" => Kind::Colon,
        ";" => Kind::Semicolon,
        _ => Kind::Unknown,
    }
}

pub struct Lexer<'a> {
    pub source: &'a str,
    pub chars: Peekable<Chars<'a>>,
    // chars: Chars<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            // chars: source.chars(),
            chars: source.chars().peekable(),
        }
    }

    pub fn analyze(&mut self) -> Vec<Token> {
        let mut token_vec = Vec::new();
        loop {
            let token = self.read_next_token();
            if token.kind == Kind::Eof {
                token_vec.push(token);
                break;
            } else {
                token_vec.push(token);
            }
        }
        token_vec
    }

    pub fn read_next_token(&mut self) -> Token {
        while let Some(c) = self.chars.peek() {
            // EBNFのprogramに該当
            match c {
                // 分離子
                ' ' | '\t' | '\n' | '\r' | '{' | '/' => {
                    let c = self.chars.next().unwrap();
                    self.comment(c);
                }
                // 字句
                _ => {
                    let start = self.offset();
                    // peekで存在を確認しているのでunwrapでpanicは起きない
                    // token()関数の呼び出し元（つまりこの関数）でchars.next()を呼び出すことで，
                    // unwrap()でpanicが起きる可能性を排除するコードの距離を短くしている
                    let c = self.chars.next().unwrap();
                    let (kind, value) = self.token(c);
                    let end = self.offset();

                    return Token {
                        kind,
                        start,
                        end,
                        value,
                    };
                }
            }
        }
        let start = self.offset();
        let end = self.offset();

        Token {
            kind: Kind::Eof,
            start,
            end,
            value: TokenValue::None,
        }
    }

    fn offset(&self) -> usize {
        // self.chars.clone().count()の計算量を調べた方がいいかもしれない
        // self.source.len()は fat pointerによりO(1)だが，後者はO(n)の可能性あり

        // イテレータを消費し，Noneを返すまでの要素数を返す
        // ので，count()の計算量はO(n)になると思う
        // ややコストが高めかもしれない
        self.source.len() - self.chars.clone().count()
    }

    fn comment(&mut self, c: char) {
        // EBNFのcomment，注釈に該当
        match c {
            '{' => {
                self.comment_brace();
            }
            '/' => {
                self.comment_slashstar();
            }
            _ => {}
        }
    }
    fn comment_brace(&mut self) {
        for c in self.chars.by_ref() {
            if c == '}' {
                break;
            }
        }
    }

    fn comment_slashstar(&mut self) {
        enum State {
            Slash,
            Star,
            Other,
        }
        let mut state = State::Slash;
        for c in self.chars.by_ref() {
            match state {
                State::Slash => {
                    if c == '*' {
                        state = State::Star;
                    }
                }
                State::Star => {
                    if c == '/' {
                        break;
                    } else if c != '*' {
                        state = State::Other;
                    }
                }
                State::Other => {
                    if c == '*' {
                        state = State::Star;
                    }
                }
            }
        }
    }

    fn token(&mut self, c: char) -> (Kind, TokenValue) {
        // EBNFのtoken，字句に該当
        match c {
            'a'..='z' | 'A'..='Z' => self.name_keyword(c),
            '0'..='9' => self.unsigned_integer(c),
            '\'' => self.string(),
            _ => self.symbol(c),
        }
    }

    fn name_keyword(&mut self, c: char) -> (Kind, TokenValue) {
        let mut buf = String::from(c);

        while let Some(c) = self.chars.peek() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    buf.push(self.chars.next().unwrap());
                }
                _ => {
                    break;
                }
            }
        }
        let kind = match_keyword(&buf);
        match kind {
            Kind::Name => (kind, TokenValue::String(buf)),
            _ => (kind, TokenValue::None),
        }
    }

    fn unsigned_integer(&mut self, c: char) -> (Kind, TokenValue) {
        let mut buf = String::from(c);

        while let Some(c) = self.chars.peek() {
            match c {
                '0'..='9' => {
                    buf.push(self.chars.next().unwrap());
                }
                _ => {
                    break;
                }
            }
        }
        (
            Kind::UnsignedInteger,
            TokenValue::Integer(buf.parse().unwrap()),
        )
    }

    fn string(&mut self) -> (Kind, TokenValue) {
        enum State {
            SingleQuote,
            Other,
        }
        let mut state = State::Other;
        let mut buf = String::new();
        while let Some(c) = self.chars.peek() {
            match state {
                State::Other => {
                    if c == &'\'' {
                        state = State::SingleQuote;
                    }
                }
                State::SingleQuote => {
                    if c == &'\'' {
                        state = State::Other;
                        // 文字列中のシングルクォートは，2つで1つのシングルクォートとして扱う
                        // そのため，ここで1つ目のシングルクォートを取り除く
                        buf.pop();
                    } else {
                        break;
                    }
                }
            }
            buf.push(self.chars.next().unwrap());
        }

        // 最後尾がシングルクォートであれば，取り除く
        if buf.ends_with('\'') {
            buf.pop();
        }

        (Kind::String, TokenValue::String(buf))
    }

    fn symbol(&mut self, c: char) -> (Kind, TokenValue) {
        let mut buf = String::from(c);

        while let Some(c) = self.chars.peek() {
            // 1文字目の段階で確定する記号があるので，その場合break
            if SYMBOLS_LEN_1.contains(&buf.as_str()) {
                break;
            }
            let cc = String::from(*c);
            if match_symbol(&cc) == Kind::Unknown {
                break;
            }
            // // 1文字目の段階で確定する記号があるので，その場合break
            // ここにこの処理を置いていたとき，なぜかレキサーのテストケースは通過する
            // パーサーのテストでは，:=を: と字句解析して異常を起こしたのに なぜ差があるのか不明
            // if SYMBOLS_LEN_1.contains(&buf.as_str()) {
            //     break;
            // }
            buf.push(self.chars.next().unwrap());
        }

        let kind = match_symbol(&buf);
        if kind != Kind::Unknown {
            (kind, TokenValue::None)
        } else {
            (kind, TokenValue::String(buf))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let source = "
        {name}
        name1 name2name3
        {keyword}
        program var array
        of begin end
        if then else
        procedure return
        call while do
        not or div and
        char integer boolean
        read write readln writeln
        true false break
        {unsigned integer}
        0 1 9 255 
        {string}
        'string'
        'string1''string2'
        {symbol}
        + - * = <> < <= > >=
        ( ) [ ] := . , : ;
        ";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.analyze();

        let expected = vec![
            (Kind::Name, TokenValue::String("name1".to_string())),
            (Kind::Name, TokenValue::String("name2name3".to_string())),
            (Kind::Program, TokenValue::None),
            (Kind::Var, TokenValue::None),
            (Kind::Array, TokenValue::None),
            (Kind::Of, TokenValue::None),
            (Kind::Begin, TokenValue::None),
            (Kind::End, TokenValue::None),
            (Kind::If, TokenValue::None),
            (Kind::Then, TokenValue::None),
            (Kind::Else, TokenValue::None),
            (Kind::Procedure, TokenValue::None),
            (Kind::Return, TokenValue::None),
            (Kind::Call, TokenValue::None),
            (Kind::While, TokenValue::None),
            (Kind::DO, TokenValue::None),
            (Kind::Not, TokenValue::None),
            (Kind::Or, TokenValue::None),
            (Kind::Div, TokenValue::None),
            (Kind::And, TokenValue::None),
            (Kind::Char, TokenValue::None),
            (Kind::Integer, TokenValue::None),
            (Kind::Boolean, TokenValue::None),
            (Kind::Read, TokenValue::None),
            (Kind::Write, TokenValue::None),
            (Kind::Readln, TokenValue::None),
            (Kind::Writeln, TokenValue::None),
            (Kind::True, TokenValue::None),
            (Kind::False, TokenValue::None),
            (Kind::Break, TokenValue::None),
            (Kind::UnsignedInteger, TokenValue::Integer(0)),
            (Kind::UnsignedInteger, TokenValue::Integer(1)),
            (Kind::UnsignedInteger, TokenValue::Integer(9)),
            (Kind::UnsignedInteger, TokenValue::Integer(255)),
            (Kind::String, TokenValue::String("string".to_string())),
            (
                Kind::String,
                TokenValue::String("string1'string2".to_string()),
            ),
            (Kind::Plus, TokenValue::None),
            (Kind::Minus, TokenValue::None),
            (Kind::Star, TokenValue::None),
            (Kind::Equal, TokenValue::None),
            (Kind::NotEq, TokenValue::None),
            (Kind::Less, TokenValue::None),
            (Kind::LessEq, TokenValue::None),
            (Kind::Great, TokenValue::None),
            (Kind::GreatEq, TokenValue::None),
            (Kind::LParen, TokenValue::None),
            (Kind::RParen, TokenValue::None),
            (Kind::LBracket, TokenValue::None),
            (Kind::RBracket, TokenValue::None),
            (Kind::Assign, TokenValue::None),
            (Kind::Dot, TokenValue::None),
            (Kind::Comma, TokenValue::None),
            (Kind::Colon, TokenValue::None),
            (Kind::Semicolon, TokenValue::None),
            (Kind::Eof, TokenValue::None),
        ];

        for (i, token) in tokens.iter().enumerate() {
            println!("{:?}", token);
            assert_eq!(token.kind, expected[i].0);
            assert_eq!(token.value, expected[i].1);
        }
    }
}
