import * as Token from "./tokens";

export class Parser {
  tokens: Token.Token[];
  index: number;
  get current(): Token.Token {
    return this.tokens[this.index];
  }
  ast: any[];

  constructor(tokens: Token.Token[]) {
    this.tokens = tokens;
    this.index = 0;
    this.ast = [];
  }

  parse(): any[] {
    this.ast.push(this.variableDeclaration());
    return this.ast;
  }

  match(type: Token.TokenType) {
    while (this.current instanceof Token.Whitespace) {
      this.advance();
    }

    if (this.current.type != type) {
      throw new Error(
        "Expected '" + type + "' but found '" + this.current.value + "'"
      );
    }

    let currentToken = { ...this.current };
    this.advance();
    return currentToken;
  }

  block() {}

  variableDeclaration(): Token.Token[] {
    let statement = [
      this.match(Token.TokenType.let),
      this.match(Token.TokenType.variable),
      this.match(Token.TokenType.assignment)
    ];

    while (this.current instanceof Token.Whitespace) {
      this.advance();
    }

    if (this.current instanceof Token.Number) {
      statement.push(this.match(Token.TokenType.number));
    } else if (this.current instanceof Token.Str) {
      statement.push(this.match(Token.TokenType.string));
    }
    return statement;
  }

  hasNext(): boolean {
    return !!this.tokens[this.index + 1];
  }

  peek(): Token.Token {
    return this.tokens[this.index + 1];
  }

  advance() {
    this.index++;
  }
}
