use ggegui::egui::{Context as EguiContext, Grid, Ui};
use ggez::Context;

use crate::{debug::DebugTerrain, engine::message::EngineMessage, engine::Engine};

impl Engine {
    pub fn debug_gui_terrain(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<EngineMessage> {
        Grid::new("terrain_draw")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Decor");
                ui.checkbox(&mut self.gui_state.draw_decor, "");
                ui.end_row();

                ui.label("Draw");
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.gui_state.debug_terrain,
                        DebugTerrain::None,
                        "Normal",
                    );
                    ui.radio_value(
                        &mut self.gui_state.debug_terrain,
                        DebugTerrain::Tiles,
                        "Tiles",
                    );
                    ui.radio_value(
                        &mut self.gui_state.debug_terrain,
                        DebugTerrain::Opacity,
                        "Opacity",
                    );
                });
                ui.end_row();
            });

        vec![]
    }
}
