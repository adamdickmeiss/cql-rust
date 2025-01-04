use std::rc::Rc;

#[cfg_attr(test, derive(Debug))]
pub struct St {
    index: String,
    index_uri: Option<String>,
    term: String,
    relation: String,
    relation_uri: Option<String>,
    modifiers: Option<Rc<CqlNode>>,
}

#[cfg_attr(test, derive(Debug))]
pub struct Boolean {
    value: String,
    left: Option<Box<CqlNode>>,
    right: Option<Box<CqlNode>>,
}

#[cfg_attr(test, derive(Debug))]
pub struct Sort {
    index: String,
    next: Option<Box<CqlNode>>,
    modifiers: Option<Box<CqlNode>>,
    search: Option<Box<CqlNode>>,
}

#[cfg_attr(test, derive(Debug))]
pub enum CqlNode {
    St(St),
    Boolean(Boolean),
    Sort(Sort),
}

impl CqlNode {
    pub(crate) fn mk_sc_dup(node: &CqlNode, term: &str) -> CqlNode {
        if let CqlNode::St(st) = node {
            let st2 = St {
                index: st.index.clone(),
                index_uri: st.index_uri.clone(),
                term: String::from(term),
                relation: st.relation.clone(),
                relation_uri: st.relation_uri.clone(),
                modifiers: st.modifiers.clone(),
            };
            return CqlNode::St(st2);
        }
        panic!("mk_sc_dup from non-st node");
    }

    pub(crate) fn mk_sc(index: &str, relation: &str, term: &str) -> CqlNode {
        let st = St {
            index: String::from(index),
            index_uri: None,
            term: String::from(term),
            relation: String::from(relation),
            relation_uri: None,
            modifiers: None,
        };
        CqlNode::St(st)
    }
    pub(crate) fn mk_boolean(
        value: &str,
        left: Option<Box<CqlNode>>,
        right: Option<Box<CqlNode>>,
    ) -> CqlNode {
        let bo = Boolean {
            value: String::from(value),
            left,
            right,
        };
        CqlNode::Boolean(bo)
    }
    pub(crate) fn mk_sort(index: &str, modifiers: Option<Box<CqlNode>>) -> CqlNode {
        let sort = Sort {
            index: String::from(index),
            modifiers,
            next: None,
            search: None,
        };
        CqlNode::Sort(sort)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_sc() {
        let my_sc = CqlNode::mk_sc("ti", "=", "value");
        assert_matches!(my_sc, CqlNode::St(n) => {
            assert_eq!(n.index, "ti");
            assert_eq!(n.relation, "=");
            assert_eq!(n.term, "value");
            assert!(n.index_uri.is_none());
            assert!(n.relation_uri.is_none());
            assert!(n.modifiers.is_none());
            assert!(n.modifiers.is_none());
        });
    }

    #[test]
    fn create_boolean() {
        let my_bool = CqlNode::mk_boolean("and", None, None);
        assert_matches!(my_bool, CqlNode::Boolean(n) => {
            assert_eq!(n.value, "and");
            assert!(n.left.is_none());
            assert!(n.right.is_none());
        });
    }

    #[test]
    fn create_sort() {
        let my_sort = CqlNode::mk_sort("date", None);
        assert_matches!(my_sort, CqlNode::Sort(n) => {
            assert_eq!(n.index, "date");
            assert!(n.modifiers.is_none());
            assert!(n.next.is_none());
            assert!(n.search.is_none());
        });
    }

    #[test]
    fn create_tree() {
        let my_sc1 = Box::new(CqlNode::mk_sc("ti", "=", "house"));
        let my_sc2 = Box::new(CqlNode::mk_sc("au", "=", "andersen"));
        let my_bool = CqlNode::mk_boolean("and", Some(my_sc1), Some(my_sc2));

        assert_matches!(my_bool, CqlNode::Boolean(n) => {
            assert_matches!(n.left.as_deref().unwrap(), CqlNode::St(n) => {
                assert_eq!("ti", n.index);
                assert_eq!("house", n.term);
            });
            assert_matches!(n.right.as_deref().unwrap(), CqlNode::St(n) => {
                assert_eq!("au", n.index);
                assert_eq!("andersen", n.term);
            });
        });
    }
}
