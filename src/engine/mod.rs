use ggez::graphics::{self};
use ggez::timer::check_update_time;
use ggez::{event, GameError};
use ggez::{Context, GameResult};
use glam::*;

use crate::config::Config;
use crate::graphics::Graphics;
use crate::network::Network;
use crate::state::State;
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
    /// The current state of the game. This struct is own by server and replicated on clients
    state: State,
    /// Printed frames since start of program
    frame_i: u64,
    /// Offset to apply to battle scene by window relative
    display_scene_offset: Vec2,
    /// Scale to apply to battle scene by window relative
    display_scene_scale: Vec2,
    // Bellow, some player display configurations
    draw_decor: bool,
}

impl Engine {
    pub fn new(config: Config, graphics: Graphics, state: State) -> GameResult<Engine> {
        let network = Network::new(config.clone())?;
        let engine = Engine {
            config,
            network,
            graphics,
            state,
            frame_i: 0,
            display_scene_offset: Vec2::new(0., 0.),
            display_scene_scale: Vec2::new(1., 1.),
            draw_decor: true,
        };
        Ok(engine)
    }

    fn init(&mut self) -> GameResult {
        match self.config.network_mode() {
            // Server own game state, so init it
            crate::NetWorkMode::Server => self.state.init()?,
            // Client initialize its state when received from server
            crate::NetWorkMode::Client => {}
        };

        if let Err(error) = self.network.init() {
            return Err(GameError::CustomError(error.to_string()));
        }

        Ok(())
    }

    fn tick(&mut self) {
        match self.config.network_mode() {
            crate::NetWorkMode::Server => self.tick_as_server(),
            crate::NetWorkMode::Client => self.tick_as_client(),
        }
    }
}

impl event::EventHandler<ggez::GameError> for Engine {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while check_update_time(ctx, self.config.target_fps()) {
            // First thing to do is to initialize the state.
            if self.frame_i == 0 {
                self.init()?;
            }

            // Execute "each frame" code
            self.tick();

            // Increment the frame counter
            self.frame_i += 1;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.graphics.clear(ctx);

        self.generate_map_sprites(self.draw_decor)?;
        self.generate_entities_sprites()?;

        // Draw entire
        let window_draw_param = graphics::DrawParam::new()
            .dest(self.display_scene_offset)
            .scale(self.display_scene_scale);
        self.graphics
            .draw(ctx, self.draw_decor, window_draw_param)?;

        graphics::present(ctx)?;
        Ok(())
    }
}
