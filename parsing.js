const grammar = require("./grammar");

function parse(input) {
  return lex(tokenize(input), []);
}
exports.parse = parse;

function tokenize(inputString) {
  let tokens = inputString.replace("\n", " \n ").split(" ");
  if (tokens[tokens.length - 1] != "EOF") tokens.push("EOF");

  return tokens;
}

let layer = 0;
function lex(tokens) {
  let tree = [];
  let position = 0;

  if (["EOF", undefined].includes(tokens[position])) {
    return tree;
  }

  while (![undefined, "EOF"].includes(tokens[position])) {
    if (tokens[position] == "let") {
      let exprTokens = [...tokens].splice(
        position,
        tokens.findIndex(token => token === "\n")
      );
      tree.push(grammar.let_declaration(exprTokens));
      position += exprTokens.length + 1;
    } else {
      throw new Error(`Unexpected token: ${tokens[position]}`);
    }
  }
  return tree;
}
