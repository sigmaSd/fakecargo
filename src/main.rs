use fakecargo::{Plan, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut should_print_exe_path = false;
    let plan = parse(args, &mut should_print_exe_path);
    let build = plan.build()?;
    build.run()?;
    if should_print_exe_path {
        println!("Compiled to: {}", build.executable().display());
    }
    Ok(())
}

fn parse(args: Vec<String>, should_print_exe_path: &mut bool) -> Plan {
    match &args
        .iter()
        .map(|v| v.as_str())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["-h"] | ["--help"] => usage_and_exit(),
        ["compile", script] => {
            *should_print_exe_path = true;
            Plan::default()
                .set_cmd("build")
                .set_cmd_args(&["--release"])
                .set_script(script)
        }
        [script] => Plan::default().set_cmd("run").set_script(script),
        [cmd, script] => Plan::default().set_cmd(cmd).set_script(script),
        args => match args.split_at(args.iter().position(|pat| pat == &"--").unwrap_or(0)) {
            ([], [cmd, cmd_args @ .., script]) => Plan::default()
                .set_cmd(cmd)
                .set_cmd_args(cmd_args)
                .set_script(script),
            ([cmd, cmd_args @ .., script], ["--", script_args @ ..]) => Plan::default()
                .set_cmd(cmd)
                .set_script(script)
                .set_cmd_args(cmd_args)
                .set_script_args(script_args),
            _ => usage_and_exit(),
        },
    }
}

fn usage_and_exit() -> ! {
    println!(
        "fakecargo \n
        Fake cargo for single rust scripts\n
  Usage:\n
	fakecargo (script) /* defaults to run the script */\n
	fakecargo (cmd) (cmd_args) (script) \n
	fakecargo (cmd) (script) -- (script-args)\n\n"
    );
    std::process::exit(0)
}
