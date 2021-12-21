fn main() {
    let action = std::env::var("DIFF_API");
    let action = action.as_deref().unwrap_or("verify");
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
            let name = input_path.file_name().unwrap().to_str().unwrap().to_owned();
            let expected = input_path.join("diff.json");
            let is_ignored = if input_path.join("new/Cargo.toml").exists()
                && input_path.join("new/Cargo.toml").exists()
            {
                action == Action::Ignore
            } else {
                true
            };
            fs_snapshot::Test {
                name,
                kind: "".into(),
                is_ignored,
                is_bench: false,
                data: fs_snapshot::Case {
                    fixture: input_path,
                    expected,
                },
            }
        },
        move |input_path| {
            let before_path = input_path.join("old/rustdoc-api.json");
            let before_raw = std::fs::read_to_string(&before_path).map_err(|e| e.to_string())?;
            let before: crate_api::Api =
                serde_json::from_str(&before_raw).map_err(|e| e.to_string())?;

            let after_path = input_path.join("new/rustdoc-api.json");
            let after_raw = std::fs::read_to_string(&after_path).map_err(|e| e.to_string())?;
            let after: crate_api::Api =
                serde_json::from_str(&after_raw).map_err(|e| e.to_string())?;

            let mut actual = Vec::new();
            crate_api::diff::diff(&before, &after, &mut actual);

            let actual = serde_json::to_string_pretty(&actual).map_err(|e| e.to_string())?;
            Ok(actual)
        },
    )
    .select(["/*/"])
    .overwrite(action == Action::Overwrite)
    .test()
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Action {
    Overwrite,
    Verify,
    Ignore,
}
