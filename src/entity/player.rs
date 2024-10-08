use crate::{
    game_data::GameData,
    sprite::{flash_material::create_sprite_color_material, indexed_sprite::IndexedSprite},
    systems::collision::{CircleCollider, ColliderType},
    timer::Timer,
};
use macroquad::prelude::*;
use std::collections::HashMap;

use super::{
    animated_sprite::{AnimatedSprite, Animation},
    entities::Ecs,
    entity_id::Entity,
    tags::{Damageable, Health},
    upgrades::CommonUpgrade,
};

pub struct PlayerData {
    pub base_move_speed: f32,
    pub sprite_offset: Vec2,
    pub base_max_hp: u8,
    pub aberration: f32,
    pub aberration_increase_timer: Timer,
    pub shadows: Vec<Entity>,
    pub upgrades: Vec<CommonUpgrade>,
}

pub struct PlayerUpgradeData {
    pub move_speed: f32,
    pub max_hp: u8,
}

impl PlayerData {
    pub fn get_upgraded_data(&self) -> PlayerUpgradeData {
        let mut max_hp = self.base_max_hp;
        let mut move_speed_percentage_increase = 0.;

        for upgrade in &self.upgrades {
            match upgrade {
                CommonUpgrade::MaxHp(increase) => max_hp += increase,
                CommonUpgrade::MoveSpeed(increase) => move_speed_percentage_increase += increase,
                _ => {}
            }
        }

        PlayerUpgradeData {
            max_hp,
            move_speed: self.base_move_speed * (1. + move_speed_percentage_increase),
        }
    }
}

pub fn spawn_player(data: &mut GameData, ecs: &mut Ecs) -> Entity {
    let indexed_sprite = IndexedSprite::new(data, "player", 16, vec2(8., 10.));
    let sprite = AnimatedSprite::new(
        indexed_sprite,
        HashMap::from([("idle".to_string(), Animation::new(vec![0], 0., false))]),
    );

    // Shadows
    let shadow1_id = data.new_entity();
    let mut shadow1_sprite = sprite.clone();
    shadow1_sprite.color = Color::from_rgba(255, 255, 255, 120);
    shadow1_sprite.visible = false;
    ecs.components
        .animated_sprites
        .insert(shadow1_id, shadow1_sprite);
    ecs.components
        .positions
        .insert(shadow1_id, vec2(180., 120.));
    ecs.components.player_entity.insert(shadow1_id, ());
    // ecs.components
    //     .materials
    //     .insert(shadow1_id, create_sprite_color_material());
    ecs.entities.push(shadow1_id);

    let shadow2_id = data.new_entity();
    let mut shadow2_sprite = sprite.clone();
    shadow2_sprite.color = Color::from_rgba(255, 255, 255, 220);
    shadow2_sprite.visible = false;
    ecs.components
        .animated_sprites
        .insert(shadow2_id, shadow2_sprite);
    ecs.components
        .positions
        .insert(shadow2_id, vec2(180., 120.));
    ecs.components.player_entity.insert(shadow2_id, ());
    // ecs.components
    //     .materials
    //     .insert(shadow2_id, create_sprite_color_material());
    ecs.entities.push(shadow2_id);

    // Player
    let id = data.new_entity();
    ecs.components.animated_sprites.insert(id, sprite.clone());

    let collider = CircleCollider {
        radius: 3.,
        coll_type: ColliderType::Player,
    };
    ecs.components.colliders.insert(id, collider);

    ecs.components.positions.insert(id, vec2(180., 120.));
    ecs.components.velocities.insert(id, Vec2::ZERO);

    let player_data = PlayerData {
        base_move_speed: 72.,
        sprite_offset: vec2(8., 10.),
        base_max_hp: 3,
        aberration: 0.,
        aberration_increase_timer: Timer::new(0.2, true),
        shadows: vec![shadow1_id, shadow2_id],
        upgrades: vec![],
    };
    ecs.components.health.insert(
        id,
        Health {
            hp: player_data.base_max_hp.into(),
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
        .insert(id, create_sprite_color_material());

    println!("PLAYER {:?}", id);

    ecs.components.player_entity.insert(id, ());

    ecs.entities.push(id);

    id
}
