// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(feature = "runtime-testing")]

use serde_generate::{
    csharp, test_utils,
    test_utils::{Choice, Runtime, Test},
    CodeGeneratorConfig,
    SourceInstaller,
};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;
use heck::CamelCase;
use std::path::Path;

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

#[test]
fn test_csharp_lcs_runtime_tests() {
    use serde_generate::SourceInstaller;

    let dir = tempdir().unwrap();

    let installer = csharp::Installer::new(dir.path().to_path_buf());
    installer.install_serde_runtime().unwrap();
    installer.install_lcs_runtime().unwrap();

    let lcs_test_dir = dir.path().join("Serde.Tests");
    std::fs::create_dir(&lcs_test_dir).unwrap();
    std::fs::copy("runtime/csharp/Serde.Tests/Serde.Tests.csproj", 
        &lcs_test_dir.join("Serde.Tests.csproj")).unwrap();
    std::fs::copy("runtime/csharp/Serde.Tests/TestLcs.cs", 
        &lcs_test_dir.join("TestLcs.cs")).unwrap();

    dotnet_build(&lcs_test_dir);
    run_mstest(&lcs_test_dir);
}

#[test]
fn test_csharp_lcs_runtime_on_simple_data() {
    test_csharp_runtime_on_simple_data(Runtime::Lcs);
}

#[test]
fn test_csharp_bincode_runtime_on_simple_data() {
    test_csharp_runtime_on_simple_data(Runtime::Bincode);
}

fn test_csharp_runtime_on_simple_data(runtime: Runtime) {
    let registry = test_utils::get_simple_registry().unwrap();
    let dir = tempdir().unwrap();

    let installer = csharp::Installer::new(dir.path().to_path_buf());
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();

    let test_dir = dirdir.path().join("Serde.Tests");
    std::fs::create_dir(&test_dir).unwrap();
    std::fs::copy("runtime/csharp/Serde.Tests/Serde.Tests.csproj", 
        &test_dir.join("Serde.Tests.csproj")).unwrap();

    // TODO: Is the CodeGenerator supposed to copy the serde runtime?
    let config =
        CodeGeneratorConfig::new("Serde.Tests".to_string()).with_encodings(vec![runtime.into()]);
    let generator = csharp::CodeGenerator::new(&config);
    generator
        .write_source_files(dir.path().to_path_buf(), &registry)
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
    test_csharp_runtime_on_supported_types(Runtime::Lcs);
}

#[test]
fn test_csharp_bincode_runtime_on_supported_types() {
    test_csharp_runtime_on_supported_types(Runtime::Bincode);
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

fn test_csharp_runtime_on_supported_types(runtime: Runtime) {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    let installer = csharp::Installer::new(dir.path().to_path_buf());
    installer.install_serde_runtime().unwrap();

    let test_dir = dir.path().join("Serde.Tests");
    std::fs::create_dir(&test_dir).unwrap();
    std::fs::copy("runtime/csharp/Serde.Tests/Serde.Tests.csproj", 
        &test_dir.join("Serde.Tests.csproj")).unwrap();

    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_encodings(vec![runtime.into()]);
    let generator = csharp::CodeGenerator::new(&config);
    generator
        .write_source_files(dir.path().to_path_buf(), &registry)
        .unwrap();

    let positive_encodings: Vec<_> = runtime
        .get_positive_samples_quick()
        .iter()
        .map(|bytes| quote_bytes(bytes))
        .collect();

    let negative_encodings: Vec<_> = runtime
        .get_negative_samples()
        .iter()
        .map(|bytes| quote_bytes(bytes))
        .collect();

    let mut source = File::create(test_dir.join("TestRuntime.cs")).unwrap();
    writeln!(
        source,
        r#"
using System;
using System.Collections.Generic;
using System.Numerics;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace Serde.Tests {{
    [TestClass]
    public class TestRuntime {{
        static readonly byte[][] positive_inputs = new byte[][] {{{0}}};
        static readonly byte[][] negative_inputs = new byte[][] {{{1}}};

        [TestMethod]
        public void TestPassFailEncoding() {{
            foreach (byte[] input in positive_inputs) {{
                SerdeData test = SerdeData.{2}Deserialize(input);
                byte[] output = test.{2}Serialize();
    
                CollectionAssert.AreEqual(input, output);
    
                // Test simple mutations of the input.
                for (int i = 0; i < input.Length; i++) {{
                    byte[] input2 = input.ToArray();
                    input2[i] ^= 0x80;
                    try {{
                        SerdeData test2 = SerdeData.{2}Deserialize(input2);
                        Assert.AreNotEqual(test2, test);
                    }} catch (DeserializationError e) {{
                        // All good
                    }}
                }}
    
            }}
    
            foreach (byte[] input in negative_inputs) {{
                try {{
                    SerdeData test = SerdeData.{2}Deserialize(input);
                    int[] bytes = new int[input.Length];
                    Arrays.setAll(bytes, n -> Math.floorMod(input[n], 256));
                    throw new Exception("Input should fail to deserialize: " + Arrays.asList(bytes));
                }} catch (DeserializationError e) {{
                        // All good
                }}
            }}
        }}
    }}
}}
"#,
        positive_encodings.join(", "),
        negative_encodings.join(", "),
        runtime.name().to_camel_case(),
    )
    .unwrap();

    dotnet_build(&test_dir);
    run_mstest(&test_dir);
}
