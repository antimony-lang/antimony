const { LetDeclaration, Operation } = require("./syntax");

module.exports = {
  expr: function(tokens) {
    let tree = [];
    if (tokens[0] == "let") {
      tree.push(this.let_declaration(tokens));
    } else {
      throw new Error(`Error parsing expression: ${tokens}`);
    }
    return tree;
  },

  /**
   * Returns an AST that represents the given let declaration
   * @param {string[]} tokens
   */
  let_declaration: function(tokens) {
    return new LetDeclaration(tokens[1], [...tokens].slice(3, tokens.length));
  },

  /**
   * Returns an AST that represents the given math operation
   * @param {string[]} tokens
   */
  operation: function(tokens) {
    return tokens.length > 1
      ? new Operation(tokens[0], tokens[1], tokens[2])
      : tokens;
  }
};
