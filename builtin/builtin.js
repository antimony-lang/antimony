/* START builtins */

function _printf(msg) {
  // Message is casted to string to prevent crash
  process.stdout.write(msg.toString());
}

function _exit(code) {
  process.exit(code);
}

function _strlen(s) {
  return s.length;
}

function _parse_int(s) {
  return parseInt(s, 10);
}

function _int_to_str(n) {
  return String(n);
}

function _read_line() {
  const fs = require("fs");
  const buf = Buffer.alloc(256);
  let line = "";
  try {
    const n = fs.readSync(0, buf, 0, buf.length, null);
    line = buf.slice(0, n).toString().replace(/\n$/, "");
  } catch (_) {}
  return line;
}

function _argc() {
  return process.argv.length - 1;
}

function _argv(i) {
  return process.argv[i + 1];
}

/* END builtins */
