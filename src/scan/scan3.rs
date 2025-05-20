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

    fn position(&self) -> usize {
        self.position
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

    /// EBNFのseparatorに該当
    fn separator(&mut self) -> Option<Token> {
        // { comment } または /* comment */ を処理する有限オートマトン
        // ここでは，{ comment } または /* comment */ の部分を無視する
        // ただし，コメントの内容は収集しておく
        while let Some(c) = self.peek() {
            dbg!("{:?}", c);

            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    // ホワイトスペースはスキップ
                    self.next_char();
                }
                '{' | '/' => return self.comment(),
                _ => unreachable!(),
            }
        }
        None
    }
    fn comment(&mut self) -> Option<Token> {
        // { comment } または /* comment */ を処理する有限オートマトン
        enum State {
            Initial,       // 初期状態
            Brace,         // { を読んだ後
            Slash,         // / を読んだ後
            SlashStar,     // /* を読んだ後（コメント内）
            StarInComment, // /*...* の状態（終了 '/' を待っている）
        }

        let mut state = State::Initial;
        let mut buf = Vec::new();
        let start = self.position;

        while let Some(c) = self.next_char() {
            match state {
                State::Initial => match c {
                    '{' => state = State::Brace,
                    '/' => state = State::Slash,
                    _ => {
                        unreachable!()
                    }
                },
                State::Brace => {
                    if c == '}' {
                        // 波括弧コメントの終了 - コメントを処理完了
                        return Some(Token {
                            kind: Kind::Unknown,
                            start,
                            end: self.position,
                            value: TokenValue::String(buf),
                        });
                    } else {
                        // コメント内容を収集（オプション）
                        buf.push(c);
                    }
                }
                State::Slash => {
                    if c == '*' {
                        state = State::SlashStar;
                    } else {
                        unreachable!()
                    }
                }
                State::SlashStar => {
                    if c == '*' {
                        state = State::StarInComment;
                    } else {
                        // コメント内容を収集（オプション）
                        buf.push(c);
                    }
                }
                State::StarInComment => {
                    if c == '/' {
                        // スラッシュスターコメントの終了 - コメントを処理したので None を返す
                        return Some(Token {
                            kind: Kind::Unknown,
                            start,
                            end: self.position,
                            value: TokenValue::String(buf),
                        });
                    } else if c != '*' {
                        // '*'が連続している場合は、StarInComment状態を維持
                        // それ以外はコメント内に戻る
                        buf.push(c);
                        state = State::SlashStar;
                    }
                    // '*'の場合はStarInComment状態を維持
                }
            }
        }

        // ファイル終端に達した場合（適切に閉じられていないコメント）
        // 未閉じのコメントはエラーとして扱う
        None
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
    fn test_comment1() {
        // 実際はコメントはトークンにならない仕様だが，試験的にトークンにしている

        let source = "{ comment }";
        let mut lexer = Lexer::new(source);
        let token = lexer.comment().unwrap();
        assert_eq!(token.kind, Kind::Unknown);
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 11);
        assert_eq!(
            token.value,
            TokenValue::String(" comment ".chars().collect())
        );
    }

    #[test]
    fn test_comment2() {
        let source = "/* comment */";
        let mut lexer = Lexer::new(source);
        let token = lexer.comment().unwrap();
        assert_eq!(token.kind, Kind::Unknown);
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 13);
        assert_eq!(
            token.value,
            TokenValue::String(" comment ".chars().collect())
        );
    }
    #[test]
    fn test_separator1() {
        let source = " \t\n\r";
        let mut lexer = Lexer::new(source);
        let token = lexer.separator();
        assert_eq!(token, None);
        assert_eq!(lexer.position, 4);
    }

    #[test]
    fn test_separator2() {
        let source = " \t\n\r{comment}\r";
        let mut lexer = Lexer::new(source);
        // 最初の4文字と最後の1文字はスキップされる
        let token = lexer.separator();
        assert_eq!(
            token,
            Some(Token {
                kind: Kind::Unknown,
                start: 4,
                end: 13,
                value: TokenValue::String("comment".chars().collect())
            })
        );
    }

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
