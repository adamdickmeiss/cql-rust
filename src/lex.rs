
#[derive(PartialEq)]
pub(crate) enum Token {
    EOS,
    EQ,
    NE,
    LT,
    GT,
    LE,
    GE,
    Exact,
    Modifier,
    LP,
    RP,
    PrefixName(String),
    SimpleString(String),
    And(String),
    Or(String),
    Not(String),
    Prox(String),
    Sortby(String),
}

pub(crate) struct Lex<'a> {
    strict: bool,
    look_ch: Option<char>,
    look: Token,
    lit: &'a mut dyn Iterator<Item = char>,
}
