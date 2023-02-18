use std::fmt::Display;

use battle_core::types::{Offset, WindowPoint};
use ggez::{event::MouseButton, input::keyboard::KeyInput, winit::event::VirtualKeyCode, Context};

use crate::{
    debug::{DebugPhysics, DebugTerrain},
    engine::{event::UIEvent, message::GuiStateMessage},
};
use serde::{Deserialize, Serialize};

use super::{message::EngineMessage, Engine};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Control {
    Soldiers,
    Map,
    Physics,
}

impl Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Control::Soldiers => f.write_str("Soldiers"),
            Control::Map => f.write_str("Map"),
            Control::Physics => f.write_str("Physics"),
        }
    }
}

enum MoveScreenMode {
    Normal,
    Speed,
}

impl MoveScreenMode {
    pub fn to_pixels_offset(&self) -> f32 {
        match self {
            MoveScreenMode::Normal => 2.0,
            MoveScreenMode::Speed => 15.0,
        }
    }
}

impl Engine {
    pub fn collect_player_inputs(&self, ctx: &mut Context) -> Vec<EngineMessage> {
        puffin::profile_scope!("collect_player_inputs");
        let mut messages = vec![];

        messages.extend(self.collect_keyboard_inputs(ctx));

        // Useful for actions expecting cursor immobilization
        let cursor_immobile_since =
            self.gui_state.get_frame_i() - self.gui_state.get_last_cursor_move_frame();
        messages.push(EngineMessage::GuiState(GuiStateMessage::PushUIEvent(
            UIEvent::ImmobileCursorSince(cursor_immobile_since),
        )));

        messages
    }

    fn collect_keyboard_inputs(&self, ctx: &mut Context) -> Vec<EngineMessage> {
        let mut messages = vec![];

        let shift_pressed = ctx.keyboard.is_key_pressed(VirtualKeyCode::LShift)
            || ctx.keyboard.is_key_pressed(VirtualKeyCode::RShift);
        let move_mode = if shift_pressed {
            MoveScreenMode::Speed
        } else {
            MoveScreenMode::Normal
        };

        // Move battle scene on the window according to user keys
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Left) {
            messages.push(EngineMessage::GuiState(
                GuiStateMessage::ApplyOnSceneDisplayOffset(Offset::new(
                    move_mode.to_pixels_offset(),
                    0.,
                )),
            ));
        }
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Right) {
            messages.push(EngineMessage::GuiState(
                GuiStateMessage::ApplyOnSceneDisplayOffset(Offset::new(
                    -move_mode.to_pixels_offset(),
                    0.,
                )),
            ));
        }
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Up) {
            messages.push(EngineMessage::GuiState(
                GuiStateMessage::ApplyOnSceneDisplayOffset(Offset::new(
                    0.,
                    move_mode.to_pixels_offset(),
                )),
            ));
        }
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Down) {
            messages.push(EngineMessage::GuiState(
                GuiStateMessage::ApplyOnSceneDisplayOffset(Offset::new(
                    0.,
                    -move_mode.to_pixels_offset(),
                )),
            ));
        }

        messages
    }

    pub fn collect_key_pressed(&self, _ctx: &mut Context, input: KeyInput) -> Vec<EngineMessage> {
        let mut messages = vec![];

        if input.keycode == Some(VirtualKeyCode::LControl)
            || input.keycode == Some(VirtualKeyCode::RControl)
        {
            messages.push(EngineMessage::GuiState(GuiStateMessage::SetControl(
                Control::Map,
            )))
        }

        messages
    }

    pub fn collect_key_released(&self, _ctx: &mut Context, input: KeyInput) -> Vec<EngineMessage> {
        let mut messages = vec![];

        match input.keycode {
            Some(VirtualKeyCode::F12) => {
                messages.push(EngineMessage::GuiState(
                    GuiStateMessage::SetDisplayDebugGui(!self.gui_state.display_debug_gui()),
                ));
            }
            Some(VirtualKeyCode::F11) => {
                let new_debug_terrain = match self.gui_state.get_debug_terrain() {
                    DebugTerrain::None => DebugTerrain::Opacity,
                    DebugTerrain::Opacity => DebugTerrain::Tiles,
                    DebugTerrain::Tiles => DebugTerrain::None,
                };
                messages.push(EngineMessage::GuiState(GuiStateMessage::SetDebugTerrain(
                    new_debug_terrain,
                )));
            }
            Some(VirtualKeyCode::F10) => {
                let new_debug_physics = self.gui_state.get_debug_physics().next();
                messages.extend(vec![
                    EngineMessage::GuiState(GuiStateMessage::SetControl(
                        self.physics_control(&new_debug_physics),
                    )),
                    EngineMessage::GuiState(GuiStateMessage::SetDebugPhysics(new_debug_physics)),
                ]);
            }
            Some(VirtualKeyCode::F9) => {
                messages.push(EngineMessage::GuiState(GuiStateMessage::ChangeSide))
            }
            Some(VirtualKeyCode::LControl) | Some(VirtualKeyCode::RControl) => messages.push(
                EngineMessage::GuiState(GuiStateMessage::SetControl(self.determine_controlling())),
            ),
            _ => {}
        };

        messages
    }

    pub fn determine_controlling(&self) -> Control {
        match self.gui_state.get_debug_physics() {
            DebugPhysics::None => Control::Soldiers,
            DebugPhysics::MosinNagantM1924GunFire | DebugPhysics::BrandtMle2731Shelling => {
                Control::Physics
            }
        }
    }

    pub fn collect_mouse_motion(
        &self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        // Instant information
        let cursor_point = WindowPoint::new(x, y);

        // Update cursor position at each frames
        messages.extend(vec![
            EngineMessage::GuiState(GuiStateMessage::SetCursorPoint(cursor_point)),
            EngineMessage::GuiState(GuiStateMessage::PushUIEvent(UIEvent::CursorMove(
                cursor_point,
            ))),
        ]);

        if let Some(left_click_down) = self.gui_state.get_left_click_down_window_point() {
            if left_click_down != &cursor_point {
                messages.push(EngineMessage::GuiState(
                    GuiStateMessage::SetCurrentCursorVector(Some((*left_click_down, cursor_point))),
                ));
                if self.gui_state.is_controlling(&Control::Map) {
                    let last_cursor_point = self.gui_state.get_current_cursor_window_point();
                    messages.push(EngineMessage::GuiState(
                        GuiStateMessage::ApplyOnSceneDisplayOffset(Offset::from_vec2(
                            cursor_point.to_vec2() - last_cursor_point.to_vec2(),
                        )),
                    ))
                }
            }
        }

        messages
    }

    pub fn collect_mouse_down(
        &self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];
        let cursor = WindowPoint::new(x, y);

        match button {
            MouseButton::Left => {
                // Update cursor down position
                messages.push(EngineMessage::GuiState(GuiStateMessage::SetLeftClickDown(
                    Some(cursor.clone()),
                )));

                // Check if any order under the cursor
                if self.gui_state.is_controlling(&Control::Soldiers) {
                    for (order, order_marker, squad_id, world_point, order_marker_i) in
                        self.battle_state.order_markers(self.gui_state.side())
                    {
                        let world_shape =
                            self.order_marker_selection_shape(&order, &order_marker, &world_point);
                        if self
                            .gui_state
                            .window_shape_from_world_shape(&world_shape)
                            .contains(&cursor)
                        {
                            let pending_order = self.create_pending_order_from_order_marker(
                                &order_marker,
                                &squad_id,
                                &Some(order_marker_i),
                                &vec![],
                            );
                            messages.push(EngineMessage::GuiState(
                                GuiStateMessage::SetPendingOrder(Some(pending_order)),
                            ));
                        }
                    }
                }
            }
            _ => {}
        }

        messages
    }

    pub fn collect_mouse_up(
        &self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        match button {
            MouseButton::Left => {
                let start_point = match self.gui_state.get_left_click_down_window_point() {
                    Some(start_point) => *start_point,
                    None => {
                        // No left button down before button up ?!
                        return vec![];
                    }
                };
                let end_point = WindowPoint::new(x, y);

                // No more longer left click down or current drag
                messages.extend(vec![
                    EngineMessage::GuiState(GuiStateMessage::SetLeftClickDown(None)),
                    EngineMessage::GuiState(GuiStateMessage::SetCurrentCursorVector(None)),
                ]);

                // Determine if it is a simple click or a drag
                if start_point != end_point {
                    messages.push(EngineMessage::GuiState(GuiStateMessage::PushUIEvent(
                        UIEvent::FinishedCursorVector(start_point, end_point),
                    )));
                } else {
                    messages.push(EngineMessage::GuiState(GuiStateMessage::PushUIEvent(
                        UIEvent::FinishedCursorLeftClick(end_point),
                    )));
                }
            }
            MouseButton::Right => {
                messages.push(EngineMessage::GuiState(GuiStateMessage::PushUIEvent(
                    UIEvent::FinishedCursorRightClick(WindowPoint::new(x, y)),
                )));
            }
            _ => {}
        }

        messages
    }

    pub fn collect_mouse_wheel(&self, _ctx: &mut Context, _x: f32, y: f32) -> Vec<EngineMessage> {
        let mut messages = vec![];

        messages.push(EngineMessage::GuiState(GuiStateMessage::ScaleUpdate(
            y / 10.0,
        )));

        messages
    }
}