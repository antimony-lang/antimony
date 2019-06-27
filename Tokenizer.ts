import { Token, TokenType, EOL } from "./tokens";

export default class Tokenizer {
  code: string;
  current: Token | undefined;
  index: number;

  constructor(code: string) {
    this.code = code;
    this.index = -1;
  }

  get hasMore(): boolean {
    return this.peek() != undefined;
  }

  peek(): string {
    return this.code[this.index + 1];
  }

  run(): Token[] {
    let tokens: Token[] = [];
    while (this.hasMore) {
      tokens.push(this.takeNext());
    }

    return tokens;
  }

  take(): string {
    this.index++;
    return this.code[this.index];
  }

  takeNext(): Token {
    let character = this.take();

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
      let token = character;
      while (Number(this.peek())) {
        token += this.take();
      }
      return new Token(TokenType.Number, token);
    }

    if (this.isWhitespace(character)) {
      return new Token(TokenType.Whitespace, character);
    }

    if (character == ";") {
      return new EOL(character);
    }

    throw new Error("Could not resolve token: " + character);
  }

  private isWhitespace(character: string): boolean {
    return character == " " || character == "\n" || character == "\t";
  }
}
