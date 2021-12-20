use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Manifest {
    pub name: String,
    pub version: cargo_metadata::Version,
    pub dependencies: Vec<Dependency>,
    pub features: HashMap<String, AnyFeature>,
}

impl Manifest {
    pub fn into_api(self, api: &mut crate::Api) {
        let mut crate_ids = HashMap::new();
        for (id, crate_) in api.crates.iter() {
            crate_ids
                .entry(crate_.name.clone())
                .or_insert_with(|| Vec::new())
                .push(id);
        }
        for dependency in self.dependencies {
            match crate_ids.get(&dependency.name) {
                Some(crate_ids) => match crate_ids.len() {
                    0 => unreachable!("Vec should only have 1+ entries"),
                    1 => {
                        api.crates.get_mut(crate_ids[0]).unwrap().version =
                            Some(dependency.version);
                    }
                    // Can't figure out which to map it to, so ignore it
                    _ => {}
                },
                // Not in the public API, can ignore
                None => {}
            }
        }
    }
}

impl<'p> From<&'p cargo_metadata::Package> for Manifest {
    fn from(pkg: &'p cargo_metadata::Package) -> Self {
        let mut features: HashMap<_, _> = pkg
            .features
            .iter()
            .map(|(k, v)| (k.to_owned(), AnyFeature::Feature(Feature::new(k, v))))
            .collect();
        let mut dependencies = Vec::new();
        for dep in &pkg.dependencies {
            let dependency = Dependency::new(dep);
            if dep.optional {
                features
                    .entry(dependency.name.clone())
                    .or_insert(AnyFeature::Dependency(dependency.clone()));
            }
            dependencies.push(dependency);
        }

        Self {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            dependencies,
            features,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnyFeature {
    Feature(Feature),
    Dependency(Dependency),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Feature {
    pub name: String,
    pub dependencies: Vec<String>,
}

impl Feature {
    fn new(name: &str, deps: &[String]) -> Self {
        Self {
            name: name.to_owned(),
            dependencies: deps.to_vec(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Dependency {
    pub name: String,
    pub version: cargo_metadata::VersionReq,
    pub rename: Option<String>,
}

impl Dependency {
    fn new(dep: &cargo_metadata::Dependency) -> Self {
        Self {
            name: dep.name.clone(),
            version: dep.req.clone(),
            rename: dep.rename.clone(),
        }
    }
}
