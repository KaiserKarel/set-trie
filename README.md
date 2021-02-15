![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# set-trie

Fast subset and superset queries based on tries.

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

License: MIT
