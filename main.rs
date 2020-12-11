// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run
use sorted::sorted;

#[sorted]
pub enum Error {
    ThatFailed,
    ThisFailed,
    SomethingFailed,
    WhoKnowsWhatFailed,
}

fn main() {}
