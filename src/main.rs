fn main() -> std::io::Result<()> {
    let temp_dir: std::path::PathBuf = std::env::temp_dir();
    let mut args = std::env::args().skip(1);

    let file: String = args.next_back().expect("No file given");
    let cmd: Vec<String> = args.collect();
    if cmd.is_empty() {
        panic!("No cmd given")
    }

    // create playground
    std::process::Command::new("cargo")
        .current_dir(&temp_dir)
        .args(&["new", "fakecargo"])
        .output()?;

    // playground dir
    let (playdir, playsrc) = {
        let mut d = temp_dir;
        d.push("fakecargo");
        let mut s = d.clone();
        s.push(std::path::Path::new("src/main.rs"));
        (d, s)
    };

    // copy file to playground
    std::fs::copy(&file, &playsrc)?;

    // run cargo cmd
    std::process::Command::new("cargo")
        .current_dir(&playdir)
        .args(&cmd)
        .spawn()?
        .wait()?;

    //copy back the src file
    std::fs::copy(&playsrc, &file)?;

    Ok(())
}
