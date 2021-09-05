# Fakecargo
fake cargo for single rust files

## Use-cases examples:

`fakecargo fmt myawesome_rust_script.rs`

`fakecargo clippy myawesome_rust_script.rs`

`fakecargo flamegraph myawesome_rust_script.rs -- arg1 arg2`

 **Example: Use external dependencies with single script (Requires `cargo-edit`):**
 
`mycoolscript.rs:`
```rust
use secret_msg::SecretMessage;
fn main() {
  println!("{}", "hello".one_way_encrypt());
}
```
`fakecargo add secret_msg mycoolscript.rs`

`fakecargo mycoolscript.rs`

## Shell replacement example
See `fake_tests.rs`, to run it:
- `fakecargo add xshell fake_tests.rs`
- `fakecargo fake_tests.rs`
