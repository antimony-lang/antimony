const { parse } = require("./parsing");
const fs = require("fs");

console.log(parse(fs.readFileSync("./test.flex").toString()));
