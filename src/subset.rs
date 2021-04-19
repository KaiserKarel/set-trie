use crate::{Node, SetTrie};

/// Iterator for [subset](SetTrie::subset) method.
#[derive(Debug, Clone)]
pub struct Subset<'a, 'b, K, T> {
    current: &'a Node<K, T>,
    next: Vec<(&'a K, &'a Node<K, T>)>,
    idx: usize,
    keys: &'b [K],
}

impl<'a, 'b, K, T> Subset<'a, 'b, K, T>
where
    K: Ord,
{
    pub(crate) fn new(trie: &'a SetTrie<K, T>, keys: &'b [K]) -> Self {
        Subset {
            current: &trie.0,
            next: vec![],
            idx: 0,
            keys,
        }
    }
}

impl<'a, 'b, K, T> Iterator for Subset<'a, 'b, K, T>
where
    K: Ord,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.current.leaves.len() {
            self.idx += 1;
            Some(&self.current.leaves[self.idx - 1])
        } else {
            if let (Some(from), Some(to)) = (self.keys.first(), self.keys.last()) {
                self.next.extend(
                    self.current
                        // technically, between inclusive is only necessary on the first iteration,
                        // where we check handle the root node. Every subsequent iter may use up-to,
                        // which saves a single binary search. For long key lengths this may matter.
                        .between_inclusive(from, to)
                        .iter()
                        .rev()
                        .map(|n| (&n.0, &n.1)),
                );

                while let Some((k, node)) = self.next.pop() {
                    if self.keys.binary_search(k).is_ok() {
                        self.idx = 0;
                        self.current = node;
                        return self.next();
                    }
                    self.next.extend(
                        node.between_inclusive(from, to)
                            .iter()
                            .map(|n| (&n.0, &n.1)),
                    )
                }
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.current.leaves.len() - self.idx, None)
    }
}

#[cfg(test)]
mod tests {
    use crate::SetTrie;

    #[test]
    fn subsets_small() {
        let mut v = SetTrie::new();
        v.insert(&[1, 2, 3], 'a');
        v.insert(&[1, 2], 'b');
        v.insert(&[0, 2, 4], 'c');
        v.insert(&[0], 'd');
        v.insert(&[0, 3], 'e');
        v.insert(&[], 'f');
        v.insert(&[2, 3], 'g');
        v.insert(&[2], 'h');
        v.insert(&[5], 'i');

        // subsets should by default return a DFS ordering.
        assert_eq!(
            v.subsets(&[&1, &2, &3, &5]).collect::<Vec<_>>(),
            vec![&'f', &'b', &'a', &'h', &'g', &'i']
        );

        // A set is its own subset.
        assert_eq!(v.subsets(&[]).collect::<Vec<_>>(), vec![&'f']);

        // Quite a specific match should work.
        assert_eq!(v.subsets(&[&5]).collect::<Vec<_>>(), vec![&'f', &'i']);

        // Non-existing key should match nothing
        assert_eq!(v.subsets(&[&6]).collect::<Vec<_>>(), vec![&'f']);
    }

    mod proptest {
        use crate::SetTrie;
        use ::proptest::prelude::*;
        use std::collections::{HashMap, HashSet};

        proptest! {
            #[test]
            #[ignore]
            fn subset(testcase: HashMap<i32, Vec<i32>>) {
                let mut trie = SetTrie::new();

                for (v, mut k) in testcase.clone() {
                    k.sort();
                    trie.insert(k.clone(), v.clone());
                    let subsets = trie.subsets(&k).collect::<Vec<_>>();

                    // we should get our just inserted item back.
                    assert!(subsets.contains(&&v));


                    // all other returned items should be a subset of K.
                    let k: HashSet<_> = k.iter().collect();
                    for value in subsets.clone() {
                        let key: HashSet<_> = testcase.get(&value).unwrap().iter().collect();
                        assert!(key.is_subset(&k));
                    }

                    // ensure that the trie has not missed any values.
                    let got: HashSet<i32> = subsets.iter().cloned().cloned().collect();
                    let want: HashSet<i32> = trie.values().cloned().filter(|i| {
                        let key: HashSet<_> = testcase.get(i).unwrap().iter().collect();
                        key.is_subset(&k)
                    }).collect();
                    assert_eq!(got, want);
                }
            }
        }
    }
}
