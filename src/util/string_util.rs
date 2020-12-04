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
