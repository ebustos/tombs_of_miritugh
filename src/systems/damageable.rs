use std::collections::HashMap;

use crate::{
    entity::{
        entities::Ecs,
        entity_id::Entity,
        events::{DamageEvent, DeathEvent},
        impact::{spawn_dust, splatter_blood},
        skull::spawn_skull,
        tags::EntityType,
    },
    game_data::GameData,
    physics::collision::Collision,
    rand_utils::rand_dir,
};
use macroquad::prelude::*;

pub fn update_damageables(ecs: &mut Ecs) {
    let damageables = ecs.check_components(|e, comps| comps.damageables.contains_key(e));

    for damageable_e in &damageables {
        let damageable = ecs.components.damageables.get_mut(damageable_e).unwrap();

        if let Some(invulnerable_timer) = &mut damageable.invulnerable_timer {
            invulnerable_timer.update();
        }

        if let Some(hit_fx_timer) = &mut damageable.hit_fx_timer {
            hit_fx_timer.update();
        }
    }
}

pub fn flash_on_damage(ecs: &mut Ecs) {
    let damageables = ecs.check_components(|e, comps| {
        comps.damageables.contains_key(e) && comps.materials.contains_key(e)
    });

    for damageable_e in &damageables {
        let damageable = ecs.components.damageables.get_mut(damageable_e).unwrap();
        let material = ecs.components.materials.get_mut(damageable_e).unwrap();

        if let Some(hit_fx_timer) = &mut damageable.hit_fx_timer {
            if !hit_fx_timer.completed() {
                let intensity = hit_fx_timer.progress() * 10. + 1.;
                let mut color = WHITE;
                color.r = intensity;
                color.g = intensity;
                color.b = intensity;
                color.a = (0.5 - hit_fx_timer.progress() % 0.5) * 2.;
                // material.set_uniform("color", color);
            } else {
                // material.set_uniform("color", WHITE);
            }
        }
    }
}

pub fn apply_damage(
    data: &mut GameData,
    ecs: &mut Ecs,
    damage_events: &mut Vec<DamageEvent>,
    blood_texture: Texture2D,
) {
    let damageables = ecs.check_components(|e, comps| {
        comps.damageables.contains_key(e) && comps.health.contains_key(e)
    });

    let mut splatter_positions = vec![];

    for damageable_e in &damageables {
        let damageable = ecs.components.damageables.get_mut(damageable_e).unwrap();
        let health = ecs.components.health.get_mut(damageable_e).unwrap();

        let mut event_indices = damage_events
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                if e.target == *damageable_e {
                    return Some(i);
                }
                None
            })
            .collect::<Vec<usize>>();
        event_indices.reverse();

        // TODO: use indices instead
        let event = damage_events.iter().find(|e| e.target == *damageable_e);

        if let Some(event) = event {
            if let Some(invulnerable_timer) = &mut damageable.invulnerable_timer {
                if invulnerable_timer.completed() {
                    health.hp -= event.damage;
                    if let Some(position) = ecs.components.positions.get(damageable_e) {
                        splatter_positions.push(*position);
                    }
                    invulnerable_timer.reset();
                    if let Some(hit_fx_timer) = &mut damageable.hit_fx_timer {
                        hit_fx_timer.reset();
                    }
                }
            }

            for index in event_indices {
                damage_events.remove(index);
            }
        }
    }

    for pos in &splatter_positions {
        splatter_blood(data, blood_texture.clone(), ecs, *pos);
    }

    damage_events.clear();
}

pub fn damage_on_collision(
    ecs: &Ecs,
    damage_events: &mut Vec<DamageEvent>,
    collisions: &HashMap<(Entity, Entity), Collision>,
) {
    let damageables = ecs.check_components(|e, comps| comps.damageables.contains_key(e));

    for damageable_e in &damageables {
        for ((source, target), _collision) in collisions.iter() {
            if target != damageable_e && source != damageable_e {
                continue;
            }
            for (e1, e2) in [(source, target), (target, source)] {
                if let Some(damage_on_coll) = ecs.components.damage_on_collision.get(e1) {
                    let apply_damage = if ecs.components.player_data.contains_key(e2) {
                        damage_on_coll.source == EntityType::Enemy
                    } else {
                        damage_on_coll.source == EntityType::Player
                    };
                    if apply_damage {
                        damage_events.push(DamageEvent {
                            source: *e1,
                            target: *e2,
                            damage: damage_on_coll.damage,
                        });
                    }
                }
            }
        }
    }
}

pub fn despawn_on_collision(
    data: &mut GameData,
    ecs: &mut Ecs,
    collisions: &HashMap<(Entity, Entity), Collision>,
    dust_texture: Texture2D,
) {
    let despawn_on_hits = ecs.check_components(|e, comps| comps.despawn_on_hit.contains_key(e));

    for despawn_e in &despawn_on_hits {
        for ((source, target), _collision) in collisions.iter() {
            for (e1, e2) in [(source, target), (target, source)] {
                if e1 == despawn_e {
                    let despawn_on_hit = ecs.components.despawn_on_hit.get(despawn_e).unwrap();
                    // TODO: not safe
                    let position = ecs.components.positions.get(despawn_e).unwrap();
                    if ecs.components.player_entity.contains_key(e2)
                        && despawn_on_hit.0 == EntityType::Player
                    {
                        spawn_dust(data, dust_texture.clone(), ecs, *position);
                        ecs.despawn(*despawn_e);
                        break;
                    };
                    if !ecs.components.player_entity.contains_key(e2)
                        && despawn_on_hit.0 == EntityType::Enemy
                    {
                        spawn_dust(data, dust_texture.clone(), ecs, *position);
                        ecs.despawn(*despawn_e);
                        break;
                    };
                }
            }
        }
    }
}

pub fn kill_entities(ecs: &mut Ecs, death_events: &mut Vec<DeathEvent>) {
    let healthies = ecs.check_components(|e, comps| comps.health.contains_key(e));

    for health_e in &healthies {
        let health = ecs.components.health.get_mut(health_e).unwrap();

        if health.hp <= 0. {
            ecs.despawn(*health_e);
            death_events.push(DeathEvent(*health_e));
        }
    }
}

pub fn handle_death(
    data: &mut GameData,
    skull_texture: Texture2D,
    ecs: &mut Ecs,
    death_events: &Vec<DeathEvent>,
) {
    let mut skull_positions = vec![];

    for ev in death_events {
        let pos = ecs.components.positions.get(&ev.0).unwrap();
        let player = ecs.components.player_entity.get(&ev.0);
        if player.is_some() {
            data.dead = true;
            data.death_screen.show();

            for _ in 0..40 {
                skull_positions.push(vec2(rand::gen_range(0., 360.), rand::gen_range(0., 240.)))
            }
        }
        spawn_skull(data, skull_texture.clone(), ecs, *pos);
    }

    for pos in skull_positions {
        spawn_skull(data, skull_texture.clone(), ecs, pos);
    }
}
