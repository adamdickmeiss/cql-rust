use std::rc::Rc;

#[cfg_attr(test, derive(Debug))]
pub struct Query {
    clause: Clause,
    sortSpec: Sort,
}

#[cfg_attr(test, derive(Debug))]
pub struct Sort {
	index: String,
	modifiers: Vec<Modifier>,
}

#[cfg_attr(test, derive(Debug))]
pub struct Modifier {
	name: String,
	relation: String,
	value: String
}

#[cfg_attr(test, derive(Debug))]
pub struct Clause {
    prefixMap: Vec<Prefix>,
    triple: Triple,
}

#[cfg_attr(test, derive(Debug))]
pub enum Triple {
    searchClause(SearchClause),
    boolClause(BoolClause),
}

#[cfg_attr(test, derive(Debug))]
pub struct Prefix {
    prefix: String,
    uri: String,
}

#[cfg_attr(test, derive(Debug))]
pub struct SearchClause {
    index: String,
    relation: String,
    modifiers: Vec<Modifier>,
    term: String
}

#[cfg_attr(test, derive(Debug))]
enum Operator {
    And,
    Or,
    Not,
    Prox
}

#[cfg_attr(test, derive(Debug))]
pub struct BoolClause {
    left: Box<Clause>,
    operator: Operator,
    modifiers: Vec<Modifier>,
    right: Box<Clause>,
}

#[cfg_attr(test, derive(Debug))]
pub struct St {
    index: String,
    index_uri: Option<String>,
    term: Option<String>,
    relation: String,
    relation_uri: Option<String>,
    modifiers: Option<Rc<St>>,
}

#[cfg_attr(test, derive(Debug))]
pub struct Boolean {
    value: String,
    left: Box<CqlNode>,
    right: Box<CqlNode>,
    modifiers: Option<Rc<St>>,
}

#[cfg_attr(test, derive(Debug))]
pub struct Root {
    search: Box<CqlNode>,
    sort: Vec<St>,
}

#[cfg_attr(test, derive(Debug))]
pub enum CqlNode {
    St(St),
    Boolean(Boolean),
    Root(Root),
}

impl CqlNode {
    pub(crate) fn mk_sc_dup(st: &St, term: &str) -> CqlNode {
        let st2 = St {
            index: st.index.clone(),
            index_uri: st.index_uri.clone(),
            term: Some(String::from(term)),
            relation: st.relation.clone(),
            relation_uri: st.relation_uri.clone(),
            modifiers: st.modifiers.clone(),
        };
        return CqlNode::St(st2);
    }

    pub(crate) fn mk_sc(
        index: &str,
        relation: &str,
        term: Option<&str>,
        modifiers: Option<Rc<St>>,
    ) -> St {
        let term = match term {
            Some(s) => Some(String::from(s)),
            _ => None,
        };
        St {
            index: String::from(index),
            index_uri: None,
            term,
            relation: String::from(relation),
            relation_uri: None,
            modifiers,
        }
    }

    pub(crate) fn mk_boolean(
        value: &str,
        left: Box<CqlNode>,
        right: Box<CqlNode>,
        modifiers: Option<Rc<St>>,
    ) -> CqlNode {
        let bo = Boolean {
            value: String::from(value),
            left,
            right,
            modifiers,
        };
        CqlNode::Boolean(bo)
    }
    pub(crate) fn mk_root(search: Box<CqlNode>, sort: Vec<St>) -> CqlNode {
        let root = Root { search, sort };
        CqlNode::Root(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_sc() {
        let n = CqlNode::mk_sc("ti", "=", Some(&"value"), None);
        assert_eq!(n.index, "ti");
        assert_eq!(n.relation, "=");
        assert!(n.term.is_some_and(|val| val == "value"));
        assert!(n.index_uri.is_none());
        assert!(n.relation_uri.is_none());
        assert!(n.modifiers.is_none());
        assert!(n.modifiers.is_none());
    }

    #[test]
    fn create_sort() {
        let sc = CqlNode::St(CqlNode::mk_sc("ti", "=", None, None));
        let my_root = CqlNode::mk_root(Box::new(sc), Vec::new());
        assert_matches!(my_root, CqlNode::Root(n) => {
                assert!(n.sort.len() == 0);
        });
    }

    #[test]
    fn create_tree() {
        let my_sc1 = Box::new(CqlNode::St(CqlNode::mk_sc("ti", "=", Some(&"house"), None)));
        let my_sc2 = Box::new(CqlNode::St(CqlNode::mk_sc(
            "au",
            "=",
            Some(&"andersen"),
            None,
        )));
        let my_bool = CqlNode::mk_boolean("And", my_sc1, my_sc2, None);

        assert_matches!(my_bool, CqlNode::Boolean(n) => {
            assert_eq!("And", n.value);
            assert_matches!(*n.left, CqlNode::St(n) => {
                assert_eq!("ti", n.index);
                assert!(n.term.is_some_and(|val| val == "house"));
            });
            assert_matches!(*n.right, CqlNode::St(n) => {
                assert_eq!("au", n.index);
                assert!(n.term.is_some_and(|val| val == "andersen"));
            });
        });
    }
}
