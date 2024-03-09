use crate::{
    game_data::GameData, sprite::indexed_sprite::IndexedSprite, systems::collision::CircleCollider,
    timer::Timer,
};
use macroquad::prelude::*;
use std::collections::HashMap;

use super::{
    animated_sprite::{AnimatedSprite, Animation},
    entities::Ecs,
    entity_id::Entity,
    tags::{Damageable, Health},
};

pub struct PlayerData {
    pub move_speed: f32,
    pub sprite_offset: Vec2,
    pub max_hp: u8,
    pub aberration: f32,
    pub aberration_increase_timer: Timer,
}

pub fn spawn_player(data: &mut GameData, texture: Texture2D, ecs: &mut Ecs) -> Entity {
    let id = data.new_entity();

    let indexed_sprite = IndexedSprite::new(texture.clone(), 16, vec2(8., 10.));
    let sprite = AnimatedSprite::new(
        indexed_sprite,
        HashMap::from([("idle".to_string(), Animation::new(vec![0], 0., false))]),
    );
    ecs.components.animated_sprites.insert(id, sprite);

    let collider = CircleCollider {
        radius: 3.,
        trigger: false,
    };
    ecs.components.colliders.insert(id, collider);

    ecs.components.positions.insert(id, vec2(180., 120.));
    ecs.components.velocities.insert(id, Vec2::ZERO);

    let player_data = PlayerData {
        move_speed: 72.,
        sprite_offset: vec2(8., 10.),
        max_hp: 3,
        aberration: 0.,
        aberration_increase_timer: Timer::new(0.2, true),
    };
    ecs.components.health.insert(
        id,
        Health {
            hp: player_data.max_hp.into(),
        },
    );
    ecs.components.player_data.insert(id, player_data);

    ecs.components.damageables.insert(
        id,
        Damageable {
            invulnerable_timer: Some(Timer::new(1., false)),
            hit_fx_timer: Some(Timer::new(0.22, false)),
        },
    );

    ecs.components
        .materials
        .insert(id, "aberration".to_string());

    println!("PLAYER {:?}", id);

    ecs.components.player_entity.insert(id, ());

    ecs.entities.push(id);
    id
}
