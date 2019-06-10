extern crate assert_cmd;

use std::process::Command;
use assert_cmd::prelude::*;

const DEFAULT_OUTPUT: &str = "Starting to compile bytecode...\nFinished bytecode compilation\nStarting to compose VM and bytecode...\n";

#[test]
fn test_empty_bc_std_vm() {
    let cmd = Command::cargo_bin("composer").unwrap()
                        .args(&["tests/data/empty/empty/main.js", "../vm/vm.js", "tests/.compiled/empty/empty"])
                        .output().unwrap();

    cmd.assert().success().stdout(DEFAULT_OUTPUT);
}

#[test]
fn test_compose_snake() {
    let cmd = Command::cargo_bin("composer").unwrap()
                .args(&["../playground/snake/unobfuscated/snake.js", "../vm/vm.js", "../playground/snake/obfuscated"])
                .output().unwrap();

    cmd.assert().success();
}
