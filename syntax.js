module.exports = {
  LetDeclaration: class {
    constructor(id, value) {
      this.id = id;
      this.value =
        value.length > 1
          ? require("./grammar").operation(
              [...value].splice(2, value.length - 1)
            )
          : value;
    }
  },
  Operation: class {
    constructor(left, operator, right) {
      this.left = left;
      this.operator = operator;
      this.right =
        right instanceof Array && right.length > 1
          ? require("./grammar").operation(
              [...right].splice(1, right.length - 1)
            )
          : right;
    }
  }
};
