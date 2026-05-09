pub fn input(prompt: &str) -> String {
    println!("{prompt}");
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).expect("read from stdin");
    s.trim().to_string()
}
