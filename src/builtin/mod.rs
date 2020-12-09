use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/builtin/"]
pub struct Builtins;
