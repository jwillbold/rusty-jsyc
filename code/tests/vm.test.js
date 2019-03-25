import VM from '/home/motu/blogpost/code/vm.js';
import * as helpers from '../helpers.js';

const testDataSet = [
  {
    name: "Empty Bytecode",
    bytecode: [],
    registers: {},
  }
]

function runVMTests(testData)
{
  try {
    var wrappedBytecode = helpers.wrapBytecode(testData[bytecode]);

    var vm = new VM();
    var result = vm.invoke(wrappedBytecode);

    if(result == 0) {
      console.log(testData[name], "passed");
    } else {
      console.warn(testData[name], "failed with return value: ", result);
    }
  } catch(e) {
    console.warn(testData[name], "failed with exception: ", e);
  }

}




runVMTests();
