if(typeof window == "undefined") {
   var window = {};
}
if(window.document === void 0) {
  window.document = {};
}
if(window.String === void 0) {
  window.String = String;
}
if(window.atob === void 0) {
  window.atob = require('atob');
}


const REGS = {
  // External dependencies
  WINDOW: 100,
  // DOCUMENT: 101,

  // Reserved registers
  STACK_PTR: 200,
  REG_BACKUP: 201,
  RETURN_VAL: 202,
  BCFUNC_RETURN: 203,

  // Common literals
  EMPTY_OBJ: 252,
  VOID: 253,
  NUM_1: 254,
  NUM_0: 255,
};

const OP = {
  // Loaders
  LOAD_STRING: 1,
  LOAD_NUM: 2,
  LOAD_FLOAT: 3,
  LOAD_LONG_NUM: 4,
  LOAD_ARRAY: 5,

  // Misc
  PROPACCESS: 10,
  FUNC_CALL: 11,
  EVAL: 12,
  CALL_BCFUNC: 13,
  RETURN_BCFUNC: 14,
  COPY: 15,
  EXIT: 16,
  COND_JUMP: 17,
  JUMP: 18,
  JUMP_COND_NEG: 19,
  BCFUNC_CALLBACK: 20,

  // Comparisons
  COMP_EQUAL: 50,
  COM_NOT_EQUAL: 51,
  COMP_LESS_THAN: 54,
  COMP_GREATHER_THAN: 55,
  COMP_LESS_THAN_EQUAL: 56,
  COMP_GREATHER_THAN_EQUAL: 57,

  // Math
  ADD: 100,
  MUL: 101,
  MINUS: 102,
  DIV: 103
};

class VM {
  constructor() {
    this.regs =  [];
    this.stack = [];
    this.ops = [];
    this.reg_backups = [];

    this.ops[OP.LOAD_STRING] = function(vm) {
      var dst = vm.getByte(), str = vm._loadString();
      vm.setReg(dst, str);
    };

    this.ops[OP.LOAD_NUM] = function(vm) {
      var dst = vm.getByte(), val = vm.getByte();
      vm.setReg(dst, val);
    };

    this.ops[OP.LOAD_FLOAT] = function(vm) {
      var dst = vm.getByte(), val = vm._loadFloat();
      vm.setReg(dst, val);
    };

    this.ops[OP.LOAD_LONG_NUM] = function(vm) {
      var dst = vm.getByte(), val = vm._loadLongNum();
      vm.setReg(dst, val);
    };

    this.ops[OP.LOAD_ARRAY] = function(vm) {
      var dst = vm.getByte(), array = vm._loadArrayFromRegister();
      vm.setReg(dst, array);
    };

    this.ops[OP.PROPACCESS] = function(vm) {
      var dst = vm.getByte(), obj = vm.getByte(), prop = vm.getByte();
      obj = vm.getReg(obj); prop = vm.getReg(prop);

      vm.setReg(dst, obj[prop]);
    };

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
      var returnReg = vm.getByte();
      var argsArray = vm._loadRegistersArray();
      vm.reg_backups.push([vm.regs.slice(), returnReg]);

      for(let i = 0; i <= argsArray.length/2; i+=2) {
        vm.setReg(argsArray[i+1], vm.getReg(argsArray[i]));
      }

      vm.setReg(REGS.STACK_PTR, funcOffset);
    }

    this.ops[OP.RETURN_BCFUNC] = function(vm) {
      var returnFromReg = vm.getByte();
      var returnData = vm.reg_backups.pop();
      var regBackups = returnData[0];
      let returnToReg = returnData[1];

      regBackups[returnToReg] = vm.getReg(returnFromReg);

      vm.regs = regBackups;
    }

    this.ops[OP.COPY] = function(vm) {
      var dst = vm.getByte(), src = vm.getByte();
      vm.setReg(dst, vm.getReg(src));
    }

    this.ops[OP.EXIT] = function(vm) {
      vm.setReg(REGS.STACK_PTR, vm.stack.length);
    }

    this.ops[OP.COND_JUMP] = function(vm) {
      var cond = vm.getByte(), jump = vm.getByte();
      cond = vm.getReg(cond);
      jump = vm.getReg(jump);

      if(cond) {
        vm.setReg(REGS.STACK_PTR, jump);
      }
    }

    this.ops[OP.JUMP] = function(vm) {
      var jump = vm.getReg(vm.getByte());
      vm.setReg(REGS.STACK_PTR, jump);
    }

    this.ops[OP.JUMP_COND_NEG] = function(vm) {
      var cond = vm.getByte(), jump = vm.getByte();
      cond = vm.getReg(cond);
      jump = vm.getReg(jump);

      if(!cond) {
        vm.setReg(REGS.STACK_PTR, jump);
      }
    }

    this.ops[OP.BCFUNC_CALLBACK] = function(vm) {
      var dst = vm.getByte(), func_offset = vm._loadLongNum(), arg_regs = vm._loadRegistersArray();
      vm.setReg(dst, function() {
        for(let i = 0; i<arg_regs.length; ++i) {
          vm.setReg(arg_regs[i], arguments[i]);
        }
        vm.runAt(func_offset)
      });
    }

    this.ops[OP.COMP_EQUAL] = function(vm) {
      var dst = vm.getByte(), left = vm.getByte(), right = vm.getByte();
      left = vm.getReg(left);
      right = vm.getReg(right);

      vm.setReg(dst, left == right);
    }

    this.ops[OP.COMP_NOT_EQUAL] = function(vm) {
      var dst = vm.getByte(), left = vm.getByte(), right = vm.getByte();
      left = vm.getReg(left);
      right = vm.getReg(right);

      vm.setReg(dst, left != right);
    }

    this.ops[OP.COMP_LESS_THAN] = function(vm) {
      var dst = vm.getByte(), left = vm.getByte(), right = vm.getByte();
      left = vm.getReg(left);
      right = vm.getReg(right);

      vm.setReg(dst, left < right);
    }

    this.ops[OP.COMP_GREATHER_THAN] = function(vm) {
      var dst = vm.getByte(), left = vm.getByte(), right = vm.getByte();
      left = vm.getReg(left);
      right = vm.getReg(right);

      vm.setReg(dst, left > right);
    }

    this.ops[OP.COMP_LESS_THAN_EQUAL] = function(vm) {
      var dst = vm.getByte(), left = vm.getByte(), right = vm.getByte();
      left = vm.getReg(left);
      right = vm.getReg(right);

      vm.setReg(dst, left <= right);
    }

    this.ops[OP.COMP_GREATHER_THAN_EQUAL] = function(vm) {
      var dst = vm.getByte(), left = vm.getByte(), right = vm.getByte();
      left = vm.getReg(left);
      right = vm.getReg(right);

      vm.setReg(dst, left >= right);
    }

    this.ops[OP.ADD] = function(vm) {
      var dst = vm.getByte(), src0 = vm.getByte(), src1 = vm.getByte();
      vm.setReg(dst, vm.regs[src0] + vm.regs[src1]);
    }

    this.ops[OP.MUL] = function(vm) {
      var dst = vm.getByte(), src0 = vm.getByte(), src1 = vm.getByte();
      vm.setReg(dst, vm.regs[src0] * vm.regs[src1]);
    }

    this.ops[OP.MINUS] = function(vm) {
      var dst = vm.getByte(), src0 = vm.getByte(), src1 = vm.getByte();
      vm.setReg(dst, vm.regs[src0] - vm.regs[src1]);
    }

    this.ops[OP.DIV] = function(vm) {
      var dst = vm.getByte(), src0 = vm.getByte(), src1 = vm.getByte();
      vm.setReg(dst, vm.regs[src0] / vm.regs[src1]);
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

  runAt(offset) {
    this.reg_backups.push([this.regs.slice(), REGS.BCFUNC_RETURN]);
    setReg(REGS.STACK_PTR, offset);
    run();
  }

  init(bytecode) {
    this.stack = this._decodeBytecode(bytecode);
    this.setReg(REGS.STACK_PTR, 0);
    this.setReg(REGS.RETURN_VAL, 0);

    this.setReg(REGS.WINDOW, window);

    this.setReg(REGS.VOID, void 0);
    this.setReg(REGS.EMPTY_OBJ, {});
  }

  _decodeBytecode(encodedBytecode) {
    var bytecode = window.atob(encodedBytecode);
    var bytes = [];
    var byteCounter = 0;
    for (var i = 0; i < bytecode.length; i++){
      var b = bytecode.charCodeAt(i);
      if (b > 255) {
        bytes[byteCounter++] = b & 255;
        b >>= 8;
      }
      bytes[byteCounter++] = b;
    }

    return bytes;
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
    var arrayLength = this.getByte();
    var array = [];

    for(var i = 0;i<arrayLength;i++) {
      array.push(this.getReg(this.getByte()));
    }

    return array;
  }

  _loadFloat() {
    var num_str = "";
    for(let i = 0; i<8; i++) {
      let x = this.getByte();
      num_str += x < 0x10 ? '0'+ x.toString(16) : x.toString(16);
    }

    var binary = parseInt(num_str, 16).toString(2);
    binary = '0'*(64-binary.length) + binary;

    var sign = (binary.charAt(0) == '1')? -1 : 1;
    var exponent = parseInt(binary.substr(1, 11), 2) - 0x3ff;
    var significandBase = binary.substr(12);
    var significandBin = '1' + significandBase;

    if (exponent == -0x3ff) {
        if (significandBase.indexOf('1') == -1) {
            return 0;
        } else {
            exponent = -0x3fe;
            significandBin = '0'+significandBase;
        }
    }

    var i = 0;
    var val = 1;
    var significand = 0;
    while (i < significandBin.length) {
        significand += val * parseInt(significandBin.charAt(i));
        val = val / 2;
        i++;
    }

    return sign * significand * Math.pow(2, exponent);
  }

  _loadLongNum() {
    var num = this.getByte() << 24 | this.getByte() << 16 |
              this.getByte() << 8 | this.getByte();
    return num;
  }

  _loadRegistersArray() {
    var arrayLength = this.getByte();
    var registers_array = [];

    for(var i = 0;i<arrayLength;i++) {
      registers_array.push(this.getByte());
    }

    return registers_array;
  }
}


module.exports = function() {
    this.REGS = REGS;
    this.OP = OP;
    this.VM = VM;
    this.window = window;
}
