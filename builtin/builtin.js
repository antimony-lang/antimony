/* START builtins */

function _printf(msg) {
  // Message is casted to string to prevent crash
  process.stdout.write(msg.toString());
}

function _exit(code) {
  process.exit(code);
}

/* END builtins */
