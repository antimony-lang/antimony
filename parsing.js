const grammar = require("./grammar");

function parse(input) {
  return lex(tokenize(input), []);
}
exports.parse = parse;

function tokenize(inputString) {
  let tokens = inputString.split(" ");
  if (tokens[tokens.length - 1] != "EOF") tokens.push("EOF");

  return tokens;
}

function lex(tokens, tree) {
  if (["EOF", undefined].includes(tokens[0])) {
    return tree;
  }

  if (tokens[0] == "let") {
    let expression = grammar.let_declaration(tokens);
    tree.push(expression);
  }

  return lex(tokens, tree);
}
