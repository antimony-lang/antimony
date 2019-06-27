const { parse } = require("./parsing");
import fs from "fs";
import Tokenizer from "./Tokenizer";
import { TokenType } from "./tokens";

let fileString = fs.readFileSync("./test.flex").toString();

let tokenizer = new Tokenizer(fileString);

let ast = tokenizer.run();
ast.forEach(token => console.log(TokenType[token.type], token.value));
