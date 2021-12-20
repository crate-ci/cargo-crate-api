pub enum Abc {
    Abc,
}

pub enum Bcd {

}

pub enum Cde {
    Abc,
    Bcd,
}

pub enum Def {
    Abc,

}

pub enum Efg {
    Abc(u8),
    Bcd,
    Cde { f: u8 },
    Def,
    Efg { f: u8 },
    Fgh { f: u16 },
    Ghi { g: u8 },
}

#[non_exhaustive]
pub enum Fgh {
}


pub enum Ghi {
}

#[non_exhaustive]
pub enum Hij {
    Abc,
}
