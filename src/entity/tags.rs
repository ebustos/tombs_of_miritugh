use crate::timer::Timer;

#[derive(Debug, PartialEq)]
pub enum EntityType {
    Player,
    Enemy,
}

pub struct DamageOnCollision {
    pub source: EntityType,
    pub damage: f32,
}

pub struct Health {
    pub hp: f32,
}

pub struct Damageable {
    pub invulnerable_timer: Option<Timer>,
    pub hit_fx_timer: Option<Timer>,
}

pub struct DespawnOnAnimEnd;
pub struct DespawnOnHit(pub EntityType);
