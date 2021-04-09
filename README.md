![maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)
![crates.io](https://img.shields.io/crates/v/set-trie.svg)
![build](https://github.com/kaiserkarel/set-trie/workflows/Tests/badge.svg)

# set-trie

Fast subset and superset queries based on tries. If you have lookup-based queries, `K -> V`, but instead of looking for
an exact match with K, you want all `K`'s which are a subset or superset of your query, then look no further.

```rust
use set_trie::SetTrie;

fn main() {
    let mut employees = SetTrie::new();
    employees.insert(&["accounting", "banking"], "Daniels");
    employees.insert(&["accounting", "banking", "crime"], "Stevens");

    assert_eq!(employees.subsets(&[&"accounting", &"banking", &"crime"]).collect::<Vec<_>>(), vec![&"Daniels", &"Stevens"]);
    assert_eq!(employees.subsets(&[&"accounting", &"banking"]).collect::<Vec<_>>(), vec![&"Daniels"]);
    assert_eq!(employees.supersets(&[&"accounting"]).collect::<Vec<_>>(), vec![&"Daniels", &"Stevens"]);
}
```

# Restrictions

Although currently not implemented in the type system, due to a lack of a trait bound over sorted iterators, set tries
require all queries to be sorted. Failing to sort the query or key will result in nonsensical results:

```rust
use set_trie::SetTrie;

fn main() {
    let mut trie = SetTrie::new();
    trie.insert(&[2, 3], "Foo");
    trie.insert(&[1, 2], "Bar");

    // although we'd expect this to contain &"Bar".
    assert_eq!(trie.subsets(&[&2, &1]).collect::<Vec<_>>(), Vec::<&&str>::new()); 
}
```

# Features
 - Subsets and supersets are lazily evaluated, through an iterative DFS algorithm.
 - Convenient `entry` API.
