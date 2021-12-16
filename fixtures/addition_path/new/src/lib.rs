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

pub mod d {}

mod e {}

mod f {
    pub use super::a::Abc;
}
