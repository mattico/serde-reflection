// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(feature = "runtime-testing")]

use heck::CamelCase;
use serde_generate::{
    csharp, test_utils,
    test_utils::{Choice, Runtime, Test},
    CodeGeneratorConfig, SourceInstaller,
};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Set to `false` to have tests generate into non-temporary directories for inspection.
/// NOTE: Set this to `true` before committing or merging!
const TEST_USE_TEMP_DIRS: bool = true;

/// Returns:
/// 1. A `PathBuf` to the directory to write test data into
/// 2. Optionally, a `tempfile::TempDir` which deletes the directory when it goes out of scope
fn create_test_dir(test_name: &'static str) -> (std::path::PathBuf, Option<tempfile::TempDir>) {
    if TEST_USE_TEMP_DIRS {
        let tempdir = tempfile::Builder::new()
            .suffix(&format!("_{}", test_name))
            .tempdir()
            .unwrap();
        (tempdir.path().to_path_buf(), Some(tempdir))
    } else {
        let mut tries = 0;
        while tries < 20 {
            let test_dir_name = if tries == 0 {
                test_name.into()
            } else {
                format!("{}_{}", test_name, tries)
            };
            let dir = std::path::Path::new("tests").join(test_dir_name).to_path_buf();
            match std::fs::create_dir(&dir) {
                Ok(()) => return (dir, None),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => tries += 1,
                Err(e) => panic!("Error creating test directory: {:?}", e),
            }
        }
        panic!("Error creating test directory: Too many existing test directories");
    }
}

fn dotnet_build(proj_dir: &Path) {
    let status = Command::new("dotnet")
        .arg("build")
        .current_dir(proj_dir)
        .status()
        .unwrap();
    assert!(status.success());
}

// TODO: switch to NUnit for better xplat
fn run_mstest(proj_dir: &Path) {
    let dll_path = proj_dir.join("bin/Debug/netcoreapp3.1/Serde.Tests.dll");
    let status = Command::new("C:/Program Files (x86)/Microsoft Visual Studio/2019/Preview/Common7/IDE/CommonExtensions/Microsoft/TestWindow/vstest.console.exe")
        .arg(&dll_path)
        .status().unwrap();
    assert!(status.success());
}

fn copy_test_project(root_dir: &Path, copy_runtime_tests: bool) -> PathBuf {
    let test_dir = root_dir.join("Serde.Tests").to_path_buf();

    std::fs::create_dir(&test_dir).unwrap();
    std::fs::copy(
        "runtime/csharp/Serde.Tests/Serde.Tests.csproj",
        &test_dir.join("Serde.Tests.csproj"),
    )
    .unwrap();

    if copy_runtime_tests {
        std::fs::copy(
            "runtime/csharp/Serde.Tests/TestLcs.cs",
            &test_dir.join("TestLcs.cs"),
        )
        .unwrap();
    }

    test_dir
}

#[test]
fn test_csharp_lcs_runtime_tests() {
    let (dir, _tmp) = create_test_dir("test_csharp_lcs_runtime_tests");
    let test_dir = copy_test_project(&dir, true);

    let installer = csharp::Installer::new(dir);
    installer.install_serde_runtime().unwrap();
    installer.install_lcs_runtime().unwrap();

    dotnet_build(&test_dir);
    run_mstest(&test_dir);
}

#[test]
fn test_csharp_lcs_runtime_on_simple_data() {
    let (dir, _tmp) = create_test_dir("test_csharp_runtime_on_simple_data");
    test_csharp_runtime_on_simple_data(dir, Runtime::Lcs);
}

#[test]
fn test_csharp_bincode_runtime_on_simple_data() {
    let (dir, _tmp) = create_test_dir("test_csharp_bincode_runtime_on_simple_data");
    test_csharp_runtime_on_simple_data(dir, Runtime::Bincode);
}

fn test_csharp_runtime_on_simple_data(dir: PathBuf, runtime: Runtime) {
    let test_dir = copy_test_project(&dir, false);

    let registry = test_utils::get_simple_registry().unwrap();

    let installer = csharp::Installer::new(dir.clone());
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_lcs_runtime().unwrap();

    // Generates code into `Serde/Tests`
    let config =
        CodeGeneratorConfig::new("Serde.Tests".to_string()).with_encodings(vec![runtime.into()]);
    let generator = csharp::CodeGenerator::new(&config);
    generator
        .write_source_files(dir, &registry)
        .unwrap();

    let reference = runtime.serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    });

    let mut source = File::create(&test_dir.join("TestRuntime.cs")).unwrap();
    writeln!(
        source,
        r#"
using System;
using System.Collections.Generic;
using System.IO;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace Serde.Tests {{
    [TestClass]
    public class TestRuntime {{
        [TestMethod]
        public void TestRoundTrip() {{
            byte[] input = new byte[] {{{0}}};

            Test test = Test.{1}Deserialize(new MemoryStream(input));

            List<uint> a = new List<uint>(new uint[] {{ 4, 6 }});
            var b = ((long)-3, (ulong)5);
            Choice c = new Choice.C((byte) 7);
            Test test2 = new Test(a, b, c);

            Assert.AreEqual(test, test2);

            byte[] output = test2.{1}Serialize();

            CollectionAssert.AreEqual(input, output);

            byte[] input2 = new byte[] {{{0}, 1}};
            Assert.ThrowsException<DeserializationException>(() => Test.{1}Deserialize(new MemoryStream(input2)));
        }}
    }}
}}
"#,
        reference
            .iter()
            .map(|x| format!("{}", *x as u8))
            .collect::<Vec<_>>()
            .join(", "),
        runtime.name().to_camel_case(),
    )
    .unwrap();

    dotnet_build(&test_dir);
    run_mstest(&test_dir);
}

#[test]
fn test_csharp_lcs_runtime_on_supported_types() {
    let (dir, _tmp) = create_test_dir("test_csharp_lcs_runtime_on_supported_types");
    test_csharp_runtime_on_supported_types(dir, Runtime::Lcs);
}

#[test]
fn test_csharp_bincode_runtime_on_supported_types() {
    let (dir, _tmp) = create_test_dir("test_csharp_bincode_runtime_on_supported_types");
    test_csharp_runtime_on_supported_types(dir, Runtime::Bincode);
}

fn quote_bytes(bytes: &[u8]) -> String {
    format!(
        "{{{}}}",
        bytes
            .iter()
            .map(|x| format!("{}", *x as u8))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn test_csharp_runtime_on_supported_types(dir: PathBuf, runtime: Runtime) {
    let test_dir = copy_test_project(&dir, false);

    let registry = test_utils::get_registry().unwrap();

    let installer = csharp::Installer::new(dir.clone());
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_lcs_runtime().unwrap();

    // Generates code into `Serde/Tests`
    let config =
        CodeGeneratorConfig::new("Serde.Tests".to_string()).with_encodings(vec![runtime.into()]);
    let generator = csharp::CodeGenerator::new(&config);
    generator
        .write_source_files(dir, &registry)
        .unwrap();

    let mut positive_encodings = runtime
        .get_positive_samples_quick()
        .iter()
        .map(|bytes| quote_bytes(bytes))
        .collect::<Vec<_>>()
        .join(",\n\t\t\tnew byte[] ");
    if positive_encodings.len() > 0 {
        positive_encodings = format!("\n\t\t\tnew byte[] {}", positive_encodings);
    }

    let mut negative_encodings = runtime
        .get_negative_samples()
        .iter()
        .map(|bytes| quote_bytes(bytes))
        .collect::<Vec<_>>()
        .join(",\n\t\t\tnew byte[] ");
    if negative_encodings.len() > 0 {
        negative_encodings = format!("\n\t\t\tnew byte[] {}", negative_encodings);
    }

    let mut source = File::create(test_dir.join("TestRuntime.cs")).unwrap();
    writeln!(
        source,
        r#"
using System;
using System.Linq;
using System.IO;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace Serde.Tests {{
    [TestClass]
    public class TestRuntime {{
        static readonly byte[][] positive_inputs = new byte[][] {{{0}}};
        static readonly byte[][] negative_inputs = new byte[][] {{{1}}};

        [TestMethod]
        public void TestPassFailEncoding() {{
            foreach (byte[] input in positive_inputs) {{
                SerdeData test = SerdeData.{2}Deserialize(new MemoryStream(input));
                byte[] output = test.{2}Serialize();
    
                CollectionAssert.AreEqual(input, output);
    
                // Test simple mutations of the input.
                for (int i = 0; i < input.Length; i++) {{
                    byte[] input2 = input.ToArray();
                    input2[i] ^= 0x80;
                    try {{
                        SerdeData test2 = SerdeData.{2}Deserialize(new MemoryStream(input2));
                        Assert.AreNotEqual(test2, test);
                    }} 
                    catch (Exception) {{ }} 
                }}
            }}
    
            foreach (byte[] input in negative_inputs) {{
                try {{
                    SerdeData test = SerdeData.{2}Deserialize(new MemoryStream(input));
                    Assert.Fail("Input should fail to deserialize: " + input.ToString());
                }}
                catch (Exception) {{ }}
            }}
        }}
    }}
}}
"#,
        positive_encodings,
        negative_encodings,
        runtime.name().to_camel_case(),
    )
    .unwrap();

    dotnet_build(&test_dir);
    run_mstest(&test_dir);
}
