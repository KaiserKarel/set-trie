#![warn(clippy::all, clippy::cargo, missing_docs)]
//! Fast subset and superset queries based on tries.
//!
//! ```rust
//! use set_trie::SetTrie;
//!
//! let mut employees = SetTrie::new();
//! employees.insert(&["accounting", "banking"], "Daniels");
//! employees.insert(&["accounting", "banking", "crime"], "Stevens");
//!
//! assert_eq!(employees.subsets(&[&"accounting", &"banking", &"crime"]).collect::<Vec<_>>(), vec![&"Daniels", &"Stevens"]);
//! assert_eq!(employees.subsets(&[&"accounting", &"banking"]).collect::<Vec<_>>(), vec![&"Daniels"]);
//! assert_eq!(employees.supersets(&[&"accounting"]).collect::<Vec<_>>(), vec![&"Daniels", &"Stevens"]);
//! ```

use crate::subset::Subset;
use crate::superset::SuperSet;
use crate::values::Values;
use std::iter::FromIterator;

mod entry;
mod subset;
mod superset;
mod values;

pub use entry::{CreatedEntry, Entry, EntryBuilder, ExistingEntry};

#[derive(Debug, Default, Eq, PartialEq)]
struct Node<K, T> {
    children: Vec<(K, Node<K, T>)>,
    leaves: Vec<T>,
}

impl<K, T> Node<K, T> {
    pub const fn new() -> Self {
        Self {
            children: vec![],
            leaves: vec![],
        }
    }
}

/// Due to the recursive nature of the implementation of Drop, large `SetTries` cause a stack overflow
/// during deallocation. Our own implementation uses an iterative algorithm to deallocate.
impl<K, T> Drop for Node<K, T> {
    fn drop(&mut self) {
        let mut stack = Vec::with_capacity(self.children.len());
        while let Some((_, child)) = self.children.pop() {
            stack.push(child);
            while let Some(mut current) = stack.pop() {
                while let Some((_, child)) = current.children.pop() {
                    stack.push(child);
                }
            }
        }
    }
}

impl<K, T> Node<K, T>
where
    K: Ord,
{
    fn has_descendant(&self, key: &K) -> bool {
        if self.children.binary_search_by(|(k, _)| k.cmp(key)).is_ok() {
            return true;
        }
        self.children
            .iter()
            .take_while(|(k, _)| k < key)
            .any(|(_, n)| n.has_descendant(key))
    }

    fn between_inclusive(&self, from: &K, to: &K) -> &[(K, Self)] {
        match (
            self.children.binary_search_by(|(k, _)| k.cmp(from)),
            self.children.binary_search_by(|(k, _)| k.cmp(to)),
        ) {
            (Ok(from), Ok(to)) | (Err(from), Ok(to)) => &self.children[from..=to],
            (Ok(from), Err(to)) | (Err(from), Err(to)) => &self.children[from..to],
        }
    }
}

/// `SetTries` allow for efficient subset and superset queries. Think of it as a
/// [`HashMap`](std::collections::HashMap), where you want the key to be within or containing a range.
///
/// ```rust
/// let mut trie = set_trie::SetTrie::new();
///
/// trie.insert(&[1, 3, 5], "foo");
/// trie.insert(&[3], "bar");
///
/// assert_eq!(trie.subsets(&[&1, &3, &5, &6]).collect::<Vec<_>>(), vec![&"foo", &"bar"]);
/// assert_eq!(trie.supersets(&[&5]).collect::<Vec<_>>(), vec![&"foo"])
/// ```
///
/// # Restrictions
///
/// Keys are required to be Ord, as the trie stores the nodes in sorted order by key. This means
/// that the caller must ensure that provided keys are in sorted order, lest nonsensical results be
/// returned.
///
/// # Performance
///
/// Subsets and Supersets are lazily evaluated. Note that superset queries are far more expensive
/// than subset queries, so attempt to structure your problem around subsets.
#[derive(Debug, Default)]
pub struct SetTrie<K, T>(Node<K, T>);

impl<K, T> SetTrie<K, T> {
    /// Create a new, empty `SetTrie`, without allocating any space for the nodes.
    #[must_use]
    pub const fn new() -> Self {
        Self(Node::new())
    }
}

impl<K, T> SetTrie<K, T>
where
    K: Ord,
{
    /// A view into a single node in the trie; which must either be created or already exists.
    #[must_use]
    pub fn entry<IK: IntoIterator<Item = K>>(
        &mut self,
        keys: IK,
    ) -> EntryBuilder<K, T, IK::IntoIter> {
        EntryBuilder::new(self, keys.into_iter())
    }

    /// Insert the item in the given node. Will create the node if needed.
    pub fn insert(&mut self, keys: impl IntoIterator<Item = K>, item: T) {
        self.entry(keys.into_iter()).and_insert(item);
    }

    /// Inserts multiple items in the given node. More performant that repeatedly calling insert.
    pub fn insert_many<IK: IntoIterator<Item = K>, IT: IntoIterator<Item = T>>(
        &mut self,
        keys: IK,
        item: IT,
    ) {
        self.entry(keys.into_iter()).and_extend(item);
    }

    /// Iterates over all subsets of `keys` using DFS, meaning that the keys are visited
    /// in order of the query:
    ///
    /// ```rust
    /// let mut trie = set_trie::SetTrie::new();
    /// trie.insert(&[1], "foo");
    /// trie.insert(&[1, 2], "bar");
    /// trie.insert(&[1, 2, 3], "baz");
    ///
    /// assert_eq!(trie.subsets(&[&1, &2, &3]).collect::<Vec<_>>(), vec![&"foo", &"bar", &"baz"]);
    /// ```
    #[must_use]
    pub fn subsets<'a, 'b>(&'a self, keys: &'b [K]) -> Subset<'a, 'b, K, T> {
        Subset::new(self, keys)
    }

    /// Iterates over all values in the trie using DFS, meaning that values are visited in order
    /// of the keys stored in the trie.
    ///
    ///
    /// ```rust
    /// let mut trie = set_trie::SetTrie::new();
    /// trie.insert(&[1], "foo");
    /// trie.insert(&[1, 2], "bar");
    /// trie.insert(&[1, 2, 3], "baz");
    ///
    /// assert_eq!(trie.values().collect::<Vec<_>>(), vec![&"foo", &"bar", &"baz"]);
    /// ```
    #[must_use]
    pub const fn values(&self) -> Values<K, T> {
        Values::new(self)
    }

    /// Iterates over all supersets of `keys` in the trie using DFS, meaning that values are visited
    /// in order of the query.
    ///
    ///
    /// ```rust
    /// let mut trie = set_trie::SetTrie::new();
    /// trie.insert(&[1], "foo");
    /// trie.insert(&[1, 2], "bar");
    /// trie.insert(&[1, 2, 3], "baz");
    ///
    /// assert_eq!(trie.supersets(&[&1]).collect::<Vec<_>>(), vec![&"foo", &"bar", &"baz"]);
    /// ```
    ///
    /// # Remarks
    ///
    /// Note that the empty set will provide the same result as values. There is currently no fast
    /// path in the trie, so if you know that your query contains no keys, use [`SetTrie::values`]
    /// instead.
    #[must_use]
    pub fn supersets<'a, 'b>(&'a self, keys: &'b [K]) -> SuperSet<'a, 'b, K, T> {
        SuperSet::new(self, keys)
    }
}

impl<I, K, T> Extend<(I, T)> for SetTrie<K, T>
where
    I: IntoIterator<Item = K>,
    K: Ord,
{
    fn extend<F: IntoIterator<Item = (I, T)>>(&mut self, iter: F) {
        for (k, t) in iter {
            self.insert(k, t);
        }
    }
}

impl<I, K, T> FromIterator<(I, T)> for SetTrie<K, T>
where
    I: IntoIterator<Item = K>,
    K: Ord,
{
    fn from_iter<F: IntoIterator<Item = (I, T)>>(iter: F) -> Self {
        let mut trie = Self::new();
        trie.extend(iter);
        trie
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod doctests {
        include!(concat!(env!("OUT_DIR"), "/skeptic-tests.rs"));
    }

    #[test]
    fn insert() {
        let mut trie = SetTrie::new();
        trie.insert(&[1], "c");
        trie.insert(&[1, 2], "c");
        trie.insert(&[1, 2, 3], "a");
        trie.insert(&[1, 2, 3], "b");
        assert_eq!(trie.entry(&[1, 2, 3]).items(), Some(&vec!["a", "b"]))
    }

    /// Due to the recursive structure; the default Drop implementation actually causes a stack
    /// overflow.
    #[test]
    fn test_stack_overflow() {
        let seed = 2000000;
        let mut trie = SetTrie::new();

        let mut current = trie.entry(0..1).or_insert(0);
        for i in 1..seed {
            current = current.entry(i - 1..i).or_insert(i)
        }
    }

    #[test]
    // https://github.com/KaiserKarel/set-trie/issues/6
    fn subsets_small2() {
        let mut v = SetTrie::new();
        v.insert(&[1, 2], 'a');
        {
            let mut s = v.subsets(&[&0, &1]);
            assert_eq!(s.next(), None);
        }
        {
            let mut s = v.subsets(&[&0, &1, &2]);
            assert_eq!(s.next(), None);
        }
        {
            let mut s = v.subsets(&[&1, &2]);
            assert_eq!(s.next(), Some(&'a'));
        }
        {
            v.insert(&[0, 2], 'a');
            let mut s = v.subsets(&[&0, &2]);
            assert_eq!(s.next(), Some(&'a'));
        }
    }

    #[test]
    // https://github.com/KaiserKarel/set-trie/issues/6
    fn supersets_small2() {
        let mut v = SetTrie::new();
        v.insert(&[1, 2], 'a');
        {
            let mut s = v.supersets(&[&0]);
            assert_eq!(s.next(), None);
        }
        {
            let mut s = v.supersets(&[]);
            assert_eq!(s.next(), Some(&'a'));
        }
        {
            let mut s = v.supersets(&[&1]);
            assert_eq!(s.next(), Some(&'a'));
        }
        {
            v.insert(&[0, 2], 'a');
            let mut s = v.supersets(&[&1, &2]);
            assert_eq!(s.next(), Some(&'a'));
        }
    }
}
