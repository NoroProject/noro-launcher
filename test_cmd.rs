use std::process::Command;

fn main() {
    let result = Command::new("./test.sh")
        .current_dir("/tmp")
        .output();
    println!("{:?}", result);
}
