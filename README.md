# Rusty-JSYC

is a JavaScript-To-Bytecode compiler written in rust. The bytecode is meant to be used in conjunction with the provided virtual machine written in JavaScript. Combined they are meant to be used as virtualization obfuscation.


### Virtualization Obfuscation

TODO

### How to use this
Compile a given JavaScript code
```Rust
TODO
```


Run the virtual machine in JavaScript context
```JavaScript
// include vm.js
// ...
var vm = new VM();
vm.init(Base64EncodedBytecode);
vm.run();
// ...
```

An example demonstrating both the compiler and the virtual machine can be found in ```playground/snake```. It features a small Snake game (snake.js).

### How to run tests
There are several test sets in this project:
  1. Node (mocha) tests:
  ```npm install && nom test```
  2. Cargo tests:
  ```cargo test```

### Current unsoundy properties
 - external member functions are only called correct if the callee expression is a direct member function
 - the 'this' pointer for external non-member functions is simply 'void 0'
 - Assignment expressions do not return a value, and thus are not really expressions
