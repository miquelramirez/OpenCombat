use battle_core::{behavior::Behavior, order::Order, state::battle::message::SideEffect};

use super::{message::RunnerMessage, Runner};

impl Runner {
    pub fn react(&mut self, messages: &Vec<RunnerMessage>) {
        // TODO : Side effects should not exists : All side effects
        // should be computed when original message is produced
        let mut side_effects = vec![];

        for message in messages {
            match message {
                RunnerMessage::BattleState(state_message) => {
                    side_effects.extend(self.battle_state.react(state_message, self.frame_i));
                }
                // These messages are destined to be directly sent to clients
                RunnerMessage::ClientsState(_) | RunnerMessage::ClientState(_, _) => {}
            }
        }

        for side_effect in &side_effects {
            self.side_effect(side_effect)
        }
    }

    // TODO : Side effects should not exists : All side effects
    // should be computed when original message is produced
    pub fn side_effect(&mut self, side_effect: &SideEffect) {
        match side_effect {
            SideEffect::SoldierFinishHisBehavior(soldier_index) => {
                let soldier = self.battle_state.soldier_mut(*soldier_index);
                soldier.set_behavior(Behavior::Idle);
                soldier.set_order(Order::Idle);
            }
            // Server ignore this side effect because concern Gui only
            SideEffect::RefreshEntityAnimation(_) => {}
        }
    }
}