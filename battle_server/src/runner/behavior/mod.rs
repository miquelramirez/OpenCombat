use battle_core::{
    behavior::{Behavior, BehaviorMode, BehaviorPropagation},
    entity::soldier::Soldier,
    order::Order,
    state::{
        battle::message::{BattleStateMessage, SoldierMessage},
        client::ClientStateMessage,
    },
    types::{Angle, SquadUuid, WorldPaths, WorldPoint},
    utils::DebugPoint,
};

use super::{message::RunnerMessage, Runner};

mod blast;
mod bullet;
mod death;
mod defend;
mod engage;
mod moves;
mod suppress;

impl Runner {
    pub fn soldier_behavior(&self, soldier: &Soldier) -> Vec<RunnerMessage> {
        puffin::profile_scope!("soldier_behavior");
        let mut messages = vec![];
        let soldier = self.battle_state.soldier(soldier.uuid());

        let behavior = match soldier.order() {
            Order::Idle => self.idle_behavior(soldier),
            Order::MoveTo(paths) => self.move_behavior(soldier, paths),
            Order::MoveFastTo(paths) => self.move_fast_behavior(soldier, paths),
            Order::SneakTo(paths) => self.sneak_to_behavior(soldier, paths),
            Order::Defend(angle) => self.defend_behavior(soldier, angle),
            Order::Hide(angle) => self.hide_behavior(soldier, angle),
            Order::EngageSquad(squad_index) => self.engage_behavior(soldier, squad_index),
            Order::SuppressFire(point) => self.suppress_fire_behavior(soldier, point),
        };

        // In case of squad leader and regularly propagation
        if self.soldier_is_squad_leader(soldier.uuid())
            && behavior.propagation() == BehaviorPropagation::Regularly
        {
            // Order must be propagated to squad members
            messages.extend(self.propagate_behavior(soldier, &behavior));
        }

        // Change behavior if computed behavior is different
        if &behavior != soldier.behavior() {
            // In case of squad leader and regularly propagation
            if self.soldier_is_squad_leader(soldier.uuid())
                && behavior.propagation() == BehaviorPropagation::OnChange
            {
                // Order must be propagated to squad members
                messages.extend(self.propagate_behavior(soldier, &behavior));
            }

            messages.extend(vec![RunnerMessage::BattleState(
                BattleStateMessage::Soldier(soldier.uuid(), SoldierMessage::SetBehavior(behavior)),
            )]);
        };

        messages
    }

    pub fn propagate_behavior(&self, leader: &Soldier, behavior: &Behavior) -> Vec<RunnerMessage> {
        assert!(self.soldier_is_squad_leader(leader.uuid()));
        let mut messages = vec![];
        let mut debug_points: Vec<DebugPoint> = vec![];

        let orders: Vec<(&Soldier, Order)> = match behavior {
            Behavior::MoveTo(_) | Behavior::MoveFastTo(_) | Behavior::SneakTo(_) => {
                match self.battle_state.soldier_behavior_mode(leader) {
                    BehaviorMode::Ground => self.propagate_move(leader.squad_uuid(), &behavior),
                    BehaviorMode::Vehicle => self.propagate_drive(leader.squad_uuid(), &behavior),
                }
            }
            Behavior::Defend(_) => {
                //
                let (orders, debug_points_) = self.propagate_defend(leader.squad_uuid(), &behavior);
                debug_points.extend(debug_points_);
                orders
            }
            Behavior::Hide(_) => {
                // TODO : Special behavior for hide
                let (orders, debug_points_) = self.propagate_defend(leader.squad_uuid(), &behavior);
                debug_points.extend(debug_points_);
                orders
            }
            Behavior::DriveTo(_) => todo!(),
            Behavior::RotateTo(_) => todo!(),
            Behavior::Idle | Behavior::Dead | Behavior::Unconscious => {
                vec![]
            }
            Behavior::SuppressFire(point) => {
                self.propagate_suppress_fire(leader.squad_uuid(), point)
            }
            Behavior::EngageSoldier(soldier_index) => {
                self.propagate_engage_soldier(&leader.squad_uuid(), soldier_index)
            }
        };

        for (subordinate, order) in orders {
            // Give order only if different from subordinate current order
            if subordinate.order() != &order {
                messages.extend(vec![RunnerMessage::BattleState(
                    BattleStateMessage::Soldier(
                        subordinate.uuid(),
                        SoldierMessage::SetOrder(order),
                    ),
                )]);
            }
        }

        for debug_point in debug_points {
            messages.push(RunnerMessage::ClientsState(
                ClientStateMessage::PushDebugPoint(debug_point),
            ))
        }

        messages
    }

    pub fn idle_behavior(&self, soldier: &Soldier) -> Behavior {
        if let Some(opponent) = self.get_soldier_opponent(soldier, None) {
            return Behavior::EngageSoldier(opponent.uuid());
        }

        if soldier.under_fire().exist() {
            // TODO : soldier angle
            Behavior::Hide(Angle(0.))
        } else {
            Behavior::Idle
        }
    }

    pub fn move_behavior(&self, soldier: &Soldier, paths: &WorldPaths) -> Behavior {
        if let Some(opponent) = self.get_soldier_opponent(soldier, None) {
            return Behavior::EngageSoldier(opponent.uuid());
        }

        match self.battle_state.soldier_behavior_mode(soldier) {
            BehaviorMode::Ground => {
                if soldier.under_fire().is_warning()
                    || soldier.under_fire().is_danger()
                    || soldier.under_fire().is_max()
                {
                    Behavior::SneakTo(paths.clone())
                } else {
                    Behavior::MoveTo(paths.clone())
                }
            }
            BehaviorMode::Vehicle => Behavior::DriveTo(paths.clone()),
        }
    }

    pub fn move_fast_behavior(&self, soldier: &Soldier, paths: &WorldPaths) -> Behavior {
        if soldier.under_fire().is_danger() || soldier.under_fire().is_max() {
            Behavior::SneakTo(paths.clone())
        } else {
            Behavior::MoveFastTo(paths.clone())
        }
    }

    pub fn sneak_to_behavior(&self, _soldier: &Soldier, paths: &WorldPaths) -> Behavior {
        Behavior::SneakTo(paths.clone())
    }

    pub fn defend_behavior(&self, _soldier: &Soldier, angle: &Angle) -> Behavior {
        Behavior::Defend(*angle)
    }

    pub fn hide_behavior(&self, _soldier: &Soldier, angle: &Angle) -> Behavior {
        Behavior::Hide(*angle)
    }

    pub fn engage_behavior(&self, soldier: &Soldier, squad_index: &SquadUuid) -> Behavior {
        // TODO : If can't see any squad opponent, but can move to covered position, move to it
        if let Some(opponent) = self.get_soldier_opponent(soldier, Some(squad_index)) {
            return Behavior::EngageSoldier(opponent.uuid());
        }

        Behavior::Idle
    }

    pub fn suppress_fire_behavior(&self, _soldier: &Soldier, point: &WorldPoint) -> Behavior {
        Behavior::SuppressFire(*point)
    }
}