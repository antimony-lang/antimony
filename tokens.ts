export enum TokenType {
  Nop = "Nop",
  Variable = "Variable",
  Number = "Number",
  Whitespace = " ",
  Let = "let",
  Assignment = "=",
  Plus = "+",
  Minus = "-",
  Multiply = "*",
  Divide = "/",
  True = "true",
  False = "false",
  If = "if",
  Curly_left = "{",
  Curly_right = "}",
  Semicolon = ";",
  Linebreak = "\n"
}

export class Token {
  type: TokenType;
  value: string;

  constructor(type: TokenType, value: string) {
    this.type = type;
    this.value = value;
  }
}

export class Variable extends Token {
  constructor(value: string) {
    super(TokenType.Variable, value);
  }
}

export class Keyword extends Token {
  constructor(value: string) {
    let type: TokenType = TokenType[value];
    super(type, value);
  }
}

export class EOL extends Token {
  constructor(value: string) {
    super(TokenType.Semicolon, value);
  }
}
