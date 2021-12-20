pub use dep_upgrade_dep::Used;

pub fn in_func(_var: dep_upgrade_dep::InFunc) {}

pub struct ConvertTo;

impl From<dep_upgrade_dep::ConvertFrom> for ConvertTo {
    fn from(_other: dep_upgrade_dep::ConvertFrom) -> ConvertTo {
        ConvertTo
    }
}
