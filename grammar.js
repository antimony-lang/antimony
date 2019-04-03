module.exports = {
  expr: function(tokens) {
    let tree = [];
    if (tokens[0] == "let") {
      tree.push(this.let_declaration(tokens));
    } else {
      throw new Error(`Unexpected token: ${tokens[position]}`);
    }
    return tree;
  },

  /**
   * Returns an AST that represents the given let declaration
   * @param {String[]} tokens
   */
  let_declaration: function(tokens) {
    return {
      type: "let_declaration",
      name: tokens[1],
      value: tokens[3]
    };
  }
};
