#![allow(dead_code)]
pub struct Abc<B> {
    field: B,
}

pub struct Def<A=u8> {
    pub field: A,
}

pub struct Def2<A=u16> {
    pub field: A,
}

pub struct Efg {
    pub field: u16,
}

pub struct Fgh {
    pub field: u8,
}

pub struct Ghi {
    field: u8,
}

pub struct Hij {
    field: u8,
}

pub struct Ijk {
    pub field1: u8,
    pub field2: u8,
}

pub struct Jkl {
    field: u8
}


pub struct Klm {
}

#[non_exhaustive]
pub struct Lmn {
}

#[non_exhaustive]
pub struct Mno {
    field: u8
}
