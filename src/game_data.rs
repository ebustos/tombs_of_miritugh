use std::collections::HashMap;

use macroquad::{audio::Sound, prelude::*};

use crate::{
    entity::{
        entities::Ecs, entity_id::Entity, mirituhg::spawn_mirituhg, player::spawn_player,
        spawner::spawn_spawner, upgrades::Upgrades,
    },
    game_state::GameState,
    input_manager::InputManager,
    items::weapon::{Launcher, Weapon},
    map::map::Map,
    rand_utils::rand_dir,
    room::Room,
    settings::GameSettings,
    timer::Timer,
    ui::{
        death_screen::DeathScreen, end_game_screen::EndGameScreen, screen_dimmer::ScreenDimmer,
        ui_data::UIData,
    },
};

pub enum GameMaterial {
    Aberration(Material),
    Color(Material),
}

pub struct Graphics {
    pub aberration_meter_material: Material,
    pub aberration_material: Material,
    pub noise1_texture: Texture2D,
    pub noise2_texture: Texture2D,
    pub materials: HashMap<String, GameMaterial>,
    pub textures: HashMap<&'static str, Texture2D>,
}

pub struct Audio {
    pub ui_switch: Sound,
    pub shoot: Sound,
    pub death: Sound,
    pub death2: Sound,
    pub spawn: Sound,
    pub kill: Sound,
    pub confirm: Sound,
    pub confirm2: Sound,
    pub hit: Sound,
    pub hit2: Sound,
    pub music1: Sound,
}

pub struct ScreenShake {
    pub distance: f32,
    pub camera_offset: Vec2,
    pub timer: Timer,
    pub event_timer: Timer,
}

impl ScreenShake {
    pub fn new() -> Self {
        Self {
            distance: 5.,
            camera_offset: Vec2::ZERO,
            timer: Timer::new(0.02, false),
            event_timer: Timer::new(0., false),
        }
    }

    pub fn shake(&mut self, duration: f32, distance: f32) {
        self.distance = distance;
        self.event_timer.time = duration;
        self.event_timer.reset();
        self.timer.reset();
    }
}

pub struct GameData {
    pub entity_index: u64,
    pub state: GameState,
    pub settings: GameSettings,
    pub ui: UIData,
    pub graphics: Graphics,
    pub audio: Audio,
    pub input: InputManager,
    pub camera: Camera2D,
    pub debug_collisions: bool,
    pub show_fps: bool,
    pub weapon: Weapon,
    pub current_room: Room,
    pub next_room: Option<Room>,
    pub maps: Vec<Map>,
    pub screen_dimmer: ScreenDimmer,
    pub map_change_requested: bool,
    pub paused: bool,
    pub pause_timer: Timer,
    pub show_pause_menu: bool,
    pub death_screen: DeathScreen,
    pub end_game_screen: EndGameScreen,
    pub dead: bool,
    pub previous_window_size: (f32, f32),
    pub game_completed: bool,
    pub item_drop_chance_increase: i32,
    pub screen_shake: ScreenShake,
    pub completed_rooms: usize,
    pub upgrades: Upgrades,
}

impl GameData {
    pub fn new(
        initial_entity_index: u64,
        settings: GameSettings,
        ui_data: UIData,
        maps: Vec<Map>,
        graphics: Graphics,
        audio: Audio,
        death_texture: Texture2D,
        end_game_texture: Texture2D,
    ) -> Self {
        let camera = Camera2D::default();
        Self {
            entity_index: initial_entity_index,
            settings,
            state: GameState::default(),
            ui: ui_data,
            graphics,
            audio,
            input: InputManager::new(),
            camera,
            debug_collisions: false,
            #[cfg(debug_assertions)]
            show_fps: true,
            #[cfg(not(debug_assertions))]
            show_fps: false,
            weapon: Weapon::Launcher(Launcher::new()),
            current_room: Room::new(0, 0.),
            next_room: None,
            maps,
            screen_dimmer: ScreenDimmer::new(),
            map_change_requested: false,
            paused: false,
            pause_timer: Timer::new(1., false),
            show_pause_menu: false,
            death_screen: DeathScreen::new(death_texture),
            end_game_screen: EndGameScreen::new(end_game_texture),
            dead: true,
            previous_window_size: (screen_width(), screen_height()),
            game_completed: false,
            item_drop_chance_increase: 0,
            screen_shake: ScreenShake::new(),
            completed_rooms: 0,
            upgrades: Upgrades::new(),
        }
    }

    pub fn reset(&mut self) {
        self.state = GameState::Intro;
        self.weapon = Weapon::Launcher(Launcher::new());
        self.current_room = Room::new(0, 3.);
        self.next_room = None;
        self.dead = false;
        self.completed_rooms = 0;
    }

    pub fn new_entity(&mut self) -> Entity {
        self.entity_index += 1;
        Entity(self.entity_index)
    }

    pub fn update(&mut self) {
        self.pause_timer.update();
        self.input.gamepads.poll();
        self.update_camera();

        let shake = &mut self.screen_shake;
        self.camera.target = vec2(360. / 2., 240. / 2.) + shake.camera_offset;
        if !self.paused {
            shake.timer.update();
            shake.event_timer.update();
            if shake.event_timer.progress() > 0. && shake.timer.completed() {
                shake.timer.reset();
                shake.camera_offset = rand_dir() * shake.event_timer.progress() * shake.distance;
            }
        }
        if shake.event_timer.just_completed() {
            shake.camera_offset = Vec2::ZERO;
        }

        for mat in &self.graphics.materials {
            match mat.1 {
                GameMaterial::Aberration(mat) => {
                    mat.set_uniform("time", get_time() as f32);
                }
                GameMaterial::Color(_mat) => {}
            }
        }
    }

    pub fn update_camera(&mut self) {
        let target_size = self.settings.resolution;
        let target = vec2(target_size.x / 2., target_size.y / 2.);

        self.camera.target = target;
        self.camera.zoom = vec2(1. / target_size.x * 2., 1. / target_size.y * 2.);
        self.camera.offset = Vec2::ZERO;
    }

    pub fn current_map(&self) -> &Map {
        &self.maps[self.current_room.map_index]
    }

    pub fn spawn_map_entities(&mut self, ecs: &mut Ecs) -> Vec2 {
        let mut player_pos = Vec2::ZERO;
        let mut spawner_positions = vec![];
        let mut spawn_boss = false;
        for (_, layer) in &self.current_map().tiled_map.layers {
            for object in &layer.objects {
                let object_pos = vec2(object.world_x + 4., object.world_y - 4.);
                // if let Some(_door_dir) = object.properties.get("door") {
                //     spawn_door(self, object_pos, ecs);
                // }
                if let Some(_door_dir) = object.properties.get("player") {
                    player_pos = object_pos;
                }
                if let Some(_) = object.properties.get("spawn") {
                    spawner_positions.push(object_pos);
                }
                if let Some(_) = object.properties.get("boss") {
                    spawn_boss = true;
                }
            }
        }

        if spawn_boss {
            spawn_mirituhg(self, vec2(180., 120.), ecs);
        }

        for pos in spawner_positions {
            spawn_spawner(self, pos, ecs);
        }

        player_pos
    }

    pub fn next_room(&mut self, ecs: &mut Ecs) {
        self.current_room.despawn(ecs);

        let map_index = match self.completed_rooms {
            0 => 1,
            _ => rand::gen_range(1, self.maps.len()),
        };

        let mut new_room = Room::new(map_index, 2. + 3. * self.completed_rooms as f32);

        new_room.available_upgrades = if self.completed_rooms == 0 {
            Upgrades::weapon_selection()
        } else {
            let players = ecs.check_components(|e, comps| comps.player_data.contains_key(e));
            let player_data = ecs.components.player_data.get(&players[0]).unwrap();
            let up_data = player_data.get_upgraded_data();
            let health = ecs.components.health.get(&players[0]).unwrap();

            if player_data.aberration >= 1. {
                // TODO: boss room
                new_room.map_index = 0;
            }

            self.upgrades.generate_upgrades(
                &self.weapon,
                up_data.max_hp as f32 - health.hp,
                player_data.aberration,
            )
        };

        self.next_room = Some(new_room);
        self.map_change_requested = true;
        self.screen_dimmer.dim();
        self.paused = true;
        self.pause_timer.reset();
        self.completed_rooms += 1;
    }
}

// pub fn reset_game(data: &mut GameData, ecs: &mut Ecs) {
//     data.reset();
//     let entities = ecs.check_components(|_, _| true);
//     for entity in entities {
//         let entity_i = ecs.entities.iter().position(|e| e == &entity).unwrap();
//         ecs.entities.remove(entity_i);
//         ecs.remove_all_components(&entity);
//     }
//     data.current_room = Room::new(0, rand::gen_range(1., 20.));
//     spawn_player(data, ecs);
//     let new_player_pos = data.spawn_map_entities(ecs);
//     data.current_room.started = true;
//     let players = ecs.check_components(|e, comps| {
//         comps.player_data.contains_key(e) && comps.positions.contains_key(e)
//     });
//     for player_e in &players {
//         let pos = ecs.components.positions.get_mut(player_e).unwrap();
//         *pos = new_player_pos;
//     }
// }
