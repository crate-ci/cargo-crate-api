pub mod a {
    pub enum Abc {}
    pub struct Def;
}

pub mod b {
    pub use super::a::Abc;
}

pub use self::a::Def;
