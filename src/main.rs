fn main() {
    run().expect("Something happened");
}

fn run() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (mut cmd, mut script) = parse(args);

    // playground dir, src and state
    let temp_dir: std::path::PathBuf = std::env::temp_dir();
    let playdir = temp_dir.as_path().join("fakecargo");
    let playsrc = playdir.join("src/main.rs");
    let mut playdir_state = vec![];

    // create playground
    if std::path::Path::exists(&playdir) {
        std::fs::remove_dir_all(&playdir)?;
    }
    std::process::Command::new("cargo")
        .current_dir(&temp_dir)
        .args(&["new", "fakecargo"])
        .output()?;

    // simulate script enviorment by
    // symlinking all files and dirs in current dir to playdir
    // TODO: Fix this for windows
    for entry in std::fs::read_dir(std::path::Path::new("./"))? {
        let entry = entry?;

        let symlink_path = {
            let mut d = playdir.clone();
            d.push(entry.file_name());
            d
        };
        if cfg!(target_os = "linux") {
            let _ = std::os::unix::fs::symlink(entry.path().canonicalize()?, symlink_path);
        }
        // Not tested on windows
        else if cfg!(target_os = "windows") {
            //On Windows, you must specify whether a symbolic link points to a file or directory why???
            if entry.file_type()?.is_file() {
                #[cfg(target_os = "windows")]
                let _ =
                    std::os::windows::fs::symlink_file(entry.path().canonicalize()?, symlink_path);
            } else if entry.file_type()?.is_dir() {
                #[cfg(target_os = "windows")]
                let _ =
                    std::os::windows::fs::symlink_dir(entry.path().canonicalize()?, symlink_path);
            }
        }
    }

    // copy script to playground
    std::fs::copy(&script[0], &playsrc)?;

    // save current playdir state so we can use it later
    // to compare and know which files/dirs where added
    for entry in std::fs::read_dir(&playdir)? {
        let entry = entry?;
        playdir_state.push(entry.file_name());
    }

    // add script args to cmd if present
    if script.len() > 1 {
        cmd.append(&mut script.split_off(1));
    }

    // run cargo cmd
    std::process::Command::new("cargo")
        .current_dir(&playdir)
        .args(&cmd)
        .spawn()?
        .wait()?;

    // Copy back the script
    std::fs::copy(&playsrc, &script[0])?;

    // Add new created files to script real directory
    // TODO: Also handle created folders
    for entry in std::fs::read_dir(&playdir)? {
        let entry = entry?;
        if !playdir_state.contains(&entry.file_name()) && entry.file_type()?.is_file() {
            std::fs::copy(
                entry.path(),
                std::path::Path::new("./").join(&entry.file_name()),
            )?;
        }
    }

    Ok(())
}

fn parse(mut args: Vec<String>) -> (Vec<String>, Vec<String>) {
    // most common case `fakecargo cmd script`
    if args.len() == 2 {
        let script = args.pop().unwrap();
        let cmd = args.pop().unwrap();
        return (vec![cmd], vec![script]);
    }

    // if its more then 3 look for flags -c -s
    let (mut cmd, mut cmd_flag) = (Vec::new(), false);
    let (mut script, mut script_flag) = (Vec::new(), false);

    for arg in args {
        if arg == "-c" {
            cmd_flag = true;
            script_flag = false;
        } else if arg == "-s" {
            script_flag = true;
            cmd_flag = false;
        } else if cmd_flag {
            cmd.push(arg);
        } else if script_flag {
            script.push(arg);
        }
    }

    // exit if wrong usage
    if cmd.is_empty() || script.is_empty() {
        usage();
        exit();
    }

    (cmd, script)
}

fn usage() {
    println!(
        "fakecargo \n
  Usage:\n
	fakecargo cmd script \n
	fakecargo -c cmd args -s script args"
    );
}
fn exit() {
    std::process::exit(0);
}
