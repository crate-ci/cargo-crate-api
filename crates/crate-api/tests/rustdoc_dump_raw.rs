fn main() {
    let action = std::env::var("RUSTDOC_DUMP_RAW");
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
        move |input| {
            let age_dir = input.parent().unwrap();
            let case_dir = age_dir.parent().unwrap();
            let name = format!(
                "{}_{}",
                case_dir.file_name().unwrap().to_str().unwrap(),
                age_dir.file_name().unwrap().to_str().unwrap()
            );
            let expected = age_dir.join("rustdoc-raw.json");
            fs_snapshot::Test {
                name,
                kind: "".into(),
                is_ignored: action == Action::Ignore,
                is_bench: false,
                data: fs_snapshot::Case {
                    fixture: input,
                    expected,
                },
            }
        },
        move |input| {
            let target_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
            let actual = crate_api::RustDocBuilder::new()
                .target_directory(target_dir.path())
                .dump_raw(input)
                .map_err(|e| e.to_string())?;
            target_dir.close().map_err(|e| e.to_string())?;
            let actual: serde_json::Value =
                serde_json::from_str(&actual).map_err(|e| e.to_string())?;
            let actual = serde_json::to_string_pretty(&actual).map_err(|e| e.to_string())?;
            Ok(actual)
        },
    )
    .select(["Cargo.toml"])
    .overwrite(action == Action::Overwrite)
    .test()
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Action {
    Overwrite,
    Verify,
    Ignore,
}
