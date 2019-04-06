// var bytecode = [50, 14, "getElementById".charCodeAt(0...13)];

// PRE-HELPERS
window = {};
if(window.document === void 0) {
  window.document = {};
}


const REGS = {
  STACK_PTR: 1,
  DOCUMENT: 2,
  RETURN_VAL: 3,
};

const OP = {
  // Loaders
  LOAD_STRING: 1,
  LOAD_NUM: 2,

  // Misc
  PROPACCESS: 10,

  // Math
  ADD: 100,
  MUL: 101,
};

class VM
{

  constructor() {
    this.regs =  [];
    this.stack = [];
    this.ops = [];

    this.ops[OP.PROPACCESS] = function(vm) {
      var dst = vm.getByte(), obj = vm.getByte(), prop = vm.getByte();
      obj = vm.getReg(obj); prop = vm.getReg(prop);

      vm.setReg(dst, obj[prop]);
    };

    this.ops[OP.LOAD_STRING] = function(vm) {
      var dst = vm.getByte(), len = vm.getByte();
      var str = "";
      for(var i = 0; i < len; i++) {
        str = String.fromCharCode(vm.getByte());
      }
      vm.setReg(dst, str);
    };

    this.ops[OP.LOAD_NUM] = function(vm) {
      var dst = vm.getByte(), val = vm.getByte();
      vm.setReg(dst, val);
    }

    this.ops[OP.ADD] = function(vm) {
      var dst = vm.getByte(), src = vm.getByte();
      vm.setReg(dst, vm.regs[dst] + vm.regs[src]);
    }

    this.ops[OP.MUL] = function(vm) {
      var dst = vm.getByte(), src = vm.getByte();
      vm.setReg(dst, vm.regs[dst] * vm.regs[src]);
    }
  }


  setReg(reg, value) {
    this.regs[reg] = value;
  };

  getReg(reg) {
    return this.regs[reg];
  };

  getByte() {
    return this.stack[this.regs[REGS.STACK_PTR]++];
  };

  run() {
    while(this.regs[REGS.STACK_PTR] < this.stack.length) {
      var op_code = this.getByte();
      var op = this.ops[op_code];
      op(this);
    }

    return this.regs[REGS.RETURN_VAL];
  };

  decodeBytecode(encodedBytecode) {
    return encodedBytecode;
  };

  init(bytecode) {
    this.stack = this.decodeBytecode(bytecode);
    this.regs[REGS.STACK_PTR] = 0;
    // this.regs[REGS.DOCUMENT] = document;
    this.regs[REGS.RETURN_VAL] = 0;
  };

}



///! HELPERS

function wrapBytecode(nonWrappedBytecode)
{
  return nonWrappedBytecode;
}




///! TESTING

const testDataSet = [
  {
    name: "Empty Bytecode",
    bytecode: [],
    registers: {},
  },
  {
    name: "Set return value 66",
    bytecode: [
      OP.LOAD_NUM, 100, 66, // LOAD NUM 66 INTO REGISTER 100
    ],
    registers: [
      [100, 66]
    ],
  },
  {
    name: "Multiply two registers",
    bytecode: [
      OP.LOAD_NUM, 100, 3, // LOAD NUM 3 INTO REGISTER 100
      OP.LOAD_NUM, 101, 2, // LOAD NUM 2 INTO REGISTER 101
      OP.MUL, 100, 101,    // MULTIPLY NUM IN REG 100 WITH NUM IN REG 101
    ],
    registers: [
      [100, 6]
    ],
  }
]

function runVMTests(testData) {
  try {
    var wrappedBytecode = wrapBytecode(testData.bytecode);

    var vm = new VM();
    vm.init(wrappedBytecode);
    const result = vm.run();

    if(result == 0) {
      for(let regData in testData.registers) {
        if(vm.getReg(regData[0]) !== regData[1]) {
          console.warn(testData.name, "failed with return value: ", result);
        }
      }

      console.log(testData.name, "passed");
    } else {
      console.warn(testData.name, "failed with return value: ", result);
    }
  } catch(e) {
    console.warn(testData.name, "failed with exception: ", e);
  }

}

for(let testData of testDataSet) {
  runVMTests(testData);
}
