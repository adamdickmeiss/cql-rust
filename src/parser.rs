use crate::error::ParseError;
use crate::lexer::Lexer;
use crate::lexer::Token;
use crate::node::CqlNode;
use crate::node::St;
use std::rc::Rc;

pub struct Parser {
    look: Token,
    lexer: Lexer,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            look: Token::EOS,
            lexer: Lexer::new(),
        }
    }

    pub fn strict(self: &mut Self, strict: bool) {
        self.lexer.strict(strict);
    }

    fn search_term(self: &mut Self) -> Option<String> {
        match &self.look {
            Token::SimpleString(name)
            | Token::PrefixName(name)
            | Token::Boolop(name)
            | Token::Sortby(name) => {
                return Some(String::from(name));
            }
            _ => return None,
        }
    }

    fn relation_symbol(self: &mut Self) -> Option<String> {
        let lead;
        match &self.look {
            Token::EQ => lead = "=",
            Token::GT => lead = ">",
            Token::LT => lead = "<",
            Token::GE => lead = ">=",
            Token::LE => lead = "<=",
            Token::NE => lead = "<>",
            Token::Exact => lead = "==",
            _ => return None,
        }
        Some(String::from(lead))
    }

    fn relation(self: &mut Self) -> Option<String> {
        if let Some(lead) = self.relation_symbol() {
            return Some(lead);
        }
        if let Token::PrefixName(name) = &self.look {
            return Some(String::from(name));
        }
        None
    }

    fn boolean(self: &mut Self) -> Option<String> {
        match &self.look {
            Token::Boolop(name) => return Some(String::from(name)),
            _ => return None,
        }
    }

    fn modifiers(
        self: &mut Self,
        get: &mut dyn Iterator<Item = char>,
    ) -> Result<Option<Rc<St>>, ParseError> {
        let mut res: Option<Rc<St>> = None;
        while let Token::Modifier = &self.look {
            self.look = self.lexer.lex(get)?;
            if let Some(modifier) = self.search_term() {
                self.look = self.lexer.lex(get)?;
                if let Some(relation) = self.relation_symbol() {
                    self.look = self.lexer.lex(get)?;
                    if let Some(value) = self.search_term() {
                        res = Some(Rc::new(CqlNode::mk_sc(
                            &modifier,
                            &relation,
                            Some(&value),
                            None,
                        )));
                        self.look = self.lexer.lex(get)?;
                    } else {
                        return Err(ParseError);
                    }
                } else {
                    res = Some(Rc::new(CqlNode::mk_sc(&modifier, "", None, None)));
                }
            } else {
                return Err(ParseError);
            }
        }
        Ok(res)
    }

    fn search_clause(
        self: &mut Self,
        get: &mut dyn Iterator<Item = char>,
        rel: &St,
    ) -> Result<CqlNode, ParseError> {
        if self.look == Token::LP {
            self.look = self.lexer.lex(get)?;
            let res = self.cql_query(get, rel)?;
            if self.look != Token::RP {
                return Err(ParseError);
            }
            self.look = self.lexer.lex(get)?;
            return Ok(res);
        }
        let n = self.search_term();
        if let Some(n) = n {
            self.look = self.lexer.lex(get)?;
            if let Some(relation) = self.relation() {
                self.look = self.lexer.lex(get)?;
                let modifiers = self.modifiers(get)?;
                let rel = CqlNode::mk_sc(&n, &relation, None, modifiers);
                return self.search_clause(get, &rel);
            }
            return Ok(CqlNode::mk_sc_dup(rel, &n));
        }
        // missing search !
        Err(ParseError)
    }

    fn scoped_clause(
        self: &mut Self,
        get: &mut dyn Iterator<Item = char>,
        rel: &St,
    ) -> Result<CqlNode, ParseError> {
        let mut left = self.search_clause(get, rel)?;
        while let Some(op) = self.boolean() {
            self.look = self.lexer.lex(get)?;
            let modifiers = self.modifiers(get)?;
            let right = self.search_clause(get, rel)?;
            left = CqlNode::mk_boolean(&op, Box::new(left), Box::new(right), modifiers);
        }
        Ok(left)
    }

    fn cql_query(
        self: &mut Self,
        get: &mut dyn Iterator<Item = char>,
        rel: &St,
    ) -> Result<CqlNode, ParseError> {
        let res = self.scoped_clause(get, rel)?;
        Ok(res)
    }

    pub fn parse(
        self: &mut Self,
        get: &mut dyn Iterator<Item = char>,
    ) -> Result<CqlNode, ParseError> {
        self.lexer.next(get);
        self.look = self.lexer.lex(get)?;
        let rel = CqlNode::mk_sc("cql.serverChoice", "=", None, None);
        let search = self.cql_query(get, &rel)?;
        let mut sort = Vec::new();
        if let Token::Sortby(_sortby) = &self.look {
            self.look = self.lexer.lex(get)?;
            while let Some(index) = &self.search_term() {
                self.look = self.lexer.lex(get)?;
                let modifiers = self.modifiers(get)?;
                sort.push(CqlNode::mk_sc(&index, "", None, modifiers));
            }
        }
        if self.look != Token::EOS {
            return Err(ParseError);
        }
        Ok(CqlNode::mk_root(Box::new(search), sort))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::BorrowMut;

    #[test]
    fn errors() {
        let mut my = Parser::new();
        let res = my.parse("  ".chars().borrow_mut());
        assert!(res.is_err());

        let res = my.parse("ti =".chars().borrow_mut());
        assert!(res.is_err());

        let res = my.parse("and )".chars().borrow_mut());
        assert!(res.is_err());

        let res = my.parse("(and".chars().borrow_mut());
        assert!(res.is_err());

        let res = my.parse("ti / ".chars().borrow_mut());
        assert!(res.is_err());

        let res = my.parse("ti / x= ".chars().borrow_mut());
        assert!(res.is_err());

        let res = my.parse("ti = / ".chars().borrow_mut());
        assert!(res.is_err());

        let res = my.parse("ti = / x= ".chars().borrow_mut());
        assert!(res.is_err());

        let res = my.parse("foo equals x".chars().borrow_mut());
        assert!(res.is_err());
    }

    #[test]
    fn ok() {
        let mut my = Parser::new();
        let res = my.parse("foo".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("and".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("adj".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("sortby".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti adj computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti = computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti > computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti >= computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti < computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti <= computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti <> computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti == computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti = /a/b=c computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti = /a=b computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("(ti = computer)".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("((ti = computer))".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti = (computer)".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a and b".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a and/x1=y1 b".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a and/x1=y1/x2=y2 b".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a sortby title".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a sortby title date".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a sortby title/a/b date/x=y".chars().borrow_mut());
        assert!(res.is_ok());
    }

    #[test]
    fn strict1() {
        let mut my = Parser::new();
        my.strict(true);
        let res = my.parse("foo equals x".chars().borrow_mut());
        assert!(res.is_ok());
    }
}
