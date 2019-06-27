import { Token, TokenType } from "./tokens";

export default class Tokenizer {
  code: string;
  current: Token;
  index: number;

  get hasMore(): boolean {
    return false;
  }

  constructor(code: string) {
    this.code = code;
    this.index = 0;
  }

  advance() {
    this.current = this.takeNext();
  }

  takeNext(): Token {
    let character = this.code[this.index];

    switch (character) {
      case "+":
        return new Token(TokenType.Plus, character);
      case "-":
        return new Token(TokenType.Minus, character);
      case "*":
        return new Token(TokenType.Multiply, character);
      case "/":
        return new Token(TokenType.Divide, character);
      default:
        break;
    }

    if (Number(character)) {
      return new Token(TokenType.Number, character);
    }

    if (this.isWhitespace(character)) {
      return new Token(TokenType.Whitespace, character);
    }
  }

  private isWhitespace(character: string): boolean {
    return character == " " || character == "\n" || character == "\t";
  }
}
