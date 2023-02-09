use std::error::Error;
use std::fmt::Display;
use std::path;
use std::path::PathBuf;

use battle_core::channel::Channel;
use battle_core::config::GuiConfig;
use battle_core::config::DEFAULT_SERVER_PUB_ADDRESS;
use battle_core::config::DEFAULT_SERVER_REP_ADDRESS;
use battle_core::game::Side;
use battle_core::map::reader::MapReader;
use battle_core::map::reader::MapReaderError;
use battle_core::message::InputMessage;
use battle_core::network::client::Client;
use battle_core::network::error::NetworkError;
use battle_core::state::battle::BattleState;
use crossbeam_channel::SendError;
use ggez::conf::WindowMode;
use ggez::event;
use ggez::GameError;
use server::EmbeddedServer;

mod audio;
mod debug;
mod engine;
mod graphics;
mod physics;
mod server;
mod ui;
mod utils;

use server::EmbeddedServerError;
use structopt::StructOpt;

pub const RESOURCE_PATH: &'static str = "resources";

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt(long = "--embedded-server")]
    embedded_server: bool,

    #[structopt(long = "--server-rep-address", default_value = DEFAULT_SERVER_REP_ADDRESS)]
    server_rep_address: String,

    #[structopt(long = "--server-bind-address", default_value = DEFAULT_SERVER_PUB_ADDRESS)]
    server_pub_address: String,

    #[structopt(long = "side")]
    side: Side,

    #[structopt(long = "profile")]
    profile: bool,

    #[structopt(long = "--profile-address", default_value = "0.0.0.0:8585")]
    profile_address: String,
}

fn main() -> Result<(), GuiError> {
    let opt = Opt::from_args();
    let channel = Channel::new();

    // Hardcoded values (will be dynamic)
    let map_name = "map1";
    let situation_name = "hardcoded";
    let resources = PathBuf::from("./resources");

    // Profiling server
    // NOTE : We must keep server object to avoid its destruction
    let _puffin_server = if opt.profile {
        let puffin_server = puffin_http::Server::new(&opt.profile_address).unwrap();
        puffin::set_scopes_on(true);
        Some(puffin_server)
    } else {
        None
    };

    // Battle server (if embedded)
    if opt.embedded_server {
        EmbeddedServer::new(&resources)
            .map_name(map_name)
            .situation_name(situation_name)
            .server_rep_address(&opt.server_rep_address)
            .server_pub_address(&opt.server_pub_address)
            .server(true)
            .start(&channel)?
    } else {
        let _client = Client::new(
            opt.server_rep_address.clone(),
            opt.server_pub_address.clone(),
            &channel,
        )
        .connect()?;
    }
    channel
        .input_sender()
        .send(vec![InputMessage::RequireCompleteSync])?;

    let context_builder = ggez::ContextBuilder::new("Open Combat", "Bastien Sevajol")
        .add_resource_path(path::PathBuf::from(format!("./{}", RESOURCE_PATH)))
        .window_mode(
            WindowMode::default()
                .dimensions(1024., 768.)
                .resizable(true),
        );
    let (mut context, event_loop) = context_builder.build()?;

    // TODO : If remote server, download map before read it
    let map = MapReader::new(map_name, &resources)?.build()?;
    let config = GuiConfig::new();
    let graphics = graphics::Graphics::new(&mut context, &map, &config)?;
    let battle_state = BattleState::empty(&map);
    let engine = engine::Engine::new(
        &mut context,
        &opt.side,
        config,
        &channel,
        graphics,
        battle_state,
    )?;

    println!("Start Gui");
    event::run(context, event_loop, engine)
}

#[derive(Debug)]
enum GuiError {
    MapReader(MapReaderError),
    RunGame(GameError),
    SendInput(SendError<Vec<InputMessage>>),
    Network(NetworkError),
    EmbeddedServer(EmbeddedServerError),
}

impl Error for GuiError {}

impl From<MapReaderError> for GuiError {
    fn from(error: MapReaderError) -> Self {
        Self::MapReader(error)
    }
}

impl From<GameError> for GuiError {
    fn from(error: GameError) -> Self {
        Self::RunGame(error)
    }
}

impl From<SendError<Vec<InputMessage>>> for GuiError {
    fn from(error: SendError<Vec<InputMessage>>) -> Self {
        Self::SendInput(error)
    }
}

impl From<NetworkError> for GuiError {
    fn from(error: NetworkError) -> Self {
        Self::Network(error)
    }
}

impl From<EmbeddedServerError> for GuiError {
    fn from(error: EmbeddedServerError) -> Self {
        Self::EmbeddedServer(error)
    }
}

impl Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiError::MapReader(error) => {
                f.write_str(&format!("Error during map load : {}", error))
            }
            GuiError::RunGame(error) => f.write_str(&format!("Running error : {}", error)),
            GuiError::SendInput(error) => {
                f.write_str(&format!("Error during input send : {}", error))
            }
            GuiError::Network(error) => f.write_str(&format!("Network error : {}", error)),
            GuiError::EmbeddedServer(error) => {
                f.write_str(&format!("Embedded server error : {}", error))
            }
        }
    }
}
