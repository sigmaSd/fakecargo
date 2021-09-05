use std::path::PathBuf;
use xshell::*;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let dir = mktemp_d()?;

    let file = scaffold(
        &dir,
        stringify!(
            fn main() {
                println!("hello");
            }
        ),
    )?;
    let out = cmd!("cargo run {file}").read()?;
    assert_eq!(out, "hello");

    let file = scaffold(
        &dir,
        stringify!(
            fn main() {
                println!("{}", std::env::args().nth(1).unwrap());
            }
        ),
    )?;
    let out = cmd!("cargo run -- run {file} -- arg1").read()?;
    assert_eq!(out, "arg1");

    let out = cmd!("cargo run -- compile {file}").read()?;
    let compiled_path = out.strip_prefix("Compiled to: ").unwrap();
    let out = cmd!("{compiled_path} hello-compiled").read()?;
    assert_eq!(out, "hello-compiled");

    Ok(())
}

fn scaffold(dir: &TempDir, data: &str) -> Result<PathBuf> {
    let file = dir.path().join("fake_test.rs");
    write_file(&file, data)?;
    Ok(file)
}
