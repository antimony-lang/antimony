/**
 * Copyright 2020 Garrit Franke
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::lexer::Position;

pub fn highlight_position_in_file(input: String, position: Position) -> String {
    let mut buf = String::new();

    let line = input.lines().nth(position.line - 1).unwrap();
    // TODO: do something better, code can be more than 9999 lines
    buf.push_str(&format!("{:>4} | {}\n", position.line, line));
    buf.push_str("     | ");

    buf.push_str(
        &line
            .chars()
            .take(position.offset - 1)
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect::<String>(),
    );
    buf.push('^');

    buf
}
