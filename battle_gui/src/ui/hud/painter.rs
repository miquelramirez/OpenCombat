use battle_core::types::WindowPoint;
use ggez::{
    graphics::{Canvas, DrawParam, MeshBuilder},
    Context, GameResult,
};

use crate::{engine::state::GuiState, ui::component::Component};

use super::Hud;

pub struct HudPainter<'a> {
    gui_state: &'a GuiState,
    hud: &'a Hud,
}

impl<'a> HudPainter<'a> {
    pub fn new(hud: &'a Hud, gui_state: &'a GuiState) -> Self {
        Self { hud, gui_state }
    }

    pub fn sprites(&self) -> Vec<DrawParam> {
        let hovered = &self.gui_state.get_current_cursor_window_point();
        [
            self.hud.background().sprites(hovered),
            self.hud.battle().sprites(hovered),
        ]
        .concat()
    }

    pub fn meshes(&self, _ctx: &Context, _mesh_builder: &mut MeshBuilder) -> GameResult<()> {
        Ok(())
    }

    pub fn draw(&self, _ctx: &mut Context, canvas: &mut Canvas) {
        self.hud.battle().draw(self.hover_point(), canvas)
    }

    fn hover_point(&self) -> &WindowPoint {
        self.gui_state.get_current_cursor_window_point()
    }
}