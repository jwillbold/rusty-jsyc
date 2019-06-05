#[macro_use]
extern crate test_exec;

const DEFAULT_OUTPUT: &str = "Starting to compile bytecode...\nFinished bytecode compilation\nStarting to compose VM and bytecode...\n";

#[test]
fn test_empty_bc_std_vm() {
    exec!{
        "composer",
        args: ["composer/tests/data/empty/empty/main.js", "vm/vm.js", "composer/tests/.compiled/empty/empty"],
        cwd: ".",
        log: true,

        code: 0,
        stdout: DEFAULT_OUTPUT,
        stderr: []
    };
}

#[test]
fn test_compose_snake() {
    exec!{
        "composer",
        args: ["playground/unobfuscated/snake_helper.js", "vm/vm.js", "playground/obfuscated"],
        cwd: ".",
        log: true,

        code: 0,
        stdout: DEFAULT_OUTPUT,
        stderr: []
    };
}
