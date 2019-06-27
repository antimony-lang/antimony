const { parse } = require("./parsing");
import fs from "fs";
import Tokenizer from "./Tokenizer";

let fileString = fs.readFileSync("./test.flex").toString();

let tokenizer = new Tokenizer(fileString);

let ast = tokenizer.run();
console.log(ast);
