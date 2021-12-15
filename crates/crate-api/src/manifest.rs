use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    name: String,
    version: cargo_metadata::Version,
    features: HashMap<String, AnyFeature>,
}

impl<'p> From<&'p cargo_metadata::Package> for Manifest {
    fn from(pkg: &'p cargo_metadata::Package) -> Self {
        let mut features: HashMap<_, _> = pkg
            .features
            .iter()
            .map(|(k, v)| (k.to_owned(), AnyFeature::Feature(Feature::new(k, v))))
            .collect();
        for dep in &pkg.dependencies {
            if let Some(feature) = Dependency::try_new(dep) {
                features
                    .entry(dep.name.clone())
                    .or_insert(AnyFeature::Dependency(feature));
            }
        }

        Self {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
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
pub struct Feature {
    name: String,
    dependencies: Vec<String>,
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
pub struct Dependency {
    name: String,
    version: cargo_metadata::VersionReq,
}

impl Dependency {
    fn try_new(dep: &cargo_metadata::Dependency) -> Option<Self> {
        dep.optional.then(|| Self {
            name: dep.name.clone(),
            version: dep.req.clone(),
        })
    }
}
