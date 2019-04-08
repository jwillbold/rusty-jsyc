// var bytecode = [50, 14, "getElementById".charCodeAt(0...13)];

// PRE-HELPERS
if(typeof window == "undefined") {
   var window = {};
}
if(window.document === void 0) {
  window.document = {};
}
if(window.String === void 0) {
  window.String = String;
}


const REGS = {
  // Internal helpers
  STACK_PTR: 1,
  RETURN_VAL: 2,
  REG_BACKUP: 3,
  BCFUNC_RETURN: 4,

  // Global context data
  WINDOW: 100,
  DOCUMENT: 101,

  // Misc
  VOID: 200,
  EMPTY_OBJ: 201
};

const OP = {
  // Loaders
  LOAD_STRING: 1,
  LOAD_NUM: 2,

  // Misc
  PROPACCESS: 10,
  FUNC_CALL: 11,
  EVAL: 12,
  CALL_BCFUNC: 13,
  RETURN_BCFUNC: 14,
  COPY: 15,
  EXIT: 16,

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
      var dst = vm.getByte(), str = vm._loadString();

      vm.setReg(dst, str);
    };

    this.ops[OP.LOAD_NUM] = function(vm) {
      var dst = vm.getByte(), val = vm.getByte();
      vm.setReg(dst, val);
    }

    this.ops[OP.FUNC_CALL] = function(vm) {
      var dst = vm.getByte(), func = vm.getByte(), funcThis = vm.getByte(),
          args = vm._loadArrayFromRegister();
      func = vm.getReg(func);
      funcThis = vm.getReg(funcThis);

      vm.setReg(dst, func.apply(funcThis, args));
    }

    this.ops[OP.EVAL] = function(vm) {
      var dst = vm.getByte(), str = vm.getByte();
      str = vm.getReg(str);

      vm.setReg(dst, eval(str));
    }

    this.ops[OP.CALL_BCFUNC] = function(vm) {
      var funcOffset = vm.getByte();

      vm.setReg(REGS.REG_BACKUP, vm.regs.slice());
      vm.setReg(REGS.STACK_PTR, funcOffset);
    }

    this.ops[OP.RETURN_BCFUNC] = function(vm) {
      var exceptions = [REGS.BCFUNC_RETURN];
      var regBackups = vm.getReg(REGS.REG_BACKUP);

      for(let exception of exceptions) {
        regBackups[exception] = vm.getReg(exception);
      }

      vm.regs = regBackups;
    }

    this.ops[OP.COPY] = function(vm) {
      var dst = vm.getByte(), src = vm.getByte();

      vm.setReg(dst, vm.getReg(src));
    }

    this.ops[OP.EXIT] = function(vm) {
      vm.setReg(REGS.STACK_PTR, vm.stack.length);
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
  }

  getReg(reg) {
    return this.regs[reg];
  }

  getByte() {
    return this.stack[this.regs[REGS.STACK_PTR]++];
  }

  run() {
    while(this.regs[REGS.STACK_PTR] < this.stack.length) {
      var op_code = this.getByte();
      var op = this.ops[op_code];
      op(this);
    }

    return this.regs[REGS.RETURN_VAL];
  }

  init(bytecode) {
    this.stack = this._decodeBytecode(bytecode);
    this.regs[REGS.STACK_PTR] = 0;
    this.regs[REGS.RETURN_VAL] = 0;
    this.regs[REGS.WINDOW] = window;
    this.regs[REGS.DOCUMENT] = window.document;
    this.regs[REGS.VOID] = void 0;
    this.regs[REGS.EMPTY_OBJ] = {};
  }

  _decodeBytecode(encodedBytecode) {
    return encodedBytecode;
  }

  _loadString() {
    // With a 1 byte string length it would only be possible to load
    // string up to a length of 256. However, this might be to short
    // load functions or so. 2 bytes and thus a maximal length of 65536
    // should be sufficient.
    var stringLength = (this.getByte() << 8) || this.getByte();
    var string = "";

    for(var i = 0;i<stringLength;i++) {
      string += String.fromCharCode(this.getByte());
    }

    return string;
  }

  _loadArrayFromRegister() {
    var arrayLength = (this.getByte() << 8) || this.getByte();
    var array = [];

    for(var i = 0;i<arrayLength;i++) {
      array.push(this.getReg(this.getByte()));
    }

    return array;
  }
}



///! HELPERS

function encodeBytecode(nonEncodedBytecode)
{
  return nonEncodedBytecode;
}

function encodeString(string)
{
  var stringLength = string.length;
  var bytecode = [stringLength & 0xff00, stringLength & 0xff];

  for(var i = 0;i<stringLength;i++) {
    bytecode.push(string.charCodeAt(i));
  }

  return bytecode;
}

function encodeRegistersArray(array)
{
  const arrayLength = array.length;

  var encodedArray = array.slice();
  encodedArray.unshift(arrayLength & 0xff00, arrayLength & 0xff);

  return encodedArray;
}




///! TESTING

const testDataSet = [
  {
    name: "Empty Bytecode",
    bytecode: [],
    expected_registers: [],
  },
  {
    name: "Set return value 66",
    bytecode: [
      OP.LOAD_NUM, 150 , 66, // LOAD NUM 66 INTO REGISTER 150
    ],
    expected_registers: [
      [150, 66]
    ],
  },
  {
    name: "Multiply two registers",
    bytecode: [
      OP.LOAD_NUM, 150, 3, // LOAD NUM 3 INTO REGISTER 100
      OP.LOAD_NUM, 151, 2, // LOAD NUM 2 INTO REGISTER 101
      OP.MUL, 150, 151,    // MULTIPLY NUM IN REG 100 WITH NUM IN REG 101
    ],
    expected_registers: [
      [150, 6]
    ],
  },
  {
    name: "Load string",
    bytecode: [
      OP.LOAD_STRING, 150, ...encodeString("Hello World")
    ],
    expected_registers: [
      [150, "Hello World"]
    ],
  },
  {
    name: "Call member function",
    init: function() {
      window.testFunc = function() { return 66; };
    },
    bytecode: [
      OP.LOAD_STRING, 150, ...encodeString("testFunc"),
      OP.PROPACCESS, 151, REGS.WINDOW, 150,
      OP.FUNC_CALL, 152, 151, REGS.WINDOW, ...encodeRegistersArray([])
    ],
    expected_registers: [
      [150, "testFunc"],
      [152, 66]
    ],
  },
  {
    name: "Call member function with arguments",
    init: function() {
      window.testFunc = function(a ,b) { return a + b; };
    },
    bytecode: [
      OP.LOAD_STRING, 150, ...encodeString("testFunc"),
      OP.PROPACCESS, 151, REGS.WINDOW, 150,
      OP.LOAD_NUM, 160, 60,
      OP.LOAD_NUM, 161, 6,
      // Cal the function
      OP.FUNC_CALL, 152, 151, REGS.WINDOW, ...encodeRegistersArray([160, 161])
    ],
    expected_registers: [
      [150, "testFunc"],
      [152, 66]
    ],
  },
  {
    name: "Create object",
    bytecode: [
      OP.LOAD_STRING, 150, ...encodeString("String"),
      OP.PROPACCESS, 151, REGS.WINDOW, 150,
      OP.FUNC_CALL, 152, 151, REGS.WINDOW, ...encodeRegistersArray([])
    ],
    expected_registers: [
      [150, "String"],
      [152, ""]
    ]
  },
  {
    name: "Call bytecode function",
    bytecode: [
      OP.LOAD_NUM, 150, 60,
      OP.LOAD_NUM, 151, 6,
      OP.CALL_BCFUNC, 12, // 15 is the offset of the bytecode below
      OP.ADD, REGS.BCFUNC_RETURN, 150,
      OP.EXIT,

      // The function: function(a ,b) { return (a+b)*2; }
      OP.ADD, 150, 151,
      OP.MUL, 150, 150,
      OP.COPY, REGS.BCFUNC_RETURN, 150,
      OP.RETURN_BCFUNC,
    ],
    expected_registers: [
      [REGS.BCFUNC_RETURN, 4416],
      [150, 60],
      [151, 6],
    ]
  },
  {
    name: "Load and call custom function",
    bytecode: [
      OP.LOAD_STRING, 150, ...encodeString("0,function(){return 66;}"),
      OP.EVAL, 150, 150,
      OP.FUNC_CALL, 151, 150, REGS.EMPTY_OBJ, ...encodeRegistersArray([]),
    ],
    expected_registers: [
      [151, 66],
    ]
  }
]

function runVMTests(testData) {
  try {
    var encodedBytecode = encodeBytecode(testData.bytecode);

    if(testData.init instanceof Function) {
      testData.init();
    }

    var vm = new VM();
    vm.init(encodedBytecode);
    const result = vm.run();

    if(result == 0) {
      var failure = false;
      for(let regData of testData.expected_registers) {
        if(vm.getReg(regData[0]) !== regData[1]) {
          failure = true;
          console.warn(testData.name, "failed. Expected register", regData[0],
                       "to be", regData[1], "but it is", vm.getReg(regData[0]));
        }
      }

      if(!failure) {
        console.log(testData.name, "passed")
      }
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
