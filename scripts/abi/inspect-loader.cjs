// Inspect the loaded call interface without invoking any native function.
const path = require('node:path');
const { lib, koffi, ReactiveError } = require(path.join(process.argv[2], 'ffi.js'));
const schema = require(path.join(process.argv[2], 'native-api.json'));

function type(value) {
  switch (value.primitive) {
    case 'Void': return value.name;
    case 'Bool': return 'bool';
    case 'String': return '*i8';
    case 'Pointer': return '*' + type(value.ref);
    case 'Record': case 'Union': return value.name;
    case 'Float32': return 'f32';
    case 'Float64': return 'f64';
    case 'Callback': return signature(value.proto);
    default:
      if (/^(U?Int)(8|16|32|64)$/.test(value.primitive)) {
        return (value.primitive.startsWith('U') ? 'u' : 'i') + value.size * 8;
      }
      throw new Error(`Unsupported ABI type: ${value.primitive}`);
  }
}
function signature(info) {
  return 'callback(' + info.arguments.map(arg => type(arg.type)).join(',') + ')->' + type(info.result);
}
const functions = Object.fromEntries(Object.entries(lib).map(([name, fn]) => {
  if (fn.info.name !== name) throw new Error(`Loader symbol mismatch: ${name}`);
  return [name, signature(fn.info)];
}));
const layouts = [];
const fields = {};
for (const [name, record] of Object.entries(schema.records)) {
  layouts.push(`layout ${name} ${koffi.sizeof(name)} ${koffi.alignof(name)}`);
  for (const field of Object.keys(record.fields)) {
    const member = koffi.type(name).members[field];
    layouts.push(`field ${name}.${field} ${member.offset}`);
    fields[`${name}.${field}`] = type(member.type);
  }
}
for (const name of schema.enums) layouts.push(`layout ${name} ${koffi.sizeof(name)} ${koffi.alignof(name)}`);
const errorValues = Object.fromEntries(Object.entries(ReactiveError).filter(([, value]) => typeof value === 'number')
  .map(([name, value]) => ['R_TUI_ERROR_' + name.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toUpperCase(), value]));
console.log(JSON.stringify({ functions, layouts, fields, errorValues }));
