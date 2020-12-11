// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use sorted::sorted;

#[sorted]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

enum ErrorKind {
    Io,
    Syntax,
    Eof,
}

fn main() {}
