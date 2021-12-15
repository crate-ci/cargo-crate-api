#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Api {
    pub crates: Crates,
}

impl Api {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
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
#[repr(transparent)]
pub struct CrateId(usize);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Crate {
    pub name: String,
}

impl Crate {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
