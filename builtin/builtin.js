/* START builtins */

function _printf(msg) {
  // Message is casted to string to prevent crash
  process.stdout.write(msg.toString());
}

function _exit(code) {
  process.exit(code);
}

function _array_len(arr) {
  return arr.length;
}

/* END builtins */
