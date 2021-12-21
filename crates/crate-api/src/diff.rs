#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Diff {
    pub category: Category,
    pub severity: Option<Severity>,
    pub id: &'static str,
    pub explanation: &'static str,
    pub before: Option<Location>,
    pub after: Option<Location>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Added,
    Removed,
    Changed,
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Forbid,
    Warn,
    Report,
    Allow,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Location {
    pub crate_id: Option<crate::CrateId>,
    pub path_id: Option<crate::PathId>,
    pub item_id: Option<crate::ItemId>,
}

pub fn diff(_before: &crate::Api, _after: &crate::Api, _changes: &mut Vec<Diff>) {}
