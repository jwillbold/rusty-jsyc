require("./test_helper.js")();
require("../vm.js")();
// var btoa = require('btoa');

///! HELPERS

function encodeBytecode(nonEncodedBytecode)
{
  return Buffer.from(nonEncodedBytecode).toString('base64')
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

const testDataSet = [
  {
    name: "Empty Bytecode",
    bytecode: [],
    expected_registers: [],
  },
  {
    name: "Load short num",
    bytecode: [
      OP.LOAD_NUM, 150 , 66, // LOAD NUM 66 INTO REGISTER 150
    ],
    expected_registers: [
      [150, 66]
    ],
  },
  {
    name: "Load float",
    bytecode: [
      OP.LOAD_FLOAT, 150, ...[0x40, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    ],
    expected_registers: [
      [150, 12.5]
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
    name: "Multiply two registers",
    bytecode: [
      OP.LOAD_NUM, 150, 3, // LOAD NUM 3 INTO REGISTER 100
      OP.LOAD_NUM, 151, 2, // LOAD NUM 2 INTO REGISTER 101
      OP.MUL, 150, 150, 151,    // MULTIPLY NUM IN REG 100 WITH NUM IN REG 101
    ],
    expected_registers: [
      [150, 6]
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
      OP.CALL_BCFUNC, 13, // 12 is the offset of the bytecode below
      OP.ADD, REGS.BCFUNC_RETURN, REGS.BCFUNC_RETURN, 150,
      OP.EXIT,

      // The function: function(a ,b) { return (a+b)*2; }
      OP.ADD, 150, 150, 151,
      OP.MUL, 150, 150, 150,
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
  var encodedBytecode = encodeBytecode(testData.bytecode);

  if(testData.init instanceof Function) {
    testData.init();
  }

  var vm = new VM();
  vm.init(encodedBytecode);

  const result = vm.run();
  assert.equal(result, 0);

  for(let regData of testData.expected_registers) {
    assert.equal(vm.getReg(regData[0]), regData[1],
                "Expected register " + regData[0] +  " to be " + regData[1] +
                " but it is " + vm.getReg(regData[0]));
  }
}


describe("VM Tests", function() {
  describe("Instructions Tests", function() {
    for(let testData of testDataSet) {
      it(testData.name, function() {
        runVMTests(testData);
      });
    }
  });
});
