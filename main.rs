// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use seq::seq;

seq!(N in 0..16 {
    #[derive(Copy, Clone, PartialEq, Debug)]
    enum Interrupt {
        #(
            Irq#N,
        )*
    }
});

fn main() {
    let interrupt = Interrupt::Irq8;

    assert_eq!(interrupt as u8, 8);
    assert_eq!(interrupt, Interrupt::Irq8);
}