module.exports = {
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
