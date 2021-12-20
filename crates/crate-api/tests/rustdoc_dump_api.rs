fn main() {
    let action = std::env::var("RUSTDOC_DUMP_API");
    let action = action.as_deref().unwrap_or("ignore");
    let action = match action {
        "overwrite" => Action::Overwrite,
        "ignore" => Action::Ignore,
        "verify" => Action::Verify,
        _ => panic!(
            "Unrecognized action {}, expected `overwrite`, `ignore`, or `verify`",
            action
        ),
    };

    fs_snapshot::Harness::new(
        "../../fixtures",
        move |input_path| {
            let age_dir = input_path.parent().unwrap();
            let case_dir = age_dir.parent().unwrap();
            let name = format!(
                "{}_{}",
                case_dir.file_name().unwrap().to_str().unwrap(),
                age_dir.file_name().unwrap().to_str().unwrap()
            );
            let expected = age_dir.join("rustdoc-api.json");
            fs_snapshot::Test {
                name,
                kind: "".into(),
                is_ignored: action == Action::Ignore,
                is_bench: false,
                data: fs_snapshot::Case {
                    fixture: input_path,
                    expected,
                },
            }
        },
        move |input_path| {
            let input = std::fs::read_to_string(&input_path).map_err(|e| e.to_string())?;
            let mut actual =
                crate_api::rustdoc::parse_raw(&input, input_path).map_err(|e| e.to_string())?;

            let manifest_path = input_path.parent().unwrap().join("Cargo.toml");
            let metadata = cargo_metadata::MetadataCommand::new()
                .manifest_path(&manifest_path)
                .exec()
                .map_err(|e| e.to_string())?;
            let root_id = metadata.resolve.unwrap().root.unwrap();
            let pkg = metadata.packages.iter().find(|p| p.id == root_id).unwrap();
            crate_api::manifest::Manifest::from(pkg).into_api(&mut actual);

            let actual = serde_json::to_string_pretty(&actual).map_err(|e| e.to_string())?;
            Ok(actual)
        },
    )
    .select(["rustdoc-raw.json"])
    .overwrite(action == Action::Overwrite)
    .test()
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Action {
    Overwrite,
    Verify,
    Ignore,
}
