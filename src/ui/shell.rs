/// Print an environment variable export to stdout with shell-specific hint to stderr.
pub fn print_env_export(var_name: &str, value: &str, context_msg: &str) {
    println!("export {var_name}=\"{value}\"");
    eprintln!("{context_msg}");

    // Shell-specific eval hint
    let shell = std::env::var("SHELL").unwrap_or_default();
    if shell.contains("fish") {
        eprintln!("Tip: <command> | source");
    } else {
        eprintln!("Tip: eval $(<command>)");
    }
}
