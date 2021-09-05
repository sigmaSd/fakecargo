use fakecargo::{Plan, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let plan = parse(args);
    let build = plan.build()?;
    build.run()
}

fn parse(args: Vec<String>) -> Plan {
    match &args
        .iter()
        .map(|v| v.as_str())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["-h"] | ["--help"] => usage_and_exit(),
        [script] => Plan::default().set_cmd("run").set_script(script),
        [cmd, script] => Plan::default().set_cmd(cmd).set_script(script),
        [cmd, script, "--", script_args @ ..] => Plan::default()
            .set_cmd(cmd)
            .set_script(script)
            .set_script_args(script_args),
        [cmd, cmd_args @ .., script] => Plan::default()
            .set_cmd(cmd)
            .set_cmd_args(cmd_args)
            .set_script(script),
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
