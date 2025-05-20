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
    String(Vec<char>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Name,
    Keyword,
    UnsignedInt,
    String,
    Symbol,
    Unknown,
}

/// キーワードとKindの対応を保持するマップを作成
/// matchで総当たりしてもいいが，こっちの方が速そう
static KEYWORDS: LazyLock<HashSet<Vec<char>>> = LazyLock::new(|| {
    [
        "program",
        "var",
        "array",
        "of",
        "begin",
        "end",
        "if",
        "then",
        "else",
        "procedure",
        "return",
        "call",
        "while",
        "do",
        "not",
        "or",
        "div",
        "and",
        "char",
        "integer",
        "boolean",
        "read",
        "write",
        "readln",
        "writeln",
        "true",
        "false",
        "break",
    ]
    .into_iter()
    .map(|k| k.chars().collect())
    .collect()
});

fn match_keyword(ident: &[char]) -> Option<Kind> {
    if KEYWORDS.contains(ident) {
        Some(Kind::Keyword)
    } else {
        None
    }
}

// 記号とKindの対応を保持するマップを作成
static SYMBOLS: LazyLock<HashSet<Vec<char>>> = LazyLock::new(|| {
    [
        "+", "-", "*", "=", "<>", "<", "<=", ">", ">=", "(", ")", "[", "]", ":=", ".", ",", ":",
        ";",
    ]
    .into_iter()
    .map(|k| k.chars().collect())
    .collect()
});

fn match_symbol(symbol: &[char]) -> Option<Kind> {
    if SYMBOLS.contains(symbol) {
        Some(Kind::Symbol)
    } else {
        None
    }
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
            let token = self.next_token();
            match token {
                Some(token) => {
                    token_vec.push(token);
                }
                None => break,
            }
        }
        token_vec
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.program()
    }

    /// EBNFのprogramに該当
    fn program(&mut self) -> Option<Token> {
        // EBNFのprogramに該当
        while let Some(c) = self.peek() {
            // EBNFのprogramに該当
            // -> program相当ならそういう風に関数切り出せ
            match c {
                // 分離子
                ' ' | '\t' | '\n' | '\r' | '{' | '/' => {
                    self.separator();
                }
                // 字句
                _ => return self.token(),
            }
        }
        None
    }

    fn separator(&mut self) -> Token {
        // EBNFのseparatorに該当
        let start = self.position;
        loop {
            let c = self.next_char().unwrap();
            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    // 空白，タブ，改行はスキップ
                    continue;
                }
                '{' | '/' => {
                    // コメント
                    self.comment(c);
                    break;
                }
                _ => {
                    unreachable!()
                }
            }
        }
        let end = self.position;
        Token {
            kind: Kind::Unknown,
            start,
            end,
            value: TokenValue::None,
        }
    }

    // コメント周りがおかしくてエラーになる可能性？ コメントの部分をトークン進められていない？
    /// EBNFのcomment，注釈に該当
    fn comment(&mut self, c: char) {
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

    /// EBNFのtoken，字句に該当
    fn token(&mut self) -> Option<Token> {
        let start = self.position;
        let c = self.peek().unwrap();
        let t = match c {
            'a'..='z' | 'A'..='Z' => self.name_keyword(),
            '0'..='9' => self.unsigned_integer(),
            '\'' => self.string(),
            _ => self.symbol(),
        };
        let end = self.position;
        t
    }

    /// 名前またはキーワードを取得
    fn name_keyword(&mut self) -> Option<Token> {
        let start = self.position;
        let c = self.next_char().unwrap();
        let mut buf = vec![c];

        while let Some(c) = self.chars.peek() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    buf.push(*c);
                    self.next_char();
                }
                _ => {
                    break;
                }
            }
        }
        let kind = match_keyword(&buf);
        match kind {
            Some(k) => {
                let end = self.position;
                Some(Token {
                    kind: k,
                    start,
                    end,
                    value: TokenValue::None,
                })
            }
            None => {
                // キーワードでなければ識別子
                let end = self.position;
                Some(Token {
                    kind: Kind::Name,
                    start,
                    end,
                    value: TokenValue::String(buf),
                })
            }
        }
    }

    fn unsigned_integer(&mut self) -> Option<Token> {
        let start = self.position;
        let c = self.next_char().unwrap();
        let mut buf = vec![c];

        while let Some(c) = self.chars.peek() {
            match c {
                '0'..='9' => {
                    buf.push(*c);
                    self.next_char();
                }
                _ => {
                    break;
                }
            }
        }
        let end = self.position;

        // Vec<char>からu32に変換
        let num_str: String = buf.iter().collect();
        let num = num_str.parse().unwrap();

        Some(Token {
            kind: Kind::UnsignedInt,
            start,
            end,
            value: TokenValue::Integer(num),
        })
    }

    fn string(&mut self) -> Option<Token> {
        enum State {
            SingleQuote,
            Other,
        }
        let mut state = State::Other;
        let mut buf = Vec::new();

        let start = self.position;
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
            buf.push(*c);
            self.next_char();
        }
        let end = self.position;

        Some(Token {
            kind: Kind::String,
            start,
            end,
            value: TokenValue::String(buf),
        })
    }

    fn symbol(&mut self) -> Option<Token> {
        let start = self.position;
        let c = self.next_char().unwrap();
        let mut buf = vec![c];

        // 現在のバッファ + 次の文字で有効な記号になるか確認
        if let Some(&next_c) = self.chars.peek() {
            let mut temp_buf = buf.clone();
            temp_buf.push(next_c);

            // 2文字の組み合わせが有効な記号であれば、次の文字も読み込む
            // if let Some(_) = SYMBOLS.get(&temp_buf) {
            if SYMBOLS.contains(&temp_buf) {
                self.next_char(); // 次の文字を消費
                buf.push(next_c);
                let end = self.position;
                return Some(Token {
                    kind: Kind::Symbol,
                    start,
                    end,
                    value: TokenValue::None,
                });
            }
        }

        // 1文字だけで完結する記号の場合
        let kind = match_symbol(&buf);
        let end = self.position;
        match kind {
            Some(k) => Some(Token {
                kind: k,
                start,
                end,
                value: TokenValue::None,
            }),
            None => {
                // 1文字だけでは有効な記号にならない場合
                // ここではUnknownとして扱う
                Some(Token {
                    kind: Kind::Unknown,
                    start,
                    end,
                    value: TokenValue::String(buf),
                })
            }
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
            (Kind::Name, TokenValue::String("name1".chars().collect())),
            (
                Kind::Name,
                TokenValue::String("name2name3".chars().collect()),
            ),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::None),
            (Kind::Keyword, TokenValue::Integer(0)),
            (Kind::Keyword, TokenValue::Integer(1)),
            (Kind::Keyword, TokenValue::Integer(9)),
            (Kind::Keyword, TokenValue::Integer(255)),
            (Kind::String, TokenValue::String("string".chars().collect())),
            (
                Kind::String,
                TokenValue::String("string1'文字列🦀".chars().collect()),
            ),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
            (Kind::Symbol, TokenValue::None),
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
