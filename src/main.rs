use std::env::{args, temp_dir};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

fn main() {
    run().expect("Something happened");
}

fn run() -> std::io::Result<()> {
    // Parse args
    let args: Vec<String> = args().skip(1).collect();
    let (cmd, script, project_name) = parse(args);

    // current dir, playground dir, src, state and project name
    let current_dir = Path::new("./");
    let temp_dir = temp_dir();
    let playdir = temp_dir.as_path().join(&project_name);
    let playsrc = playdir.join("src/main.rs");
    let mut playdir_state = vec![];

    // create playground
    Command::new("cargo")
        .current_dir(&temp_dir)
        .args(&["new", &project_name])
        .output()?;

    // Get rid of a lot of problems by doing a soft reset
    for entry in fs::read_dir(&playdir)? {
        let entry = entry?;

        if !["src", "Cargo.toml", "target"]
            .contains(&entry.file_name().into_string().unwrap().as_str())
        {
            let path = entry.path();
            if path.is_dir() {
                let _ = fs::remove_dir_all(&path);
            } else {
                let _ = fs::remove_file(&path);
            }
        }
    }

    // simulate script enviorment by
    // symlinking all files and dirs in current dir to playdir
    // TODO: Fix this for windows
    for entry in fs::read_dir(&current_dir)? {
        let entry = entry?;

        let symlink_path = {
            let mut d = playdir.clone();
            d.push(entry.file_name());
            d
        };
        #[cfg(target_os = "linux")]
        let _ = std::os::unix::fs::symlink(entry.path().canonicalize()?, symlink_path);
        // Not tested on windows
        //On Windows, you must specify whether a symbolic link points to a file or directory why???
        #[cfg(target_os = "windows")]
        {
            if entry.file_type()?.is_file() {
                let _ =
                    std::os::windows::fs::symlink_file(entry.path().canonicalize()?, symlink_path);
            } else if entry.file_type()?.is_dir() {
                let _ =
                    std::os::windows::fs::symlink_dir(entry.path().canonicalize()?, symlink_path);
            }
        }
    }

    // copy script to playground
    fs::copy(&script, &playsrc)?;

    // save current playdir state so we can use it later
    // to compare and know which files/dirs where added
    for entry in fs::read_dir(&playdir)? {
        let entry = entry?;
        playdir_state.push(entry.file_name());
    }

    // run cargo cmd
    Command::new("cargo")
        .current_dir(&playdir)
        .args(&cmd)
        .spawn()?
        .wait()?;

    // Copy back the script
    fs::copy(&playsrc, &script)?;

    // Add newly created files and dirs to script real directory
    for entry in fs::read_dir(&playdir)? {
        let entry = entry?;
        if !playdir_state.contains(&entry.file_name()) {
            copy_entry(&entry.path(), &&current_dir.join(&entry.file_name()))?;
        }
    }

    Ok(())
}

fn parse(mut args: Vec<String>) -> (Vec<String>, PathBuf, String) {
    // look for help
    if args.contains(&"-h".to_string()) | args.contains(&"--help".to_string()) {
        usage();
        std::process::exit(0)
    }
    // New: special case `fakecargo script`
    // Since its actually whats most commenly used
    if args.len() == 1 {
        let script = args.pop().unwrap();
        let cmd = String::from("run");
        let project_name = project_from_script(&script);
        return (vec![cmd], Path::new(&script).to_path_buf(), project_name);
    }

    // most common case `fakecargo cmd script`
    if args.len() == 2 {
        let script = args.pop().unwrap();
        let cmd = args.pop().unwrap();
        let project_name = project_from_script(&script);

        // Clean(Hard reset) and exit if specifed
        // fakecargo fakeclean script
        if clean(&cmd, &project_name) {
            exit(0);
        }

        return (vec![cmd], Path::new(&script).to_path_buf(), project_name);
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
        exit(0);
    }

    // add script args to cmd if present
    if script.len() > 1 {
        cmd.append(&mut script.split_off(1));
    }

    let script = script.pop().unwrap();
    let project_name = project_from_script(&script);

    (cmd, Path::new(&script).to_path_buf(), project_name)
}

fn clean(cmd: &str, project_name: &str) -> bool {
    if cmd == "fakeclean" {
        let playdir = temp_dir().as_path().join(&project_name);

        let _ = fs::remove_dir_all(&playdir);
        return true;
    }
    false
}

// helper functions
fn copy_entry(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        let _ = fs::DirBuilder::new().create(&dst);

        for sub_entry in fs::read_dir(src)? {
            let sub_entry = sub_entry?;
            let path = sub_entry.path();
            let dst = dst.join(&path.file_name().unwrap());
            copy_entry(&path, &dst)?;
        }
    } else {
        fs::copy(src, dst)?;
    }
    Ok(())
}

fn project_from_script(script: &str) -> String {
    Path::new(&script.replace(".rs", ""))
        .file_name()
        .expect("Please specify a valid script to run")
        .to_str()
        .unwrap()
        .to_owned()
}

// Usage
fn usage() {
    println!(
        "fakecargo \n
        Fake cargo for single rust scripts\n
  Usage:\n
	fakecargo script /* defaults to run the script */\n
	fakecargo cmd script \n
	fakecargo -c cmd args -s script args\n\n
  To reset fakecargo:\n
        fakecargo fakeclean script"
    );
}
