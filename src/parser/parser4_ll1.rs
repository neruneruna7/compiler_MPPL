use error::SyntaxError;
use st::{Node, NodeKind};

use crate::scan::scan3::{self, Kind, Lexer, Token};

mod error;
mod first_set;
mod st;

pub(crate) type SyntaxResult = std::result::Result<Node, SyntaxError>;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum SyntaxKind {
    Program,
    Block,
    VariableDeclaration,
    VariableNames,
    VariableName,
    Type,
    StandardType,
    ArrayType,
    SubprogramDeclaration,
    ProcedureName,
    FormalParameters,
    CompoundStatement,
    Statement,
    ConditionStatement,
    IterationStatement,
    ExitStatement,
    CallStatement,
    Expressions,
    ReturnStatement,
    AssignmentStatement,
    LeftPart,
    Variable,
    Expression,
    SimpleExpression,
    Term,
    Factor,
    Constant,
    MultiplicativeOperator,
    AdditiveOperator,
    RelationalOperator,
    InputStatement,
    OutputStatement,
    OutputFormat,
    EmptyStatement,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    lookahead: Token,
    cur_token: Kind,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let init_token = lexer.next_token();
        Self {
            lexer,
            lookahead: init_token,
            cur_token: Kind::Program,
        }
    }

    fn match_token(&self, kind: scan3::Kind) -> bool {
        if self.lookahead.kind == kind {
            true
        } else {
            false
        }
    }

    fn match_consume_token(&mut self, kind: scan3::Kind) -> SyntaxResult {
        if self.match_token(kind) {
            self.cur_token = kind;
            let current = self.lookahead.clone();
            self.lookahead = self.lexer.next_token();
            println!("consume token: {:?}, lookahead: {:?}", kind, self.lookahead);
            Ok(Node::new(NodeKind::Token(current), None))
        } else {
            println!(
                "consume token error: {:?}, lookahead: {:?}",
                kind, self.lookahead
            );
            Err(error::SyntaxError::new(self, &[kind], &[]))
        }
    }

    fn match_syntax_first_token(&self, syntax: SyntaxKind) -> bool {
        if syntax == SyntaxKind::EmptyStatement {
            // 空文の場合，何が来てもtrue
            return true;
        }
        let lk = self.lookahead.kind;
        let binding = first_set::FIRST_SETS;
        let tokens = binding.iter().find(|x| x.symbol == syntax).unwrap();
        tokens.first_set.contains(&lk)
    }

    fn match_consume_syntax(&mut self, syntax: SyntaxKind) -> SyntaxResult {
        if self.match_syntax_first_token(syntax) {
            let node = match syntax {
                SyntaxKind::Program => unreachable!(),
                SyntaxKind::Block => self.block()?,
                SyntaxKind::VariableDeclaration => self.variable_declaration_part()?,
                SyntaxKind::VariableNames => self.variable_names()?,
                SyntaxKind::VariableName => self.valriable_name()?,
                SyntaxKind::Type => self.type_()?,
                SyntaxKind::StandardType => self.standard_type()?,
                SyntaxKind::ArrayType => self.array_type()?,
                SyntaxKind::SubprogramDeclaration => self.subprogram_declaration()?,
                SyntaxKind::ProcedureName => self.procedure_name()?,
                SyntaxKind::FormalParameters => self.formal_parameters()?,
                SyntaxKind::CompoundStatement => self.compound_statement()?,
                SyntaxKind::Statement => self.statement()?,
                SyntaxKind::ConditionStatement => self.condnition_statement()?,
                SyntaxKind::IterationStatement => self.iteration_statement()?,
                SyntaxKind::ExitStatement => self.exit_statement()?,
                SyntaxKind::CallStatement => self.call_statement()?,
                SyntaxKind::Expressions => self.expressions()?,
                SyntaxKind::ReturnStatement => self.return_statement()?,
                SyntaxKind::AssignmentStatement => self.assignment_statement()?,
                SyntaxKind::LeftPart => self.left_part()?,
                SyntaxKind::Variable => self.variable()?,
                SyntaxKind::Expression => self.expression()?,
                SyntaxKind::SimpleExpression => self.simple_expression()?,
                SyntaxKind::Term => self.term()?,
                SyntaxKind::Factor => self.factor()?,
                SyntaxKind::Constant => self.constant()?,
                SyntaxKind::MultiplicativeOperator => self.multiplicative_operator()?,
                SyntaxKind::AdditiveOperator => self.additive_operator()?,
                SyntaxKind::RelationalOperator => self.relational_operator()?,
                SyntaxKind::InputStatement => self.input_statement()?,
                SyntaxKind::OutputStatement => self.output_statement()?,
                SyntaxKind::OutputFormat => self.output_format()?,
                SyntaxKind::EmptyStatement => self.empty_statement()?,
            };
            Ok(node)
        } else {
            Err(error::SyntaxError::new(self, &[], &[syntax]))
        }
    }

    /// パースの開始
    /// "program" "名前" ";" ブロック "."
    pub fn parse_program(&mut self) -> SyntaxResult {
        // マクロ構文のprogramに該当
        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::Program),
            Some(vec![
                self.match_consume_token(Kind::Program)?,
                self.match_consume_token(Kind::Name)?,
                self.match_consume_token(Kind::Semicolon)?,
                self.match_consume_syntax(SyntaxKind::Block)?,
                self.match_consume_token(Kind::Dot)?,
            ]),
        ))
    }

    /// { 変数宣言部 | 副プログラム宣言 } 複合文
    fn block(&mut self) -> SyntaxResult {
        let mut nodes = vec![];
        loop {
            match self.lookahead.kind {
                _ if self.match_syntax_first_token(SyntaxKind::VariableDeclaration) => {
                    nodes.push(self.match_consume_syntax(SyntaxKind::VariableDeclaration)?)
                }
                _ if self.match_syntax_first_token(SyntaxKind::SubprogramDeclaration) => {
                    nodes.push(self.match_consume_syntax(SyntaxKind::SubprogramDeclaration)?)
                }
                _ => break,
            }
        }
        nodes.push(self.match_consume_syntax(SyntaxKind::CompoundStatement)?);
        Ok(Node::new(NodeKind::Syntax(SyntaxKind::Block), Some(nodes)))
    }

    /// "var" 変数名の並び ":" 型 ";" { 変数名の並び ":" 型 ";" }
    fn variable_declaration_part(&mut self) -> SyntaxResult {
        let mut nodes = vec![
            self.match_consume_token(Kind::Var)?,
            self.match_consume_syntax(SyntaxKind::VariableNames)?,
            self.match_consume_token(Kind::Colon)?,
            self.match_consume_syntax(SyntaxKind::Type)?,
            self.match_consume_token(Kind::Semicolon)?,
        ];
        while self.lookahead.kind == Kind::Name {
            let n = vec![
                self.match_consume_syntax(SyntaxKind::VariableNames)?,
                self.match_consume_token(Kind::Colon)?,
                self.match_consume_syntax(SyntaxKind::Type)?,
                self.match_consume_token(Kind::Semicolon)?,
            ];
            nodes.extend(n);
        }

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::VariableDeclaration),
            Some(nodes),
        ))
    }

    /// 変数名 { "," 変数名 }
    fn variable_names(&mut self) -> SyntaxResult {
        let mut nodes = vec![self.match_consume_syntax(SyntaxKind::VariableName)?];

        while self.lookahead.kind == Kind::Comma {
            nodes.push(self.match_consume_token(Kind::Comma)?);
            nodes.push(self.match_consume_syntax(SyntaxKind::VariableName)?);
        }
        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::VariableNames),
            Some(nodes),
        ))
    }

    /// "名前"
    fn valriable_name(&mut self) -> SyntaxResult {
        let nodes = vec![self.match_consume_token(Kind::Name)?];
        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::VariableName),
            Some(nodes),
        ))
    }

    /// 標準型 | 配列型
    /// 予約語に引っかかるのを防ぐため，アンダーバーをつけている
    fn type_(&mut self) -> SyntaxResult {
        let err = error::SyntaxError::new(
            self,
            &[],
            &[SyntaxKind::StandardType, SyntaxKind::ArrayType],
        );
        let nodes = match self.lookahead.kind {
            _ if self.match_syntax_first_token(SyntaxKind::ArrayType) => self.array_type()?,
            _ if self.match_syntax_first_token(SyntaxKind::StandardType) => {
                self.match_consume_syntax(SyntaxKind::StandardType)?
            }
            _ => return Err(err),
        };
        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::Type),
            Some(vec![nodes]),
        ))
    }

    /// "integer" | "boolean" | "char"
    fn standard_type(&mut self) -> SyntaxResult {
        let err = error::SyntaxError::new(self, &[], &[SyntaxKind::StandardType]);
        let nodes = match self.lookahead.kind {
            Kind::Integer => self.match_consume_token(Kind::Integer)?,
            Kind::Boolean => self.match_consume_token(Kind::Boolean)?,
            Kind::Char => self.match_consume_token(Kind::Char)?,
            _ => return Err(err),
        };
        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::StandardType),
            Some(vec![nodes]),
        ))
    }

    /// "array" "[" "符号なし整数" "]" "of" 標準型
    fn array_type(&mut self) -> SyntaxResult {
        let nodes = vec![
            self.match_consume_token(Kind::Array)?,
            self.match_consume_token(Kind::LBracket)?,
            self.match_consume_token(Kind::UnsignedInteger)?,
            self.match_consume_token(Kind::RBracket)?,
            self.match_consume_token(Kind::Of)?,
            self.match_consume_syntax(SyntaxKind::StandardType)?,
        ];
        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::ArrayType),
            Some(nodes),
        ))
    }

    /// "procedure" 手続き名 [ 仮引数部 ] ";" [ 変数宣言部 ] 複合文 ";"
    fn subprogram_declaration(&mut self) -> SyntaxResult {
        let mut nodes = vec![
            self.match_consume_token(Kind::Procedure)?,
            self.match_consume_syntax(SyntaxKind::ProcedureName)?,
        ];
        if self.match_syntax_first_token(SyntaxKind::FormalParameters) {
            nodes.push(self.match_consume_syntax(SyntaxKind::FormalParameters)?);
        }
        nodes.push(self.match_consume_token(Kind::Semicolon)?);
        if self.match_syntax_first_token(SyntaxKind::VariableDeclaration) {
            nodes.push(self.match_consume_syntax(SyntaxKind::VariableDeclaration)?);
        }
        nodes.push(self.match_consume_syntax(SyntaxKind::CompoundStatement)?);
        nodes.push(self.match_consume_token(Kind::Semicolon)?);

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::SubprogramDeclaration),
            Some(nodes),
        ))
    }

    /// "名前"
    fn procedure_name(&mut self) -> SyntaxResult {
        let nodes = vec![self.match_consume_token(Kind::Name)?];
        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::ProcedureName),
            Some(nodes),
        ))
    }

    /// "(" 変数名の並び ":" 型 { ";" 変数名の並び ":" 型 } ")"
    fn formal_parameters(&mut self) -> SyntaxResult {
        let mut nodes = vec![
            self.match_consume_token(Kind::LParen)?,
            self.match_consume_syntax(SyntaxKind::VariableNames)?,
            self.match_consume_token(Kind::Colon)?,
            self.match_consume_syntax(SyntaxKind::Type)?,
        ];
        while self.lookahead.kind == Kind::Semicolon {
            let n = vec![
                self.match_consume_token(Kind::Semicolon)?,
                self.match_consume_syntax(SyntaxKind::VariableNames)?,
                self.match_consume_token(Kind::Colon)?,
                self.match_consume_syntax(SyntaxKind::Type)?,
            ];
            nodes.extend(n);
        }
        nodes.push(self.match_consume_token(Kind::RParen)?);

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::FormalParameters),
            Some(nodes),
        ))
    }

    /// "begin" 文 { ";" 文 } "end"
    fn compound_statement(&mut self) -> SyntaxResult {
        let mut nodes = vec![
            self.match_consume_token(Kind::Begin)?,
            self.match_consume_syntax(SyntaxKind::Statement)?,
        ];
        while self.lookahead.kind == Kind::Semicolon {
            let n = vec![
                self.match_consume_token(Kind::Semicolon)?,
                self.match_consume_syntax(SyntaxKind::Statement)?,
            ];
            nodes.extend(n);
        }
        nodes.push(self.match_consume_token(Kind::End)?);

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::CompoundStatement),
            Some(nodes),
        ))
    }

    /// 代入文 | 分岐文 | 繰り返し文 | 脱出文 | 手続き呼び出し文 | 複合文 | 戻り文 | 入力文
    /// | 出力文 | 複合文 | 空文
    fn statement(&mut self) -> SyntaxResult {
        let nodes = match self.lookahead.kind {
            _ if self.match_syntax_first_token(SyntaxKind::AssignmentStatement) => {
                self.match_consume_syntax(SyntaxKind::AssignmentStatement)?
            }
            _ if self.match_syntax_first_token(SyntaxKind::ConditionStatement) => {
                self.match_consume_syntax(SyntaxKind::ConditionStatement)?
            }
            _ if self.match_syntax_first_token(SyntaxKind::IterationStatement) => {
                self.match_consume_syntax(SyntaxKind::IterationStatement)?
            }
            _ if self.match_syntax_first_token(SyntaxKind::ExitStatement) => {
                self.match_consume_syntax(SyntaxKind::ExitStatement)?
            }
            _ if self.match_syntax_first_token(SyntaxKind::CallStatement) => {
                self.match_consume_syntax(SyntaxKind::CallStatement)?
            }
            _ if self.match_syntax_first_token(SyntaxKind::CompoundStatement) => {
                self.match_consume_syntax(SyntaxKind::CompoundStatement)?
            }
            _ if self.match_syntax_first_token(SyntaxKind::ReturnStatement) => {
                self.match_consume_syntax(SyntaxKind::ReturnStatement)?
            }
            _ if self.match_syntax_first_token(SyntaxKind::InputStatement) => {
                self.match_consume_syntax(SyntaxKind::InputStatement)?
            }
            _ if self.match_syntax_first_token(SyntaxKind::OutputStatement) => {
                self.match_consume_syntax(SyntaxKind::OutputStatement)?
            }
            // 空文なので何もしなくていい はず
            _ => self.match_consume_syntax(SyntaxKind::EmptyStatement)?,
        };

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::Statement),
            Some(vec![nodes]),
        ))
    }

    /// "if" 式 "then" 文 [ "else" 文 ]
    fn condnition_statement(&mut self) -> SyntaxResult {
        let mut nodes = vec![
            self.match_consume_token(Kind::If)?,
            self.match_consume_syntax(SyntaxKind::Expression)?,
            self.match_consume_token(Kind::Then)?,
            self.match_consume_syntax(SyntaxKind::Statement)?,
        ];
        while self.lookahead.kind == Kind::Else {
            let n = vec![
                self.match_consume_token(Kind::Else)?,
                self.match_consume_syntax(SyntaxKind::Statement)?,
            ];
            nodes.extend(n);
        }

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::ConditionStatement),
            Some(nodes),
        ))
    }

    /// "while" 式 "do" 文
    fn iteration_statement(&mut self) -> SyntaxResult {
        let nodes = vec![
            self.match_consume_token(Kind::While)?,
            self.match_consume_syntax(SyntaxKind::Expression)?,
            self.match_consume_token(Kind::DO)?,
            self.match_consume_syntax(SyntaxKind::Statement)?,
        ];

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::IterationStatement),
            Some(nodes),
        ))
    }

    /// "break"
    fn exit_statement(&mut self) -> SyntaxResult {
        let nodes = vec![self.match_consume_token(Kind::Break)?];

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::ExitStatement),
            Some(nodes),
        ))
    }

    /// "call" 手続き名 [ "(" 式の並び ")" ]
    fn call_statement(&mut self) -> SyntaxResult {
        let mut nodes = vec![
            self.match_consume_token(Kind::Call)?,
            self.match_consume_syntax(SyntaxKind::ProcedureName)?,
        ];
        if self.lookahead.kind == Kind::LParen {
            let n = vec![
                self.match_consume_token(Kind::LParen)?,
                self.match_consume_syntax(SyntaxKind::Expressions)?,
                self.match_consume_token(Kind::RParen)?,
            ];
            nodes.extend(n);
        }

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::CallStatement),
            Some(nodes),
        ))
    }

    /// 式 { "," 式 }
    fn expressions(&mut self) -> SyntaxResult {
        let mut nodes = vec![self.match_consume_syntax(SyntaxKind::Expression)?];
        while self.lookahead.kind == Kind::Comma {
            let n = vec![
                self.match_consume_token(Kind::Comma)?,
                self.match_consume_syntax(SyntaxKind::Expression)?,
            ];
            nodes.extend(n);
        }

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::Expressions),
            Some(nodes),
        ))
    }

    /// return
    fn return_statement(&mut self) -> SyntaxResult {
        let n = self.match_consume_token(Kind::Return)?;

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::ReturnStatement),
            Some(vec![n]),
        ))
    }

    /// 左辺部 ":=" 式
    fn assignment_statement(&mut self) -> SyntaxResult {
        let nodes = vec![
            self.match_consume_syntax(SyntaxKind::LeftPart)?,
            self.match_consume_token(Kind::Assign)?,
            self.match_consume_syntax(SyntaxKind::Expression)?,
        ];

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::AssignmentStatement),
            Some(nodes),
        ))
    }

    /// 変数
    fn left_part(&mut self) -> SyntaxResult {
        let n = self.match_consume_syntax(SyntaxKind::Variable)?;
        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::LeftPart),
            Some(vec![n]),
        ))
    }

    /// 変数名 [ "[" 式 "]" ]
    fn variable(&mut self) -> SyntaxResult {
        let mut nodes = vec![self.match_consume_syntax(SyntaxKind::VariableName)?];
        if self.lookahead.kind == Kind::LBracket {
            let n = vec![
                self.match_consume_token(Kind::LBracket)?,
                self.match_consume_syntax(SyntaxKind::Expression)?,
                self.match_consume_token(Kind::RBracket)?,
            ];
            nodes.extend(n);
        }

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::Variable),
            Some(nodes),
        ))
    }

    /// 単純式 { 関係演算子 単純式 }
    fn expression(&mut self) -> SyntaxResult {
        let mut nodes = vec![self.match_consume_syntax(SyntaxKind::SimpleExpression)?];
        if self.match_syntax_first_token(SyntaxKind::RelationalOperator) {
            let n = vec![
                self.match_consume_syntax(SyntaxKind::RelationalOperator)?,
                self.match_consume_syntax(SyntaxKind::SimpleExpression)?,
            ];
            nodes.extend(n);
        }

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::Expression),
            Some(nodes),
        ))
    }

    /// [ "+" | "-" ] 項 { 加法演算子 項 }
    fn simple_expression(&mut self) -> SyntaxResult {
        let mut nodes = vec![];
        match self.lookahead.kind {
            Kind::Plus => nodes.push(self.match_consume_token(Kind::Plus)?),
            Kind::Minus => nodes.push(self.match_consume_token(Kind::Minus)?),
            _ => {}
        }
        nodes.push(self.match_consume_syntax(SyntaxKind::Term)?);
        while self.match_syntax_first_token(SyntaxKind::AdditiveOperator) {
            let n = vec![
                self.match_consume_syntax(SyntaxKind::AdditiveOperator)?,
                self.match_consume_syntax(SyntaxKind::Term)?,
            ];
            nodes.extend(n);
        }

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::SimpleExpression),
            Some(nodes),
        ))
    }

    /// 因子 { 乗法演算子 因子 }
    fn term(&mut self) -> SyntaxResult {
        let mut nodes = vec![self.match_consume_syntax(SyntaxKind::Factor)?];
        while self.match_syntax_first_token(SyntaxKind::MultiplicativeOperator) {
            let n = vec![
                self.match_consume_syntax(SyntaxKind::MultiplicativeOperator)?,
                self.match_consume_syntax(SyntaxKind::Factor)?,
            ];
            nodes.extend(n);
        }

        Ok(Node::new(NodeKind::Syntax(SyntaxKind::Term), Some(nodes)))
    }

    /// 変数 | 定数 | "(" 式 ")" | "not" 因子 | 標準型 "(" 式 ")"
    fn factor(&mut self) -> SyntaxResult {
        let err = error::SyntaxError::new(
            self,
            &[Kind::LParen, Kind::Not],
            &[
                SyntaxKind::Variable,
                SyntaxKind::Constant,
                SyntaxKind::StandardType,
            ],
        );

        let mut nodes = vec![];
        match self.lookahead.kind {
            _ if self.match_syntax_first_token(SyntaxKind::Variable) => {
                nodes.push(self.match_consume_syntax(SyntaxKind::Variable)?)
            }
            _ if self.match_syntax_first_token(SyntaxKind::Constant) => {
                nodes.push(self.match_consume_syntax(SyntaxKind::Constant)?)
            }
            Kind::LParen => {
                let n = vec![
                    self.match_consume_token(Kind::LParen)?,
                    self.match_consume_syntax(SyntaxKind::Expression)?,
                    self.match_consume_token(Kind::RParen)?,
                ];
                nodes.extend(n);
            }
            Kind::Not => {
                let n = vec![
                    self.match_consume_token(Kind::Not)?,
                    self.match_consume_syntax(SyntaxKind::Factor)?,
                ];
                nodes.extend(n);
            }
            _ if self.match_syntax_first_token(SyntaxKind::StandardType) => {
                let n = vec![
                    self.match_consume_syntax(SyntaxKind::StandardType)?,
                    self.match_consume_token(Kind::LParen)?,
                    self.match_consume_syntax(SyntaxKind::Expression)?,
                    self.match_consume_token(Kind::RParen)?,
                ];
                nodes.extend(n);
            }
            _ => Err(err)?,
        }

        Ok(Node::new(NodeKind::Syntax(SyntaxKind::Factor), Some(nodes)))
    }

    /// "符号なし整数" | "true" | "false" | "文字列"
    fn constant(&mut self) -> SyntaxResult {
        let err = error::SyntaxError::new(
            self,
            &[Kind::UnsignedInteger, Kind::True, Kind::False, Kind::String],
            &[],
        );
        let nodes = match self.lookahead.kind {
            Kind::UnsignedInteger => self.match_consume_token(Kind::UnsignedInteger)?,
            Kind::True => self.match_consume_token(Kind::True)?,
            Kind::False => self.match_consume_token(Kind::False)?,
            Kind::String => self.match_consume_token(Kind::String)?,
            _ => Err(err)?,
        };

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::Constant),
            Some(vec![nodes]),
        ))
    }

    /// "*" | "div" | "and"
    fn multiplicative_operator(&mut self) -> SyntaxResult {
        let err = error::SyntaxError::new(self, &[Kind::Star, Kind::Div, Kind::And], &[]);

        let nodes = match self.lookahead.kind {
            Kind::Star => self.match_consume_token(Kind::Star)?,
            Kind::Div => self.match_consume_token(Kind::Div)?,
            Kind::And => self.match_consume_token(Kind::And)?,
            _ => Err(err)?,
        };

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::MultiplicativeOperator),
            Some(vec![nodes]),
        ))
    }

    /// "+" | "-" | "or"
    fn additive_operator(&mut self) -> SyntaxResult {
        let err = error::SyntaxError::new(self, &[Kind::Plus, Kind::Minus, Kind::Or], &[]);
        let nodes = match self.lookahead.kind {
            Kind::Plus => self.match_consume_token(Kind::Plus)?,
            Kind::Minus => self.match_consume_token(Kind::Minus)?,
            Kind::Or => self.match_consume_token(Kind::Or)?,
            _ => Err(err)?,
        };
        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::AdditiveOperator),
            Some(vec![nodes]),
        ))
    }

    /// "=" | "<>" | "<" | "<=" | ">" | ">="
    fn relational_operator(&mut self) -> SyntaxResult {
        let err = error::SyntaxError::new(
            self,
            &[
                Kind::Equal,
                Kind::NotEq,
                Kind::Less,
                Kind::LessEq,
                Kind::Great,
                Kind::GreatEq,
            ],
            &[],
        );
        let nodes = match self.lookahead.kind {
            Kind::Equal => self.match_consume_token(Kind::Equal)?,
            Kind::NotEq => self.match_consume_token(Kind::NotEq)?,
            Kind::Less => self.match_consume_token(Kind::Less)?,
            Kind::LessEq => self.match_consume_token(Kind::LessEq)?,
            Kind::Great => self.match_consume_token(Kind::Great)?,
            Kind::GreatEq => self.match_consume_token(Kind::GreatEq)?,
            _ => Err(err)?,
        };

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::RelationalOperator),
            Some(vec![nodes]),
        ))
    }

    /// ( "read" | "readln" ) [ "(" 変数 { "," 変数 } ")" ]
    fn input_statement(&mut self) -> SyntaxResult {
        let err = error::SyntaxError::new(self, &[Kind::Read, Kind::Readln], &[]);
        let mut nodes = vec![];
        let n = match self.lookahead.kind {
            Kind::Read => self.match_consume_token(Kind::Read)?,
            Kind::Readln => self.match_consume_token(Kind::Readln)?,
            _ => Err(err)?,
        };
        nodes.push(n);

        if self.lookahead.kind == Kind::LParen {
            let mut n = vec![
                self.match_consume_token(Kind::LParen)?,
                self.match_consume_syntax(SyntaxKind::Variable)?,
            ];
            while self.lookahead.kind == Kind::Comma {
                n.push(self.match_consume_token(Kind::Comma)?);
                n.push(self.match_consume_syntax(SyntaxKind::Variable)?);
            }
            n.push(self.match_consume_token(Kind::RParen)?);
            nodes.extend(n);
        }

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::InputStatement),
            Some(nodes),
        ))
    }

    /// ( "write" | "writeln" ) [ "(" 出力指定 { "," 出力指定 } ")" ]
    fn output_statement(&mut self) -> SyntaxResult {
        let err = error::SyntaxError::new(self, &[Kind::Write, Kind::Writeln], &[]);
        let mut nodes = vec![];
        let n = match self.lookahead.kind {
            Kind::Write => self.match_consume_token(Kind::Write)?,
            Kind::Writeln => self.match_consume_token(Kind::Writeln)?,
            _ => Err(err)?,
        };
        nodes.push(n);

        if self.lookahead.kind == Kind::LParen {
            let mut n = vec![
                self.match_consume_token(Kind::LParen)?,
                self.match_consume_syntax(SyntaxKind::OutputFormat)?,
            ];
            while self.lookahead.kind == Kind::Comma {
                let n2 = vec![
                    self.match_consume_token(Kind::Comma)?,
                    self.match_consume_syntax(SyntaxKind::OutputFormat)?,
                ];
                n.extend(n2);
            }
            n.push(self.match_consume_token(Kind::RParen)?);
            nodes.extend(n);
        }

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::OutputStatement),
            Some(nodes),
        ))
    }

    /// 式 [ ":" "符号なし整数" ] | "文字列"
    fn output_format(&mut self) -> SyntaxResult {
        let err = error::SyntaxError::new(self, &[Kind::String], &[SyntaxKind::Expression]);

        let mut nodes = vec![];
        match self.lookahead.kind {
            Kind::String => nodes.push(self.match_consume_token(Kind::String)?),
            _ if self.match_syntax_first_token(SyntaxKind::Expression) => {
                let mut n = vec![self.match_consume_syntax(SyntaxKind::Expression)?];

                if self.lookahead.kind == Kind::Colon {
                    let n2 = vec![
                        self.match_consume_token(Kind::Colon)?,
                        self.match_consume_token(Kind::UnsignedInteger)?,
                    ];
                    n.extend(n2);
                }
                nodes.extend(n);
            }
            _ => Err(err)?,
        }

        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::OutputFormat),
            Some(nodes),
        ))
    }

    fn empty_statement(&mut self) -> SyntaxResult {
        Ok(Node::new(
            NodeKind::Syntax(SyntaxKind::EmptyStatement),
            None,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::scan::scan3::Lexer;

    // ./parse/samples/1.mpl
    // ./parse/answes/1.mpl
    // を読み込む
    const TEST_SOURCE_COUNT: usize = 20;
    #[test]
    fn tests_parser() {
        for i in 1..=TEST_SOURCE_COUNT {
            let source =
                std::fs::read_to_string(format!("test_source/perse/samples/{}.mpl", i)).unwrap();
            let answer =
                std::fs::read_to_string(format!("test_source/perse/answers/{}.mpl", i)).unwrap();
            let lexer = Lexer::new(&source);
            let mut parser = Parser::new(lexer);
            match parser.parse_program() {
                Ok(_) => println!("{} parsing OK \n{}", i, source),
                Err(e) => {
                    eprintln!("Err {}:  {}", i, e);
                    // assert_eq!(e.lexeicalized_source.trim(), answer.trim())
                    // ソースコードと正解例をトークン化し，トークン化した結果を比較する
                    // トークンの位置情報はいらないので，取り除いたものを比較する
                    let mut lex = Lexer::new(&e.lexeicalized_source);
                    let lexeicalized_err_source = lex
                        .analyze()
                        .iter()
                        .map(|t| (t.kind, t.value.clone()))
                        .collect::<Vec<_>>();
                    let mut lex = Lexer::new(&answer);
                    let lexeicalized_answer = lex
                        .analyze()
                        .iter()
                        .map(|t| (t.kind, t.value.clone()))
                        .collect::<Vec<_>>();
                    assert_eq!(lexeicalized_err_source, lexeicalized_answer);
                }
            }
        }
    }
}
