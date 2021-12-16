pub struct Item;

#[macro_export]
macro_rules! baz {
    () => {
        Item
    };
}

#[macro_export]
macro_rules! qux2 {
    () => {
        Item
    };
}

pub fn abc() -> Item {
    baz!()
}
