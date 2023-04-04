use std::fmt::Display;

use battle_core::{
    audio::Sound,
    types::{Offset, WindowPoint},
};
use ggez::{event::MouseButton, input::keyboard::KeyInput, winit::event::VirtualKeyCode, Context};

use crate::{
    debug::DebugPhysics,
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
                GuiStateMessage::ApplyOnDisplaySceneOffset(Offset::new(
                    move_mode.to_pixels_offset(),
                    0.,
                )),
            ));
        }
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Right) {
            messages.push(EngineMessage::GuiState(
                GuiStateMessage::ApplyOnDisplaySceneOffset(Offset::new(
                    -move_mode.to_pixels_offset(),
                    0.,
                )),
            ));
        }
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Up) {
            messages.push(EngineMessage::GuiState(
                GuiStateMessage::ApplyOnDisplaySceneOffset(Offset::new(
                    0.,
                    move_mode.to_pixels_offset(),
                )),
            ));
        }
        if ctx.keyboard.is_key_pressed(VirtualKeyCode::Down) {
            messages.push(EngineMessage::GuiState(
                GuiStateMessage::ApplyOnDisplaySceneOffset(Offset::new(
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
            Some(VirtualKeyCode::F4) => {
                messages.push(EngineMessage::MakeASave);
            }
            Some(VirtualKeyCode::F5) => {
                messages.push(EngineMessage::TryLoadLastSave);
            }
            Some(VirtualKeyCode::F12) => {
                messages.push(EngineMessage::GuiState(
                    GuiStateMessage::SetDisplayDebugGui(!self.gui_state.display_debug_gui()),
                ));
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
                        GuiStateMessage::ApplyOnDisplaySceneOffset(Offset::from_vec2(
                            cursor_point.to_vec2() - last_cursor_point.to_vec2(),
                        )),
                    ))
                }

                // Begin drag squad if this mouve is after a click on soldier and cursor is still hover the soldier
                if self.battle_state.phase().placement() && self.gui_state.dragged_squad().is_none()
                {
                    let world_point = self.gui_state.get_current_cursor_world_point();
                    if let (Some(started_with_soldier_index), Some(soldier_index)) = (
                        self.gui_state.begin_click_on_soldier(),
                        self.get_soldiers_at_point(world_point).first(),
                    ) {
                        if started_with_soldier_index == started_with_soldier_index {
                            let squad_index =
                                self.battle_state.soldier(*soldier_index).squad_uuid();
                            messages.push(EngineMessage::GuiState(GuiStateMessage::SetDragSquad(
                                Some(squad_index),
                            )));
                        }
                    }
                }
            }
        }

        let point = self.gui_state.get_current_cursor_window_point();
        let cursor_in_control = self.hud.contains(&vec![&point]);
        messages.push(EngineMessage::GuiState(GuiStateMessage::SetCursorInHud(
            cursor_in_control,
        )));

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
                if self.gui_state.is_controlling(&Control::Soldiers)
                    && !self.gui_state.cursor_in_hud()
                {
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

                    if self.battle_state.phase().placement() && !self.gui_state.cursor_in_hud() {
                        let world_point = self.gui_state.get_current_cursor_world_point();
                        if let Some(soldier_index) = self.get_soldiers_at_point(world_point).first()
                        {
                            messages.push(EngineMessage::GuiState(
                                GuiStateMessage::SetBeginClickOnSoldier(Some(*soldier_index)),
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
                let end_point = WindowPoint::new(x, y);
                let start_point = self
                    .gui_state
                    .get_left_click_down_window_point()
                    .unwrap_or(end_point);

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

                if let Some(squad_index) = self.gui_state.dragged_squad() {
                    if self.gui_state.cursor_in_hud() {
                        messages.push(EngineMessage::PlaySound(Sound::Bip1))
                    } else {
                        let world_end_point =
                            self.gui_state.world_point_from_window_point(end_point);
                        messages.push(EngineMessage::GuiState(GuiStateMessage::PushUIEvent(
                            UIEvent::DropSquadTo(*squad_index, world_end_point),
                        )));
                    }
                }

                if let Some(component) = self.hud.hovered_by(&vec![&start_point, &end_point]) {
                    if let Some(event) = component.event() {
                        messages.extend(self.hud_event(event));
                    }
                    if let Some(sound) = component.sound() {
                        messages.push(EngineMessage::PlaySound(sound));
                    }
                };

                messages.push(EngineMessage::GuiState(GuiStateMessage::SetDragSquad(None)));
                messages.push(EngineMessage::GuiState(
                    GuiStateMessage::SetBeginClickOnSoldier(None),
                ))
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

    pub fn collect_mouse_wheel(&self, ctx: &mut Context, _x: f32, y: f32) -> Vec<EngineMessage> {
        let mut messages = vec![];

        if !self.gui_state.debug_gui_hovered && !self.gui_state.cursor_in_hud() {
            let zoom = if y > 0. {
                self.gui_state.zoom.next()
            } else {
                self.gui_state.zoom.previous()
            };

            let half_screen_width = ctx.gfx.drawable_size().0 / 2.;
            let half_screen_height = ctx.gfx.drawable_size().1 / 2.;
            let world_point = self
                .gui_state
                .world_point_from_window_point(WindowPoint::new(
                    half_screen_width,
                    half_screen_height,
                ));

            let half_screen_width = ctx.gfx.drawable_size().0 / 2.;
            let half_screen_height = ctx.gfx.drawable_size().1 / 2.;
            let new_offset_x = -(world_point.x * zoom.factor()) + half_screen_width;
            let new_offset_y = -(world_point.y * zoom.factor()) + half_screen_height;

            messages.push(EngineMessage::GuiState(GuiStateMessage::SetZoom(zoom)));
            messages.push(EngineMessage::GuiState(
                GuiStateMessage::SetDisplaySceneOffset(Offset::new(new_offset_x, new_offset_y)),
            ));
        }

        messages
    }
}
