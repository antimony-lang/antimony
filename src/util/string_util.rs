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
    // TODO: Chain without collecting in between
    input
        .chars()
        .skip(position.raw)
        .take_while(|c| c != &'\n')
        .collect::<String>()
        .chars()
        .rev()
        .take_while(|c| c != &'\n')
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>()
}
