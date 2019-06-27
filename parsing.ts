import { Token } from "./tokens";

export function parse(input: string) {
  return lex(tokenize(input));
}

export function tokenize(input: string) {
  let tokens: Token[] = [];

  return tokens;
}

export function lex(tokens: Token[]) {
  return tokens;
}
