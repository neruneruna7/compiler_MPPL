use std::{
    collections::{HashMap, HashSet},
    iter::Peekable,
    str::Chars,
    sync::LazyLock,
};

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

/// キーワードとKindの対応を保持するマップを作成
/// matchで総当たりしてもいいが，こっちの方が速そう
static KEYWORDS: LazyLock<HashMap<&'static str, Kind>> = LazyLock::new(|| {
    [
        ("program", Kind::Program),
        ("var", Kind::Var),
        ("array", Kind::Array),
        ("of", Kind::Of),
        ("begin", Kind::Begin),
        ("end", Kind::End),
        ("if", Kind::If),
        ("then", Kind::Then),
        ("else", Kind::Else),
        ("procedure", Kind::Procedure),
        ("return", Kind::Return),
        ("call", Kind::Call),
        ("while", Kind::While),
        ("do", Kind::DO),
        ("not", Kind::Not),
        ("or", Kind::Or),
        ("div", Kind::Div),
        ("and", Kind::And),
        ("char", Kind::Char),
        ("integer", Kind::Integer),
        ("boolean", Kind::Boolean),
        ("read", Kind::Read),
        ("write", Kind::Write),
        ("readln", Kind::Readln),
        ("writeln", Kind::Writeln),
        ("true", Kind::True),
        ("false", Kind::False),
        ("break", Kind::Break),
    ]
    .into_iter()
    .collect()
});

fn match_keyword(ident: &str) -> Kind {
    KEYWORDS.get(ident).copied().unwrap_or(Kind::Name)
}

// 記号とKindの対応を保持するマップを作成
static SYMBOLS: LazyLock<HashMap<&'static str, Kind>> = LazyLock::new(|| {
    [
        ("+", Kind::Plus),
        ("-", Kind::Minus),
        ("*", Kind::Star),
        ("=", Kind::Equal),
        ("<>", Kind::NotEq),
        ("<", Kind::Less),
        ("<=", Kind::LessEq),
        (">", Kind::Great),
        (">=", Kind::GreatEq),
        ("(", Kind::LParen),
        (")", Kind::RParen),
        ("[", Kind::LBracket),
        ("]", Kind::RBracket),
        (":=", Kind::Assign),
        (".", Kind::Dot),
        (",", Kind::Comma),
        (":", Kind::Colon),
        (";", Kind::Semicolon),
    ]
    .into_iter()
    .collect()
});

fn match_symbol(symbol: &str) -> Kind {
    SYMBOLS.get(symbol).copied().unwrap_or(Kind::Unknown)
}

pub struct Lexer<'a> {
    pub source: &'a str,
    pub chars: Peekable<Chars<'a>>,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            // chars: source.chars(),
            chars: source.chars().peekable(),
            position: 0,
        }
    }

    // 次の文字を見る（消費しない）
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    // 文字を消費するたびにpositionを更新
    fn next_char(&mut self) -> Option<char> {
        if let Some(c) = self.chars.next() {
            // self.position += c.len_utf8(); //マルチバイト文字のときに，オフセットがおかしくなるのでこれはだめ
            // あくまで文字数を見ている
            self.position += 1;
            Some(c)
        } else {
            None
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
        while let Some(c) = self.peek() {
            // EBNFのprogramに該当
            match c {
                // 分離子
                ' ' | '\t' | '\n' | '\r' | '{' | '/' => {
                    let c = self.next_char().unwrap();
                    self.comment(c);
                }
                // 字句
                _ => {
                    let start = self.position;
                    // peekで存在を確認しているのでunwrapでpanicは起きない
                    // token()関数の呼び出し元（つまりこの関数）でnext_char()を呼び出すことで，
                    // unwrap()でpanicが起きる可能性を排除するコードの距離を短くしている
                    let c = self.next_char().unwrap();
                    let (kind, value) = self.token(c);
                    let end = self.position;

                    return Token {
                        kind,
                        start,
                        end,
                        value,
                    };
                }
            }
        }
        let start = self.position;
        let end = self.position;

        Token {
            kind: Kind::Eof,
            start,
            end,
            value: TokenValue::None,
        }
    }

    // コメント周りがおかしくてエラーになる可能性？ コメントの部分をトークン進められていない？
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
        while let Some(c) = self.next_char() {
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
        while let Some(c) = self.next_char() {
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
                    buf.push(self.next_char().unwrap());
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
                    buf.push(self.next_char().unwrap());
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
            buf.push(self.next_char().unwrap());
        }

        // 最後尾がシングルクォートであれば，取り除く
        if buf.ends_with('\'') {
            buf.pop();
        }

        (Kind::String, TokenValue::String(buf))
    }

    fn symbol(&mut self, c: char) -> (Kind, TokenValue) {
        let mut buf = String::from(c);

        // 現在のバッファ + 次の文字で有効な記号になるか確認
        if let Some(&next_c) = self.chars.peek() {
            let mut temp_buf = buf.clone();
            temp_buf.push(next_c);

            // 2文字の組み合わせが有効な記号であれば、次の文字も読み込む
            if let Some(&kind) = SYMBOLS.get(temp_buf.as_str()) {
                self.next_char(); // 次の文字を消費
                buf.push(next_c);
                return (kind, TokenValue::None);
            }
        }
        let a = 1;

        // 1文字だけで完結する記号の場合
        let kind = match_symbol(&buf);
        if kind != Kind::Unknown {
            (kind, TokenValue::None)
        } else {
            // 不明な記号の場合、単にStringとして返す
            (Kind::Unknown, TokenValue::String(buf))
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
        'string1''文字列🦀'
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
                TokenValue::String("string1'文字列🦀".to_string()),
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

    #[test]
    fn test_offset() {
        let source = "abc abc abc";
        let mut lexer = Lexer::new(source);
        let lex = lexer.analyze();
        let ans = vec![
            (0, 3),
            (4, 7),
            (8, 11),
            (11, 11), // EOF
        ];
        eprintln!("{:?}", lex);
        for (i, token) in lex.iter().enumerate() {
            assert_eq!(token.start, ans[i].0);
            assert_eq!(token.end, ans[i].1);
        }
    }
}
