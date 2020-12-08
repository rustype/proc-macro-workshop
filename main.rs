// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use derive_debug::CustomDebug;

#[derive(CustomDebug)]
pub struct Field<T> {
    value: T,
    #[debug = "0b{:08b}"]
    bitmask: u8,
}

fn main() {
    let f = Field {
        value: "F",
        bitmask: 0b00011100,
    };

    let debug = format!("{:?}", f);
    let expected = r#"Field { value: "F", bitmask: 0b00011100 }"#;

    assert_eq!(debug, expected);
}
