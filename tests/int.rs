use std::{env::temp_dir, fs::create_dir_all, path::PathBuf};

use fakecargo::{Plan, Result};

#[test]
fn t1() -> Result<()> {
    let file = scaffold()?;
    let plan = Plan::default().set_cmd("run").set_script(&file);
    plan.build()?.run()?;
    let plan = Plan::default().set_cmd("fmt").set_script(&file);
    plan.build()?.run()?;
    let plan = Plan::default().set_cmd("clippy").set_script(&file);
    plan.build()?.run()?;
    let plan = Plan::default()
        .set_cmd("run")
        .set_cmd_args(&["--offline"])
        .set_script(&file)
        .set_script_args(&["a"]);
    plan.build()?.run()?;

    Ok(())
}

fn scaffold() -> Result<PathBuf> {
    let dir = temp_dir().join("fakecargo_tests");
    let _ = create_dir_all(&dir);
    let file = dir.join("fake_test.rs");
    let data = "fn main() { let _a = \"hello\"; }";
    std::fs::write(&file, data)?;
    Ok(file)
}
