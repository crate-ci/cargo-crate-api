use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Manifest {
    pub name: String,
    pub version: cargo_metadata::Version,
    pub dependencies: Vec<Dependency>,
    pub features: HashMap<String, AnyFeature>,
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
}

impl Dependency {
    fn new(dep: &cargo_metadata::Dependency) -> Self {
        Self {
            name: dep.rename.clone().unwrap_or_else(|| dep.name.clone()),
            version: dep.req.clone(),
        }
    }
}
