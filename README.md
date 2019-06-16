[![Build Status](https://travis-ci.com/jwillbold/rusty-jsyc.svg?token=hPh87VpFt3MQPwdySdkS&branch=master)](https://travis-ci.com/jwillbold/rusty-jsyc)
[![codecov](https://codecov.io/gh/jwillbold/rusty-jsyc/branch/master/graph/badge.svg?token=puTrXEsmcx)](https://codecov.io/gh/jwillbold/rusty-jsyc)


# Rusty-JSYC

Rusty-JSYC (JavaScript bYtecode Compiler) is a JavaScript-To-Bytecode compiler written in Rust. The bytecode is meant to be used in conjunction with the provided virtual machine written in JavaScript. In combination they form the components for a virtualization obfuscation.

## How to use this
You must first compile the given JavaScript code. After that you can execute it with the provided virtual machine.

#### Compile your JavaScript code

You can either use the provided command line tool:

```Bash
cargo run </path/to/javascript.js> </path/to/vm-template.js> </output/dir> -d
```

or use the compiler as a library and call it from your own rust code:

```Rust
extern crate jsyc_compiler;

use jsyc_compiler::{JSSourceCode, BytecodeCompiler};

fn main() {
  let js_code = JSSourceCode::new("console.log('Hello World');".into());
  let mut compiler = BytecodeCompiler::new();

  let bytecode = compiler.compile(&js_code).expect("Failed to compile code");
  println!("Bytecode: {}", bytecode);

  let depedencies = compiler.decl_dependencies();
  println!("Depedencies: {:?}", depedencies);

  let base64_bytecode = bytecode.encode_base64();
  println!("Base64-encoded bytecode: {}", base64_bytecode);
}
```

In your Cargo.Toml:
```Toml
[dependencies]
jsyc_compiler = "~0.1"
```

#### Run the virtual machine
```JavaScript
// include vm.js
// ...
var vm = new VM();
vm.init(Base64EncodedBytecode);
requestIdleCallback(() => vm.run());
// ...
```
Replace ``Base64EncodedBytecode`` with the actual base64 encoded bytecode.

#### Playground example

An example demonstrating both the compiler and the virtual machine can be found in ``playground/snake``. It features a small Snake game (snake.js).
You can compile this with:
```Bash
cargo run "playground/snake/unobfuscated/snake.js" "vm/vm.js" "playground/snake/obfuscated" "playground/snake/unobfuscated/index.html"
```
After compilation, open the index.html file in your browser.
```
/path/to/rusty-jsyc/playground/snake/obfuscated/index.html
```
This was tested in Chrome 74 and Firefox 67. However, any ES6 capable browser should be compatible.

## Virtualization Obfuscation
Virtualization obfuscation is a state-of-the-art obfuscation scheme. It obfuscates the code by compiling it into bytecode which is then executed by a virtual machine (VM). Thus, the VM gets distributed along with the compiled bytecode. It is then called with this bytecode and executes it, and is thereby executing the actual code.

Since the bytecode is executed instruction by instruction, the original code is never restored anywhere. So, any potential attacker must first reverse engineer the VM, which may be heavily obfuscated, must then understand the underlying architecture and instruction-set before analyzing the actual bytecode. Since any two virtualization obfuscations are potentially different, the use of automated tools is limited. [[1](1)][[2](2)]

### Compatibility

#### Interactions between the virtual and real JavaScript context
It is possible to provide the functions defined in the virtual JavaScript context to the real JavaScript context.
```JavaScript
// Compiled JavaScript
function my_secret_function(a, b, c) { return a*b+c; }
window.important_secret_function = my_secret_function;
```

```JavaScript
// Non-Compiled JavaScript
var my_secret_function = window.important_secret_function;
my_secret_function(10, 20, 1337);
```

It does not need to be ``window``, any object instance know to both contexts will work. When calling ``my_secret_function`` the virtual machine will start the execution of the corresponding bytecode chunk. Thus, calling a function this way does not reveal any more information on the implementation than just calling it inside the compiled JavaScript.

#### Current unsound properties
These are the properties that are not reflected by the bytecode as they would be in real JavaScript.
 - the 'this' pointer for external non-member functions is simply 'void 0'
 - Assignment expressions do not return a value, and thus are not really expressions
 - If you declare a variable without assignment it's value will be unknown. Thus it might or might not be undefined (void 0). (It will be undefined but not JavaScript's undefined (void 0))
 - ``let`` and ``const`` declarations are treated as ``var`` declarations

#### Unsupported JavaScript syntaxes
This compiler currently only supports a subset of JavaScript features. Currently missing are
 - Object related notations ({}, new, this, super)
 - for-of and for-in loops
 - try and throw structures
 - break, continue, with, await, class and switch keywords
 - labels
 - function expressions and arrow function (Regular functions are allowed)
  - function expressions and arrow functions can be realized with:
  ```JavaScript
  var func_expr = eval("0, function(x) {return x*x;}");
  ```
  However, they do not support references to variables defined in the compiled JavaScript.
 - tagged template expressions
 - spread, rest and sequence notations

### How to run tests
There are several test sets in this project:
 1. Cargo tests: ``cargo test``
 2. Node (mocha) tests:``npm install && npm test``

_____________________________________
[1]: http://static.usenix.org/event/woot09/tech/full_papers/rolles.pdf
*1*: Rolf Rolles. Unpacking virtualization obfuscators. USENIX Workshop on Offensive Technologies (WOOT), 2009.

[2]: https://dslab.epfl.ch/pubs/staticVirtObf.pdf
*2*: Johannes Kinder. Towards static analysis of virtualization-obfuscated binaries. Reverse Engineering (WCRE), 2012 19th Working Conference.
