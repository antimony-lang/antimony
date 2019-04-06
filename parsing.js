const grammar = require("./grammar");

function parse(input) {
  return lex(tokenize(input), []);
}
exports.parse = parse;

function tokenize(inputString) {
  let tokens = inputString
    .split("")
    .map(token => (token == ";" ? " ; " : token))
    .map(token => (token == "(" ? " ( " : token))
    .map(token => (token == ")" ? " ) " : token))
    .map(token => (token == "{" ? " { " : token))
    .map(token => (token == "}" ? " } " : token))
    .filter(token => token != "\n")
    .join("")
    .split(" ")
    .filter(token => token != "");
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
    let exprTokens = [...tokens]
      .splice(
        position,
        [...tokens]
          .splice(position, tokens.length - 1)
          .findIndex(token => token === ";") + position
      )
      .filter(token => !["EOF", ";"].includes(token));
    if (exprTokens.length > 0) {
      tree.push(grammar.expr(exprTokens));
    }
    position += exprTokens.length + 1;
  }
  return tree;
}
