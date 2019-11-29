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
  encodedArray.unshift(arrayLength & 0xff);

  return encodedArray;
}

function encodeLongNum(num)
{
  return [(num >> 24) & 0xff, (num >> 16) & 0xff, (num >> 8) & 0xff, (num >> 0) & 0xff]
}

var window = {
  document: {},
  String: String
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
    name: "Load float (12.5)",
    bytecode: [
      OP.LOAD_FLOAT, 150, ...[0x40, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    ],
    expected_registers: [
      [150, 12.5]
    ],
  },
  {
    name: "Load float (0.5)",
    bytecode: [
      OP.LOAD_FLOAT, 150, ...[0x3f, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    ],
    expected_registers: [
      [150, 0.5]
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
    name: "Load array",
    bytecode: [
      OP.LOAD_STRING, 150, ...encodeString("Hello World"),
      OP.LOAD_NUM, 151, 120,
      OP.LOAD_FLOAT, 152, ...[0x40, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
      OP.LOAD_ARRAY, 153, ...encodeRegistersArray([150, 151, 152])
    ],
    expected_registers: [
      [150, "Hello World"],
      [151, 120],
      [152, 12.5],
      [153, ["Hello World", 120, 12.5]]
    ],
  },
  {
    name: "Load long num positiv",
    bytecode: [
      OP.LOAD_LONG_NUM, 150, 0x0f, 0x00, 0xff, 0x1f
    ],
    expected_registers: [
      [150, 0x0f00ff1f],
    ],
  },
  {
    name: "Load long num negative",
    bytecode: [
      OP.LOAD_LONG_NUM, 150, 0xff, 0x00, 0xff, 0x1f
    ],
    expected_registers: [
      [150, -0x00ff00e1],
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
      // 22 is the offset of the bytecode function below
      OP.CALL_BCFUNC, ...encodeLongNum(22), 160, ...encodeRegistersArray([152, 150, 153, 151]),
      OP.ADD, 160, 160, 150,
      OP.EXIT,

      // The function: function(a, b) { return (a+b)^2; }
      // a: 152
      // b: 153
      OP.ADD, 152, 152, 153,
      OP.MUL, 152, 152, 152,
      OP.RETURN_BCFUNC, 152, ...encodeRegistersArray([])
    ],
    expected_registers: [
      [150, 60],
      [151, 6],
      [160, 4416]
    ]
  },
  {
    name: "Load and call custom function",
    bytecode: [
      OP.LOAD_STRING, 150, ...encodeString("0,function(){return 66;}"),
      OP.EVAL, 150, 150,
      OP.FUNC_CALL, 151, 150, 252, ...encodeRegistersArray([]),
    ],
    expected_registers: [
      [151, 66],
    ]
  },
  // { // TODO: This should work, but it requires 'null' to work
  //   // currently, 'null' is recognized as identifier by ressa
  //   name: "Try-Throw",
  //   init_regeisters: [
  //     [0, console],
  //     [1, JSON],
  //     [2, Object]
  //   ],
  //   bytecode: [
  //     OP.TRY, 7, ...encodeLongNum(49), ...encodeLongNum(72),
  //     OP.LOAD_STRING, 4, ...encodeString('{"x": 100}'),
  //     OP.LOAD_STRING, 6, ...encodeString("parse"),
  //     OP.PROPACCESS, 5, 1, 6,
  //     OP.FUNC_CALL, 3, 5, 1, ...encodeRegistersArray([4]),
  //     OP.LOAD_LONG_NUM, 200, ...encodeLongNum(98),
  //     OP.LOAD_STRING, 9, ...encodeString("log"),
  //     OP.PROPACCESS, 8, 0, 9,
  //     OP.FUNC_CALL, 202, 8, 0, ...encodeRegistersArray([7]),
  //     OP.LOAD_LONG_NUM, 200, ...encodeLongNum(98),
  //     OP.LOAD_STRING, 8, ...encodeString("create"),
  //     OP.PROPACCESS, 7, 2, 8,
  //     OP.FUNC_CALL, 3, 7, 2, ...encodeRegistersArray([9]),
  //     OP.LOAD_LONG_NUM, 200, ...encodeLongNum(98)
  //   ],
  //   expected_registers: [
  //     [2, {x: 100}]
  //   ]
  // }
  {
    name: "Try-Throw (no-throw)",
    init_regeisters: [
      [0, console],
      [1, JSON],
      [2, {}]
    ],
    bytecode: [
      OP.TRY, 7, ...encodeLongNum(49), ...encodeLongNum(58),
      OP.LOAD_STRING, 4, ...encodeString('{"x": 100}'),
      OP.LOAD_STRING, 6, ...encodeString("parse"),
      OP.PROPACCESS, 5, 1, 6,
      OP.FUNC_CALL, 3, 5, 1, ...encodeRegistersArray([4]),
      OP.LOAD_LONG_NUM, 200, ...encodeLongNum(81),
      OP.COPY, 3, 2,
      OP.LOAD_LONG_NUM, 200, ...encodeLongNum(81),
      OP.LOAD_STRING, 8, ...encodeString("log"),
      OP.PROPACCESS, 7, 0, 8,
      OP.FUNC_CALL, 202, 7, 0, ...encodeRegistersArray([3]),
      OP.LOAD_LONG_NUM, 200, ...encodeLongNum(81)
    ],
    expected_registers: [
      [3, {x: 100}]
    ]
  },
  {
    name: "Try-Throw (throw)",
    init_regeisters: [
      [0, console],
      [1, JSON],
      [2, {}]
    ],
    bytecode: [
      OP.TRY, 7, ...encodeLongNum(57), ...encodeLongNum(66),
      OP.LOAD_STRING, 4, ...encodeString('{invalid, invalid}'),
      OP.LOAD_STRING, 6, ...encodeString("parse"),
      OP.PROPACCESS, 5, 1, 6,
      OP.FUNC_CALL, 3, 5, 1, ...encodeRegistersArray([4]),
      OP.LOAD_LONG_NUM, 200, ...encodeLongNum(89),
      OP.COPY, 3, 2,
      OP.LOAD_LONG_NUM, 200, ...encodeLongNum(89),
      OP.LOAD_STRING, 8, ...encodeString("log"),
      OP.PROPACCESS, 7, 0, 8,
      OP.FUNC_CALL, 202, 7, 0, ...encodeRegistersArray([3]),
      OP.LOAD_LONG_NUM, 200, ...encodeLongNum(89)
    ],
    expected_registers: [
      [3, {}]
    ]
  }
]

function runVMTests(testData) {
  var encodedBytecode = encodeBytecode(testData.bytecode);

  if(testData.init instanceof Function) {
    testData.init();
  }

  var vm = new VM();

  if(typeof testData.init_regeisters !== "undefined") {
    // console.log(testData.init_regeisters[1]);
    for(let reg_init of testData.init_regeisters) {
      // console.log(reg_init);
      vm.setReg(reg_init[0], reg_init[1])
    }
  }

  vm.atob = require("atob");
  vm.setReg(REGS.WINDOW, window);

  vm.init(encodedBytecode);

  const result = vm.run();
  assert.equal(result, 0);

  for(let regData of testData.expected_registers) {
    assert.deepEqual(vm.getReg(regData[0]), regData[1],
                "Expected register " + regData[0] +  " to be " + regData[1] +
                " but it is " + vm.getReg(regData[0]));
  }
}


describe("VM Tests", function() {
  describe("Instructions Tests", function() {
    for(let testData of testDataSet) {
      it(testData.name, () => runVMTests(testData));
    }
  });
});
