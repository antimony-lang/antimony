const types = {
  whitespace: " ",
  let: "let",
  variable: undefined,
  number: undefined,
  assignment: "=",
  plus: "+",
  minus: "-",
  multiply: "*",
  divide: "/",
  if: "if",
  curly_left: "{",
  curly_right: "}",
  semicolon: ";",
  linebreak: "\n"
};
exports.types = types;

class Token {
  constructor(type, value) {
    this.type = type;
    this.value = value;
  }
}
exports.Token = Token;

class Variable extends Token {
  constructor(value) {
    this.type = types.variable;
    this.value = value;
  }
}
exports.Variable = Variable;

class Keyword extends Token {
  constructor(value) {
    types.forEach((rawValue, key) => {
      if (value === rawValue) {
        this.type = types[key];
      }
    });

    if (!this.type) {
      throw new Error(value, "could not be parsed");
    }
  }
}
exports.Keyword = Keyword;
