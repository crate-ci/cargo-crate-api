#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Api {
    pub root_id: Option<PathId>,
    pub paths: Paths,
    pub items: Items,
    pub crates: Crates,
    pub features: std::collections::BTreeMap<String, AnyFeature>,
}

impl Api {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Paths {
    paths: Vec<(PathId, Path)>,
}

impl Paths {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, path_: Path) -> PathId {
        let id = PathId(self.paths.len());
        self.paths.push((id, path_));
        id
    }

    pub fn get(&self, id: PathId) -> Option<&Path> {
        self.paths.get(id.0).map(|(_i, c)| c)
    }

    pub fn get_mut(&mut self, id: PathId) -> Option<&mut Path> {
        self.paths.get_mut(id.0).map(|(_i, c)| c)
    }

    pub fn iter(&self) -> impl Iterator<Item = (PathId, &Path)> {
        self.paths.iter().map(|(i, c)| (*i, c))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (PathId, &mut Path)> {
        self.paths.iter_mut().map(|(i, c)| (*i, c))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(transparent)]
pub struct PathId(usize);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Path {
    pub crate_id: Option<CrateId>,
    pub path: String,
    pub kind: PathKind,
    pub span: Option<Span>,
    pub item_id: Option<ItemId>,
    pub children: Vec<PathId>,
}

impl Path {
    pub fn new(kind: PathKind, path: impl Into<String>) -> Self {
        Self {
            crate_id: None,
            path: path.into(),
            kind,
            span: None,
            item_id: None,
            children: Vec::new(),
        }
    }
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PathKind {
    Module,
    ExternCrate,
    Import,
    Struct,
    Union,
    Enum,
    Variant,
    Function,
    Typedef,
    OpaqueTy,
    Constant,
    Trait,
    TraitAlias,
    Method,
    Impl,
    Static,
    ForeignType,
    Macro,
    ProcAttribute,
    ProcDerive,
    AssocConst,
    AssocType,
    Primitive,
    Keyword,
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Items {
    items: Vec<(ItemId, Item)>,
}

impl Items {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, item_: Item) -> ItemId {
        let id = ItemId(self.items.len());
        self.items.push((id, item_));
        id
    }

    pub fn get(&self, id: ItemId) -> Option<&Item> {
        self.items.get(id.0).map(|(_i, c)| c)
    }

    pub fn get_mut(&mut self, id: ItemId) -> Option<&mut Item> {
        self.items.get_mut(id.0).map(|(_i, c)| c)
    }

    pub fn iter(&self) -> impl Iterator<Item = (ItemId, &Item)> {
        self.items.iter().map(|(i, c)| (*i, c))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (ItemId, &mut Item)> {
        self.items.iter_mut().map(|(i, c)| (*i, c))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(transparent)]
pub struct ItemId(usize);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Item {
    pub crate_id: Option<CrateId>,
    pub name: Option<String>,
    pub span: Option<Span>,
}

impl Item {
    pub fn new() -> Self {
        Self {
            crate_id: None,
            name: None,
            span: None,
        }
    }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Crates {
    crates: Vec<(CrateId, Crate)>,
}

impl Crates {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, crate_: Crate) -> CrateId {
        let id = CrateId(self.crates.len());
        self.crates.push((id, crate_));
        id
    }

    pub fn get(&self, id: CrateId) -> Option<&Crate> {
        self.crates.get(id.0).map(|(_i, c)| c)
    }

    pub fn get_mut(&mut self, id: CrateId) -> Option<&mut Crate> {
        self.crates.get_mut(id.0).map(|(_i, c)| c)
    }

    pub fn iter(&self) -> impl Iterator<Item = (CrateId, &Crate)> {
        self.crates.iter().map(|(i, c)| (*i, c))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (CrateId, &mut Crate)> {
        self.crates.iter_mut().map(|(i, c)| (*i, c))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(transparent)]
pub struct CrateId(usize);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Crate {
    pub name: String,
    pub version: Option<cargo_metadata::VersionReq>,
}

impl Crate {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Span {
    /// The relative path to the source file for this span
    pub filename: std::path::PathBuf,
    /// Zero indexed Line and Column of the first character of the `Span`
    pub begin: (usize, usize),
    /// Zero indexed Line and Column of the last character of the `Span`
    pub end: (usize, usize),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "kind")]
pub enum AnyFeature {
    Feature(Feature),
    OptionalDependency(OptionalDependency),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Feature {
    pub name: String,
    pub dependencies: Vec<String>,
}

impl Feature {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dependencies: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct OptionalDependency {
    /// The name used to activate this dependency
    pub name: String,
    /// If renamed, the actual dependency name
    pub package: Option<String>,
}

impl OptionalDependency {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            package: None,
        }
    }
}
