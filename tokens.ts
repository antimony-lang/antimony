export enum TokenType {
  nop = "Nop",
  variable = "Variable",
  number = "Number",
  whitespace = " ",
  let = "let",
  assignment = "=",
  plus = "+",
  minus = "-",
  multiply = "*",
  divide = "/",
  true = "true",
  false = "false",
  if = "if",
  curly_left = "{",
  curly_right = "}",
  semicolon = ";",
  linebreak = "\n"
}

export class Token {
  type: TokenType;
  value: string;

  constructor(type: TokenType, value: string) {
    this.type = type;
    this.value = value;
  }
}

export class Number extends Token {
  constructor(value: string) {
    super(TokenType.number, value);
  }
}

export class Variable extends Token {
  constructor(value: string) {
    super(TokenType.variable, value);
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
    super(TokenType.semicolon, value);
  }
}

export class Whitespace extends Token {
  constructor(value: string) {
    super(TokenType.whitespace, value);
  }
}
