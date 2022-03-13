use ggez::graphics::{self};
use ggez::timer::check_update_time;
use ggez::{event, GameError};
use ggez::{Context, GameResult};

use crate::config::Config;
use crate::graphics::Graphics;
use crate::network::Network;
use crate::state::local::LocalState;
use crate::state::shared::SharedState;
mod animate;
mod client;
mod draw;
mod entity;
mod input;
mod network;
mod order;
mod react;
mod server;
mod update;

pub struct Engine {
    config: Config,
    network: Network,
    graphics: Graphics,
    /// The current shared state of the game. This struct is own by server and replicated on clients
    shared_state: SharedState,
    /// The current local state of the game. This struct is own by client and server and are not related
    local_state: LocalState,
}

impl Engine {
    pub fn new(
        config: Config,
        graphics: Graphics,
        shared_state: SharedState,
    ) -> GameResult<Engine> {
        let network = Network::new(config.clone())?;
        let local_state = LocalState::new();
        let engine = Engine {
            config,
            network,
            graphics,
            shared_state,
            local_state,
        };
        Ok(engine)
    }

    fn init(&mut self) -> GameResult {
        match self.config.network_mode() {
            // Server own game shared shared state, so init it
            crate::NetWorkMode::Server => self.shared_state.init()?,
            // Client initialize its shared state when received from server
            crate::NetWorkMode::Client => {}
        };

        if let Err(error) = self.network.init() {
            return Err(GameError::CustomError(error.to_string()));
        }

        Ok(())
    }

    fn tick(&mut self, ctx: &mut Context) {
        match self.config.network_mode() {
            crate::NetWorkMode::Server => self.tick_as_server(ctx),
            crate::NetWorkMode::Client => self.tick_as_client(ctx),
        }
    }
}

impl event::EventHandler<ggez::GameError> for Engine {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while check_update_time(ctx, self.config.target_fps()) {
            // First thing to do is to initialize the shared state.
            if self.local_state.frame_i == 0 {
                self.init()?;
            }

            // Execute "each frame" code
            self.tick(ctx);

            // Increment the frame counter
            self.local_state.frame_i += 1;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.graphics.clear(ctx);

        self.generate_map_sprites(self.local_state.draw_decor)?;
        self.generate_entities_sprites()?;

        // Draw entire
        let window_draw_param = graphics::DrawParam::new()
            .dest(self.local_state.display_scene_offset)
            .scale(self.local_state.display_scene_scale);
        self.graphics
            .draw(ctx, self.local_state.draw_decor, window_draw_param)?;

        graphics::present(ctx)?;
        Ok(())
    }
}
