use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../assets/"]
pub struct Assets;

#[derive(Clone, Copy)]
pub enum ServerMessage {
    Ready(u16),
}
