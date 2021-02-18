// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run






use derive_builder::Builder;
use derive_debug::CustomDebug;

#[derive(CustomDebug)]
pub struct Field<T> {
    value: T,
    #[debug = "0b{:08b}"]
    bitmask: u8,
}



fn main() {}
