use battle_core::{
    behavior::{Behavior, Body},
    order::Order,
    state::battle::message::{BattleStateMessage, SoldierMessage},
    types::{SoldierIndex, WorldPaths},
};

use super::{message::RunnerMessage, Runner};

impl Runner {
    pub fn movement_updates(
        &self,
        soldier_index: SoldierIndex,
        path: &WorldPaths,
    ) -> Vec<RunnerMessage> {
        let mut messages = vec![];
        let soldier = self.battle_state.soldier(soldier_index);
        let point = path.next_point().expect("Must have point in path");

        // There is a next point in path, go to it
        let velocity = self
            .config
            .behavior_velocity(soldier.behavior())
            .expect("Entity behavior must have velocity when move code called");
        let vector = (point.to_vec2() - soldier.world_point().to_vec2()).normalize() * velocity;

        // Point reached
        if vector.is_nan()
            || (soldier.world_point().to_vec2() - point.to_vec2()).length() <= vector.length()
        {
            // If it is the last point, move is finished
            if path.is_last_point().expect("Must contain points") {
                let (behavior, order) = if let Some(then_order) = soldier.order().then() {
                    (
                        Behavior::from_order(&then_order, soldier, &self.battle_state),
                        then_order,
                    )
                } else {
                    (
                        Behavior::Idle(Body::from_soldier(soldier, &self.battle_state)),
                        Order::Idle,
                    )
                };

                messages.extend(vec![
                    RunnerMessage::BattleState(BattleStateMessage::Soldier(
                        soldier_index,
                        SoldierMessage::SetBehavior(behavior),
                    )),
                    RunnerMessage::BattleState(BattleStateMessage::Soldier(
                        soldier_index,
                        SoldierMessage::SetOrder(order),
                    )),
                ]);
            } else {
                messages.push(RunnerMessage::BattleState(BattleStateMessage::Soldier(
                    soldier_index,
                    SoldierMessage::ReachBehaviorStep,
                )));
            }

            // Movement required
        } else {
            let new_point = soldier.world_point().apply(vector);
            messages.push(RunnerMessage::BattleState(BattleStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetWorldPosition(new_point),
            )));
        }

        messages
    }
}
