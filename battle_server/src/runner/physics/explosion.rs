use battle_core::audio::Sound;
use battle_core::entity::soldier::Soldier;
use battle_core::entity::vehicle::Vehicle;
use battle_core::game::explosive::ExplosiveType;
use battle_core::physics::event::explosion::Explosion;
use battle_core::physics::utils::distance_between_points;
use battle_core::state::client::ClientStateMessage;

use battle_core::types::Distance;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::runner::message::RunnerMessage;
use crate::runner::Runner;

impl Runner {
    pub fn tick_explosions(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_explosions");
        let mut messages = vec![];

        for explosion in self.battle_state.explosions() {
            if explosion.effective(*self.battle_state.frame_i()) {
                messages.extend(self.explosion_effects(explosion))
            }
        }

        messages
    }

    // TODO : find algorithm kill/injure about explosives + terrain + position
    fn explosion_effects(&self, explosion: &Explosion) -> Vec<RunnerMessage> {
        puffin::profile_scope!(
            "explosion_effects",
            format!("start={} end={}", explosion.start(), explosion.end())
        );
        let mut messages = vec![];
        let point = explosion.point();
        let explosive_type = explosion.type_();

        for soldier in self.battle_state.soldiers() {
            if !soldier.can_feel_explosion() {
                continue;
            }

            // Simple for now, but if in vehicle, don't be affected
            if self
                .battle_state
                .soldier_vehicle_place(soldier.uuid())
                .is_some()
            {
                continue;
            }

            let distance = distance_between_points(&soldier.world_point(), point);

            // TODO : Move into dedicated struct ?
            if let (
                Some(direct_death_rayons),
                Some(regressive_death_rayon),
                Some(regressive_injured_rayon),
            ) = (
                self.config
                    .explosive_direct_death_rayon
                    .get(explosion.type_()),
                self.config
                    .explosive_regressive_death_rayon
                    .get(explosion.type_()),
                self.config
                    .explosive_regressive_injured_rayon
                    .get(explosion.type_()),
            ) {
                if &distance < direct_death_rayons {
                    messages.extend(self.killing_blast_effects(soldier));
                } else if &distance <= regressive_death_rayon
                    || &distance <= regressive_injured_rayon
                {
                    let mut rng = rand::thread_rng();
                    let percent = 1.0
                        - (distance.millimeters() as f32
                            / regressive_death_rayon.millimeters() as f32);
                    let roll = rng.gen_range(0.0..1.0);

                    if roll <= percent {
                        messages.extend(self.killing_blast_effects(soldier));
                    } else {
                        let percent = 1.0
                            - (distance.millimeters() as f32
                                / regressive_injured_rayon.millimeters() as f32);
                        let roll = rng.gen_range(0.0..1.0);

                        if roll <= percent {
                            messages.extend(self.stunning_blast_effects(soldier));
                        } else {
                            messages.extend(self.proximity_blast_effects(soldier, distance));
                        }
                    }
                } else if distance.meters() < 100 {
                    messages.extend(self.proximity_blast_effects(soldier, distance));
                }
            }
        }

        for vehicle in self.battle_state.vehicles() {
            if vehicle.chassis_shape().contains(point) {
                messages.extend(self.vehicle_shell_impact_effects(vehicle, explosive_type));
            }
        }

        messages
    }

    fn killing_blast_effects(&self, soldier: &Soldier) -> Vec<RunnerMessage> {
        puffin::profile_scope!("killing_blast_effects", soldier.uuid().to_string());
        let mut messages = self.soldier_die(soldier.uuid());

        if soldier.can_produce_sound() {
            let pick_from = vec![
                Sound::MaleScreaming1,
                Sound::MaleScreaming2,
                Sound::MaleScreaming3,
                Sound::MaleScreaming4,
                Sound::MaleScreaming5,
                Sound::MaleDie1,
                Sound::MaleDie2,
                Sound::MaleDie3,
                Sound::MaleDie4,
                Sound::MaleDie5,
                Sound::MaleDie6,
                Sound::MaleDie7,
                Sound::MaleDie8,
            ];
            messages.push(RunnerMessage::ClientsState(
                ClientStateMessage::PlayBattleSound(
                    *pick_from
                        .choose(&mut rand::thread_rng())
                        .expect("Must one be chosen"),
                ),
            ))
        }

        messages
    }

    fn stunning_blast_effects(&self, soldier: &Soldier) -> Vec<RunnerMessage> {
        puffin::profile_scope!("stunning_blast_effects", soldier.uuid().to_string());
        self.soldier_blast_stunned(soldier.uuid())
    }

    fn proximity_blast_effects(&self, soldier: &Soldier, distance: Distance) -> Vec<RunnerMessage> {
        puffin::profile_scope!("proximity_blast_effects", soldier.uuid().to_string());
        self.soldier_blast(soldier.uuid(), distance)
    }

    fn vehicle_shell_impact_effects(
        &self,
        vehicle: &Vehicle,
        _explosive: &ExplosiveType,
    ) -> Vec<RunnerMessage> {
        puffin::profile_scope!("vehicle_shell_impact_effects", vehicle.uuid().to_string());
        // TODO effects on soldiers (with a real explosive algorithm)
        let pick_from = vec![Sound::MetalHit1];
        vec![RunnerMessage::ClientsState(
            ClientStateMessage::PlayBattleSound(
                *pick_from
                    .choose(&mut rand::thread_rng())
                    .expect("Must one be chosen"),
            ),
        )]
    }
}
