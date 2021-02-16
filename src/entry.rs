#![allow(clippy::module_name_repetitions)]

use crate::{Node, SetTrie};

/// `EntryBuilder` for the [entry](SetTrie::entry) method. Entries are lazily evaluated, thus the builder
/// is used to provide the configuration, while the [entry](Entry) is already evaluated.
pub struct EntryBuilder<'a, K, T, IK>
where
    IK: Iterator<Item = K> + 'a,
    K: Ord,
{
    node: &'a mut Node<K, T>,
    keys: IK,
}

impl<'a, K, T, IK> EntryBuilder<'a, K, T, IK>
where
    IK: Iterator<Item = K> + 'a,
    K: Ord,
{
    pub(crate) fn new(trie: &'a mut SetTrie<K, T>, keys: IK) -> Self {
        EntryBuilder {
            node: &mut trie.0,
            keys,
        }
    }

    pub(crate) fn from_node(node: &'a mut Node<K, T>, keys: IK) -> Self {
        EntryBuilder { node, keys }
    }
}

/// A view into a node of a [`SetTrie`](SetTrie), either created or already existing.
pub enum Entry<'a, K, T>
where
    K: Ord,
{
    /// Indicates that the entry was created.
    Created(CreatedEntry<'a, K, T>),

    /// Indicates that the entry already existed.
    Existing(ExistingEntry<'a, K, T>),
}

/// Indicates that the entry was created.
pub struct CreatedEntry<'a, K, T>
where
    K: Ord,
{
    node: &'a mut Node<K, T>,
}

/// Indicates that the entry already exists.
pub struct ExistingEntry<'a, K, T>
where
    K: Ord,
{
    node: &'a mut Node<K, T>,
}

impl<'a, K, T, IK> EntryBuilder<'a, K, T, IK>
where
    IK: Iterator<Item = K> + 'a,
    K: Ord,
{
    /// Extends the entry, creating it if needed
    pub fn and_extend(self, default: impl IntoIterator<Item = T>) -> Entry<'a, K, T> {
        match self.or_create() {
            Entry::Existing(e) => {
                e.node.leaves.extend(default.into_iter());
                Entry::Existing(e)
            }
            Entry::Created(e) => {
                e.node.leaves.extend(default.into_iter());
                Entry::Created(e)
            }
        }
    }

    /// Inserts into the entry, creating it if needed
    pub fn and_insert(self, default: T) -> Entry<'a, K, T> {
        match self.or_create() {
            Entry::Existing(e) => {
                e.node.leaves.push(default);
                Entry::Existing(e)
            }
            Entry::Created(e) => {
                e.node.leaves.push(default);
                Entry::Created(e)
            }
        }
    }

    /// Finds the entry, and if it does not exist, extends with the provided value.
    pub fn or_extend(self, default: impl IntoIterator<Item = T>) -> Entry<'a, K, T> {
        match self.or_create() {
            entry @ Entry::Existing(_) => entry,
            Entry::Created(e) => {
                e.node.leaves.extend(default.into_iter());
                Entry::Created(e)
            }
        }
    }

    /// Finds the entry, and if it does not exist, inserts the value.
    pub fn or_insert(self, default: T) -> Entry<'a, K, T> {
        match self.or_create() {
            entry @ Entry::Existing(_) => entry,
            Entry::Created(e) => {
                e.node.leaves.push(default);
                Entry::Created(e)
            }
        }
    }

    /// Finds the entry, and if it does not exist, creates it.
    pub fn or_create(self) -> Entry<'a, K, T> {
        let mut node = self.node;
        let mut created = false;

        for key in self.keys {
            node = match node.children.binary_search_by(|(k, _)| k.cmp(&key)) {
                Ok(idx) => &mut (node.children[idx].1),
                Err(idx) => {
                    created = true;
                    node.children.insert(idx, (key, Node::new()));
                    &mut (node.children[idx].1)
                }
            }
        }

        if created {
            return Entry::Created(CreatedEntry { node });
        }
        Entry::Existing(ExistingEntry { node })
    }

    /// Finds the entry, but does not create one. This method short circuits on the first missing
    /// key.
    pub fn find(self) -> Option<ExistingEntry<'a, K, T>> {
        let mut node = self.node;

        for key in self.keys {
            node = match node.children.binary_search_by(|(k, _)| k.cmp(&key)) {
                Ok(idx) => &mut (node.children[idx].1),
                Err(_) => return None,
            }
        }
        Some(ExistingEntry { node })
    }

    /// Returns all associated items of an entry.
    pub fn items(self) -> Option<&'a Vec<T>> {
        self.find().map(|node| &node.node.leaves)
    }

    /// Mutably returns all associated items of an entry.
    pub fn items_mut(self) -> Option<&'a mut Vec<T>> {
        self.find().map(|node| &mut node.node.leaves)
    }
}

impl<'a, K, T> Entry<'a, K, T>
where
    K: Ord,
{
    fn node(&self) -> &Node<K, T> {
        match self {
            Entry::Existing(e) => e.node,
            Entry::Created(e) => e.node,
        }
    }

    fn node_mut(&mut self) -> &mut Node<K, T> {
        match self {
            Entry::Existing(e) => e.node,
            Entry::Created(e) => e.node,
        }
    }

    /// Returns all associated items of an entry.
    #[must_use]
    pub fn items(&self) -> &Vec<T> {
        &self.node().leaves
    }

    /// Mutably returns all associated items of an entry.
    #[must_use]
    pub fn items_mut(&mut self) -> &mut Vec<T> {
        &mut self.node_mut().leaves
    }

    /// Provides a view into a child of the entry. If you are sequentially inserting longer keys,
    /// reusing the entry is more efficient than starting from the root.
    ///
    /// ```rust
    /// // Equivalent to first inserting 0..1, then 0..2 etc.
    ///
    /// let mut trie = set_trie::SetTrie::new();
    /// let mut current = trie.entry(0..1).or_insert(0);
    ///     for i in 1..5000 {
    ///         current = {
    ///             current.entry(i-1..i).or_insert(i)
    ///         }
    ///     }
    /// ```
    #[must_use]
    pub fn entry<IK: IntoIterator<Item = K>>(
        self,
        keys: IK,
    ) -> EntryBuilder<'a, K, T, IK::IntoIter> {
        match self {
            Entry::Created(e) => EntryBuilder::from_node(e.node, keys.into_iter()),
            Entry::Existing(e) => EntryBuilder::from_node(e.node, keys.into_iter()),
        }
    }
}
