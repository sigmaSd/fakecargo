# fakecargo
fake cargo for single rust files

# Usecases examples:

`fakecargo fmt myawesome_rust_script.rs`

`fakecargo clippy myawesome_rust_script.rs`

`fakecargo -c flamegraph -s myawesome_rust_script.rs arg1 arg2`

# Example: Use external dependencies with single scripts:
**mycoolscript.rs:**
```rust
use secret_msg::SecretMessage;
fn main() {
  println!("{}", "hello".one_way_encrypt());
}
```
`fakecargo -c add secret_msg -s mycoolscript.rs`

`fakecargo r mycoolscript.rs`