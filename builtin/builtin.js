/* START builtins */

function _printf(msg) {
  // Message is casted to string to prevent crash
  process.stdout.write(msg.toString());
}

/* END builtins */
