// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use seq::seq;

seq!(N in 1..4 {
    fn f#N () -> u64 {
        N * 2
    }
});

// This f0 is written separately to detect whether your macro correctly starts
// with the first iteration at N=1 as specified in the invocation. If the macro
// incorrectly started at N=0 like in the previous tests cases, the first
// generated function would conflict with this one and the program would not
// compile.
fn f0() -> u64 {
    100
}

fn main() {
    let sum = f0() + f1() + f2() + f3();

    assert_eq!(sum, 100 + 2 + 4 + 6);
}
