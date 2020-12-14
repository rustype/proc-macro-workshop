// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use seq::seq;

macro_rules! expand_to_nothing {
    ($arg:literal) => {
        // nothing
    };
}

seq!(N in 0..4 {
    expand_to_nothing!(N);
});

fn main() {}
