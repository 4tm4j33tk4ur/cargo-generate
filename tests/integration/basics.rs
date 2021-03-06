extern crate predicates;

use helpers::project_builder::dir;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;

#[test]
fn it_substitutes_projectname_in_cargo_toml() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        ).init_git()
        .build();

    let dir = dir("main").build();

    Command::main_binary()
        .unwrap()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        dir.read("foobar-project/Cargo.toml")
            .contains("foobar-project")
    );
}

#[test]
fn it_kebabcases_projectname_when_passed_to_flag() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        ).init_git()
        .build();

    let dir = dir("main").build();

    Command::main_binary()
        .unwrap()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar_project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        dir.read("foobar-project/Cargo.toml")
            .contains("foobar-project")
    );
}

#[test]
fn it_substitutes_cratename_in_a_rust_file() {
    let template = dir("template")
        .file(
            "main.rs",
            r#"
extern crate {{crate_name}};          
"#,
        ).init_git()
        .build();

    let dir = dir("main").build();

    Command::main_binary()
        .unwrap()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let file = dir.read("foobar-project/main.rs");
    assert!(file.contains("foobar_project"));
    assert!(!file.contains("foobar-project"));
}

#[test]
fn short_commands_work() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        ).init_git()
        .build();

    let dir = dir("main").build();

    Command::main_binary()
        .unwrap()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        dir.read("foobar-project/Cargo.toml")
            .contains("foobar-project")
    );
}

#[test]
fn it_allows_user_defined_projectname_when_passing_force_flag() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        ).init_git()
        .build();

    let dir = dir("main").build();

    Command::main_binary()
        .unwrap()
        .arg("generate")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar_project")
        .arg("--force")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        dir.read("foobar_project/Cargo.toml")
            .contains("foobar_project")
    );
}

#[test]
fn it_removes_unneeded_files() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        ).file(
            ".genignore",
            r#".genignore
deleteme.sh
*.trash
"#,
        ).file("deleteme.sh", r#"Nothing to see here"#)
        .file("deleteme.trash", r#"This is trash"#)
        .file("notme.sh", r#"I'm here!"#)
        .init_git()
        .build();

    let dir = dir("main").build();

    Command::main_binary()
        .unwrap()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(dir.exists("foobar-project/notme.sh"), true);
    assert_eq!(dir.exists("foobar-project/.genignore"), false);
    assert_eq!(dir.exists("foobar-project/deleteme.sh"), false);
    assert_eq!(dir.exists("foobar-project/deleteme.trash"), false);
}

#[test]
fn it_does_not_remove_files_from_outside_project_dir() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        ).file(
            ".genignore",
            r#".genignore
../dangerous.todelete.cargogeneratetests
"#,
        ).init_git()
        .build();

    let dir = dir("main").build();

    let dangerous_file = template
        .path()
        .join("..")
        .join("dangerous.todelete.cargogeneratetests");

    fs::write(&dangerous_file, "YOU BETTER NOT").expect(&format!(
        "Could not write {}",
        dangerous_file.to_str().expect("Could not read path.")
    ));

    Command::main_binary()
        .unwrap()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        fs::metadata(&dangerous_file)
            .expect("should exist")
            .is_file()
    );
    fs::remove_file(&dangerous_file).expect("failed to clean up test file");
}

#[test]
fn errant_ignore_entry_doesnt_affect_template_files() {
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        ).file(
            ".genignore",
            r#".genignore
../dangerous.todelete.cargogeneratetests
"#,
        ).file("./dangerous.todelete.cargogeneratetests", "IM FINE OK")
        .init_git()
        .build();

    let dir = dir("main").build();

    Command::main_binary()
        .unwrap()
        .arg("gen")
        .arg("--git")
        .arg(template.path())
        .arg("-n")
        .arg("foobar-project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        fs::metadata(
            template
                .path()
                .join("dangerous.todelete.cargogeneratetests")
        ).expect("should exist")
        .is_file()
    );
}

#[test]
fn it_allows_a_git_branch_to_be_specified() {
    // Build and commit on mater
    let template = dir("template")
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project"
version = "0.1.0"
"#,
        ).init_git()
        .branch("baz")
        .build();

    let dir = dir("main").build();

    Command::main_binary()
        .unwrap()
        .arg("generate")
        .arg("--branch")
        .arg("baz")
        .arg("--git")
        .arg(template.path())
        .arg("--name")
        .arg("foobar-project")
        .current_dir(&dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        dir.read("foobar-project/Cargo.toml")
            .contains("foobar-project")
    );
}
