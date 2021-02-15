use crate::{Node, SetTrie};

/// Iterator for [`SetTrie::values`].
pub struct Values<'a, K, T> {
    idx: usize,
    current: &'a Node<K, T>,
    nodes: Vec<&'a Node<K, T>>,
}

impl<'a, K, T> Values<'a, K, T> {
    #[must_use]
    pub(crate) const fn new(trie: &SetTrie<K, T>) -> Values<K, T> {
        Values {
            idx: 0,
            current: &trie.0,
            nodes: vec![],
        }
    }
}

impl<'a, K, T> Iterator for Values<'a, K, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.current.leaves.len() {
            self.idx += 1;
            Some(&self.current.leaves[self.idx - 1])
        } else {
            self.nodes
                .extend(self.current.children.iter().map(|n| &n.1).rev());
            if let Some(next) = self.nodes.pop() {
                self.current = next;
                self.idx = 0;
                return self.next();
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    mod proptest {
        use crate::SetTrie;
        use ::proptest::prelude::*;
        use std::collections::{HashMap, HashSet};

        proptest! {
            #[test]
            #[ignore]
            fn values(testcase: HashMap<i32, Vec<i32>>) {
                let mut trie = SetTrie::new();

                for (v, mut k) in testcase.clone() {
                    k.sort();
                    trie.insert(k.clone(), v.clone());
                }

                let vals: HashSet<_> = trie.values().collect();
                let expected: HashSet<_> = testcase.keys().collect();
                assert_eq!(vals, expected);
            }
        }
    }
}
