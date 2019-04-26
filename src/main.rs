use std::env::{args, temp_dir};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

fn main() {
    run().expect("Something happened");
}

fn run() -> std::io::Result<()> {
    // playground dir, src and state
    let temp_dir = temp_dir();
    let playdir = temp_dir.as_path().join("fakecargo");
    let playsrc = playdir.join("src/main.rs");
    let mut playdir_state = vec![];

    // parse args
    let args: Vec<String> = args().skip(1).collect();
    // Clean and exit if specifed
    if clean(&args, &playdir) {
        return Ok(());
    }
    let (mut cmd, mut script) = parse(args);

    // create playground
    Command::new("cargo")
        .current_dir(&temp_dir)
        .args(&["new", "fakecargo"])
        .output()?;

    // simulate script enviorment by
    // symlinking all files and dirs in current dir to playdir
    // TODO: Fix this for windows
    for entry in fs::read_dir(Path::new("./"))? {
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
    fs::copy(&script[0], &playsrc)?;

    // save current playdir state so we can use it later
    // to compare and know which files/dirs where added
    for entry in fs::read_dir(&playdir)? {
        let entry = entry?;
        playdir_state.push(entry.file_name());
    }

    // add script args to cmd if present
    if script.len() > 1 {
        cmd.append(&mut script.split_off(1));
    }

    // run cargo cmd
    Command::new("cargo")
        .current_dir(&playdir)
        .args(&cmd)
        .spawn()?
        .wait()?;

    // Copy back the script
    fs::copy(&playsrc, &script[0])?;

    // Add new created files to script real directory
    // TODO: Also handle created folders
    for entry in fs::read_dir(&playdir)? {
        let entry = entry?;
        if !playdir_state.contains(&entry.file_name()) && entry.file_type()?.is_file() {
            fs::copy(entry.path(), Path::new("./").join(&entry.file_name()))?;
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
        exit(0);
    }

    (cmd, script)
}

fn clean(args: &[String], playdir: &PathBuf) -> bool {
    if args.contains(&"fakeclean".to_string()) {
        let _ = fs::remove_dir_all(&playdir);
        return true;
    }
    false
}

fn usage() {
    println!(
        "fakecargo \n
        Fake cargo for single rust scripts\n
  Usage:\n
	fakecargo cmd script \n
	fakecargo -c cmd args -s script args\n\n
  To reset fakecargo:\n
        fakecargo fakeclean script"
    );
}
