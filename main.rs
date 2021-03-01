// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run


use bitfield::*;

//#[bitfield]
pub struct MyFourBytes {
    a: B1,
    b: B3,
    c: B4,
    d: B24,
}

fn main() {
    assert_eq!(<B24 as Specifier>::BITS, 24);
}