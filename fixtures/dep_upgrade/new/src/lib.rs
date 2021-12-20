pub use dep::Used;

pub fn in_func(_var: dep::InFunc) {}

pub struct ConvertTo;

impl From<dep::ConvertFrom> for ConvertTo {
    fn from(_other: dep::ConvertFrom) -> ConvertTo {
        ConvertTo
    }
}
