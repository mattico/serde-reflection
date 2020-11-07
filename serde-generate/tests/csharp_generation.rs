// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{csharp, test_utils, CodeGeneratorConfig, Encoding};
use std::collections::BTreeMap;
use std::process::Command;
use tempfile::{tempdir, TempDir};

fn test_that_csharp_code_compiles_with_config(
    config: &CodeGeneratorConfig,
) -> (TempDir, std::path::PathBuf) {
    use serde_generate::SourceInstaller;

    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    let installer = csharp::Installer::new(dir.path().to_path_buf());
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_lcs_runtime().unwrap();

    std::fs::copy("runtime/csharp/Serde.sln", dir.path().join("Serde.sln")).unwrap();

    let lcs_test_dir = dir.path().join("Serde.Lcs.Tests");
    std::fs::create_dir(&lcs_test_dir).unwrap();
    std::fs::copy("runtime/csharp/Serde.Lcs.Tests/Serde.Lcs.Tests.csproj", 
        &lcs_test_dir.join("Serde.Lcs.Tests.csproj")).unwrap();

    let generate_dir = dir.path().join("Serde.GenerationTest");
    std::fs::create_dir(&generate_dir).unwrap();
    std::fs::copy("runtime/csharp/Serde.GenerationTest/Serde.GenerationTest.csproj", 
        &generate_dir.join("Serde.GenerationTest.csproj")).unwrap();

    let status = Command::new("dotnet")
        .arg("build")
        .current_dir(dir.path())
        .status()
        .unwrap();
    assert!(status.success());

    let path = dir.path().join("testing");
    (dir, path)
}

#[test]
fn test_that_csharp_code_compiles() {
    let config = CodeGeneratorConfig::new("Serde.GenerationTest".to_string());
    test_that_csharp_code_compiles_with_config(&config);
}

#[test]
fn test_that_csharp_code_compiles_without_serialization() {
    let config = CodeGeneratorConfig::new("Serde.GenerationTest".to_string()).with_serialization(false);
    test_that_csharp_code_compiles_with_config(&config);
}

#[test]
fn test_that_csharp_code_compiles_with_lcs() {
    let config =
        CodeGeneratorConfig::new("Serde.GenerationTest".to_string()).with_encodings(vec![Encoding::Lcs]);
    test_that_csharp_code_compiles_with_config(&config);
}

#[test]
fn test_that_csharp_code_compiles_with_bincode() {
    let config =
        CodeGeneratorConfig::new("Serde.GenerationTest".to_string()).with_encodings(vec![Encoding::Bincode]);
    test_that_csharp_code_compiles_with_config(&config);
}

#[test]
fn test_that_csharp_code_compiles_with_comments() {
    let comments = vec![(
        vec!["testing".to_string(), "SerdeData".to_string()],
        "Some\ncomments".to_string(),
    )]
    .into_iter()
    .collect();
    let config = CodeGeneratorConfig::new("Serde.GenerationTest".to_string()).with_comments(comments);

    let (_dir, path) = test_that_csharp_code_compiles_with_config(&config);

    // Comment was correctly generated.
    let content = std::fs::read_to_string(path.join("SerdeData.cs")).unwrap();
    assert!(content.contains(
        r#"
/**
 * Some
 * comments
 */
"#
    ));
}

#[test]
fn test_csharp_code_with_external_definitions() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    // (wrongly) Declare TraitHelpers as external.
    let mut definitions = BTreeMap::new();
    definitions.insert("foo".to_string(), vec!["TraitHelpers".to_string()]);
    let config =
        CodeGeneratorConfig::new("Serde.GenerationTest".to_string()).with_external_definitions(definitions);
    let generator = csharp::CodeGenerator::new(&config);

    generator
        .write_source_files(dir.path().to_path_buf(), &registry)
        .unwrap();

    // References were updated.
    let content = std::fs::read_to_string(dir.path().join("testing/SerdeData.cs")).unwrap();
    assert!(content.contains("foo.TraitHelpers."));
}

#[test]
fn test_that_csharp_code_compiles_with_custom_code() {
    let comments = vec![(
        vec!["testing".to_string(), "SerdeData".to_string()],
        "SerdeData me() {{ return this; }}".to_string(),
    )]
    .into_iter()
    .collect();
    let config = CodeGeneratorConfig::new("Serde.GenerationTest".to_string()).with_comments(comments);

    let (_dir, path) = test_that_csharp_code_compiles_with_config(&config);

    // Comment was correctly generated.
    let content = std::fs::read_to_string(path.join("SerdeData.cs")).unwrap();
    assert!(content.contains("me()"));
}
