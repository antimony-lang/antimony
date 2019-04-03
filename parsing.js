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
function lex(tokens, tree) {
  if (tree[layer] == undefined) {
    tree[layer] = [];
  }
  if (["(", "{"].includes(tokens.shift())) {
    layer++;
    if (tree[layer] == undefined) {
      tree[layer] = [];
    }
  } else if ([")", "}"].includes(tokens.shift())) {
    layer--;
  }

  if (["EOF", undefined].includes(tokens[0])) {
    return tree;
  }

  if (tokens[0] == "let") {
    let expression = grammar.let_declaration(tokens);
    tree[layer].push(expression);
  } else {
    throw new Error(`Unexpected token: ${tokens[0]}`);
  }

  return lex(tokens, tree);
}
