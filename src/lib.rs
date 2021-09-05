use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Default)]
pub struct Plan {
    cmd: Option<String>,
    cmd_args: Option<Vec<String>>,
    script: Option<PathBuf>,
    script_args: Option<Vec<String>>,
    project_name: Option<String>,
}
impl Plan {
    pub fn set_cmd(self, cmd: impl ToString) -> Self {
        let cmd = cmd.to_string();
        Self {
            cmd: Some(cmd),
            ..self
        }
    }
    pub fn set_script(self, script: impl Into<PathBuf>) -> Self {
        let script = script.into();
        let project_name = script.file_stem().unwrap().to_str().unwrap().to_string();
        Self {
            script: Some(script),
            project_name: Some(project_name),
            ..self
        }
    }

    pub fn set_script_args(self, script_args: &[&str]) -> Self {
        Self {
            script_args: Some(script_args.iter().map(ToString::to_string).collect()),
            ..self
        }
    }
    pub fn set_cmd_args(self, cmd_args: &[&str]) -> Self {
        Self {
            cmd_args: Some(cmd_args.iter().map(ToString::to_string).collect()),
            ..self
        }
    }
    pub fn build(self) -> Result<Build> {
        Ok(Build {
            cmd: self.cmd.ok_or("cmd is not specified")?,
            script: self.script.ok_or("script is not specified")?,
            project_name: self.project_name.unwrap(),
            script_args: self.script_args.unwrap_or_default(),
            cmd_args: self.cmd_args.unwrap_or_default(),
        })
    }
}

#[derive(Debug)]
pub struct Build {
    cmd: String,
    script: PathBuf,
    project_name: String,
    script_args: Vec<String>,
    cmd_args: Vec<String>,
}
impl Build {
    fn root_dir(&self) -> PathBuf {
        env::temp_dir().join("fakecargo")
    }

    fn create_ground(&self) -> Result<()> {
        if let Err(e) = std::fs::create_dir(self.root_dir()) {
            if e.kind() != ErrorKind::AlreadyExists {
                return Err(e.into());
            }
        }
        Command::new("cargo")
            .current_dir(&self.root_dir())
            .args(&["new", &self.project_name])
            .output()?;
        Ok(())
    }
    fn copy_script(&self, direction: Direction) -> Result<()> {
        match direction {
            Direction::ToFakeGround => {
                fs::copy(&self.script, self.fake_src())?;
            }
            Direction::FromFakeGround => {
                fs::copy(&self.fake_src(), &self.script)?;
            }
        }
        Ok(())
    }
    fn execute(&self) -> Result<()> {
        if &self.cmd == "run" || &self.cmd == "r" {
            // run cargo cmd
            let status = Command::new("cargo")
                .arg("build")
                .args(&self.cmd_args)
                .current_dir(&self.fake_dir())
                .spawn()?
                .wait()?;
            if !status.success() {
                return Err("failed to build script".into());
            }

            Command::new(self.executable())
                .args(&self.script_args)
                .spawn()?
                .wait()?;
        } else {
            // run cargo cmd
            Command::new("cargo")
                .current_dir(&self.fake_dir())
                .arg(&self.cmd)
                .args(&self.cmd_args)
                .arg("--")
                .args(&self.script_args)
                .spawn()?
                .wait()?;
        }
        Ok(())
    }
    fn fake_dir(&self) -> PathBuf {
        self.root_dir().join(&self.project_name)
    }
    fn fake_src(&self) -> PathBuf {
        self.fake_dir().join("src/main.rs")
    }

    fn executable(&self) -> PathBuf {
        let target_dir = if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
            target_dir.into()
        } else {
            self.fake_dir().join("target")
        };
        target_dir.join("debug").join(&self.project_name)
    }

    pub fn run(&self) -> Result<()> {
        // create fakeground
        self.create_ground()?;
        // copy script to fakeground
        self.copy_script(Direction::ToFakeGround)?;
        // run user commands
        self.execute()?;
        // copy the script back, useful if the script was formatted with cargo fmt for example
        self.copy_script(Direction::FromFakeGround)?;
        Ok(())
    }
}

enum Direction {
    ToFakeGround,
    FromFakeGround,
}
