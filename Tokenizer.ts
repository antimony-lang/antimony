import {
  Token,
  TokenType,
  EOL,
  Keyword,
  Variable,
  Whitespace,
  Number as Num,
  Str
} from "./tokens";

export default class Tokenizer {
  code: string;
  current: Token | undefined;
  index: number;

  visitor = {
    number: (character: string): Num => {
      let token = character;
      while (Number(this.peek())) {
        token += this.take();
      }
      return new Num(token);
    },

    string: (character: string): Token => {
      let token = character;
      while (this.peek() != '"') {
        if (["\n", "EOF"].includes(this.peek())) {
          throw new Error("Expected '\"' but found " + "\\n");
        }
        token += this.take();
      }
      // Append right quote
      token += this.take();

      return new Str(token);
    }
  };

  constructor(code: string) {
    this.code = code;
    // Mark end of file
    this.code += " EOF";
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
        return new Token(TokenType.plus, character);
      case "-":
        return new Token(TokenType.minus, character);
      case "*":
        return new Token(TokenType.multiply, character);
      case "/":
        return new Token(TokenType.divide, character);
      case "=":
        return new Token(TokenType.assignment, character);
      case '"':
        return this.visitor.string(character);
      default:
        break;
    }

    if (Number(character)) {
      return this.visitor.number(character);
    }

    if (this.isWhitespace(character)) {
      return new Whitespace(character);
    }

    if (character == ";") {
      return new EOL(character);
    }

    if (this.isLetter(character)) {
      let token = character;
      while (this.isLetter(this.peek())) {
        token += this.take();
      }

      if (new Keyword(token).type) return new Keyword(token);
      else return new Variable(token);
    }

    throw new Error("Could not resolve token: " + character);
  }

  private isWhitespace(character: string): boolean {
    return character == " " || character == "\n" || character == "\t";
  }

  private isLetter(str: string) {
    if (!str) {
      return false;
    }
    return str.length === 1 && str.match(/[a-zA-Z_]/i);
  }
}
