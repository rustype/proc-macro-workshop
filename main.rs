// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use sorted::sorted;

#[sorted]
pub enum Conference {
    RustBeltRust,
    RustConf,
    RustFest,
    RustLatam,
    RustRush,
}

impl Conference {
    #[sorted::check]
    pub fn region(&self) -> &str {
        use self::Conference::*;

        #[sorted]
        match self {
            RustFest => "Europe",
            RustLatam => "Latin America",
            _ => "elsewhere",
        }
    }
}

fn main() {}
