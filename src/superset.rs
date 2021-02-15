use crate::{Node, SetTrie};

/// Iterator for [superset](SetTrie::superset) method.
pub struct SuperSet<'a, 'b, K, T> {
    idx: usize,
    current: (bool, bool, &'a Node<K, T>),
    next: Vec<(bool, &'a K, &'a Node<K, T>)>,
    keys: &'b [K],
}

impl<'a, 'b, K, T> SuperSet<'a, 'b, K, T>
where
    K: Ord,
{
    pub(crate) fn new(trie: &'a SetTrie<K, T>, keys: &'b [K]) -> Self {
        SuperSet {
            current: (keys.is_empty(), keys.is_empty(), &trie.0),
            next: vec![],
            // if we are queried for the empty set, we want to process root, else jump to evaluating
            // the descendants of root.
            idx: if keys.is_empty() {
                0
            } else {
                trie.0.leaves.len()
            },
            keys,
        }
    }
}

impl<'a, 'b, K, T> Iterator for SuperSet<'a, 'b, K, T>
where
    K: Ord,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let (encountered_first, is_superset, current) = self.current;

        if is_superset && self.idx < current.leaves.len() {
            self.idx += 1;
            Some(&current.leaves[self.idx - 1])
        } else if let (Some(first), Some(last)) = (self.keys.first(), self.keys.last()) {
            self.next.extend(
                current
                    .children
                    .iter()
                    .map(|(k, n)| {
                        (
                            // If we have encountered a first, any child is a candidate. If our
                            // own key is greater than the first key, and we have not yet
                            // encountered the first key, then we can never be a superset.
                            (n.has_descendant(first) || k == first || encountered_first)
                                && (k <= first || encountered_first),
                            k,
                            n,
                        )
                    })
                    .filter(|n| n.0)
                    .rev(),
            );

            if let Some((b, k, n)) = self.next.pop() {
                self.current = (b, k >= last || is_superset, n);
                self.idx = 0;
                return self.next();
            }
            None
        // empty set is queried, thus we can include every single item
        } else {
            let next = current.children.iter().map(|(k, n)| (true, k, n));
            self.next.extend(next.rev());

            if let Some((b, _, n)) = self.next.pop() {
                self.current = (b, true, n);
                self.idx = 0;
                return self.next();
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.current.2.leaves.len() - self.idx, None)
    }
}

#[cfg(test)]
mod tests {
    use crate::SetTrie;

    #[test]
    fn superset_small() {
        let mut v = SetTrie::new();
        v.insert(&[1, 2, 3], 'a');
        v.insert(&[1, 2, 4], 'b');
        v.insert(&[0, 2, 4], 'c');

        let qr = v.supersets(&[&1, &2]).collect::<Vec<_>>();
        assert_eq!(qr, vec![&'a', &'b']);
    }

    #[test]
    fn superset_medium() {
        let mut trie = SetTrie::new();
        trie.insert(&[1], "a");
        trie.insert(&[2], "b");
        trie.insert(&[2, 3], "c");
        trie.insert(&[3], "d");
        trie.insert(&[2, 3, 4], "e");

        assert_eq!(
            trie.supersets(&[]).collect::<Vec<_>>(),
            vec![&"a", &"b", &"c", &"e", &"d"]
        );

        assert_eq!(
            trie.supersets(&[&2]).collect::<Vec<_>>(),
            vec![&"b", &"c", &"e"]
        );

        assert_eq!(trie.supersets(&[&1]).collect::<Vec<_>>(), vec![&"a"]);

        assert_eq!(
            trie.supersets(&[&2, &3]).collect::<Vec<_>>(),
            vec![&"c", &"e"]
        );

        assert_eq!(
            trie.supersets(&[&3]).collect::<Vec<_>>(),
            vec![&"c", &"e", &"d"]
        );
    }

    mod proptest {
        use crate::SetTrie;
        use ::proptest::prelude::*;
        use std::collections::{HashMap, HashSet};

        proptest! {
            #[test]
            #[ignore]
            fn superset(testcase: HashMap<i32, Vec<i32>>) {
                let mut trie = SetTrie::new();

                for (v, mut k) in testcase.clone() {
                    k.sort();
                    trie.insert(k.clone(), v.clone());
                    let supersets = trie.supersets(&k).collect::<Vec<_>>();

                    // we should get our just inserted item back.
                    assert!(supersets.contains(&&v));


                    // all other returned items should be a superset of K.
                    let k: HashSet<_> = k.iter().collect();
                    for value in supersets.clone() {
                        let key: HashSet<_> = testcase.get(&value).unwrap().iter().collect();
                        assert!(key.is_superset(&k));
                    }

                    // ensure that the trie has not missed any values.
                    let got: HashSet<i32> = supersets.iter().cloned().cloned().collect();
                    let want: HashSet<i32> = trie.values().cloned().filter(|i| {
                        let key: HashSet<_> = testcase.get(i).unwrap().iter().collect();
                        key.is_superset(&k)
                    }).collect();
                    assert_eq!(got, want);
                }
            }
        }
    }
}
