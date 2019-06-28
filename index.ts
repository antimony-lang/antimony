import fs from "fs";
import Tokenizer from "./Tokenizer";
import { Parser } from "./Parser";

let fileString = fs.readFileSync("./test.flex").toString();

let tokens = new Tokenizer(fileString).run();
let parser = new Parser(tokens);
let ast = parser.parse();
console.log(ast);
