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

function lex(tokens) {
  let tree = [];
  let position = 0;

  if (["EOF", undefined].includes(tokens[position])) {
    return tree;
  }

  while (![undefined, "EOF"].includes(tokens[position])) {
    let exprTokens = [...tokens].splice(
      position,
      tokens.findIndex(token => [";".includes(token)]) + position
    );
    if (exprTokens.length > 0) {
      tree.push(grammar.expr(exprTokens));
    }
    position += exprTokens.length + 1;
  }
  return tree;
}
