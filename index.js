const { parse } = require("./parsing");
const fs = require("fs");

let fileString = fs.readFileSync("./test.flex").toString();

let ast = parse(fileString);
console.log(ast);
