// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run
use seq::seq;

seq!(N in 0..4 {
    compile_error!(concat!("error number ", stringify!(N)));
});

fn main() {}
