use owo_colors::OwoColorize;

pub fn header(msg: &str) {
    println!("\n{}", msg.bold());
}

pub fn action(msg: &str) {
    println!("{} {}", "==>".bold().cyan(), msg.bold());
}

#[allow(dead_code)]
pub fn step(msg: &str) {
    println!("  {} {}", "→".green(), msg);
}

pub fn success(msg: &str) {
    println!("{} {}", "✔".green(), msg);
}

pub fn bad(msg: &str) {
    eprintln!("{} {}", "✘".red(), msg);
}

pub fn hint(msg: &str) {
    eprintln!("{}: {}", "hint".dimmed(), msg);
}

pub fn warning(msg: &str) {
    eprintln!("{}: {}", "warning".yellow(), msg);
}

pub fn info(msg: &str) {
    println!("{}", msg.dimmed());
}

pub fn error(msg: &str) {
    hint(msg);
}
