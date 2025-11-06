use std::io::{self, Write};

pub fn pause_before_exit() {
    println!("按回车键继续...");
    let _ = io::stdout().flush();
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);
}