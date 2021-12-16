pub mod a {
    pub struct Abc;
}

pub mod b {
    pub use super::a::*;
}

pub mod c {
    pub use super::a::Abc;
}

pub use self::a::Abc;

mod d {
    pub use super::a::Abc;
}
