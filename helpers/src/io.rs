use std::io::Write;

pub fn input(prompt: &str) -> String {
    print!("{prompt}");
    std::io::stdout().flush().expect("flushed stdout");
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).expect("read from stdin");
    s.trim().to_string()
}
