pub struct Item;

#[macro_export]
macro_rules! bar {
    () => {
        Item
    };
}

#[macro_export]
macro_rules! quux2 {
    () => {
        Item
    };
}

pub fn abc() -> Item {
    bar!()
}
