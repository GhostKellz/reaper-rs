use colored::*;

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "error:".red().bold(), msg.red());
}

pub fn print_success(msg: &str) {
    println!("{} {}", "success:".green().bold(), msg.green());
}

pub fn print_info(msg: &str) {
    println!("{} {}", "info:".cyan().bold(), msg.cyan());
}
