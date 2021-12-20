#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Raw {
    pub manifest_path: std::path::PathBuf,
    pub rustdoc: Option<rustdoc_json_types_fork::Crate>,
    pub manifest: Option<crate_api::manifest::Manifest>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Api {
    pub manifest_path: std::path::PathBuf,
    pub api: crate_api::Api,
}
