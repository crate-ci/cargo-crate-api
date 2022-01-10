use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Id {
    pub name: &'static str,
    pub explanation: &'static str,
    pub category: Category,
    pub default_severity: Severity,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Diff {
    pub severity: Severity,
    pub id: Id,
    pub before: Option<Location>,
    pub after: Option<Location>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Unknown,
    Added,
    Removed,
    Changed,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Allow,
    Report,
    Warn,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Location {
    pub crate_id: Option<crate::CrateId>,
    pub path_id: Option<crate::PathId>,
    pub item_id: Option<crate::ItemId>,
}

pub fn diff(before: &crate::Api, after: &crate::Api, changes: &mut Vec<Diff>) {
    public_dependencies(before, after, changes);
}

pub const ALL_IDS: &[Id] = &[
    DEPENDENCY_REMOVED,
    DEPENDENCY_ADDED,
    DEPENDENCY_AMBIGUOUS,
    DEPENDENCY_REQUIREMENT,
];

pub const DEPENDENCY_REMOVED: Id = Id {
    name: "dependency-removed",
    explanation: "Public dependency removed because of an API change",
    category: Category::Removed,
    // This is a side effect of an API change but not an API change in of itself
    default_severity: Severity::Allow,
};

pub const DEPENDENCY_ADDED: Id = Id {
    name: "dependency-added",
    explanation: "Public dependency removed because of an API change",
    category: Category::Added,
    // In case people weren't aware they added a dependency to their public API
    default_severity: Severity::Report,
};

pub const DEPENDENCY_AMBIGUOUS: Id = Id {
    name: "dependency-ambiguous",
    explanation: "Could not determine the dependency version to check it",
    category: Category::Unknown,
    // In case people weren't aware they added a dependency to their public API
    default_severity: Severity::Allow,
};

pub const DEPENDENCY_REQUIREMENT: Id = Id {
    name: "dependency-requirement",
    explanation: "Changing the major version requirements breaks compatibility",
    category: Category::Changed,
    default_severity: Severity::Warn,
};

pub fn public_dependencies(before: &crate::Api, after: &crate::Api, changes: &mut Vec<Diff>) {
    let before_by_name: HashMap<_, _> = before
        .crates
        .iter()
        .map(|(id, crate_)| (crate_.name.as_str(), id))
        .collect();
    let after_by_name: HashMap<_, _> = after
        .crates
        .iter()
        .map(|(id, crate_)| (crate_.name.as_str(), id))
        .collect();

    let before_names: HashSet<_> = before_by_name.keys().collect();
    let after_names: HashSet<_> = after_by_name.keys().collect();

    for removed_name in before_names.difference(&after_names) {
        let before_crate_id = *before_by_name.get(*removed_name).unwrap();
        changes.push(Diff {
            severity: DEPENDENCY_REMOVED.default_severity,
            id: DEPENDENCY_REMOVED,
            before: Some(Location {
                crate_id: Some(before_crate_id),
                ..Default::default()
            }),
            after: None,
        });
    }

    for added_name in after_names.difference(&before_names) {
        let after_crate_id = *after_by_name.get(*added_name).unwrap();
        changes.push(Diff {
            severity: DEPENDENCY_ADDED.default_severity,
            id: DEPENDENCY_ADDED,
            before: None,
            after: Some(Location {
                crate_id: Some(after_crate_id),
                ..Default::default()
            }),
        });
    }

    for common_name in after_names.intersection(&before_names) {
        let before_crate_id = *before_by_name.get(*common_name).unwrap();
        let before_crate = before.crates.get(before_crate_id).unwrap();
        let after_crate_id = *after_by_name.get(*common_name).unwrap();
        let after_crate = after.crates.get(after_crate_id).unwrap();

        if before_crate.version.is_none() || after_crate.version.is_none() {
            changes.push(Diff {
                severity: DEPENDENCY_AMBIGUOUS.default_severity,
                id: DEPENDENCY_AMBIGUOUS,
                before: Some(Location {
                    crate_id: Some(before_crate_id),
                    ..Default::default()
                }),
                after: Some(Location {
                    crate_id: Some(after_crate_id),
                    ..Default::default()
                }),
            });
        } else if before_crate.version == after_crate.version {
        } else {
            let (before_lower, before_upper) = breaking(before_crate.version.as_ref().unwrap());
            let before_lower = before_lower.unwrap_or((0, 0, 0));
            let before_upper = before_upper.unwrap_or((u64::MAX, u64::MAX, u64::MAX));

            let (after_lower, after_upper) = breaking(after_crate.version.as_ref().unwrap());
            let after_lower = after_lower.unwrap_or((0, 0, 0));
            let after_upper = after_upper.unwrap_or((u64::MAX, u64::MAX, u64::MAX));

            if before_lower < after_lower || after_upper < before_upper {
                changes.push(Diff {
                    severity: DEPENDENCY_REQUIREMENT.default_severity,
                    id: DEPENDENCY_REQUIREMENT,
                    before: Some(Location {
                        crate_id: Some(before_crate_id),
                        ..Default::default()
                    }),
                    after: Some(Location {
                        crate_id: Some(after_crate_id),
                        ..Default::default()
                    }),
                });
            }
        }
    }
}

fn breaking(version: &semver::VersionReq) -> VersionRange {
    if *version == semver::VersionReq::STAR {
        return (None, None);
    }

    let mut lower = None;
    let mut upper = None;
    for comparator in &version.comparators {
        let (current_lower, current_upper) = breaking_comparator(comparator);
        if let Some(current_lower) = current_lower {
            lower.get_or_insert(current_lower);
            lower = Some(lower.unwrap().max(current_lower));
        }
        if let Some(current_upper) = current_upper {
            upper.get_or_insert(current_upper);
            upper = Some(upper.unwrap().min(current_upper));
        }
    }

    (lower, upper)
}

type VersionParts = (u64, u64, u64);

type VersionRange = (Option<VersionParts>, Option<VersionParts>);

fn breaking_comparator(comparator: &semver::Comparator) -> VersionRange {
    match comparator.op {
        semver::Op::Exact => {
            if let Some(major) = exact_break(comparator) {
                (Some(major), Some(major))
            } else {
                (None, None)
            }
        }
        semver::Op::Greater => {
            let major = if 1 <= comparator.major {
                let major = comparator.major;
                let major = if comparator.minor.is_none() {
                    major + 1
                } else {
                    major
                };
                (major, 0, 0)
            } else if comparator.minor.is_none() {
                return (None, None);
            } else if 1 <= comparator.minor.unwrap() {
                let major = comparator.minor.unwrap();
                let major = if comparator.patch.is_none() {
                    major + 1
                } else {
                    major
                };
                (0, major, 0)
            } else if comparator.patch.is_none() {
                return (None, None);
            } else {
                let major = comparator.patch.unwrap() + 1;
                (0, 0, major)
            };
            (Some(major), None)
        }
        semver::Op::GreaterEq => {
            if let Some(major) = exact_break(comparator) {
                (Some(major), None)
            } else {
                (None, None)
            }
        }
        semver::Op::Less => {
            let major = if 1 <= comparator.major {
                let major = comparator.major;
                let major = if comparator.minor.is_none() {
                    major - 1
                } else {
                    major
                };
                (major, 0, 0)
            } else if comparator.minor.is_none() {
                return (None, None);
            } else if 1 <= comparator.minor.unwrap() {
                let major = comparator.minor.unwrap();
                let major = if comparator.patch.is_none() {
                    major - 1
                } else {
                    major
                };
                (0, major, 0)
            } else if comparator.patch.is_none() {
                return (None, None);
            } else {
                let major = comparator.patch.unwrap();
                let major = if major == 0 { major } else { major - 1 };
                (0, 0, major)
            };
            (None, Some(major))
        }
        semver::Op::LessEq => {
            if let Some(major) = exact_break(comparator) {
                (None, Some(major))
            } else {
                (None, None)
            }
        }
        semver::Op::Tilde => {
            if let Some(major) = exact_break(comparator) {
                (Some(major), Some(major))
            } else {
                (None, None)
            }
        }
        semver::Op::Caret => {
            if let Some(major) = exact_break(comparator) {
                (Some(major), Some(major))
            } else {
                (None, None)
            }
        }
        semver::Op::Wildcard => {
            if let Some(major) = exact_break(comparator) {
                (Some(major), Some(major))
            } else {
                (None, None)
            }
        }
        _ => (None, None),
    }
}

fn exact_break(comparator: &semver::Comparator) -> Option<(u64, u64, u64)> {
    if 1 <= comparator.major {
        let major = comparator.major;
        Some((major, 0, 0))
    } else if comparator.minor.is_none() {
        None
    } else if 1 <= comparator.minor.unwrap() {
        let major = comparator.minor.unwrap();
        Some((0, major, 0))
    } else if comparator.patch.is_none() {
        None
    } else {
        let major = comparator.patch.unwrap();
        Some((0, 0, major))
    }
}
