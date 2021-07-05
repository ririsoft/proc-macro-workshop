// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use derive_builder::Builder;

#[derive(Builder)]
pub struct Command {
    _executable: String,
    _args: Vec<String>,
    _env: Vec<String>,
    _current_dir: String,
}

fn main() {
    let _cmd = Command::builder().build().unwrap();
}
