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
These are the properties that not reflected by the bytecode as they would be in real JavaScript.
 - external member functions are only called correct if the callee expression is a direct member function
 - the 'this' pointer for external non-member functions is simply 'void 0'
 - Assignment expressions do not return a value, and thus are not really expressions
 - If you declare a variable without assignment it's value will be unknown. Thus it will might or might no be undefined (void 0). (It will be undefined but not JavaScript's undefined (void 0))
