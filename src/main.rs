use std::collections::HashMap;

use entity::{
    entities::Ecs,
    entity_id::Entity,
    events::{DamageEvent, DeathEvent},
    player::spawn_player,
};
use fps_counter::FPSCounter;
use game_state::GameState;
use items::weapon::{Shooter, Weapon};
use macroquad::{audio, miniquad::window::set_mouse_cursor, prelude::*};
use macroquad_tiled::load_map;
use room::Room;
use settings::{GameSettings, WindowSize};
use systems::{
    collision::draw_colliders,
    damageable::{
        apply_damage, damage_on_collision, despawn_on_collision, flash_on_damage,
        handle_enemy_death, kill_entities, update_damageables,
    },
    door::handle_door_collisions,
    enemy::update_enemies,
    movement::move_entities,
    player::update_player,
    spawn::spawn_creatures,
    sprite::{draw_animated_sprites, update_animated_sprites},
    timer::update_timers,
    weapon::update_weapon,
};
use ui::{
    hud::{create_aberration_material, draw_aberration_meter},
    pause_menu::pause_menu,
    ui_data::UIData,
};

use crate::{
    game_data::{GameData, Sprites},
    input_manager::{Action, InputManager},
    map::map::Map,
    sprite::indexed_sprite::IndexedSprite,
    ui::hud::draw_hp,
};

mod entity;
mod fps_counter;
mod game_data;
mod game_state;
mod input_manager;
mod items;
mod map;
mod physics;
mod rand_utils;
mod room;
mod settings;
mod sprite;
mod systems;
mod timer;
mod ui;

fn window_conf() -> Conf {
    Conf {
        window_title: "Acrola Jam 0".to_owned(),
        fullscreen: false,
        window_resizable: true,
        window_width: 1440,
        window_height: 960,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandOnly,
            framebuffer_alpha: false,
            swap_interval: None,
            ..Default::default()
        },
        sample_count: 0,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    set_pc_assets_folder("assets");

    let render_target = render_target(360, 240);
    render_target.texture.set_filter(FilterMode::Nearest);

    let mut font = load_ttf_font("fonts/Bitfantasy.ttf").await.unwrap();
    font.set_filter(FilterMode::Nearest);

    let mut icon_font = load_ttf_font("fonts/Zicons.ttf").await.unwrap();
    icon_font.set_filter(FilterMode::Nearest);

    // UI assets
    let button_texture: Texture2D = load_texture("ui/button_bg.png").await.unwrap();
    button_texture.set_filter(FilterMode::Nearest);
    let button_texture_hover: Texture2D = load_texture("ui/button_bg_hover.png").await.unwrap();
    button_texture_hover.set_filter(FilterMode::Nearest);
    let button_texture_pressed: Texture2D = load_texture("ui/button_bg_clicked.png").await.unwrap();
    button_texture_pressed.set_filter(FilterMode::Nearest);
    let frame_texture: Texture2D = load_texture("ui/window_bg.png").await.unwrap();
    frame_texture.set_filter(FilterMode::Nearest);
    let focus_bg_texture: Texture2D = load_texture("ui/focus_bg.png").await.unwrap();
    focus_bg_texture.set_filter(FilterMode::Nearest);

    // Sfx
    let button_click_sfx = audio::load_sound("audio/ui/bookClose.ogg").await.unwrap();

    let ui_data = UIData {
        button_texture: button_texture,
        button_texture_hover: button_texture_hover,
        button_texture_pressed: button_texture_pressed,
        button_click_sfx: button_click_sfx,
        frame_texture: frame_texture.clone(),
        focus_background_texture: focus_bg_texture,
        font: font.clone(),
        icon_font: icon_font.clone(),
        text_color: Color::from_hex(0xe4d2aa),
        text_shadow_color: Color::from_hex(0xb4202a),
        focus: None,
    };

    let mut fullscreen = false;

    let camera = Camera2D::default();

    let mut fps_counter = FPSCounter::default();

    let mut paused = false;

    let hud_heart_texture: Texture2D = load_texture("ui/heart_01.png").await.unwrap();
    hud_heart_texture.set_filter(FilterMode::Nearest);
    let hopper_texture: Texture2D = load_texture("entities/hopper_01.png").await.unwrap();
    hopper_texture.set_filter(FilterMode::Nearest);
    let skull_texture: Texture2D = load_texture("entities/skull_01.png").await.unwrap();
    skull_texture.set_filter(FilterMode::Nearest);
    let bullet_texture: Texture2D = load_texture("entities/bullet_01.png").await.unwrap();
    bullet_texture.set_filter(FilterMode::Nearest);
    let dust_texture: Texture2D = load_texture("entities/dust_01.png").await.unwrap();
    dust_texture.set_filter(FilterMode::Nearest);
    let blood_texture: Texture2D = load_texture("entities/blood_01.png").await.unwrap();
    blood_texture.set_filter(FilterMode::Nearest);
    let aberration_meter_texture: Texture2D =
        load_texture("ui/aberration_meter.png").await.unwrap();
    aberration_meter_texture.set_filter(FilterMode::Nearest);
    let noise1_texture: Texture2D = load_texture("entities/Perlin_16-128x128.png")
        .await
        .unwrap();
    noise1_texture.set_filter(FilterMode::Nearest);
    let noise2_texture: Texture2D = load_texture("entities/Perlin_15-128x128.png")
        .await
        .unwrap();
    noise2_texture.set_filter(FilterMode::Nearest);
    let aberration_meter_mask_texture: Texture2D =
        load_texture("ui/aberration_meter_mask.png").await.unwrap();
    aberration_meter_mask_texture.set_filter(FilterMode::Nearest);

    let aberration_material = create_aberration_material();

    let sprites = Sprites {
        hud_heart: IndexedSprite::new(hud_heart_texture, 16, Vec2::ZERO),
        aberration_meter: IndexedSprite::new(aberration_meter_texture, 48, Vec2::ZERO),
        aberration_material,
    };

    let settings = GameSettings::default();

    let mut entity_index = 0;

    // Map
    let tileset = load_texture("map/tileset_01.png").await.unwrap();
    tileset.set_filter(FilterMode::Nearest);

    let tiled_map1_json = load_string("map/example_01.tmj").await.unwrap();
    let tiled_map1 = load_map(
        tiled_map1_json.as_str(),
        &[("tileset_01.png", tileset.clone())],
        &[],
    )
    .unwrap();
    entity_index += 1;
    let map1 = Map::new(Entity(entity_index), &settings, tiled_map1);

    let tiled_map2_json = load_string("map/map2.tmj").await.unwrap();
    let tiled_map2 = load_map(
        tiled_map2_json.as_str(),
        &[("tileset_01.png", tileset.clone())],
        &[],
    )
    .unwrap();
    entity_index += 1;
    let map2 = Map::new(Entity(entity_index), &settings, tiled_map2);

    let tiled_map3_json = load_string("map/map3.tmj").await.unwrap();
    let tiled_map3 = load_map(
        tiled_map3_json.as_str(),
        &[("tileset_01.png", tileset.clone())],
        &[],
    )
    .unwrap();
    entity_index += 1;
    let map3 = Map::new(Entity(entity_index), &settings, tiled_map3);

    let maps = vec![map1, map2, map3];
    let mut data = GameData {
        entity_index,
        settings,
        state: GameState::default(),
        ui: ui_data,
        sprites,
        input: InputManager::new(),
        camera,
        debug_collisions: false,
        #[cfg(debug_assertions)]
        show_fps: true,
        #[cfg(not(debug_assertions))]
        show_fps: false,
        weapon: Weapon::Shooter(Shooter::new()),
        current_room: Room::new(maps.len(), 3.),
        maps,
    };
    data.settings.set_window_size(WindowSize::W1440);

    let player_texture: Texture2D = load_texture("entities/player_01.png").await.unwrap();
    player_texture.set_filter(FilterMode::Nearest);

    let mut ecs = Ecs::default();

    spawn_player(&mut data, player_texture, &mut ecs);

    data.spawn_map_entities(&mut ecs);

    let mut collisions = HashMap::new();
    let mut damage_events = Vec::<DamageEvent>::new();
    let mut death_events = Vec::<DeathEvent>::new();

    data.sprites
        .aberration_material
        .set_texture("noise1", noise1_texture.clone());
    data.sprites
        .aberration_material
        .set_texture("noise2", noise2_texture.clone());
    data.sprites
        .aberration_material
        .set_texture("mask", aberration_meter_mask_texture.clone());

    loop {
        let despawned_entities = &ecs.marked_for_despawn.clone();
        for entity in despawned_entities {
            let entity_i = ecs.entities.iter().position(|e| e == entity);
            if let Some(index) = entity_i {
                ecs.entities.remove(index);
                ecs.remove_all_components(entity);
            }
        }
        ecs.marked_for_despawn.clear();

        death_events.clear();

        data.sprites
            .aberration_material
            .set_texture("noise1", noise1_texture.clone());
        data.sprites
            .aberration_material
            .set_texture("noise2", noise2_texture.clone());
        data.sprites
            .aberration_material
            .set_uniform("intensity", 1.2f32);
        data.sprites
            .aberration_material
            .set_uniform("time", get_time() as f32);

        data.update();
        set_mouse_cursor(miniquad::CursorIcon::Default);

        set_camera(&data.camera);

        clear_background(BLACK);

        if (is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt))
            && is_key_pressed(KeyCode::Enter)
        {
            fullscreen = !fullscreen;

            if fullscreen {
                data.settings
                    .set_window_size(settings::WindowSize::Fullscreen);
            } else {
                data.settings.set_window_size(WindowSize::default());
            }
        }

        if is_key_pressed(KeyCode::F1) {
            data.debug_collisions = !data.debug_collisions;
        }

        if is_key_pressed(KeyCode::F6) {
            data.current_room.despawn(&mut ecs);
            data.current_room = Room::new(data.maps.len(), rand::gen_range(1., 20.));
            let new_player_pos = data.spawn_map_entities(&mut ecs);
            let players = ecs.check_components(|e, comps| {
                comps.player_data.contains_key(e) && comps.positions.contains_key(e)
            });
            for player_e in &players {
                let pos = ecs.components.positions.get_mut(player_e).unwrap();
                *pos = new_player_pos;
            }
        }

        if data.input.is_just_pressed(Action::Pause) {
            paused = !paused;
            if !paused {
                data.ui.focus = None;
            }
        }

        data.current_map().draw_base();

        spawn_creatures(&mut data, &mut ecs, &hopper_texture);
        update_timers(&mut ecs);
        update_damageables(&mut ecs);
        damage_on_collision(&ecs, &mut damage_events, &collisions);
        despawn_on_collision(&mut data, &mut ecs, &collisions, dust_texture.clone());
        apply_damage(
            &mut data,
            &mut ecs,
            &mut damage_events,
            blood_texture.clone(),
        );
        kill_entities(&mut ecs, &mut death_events);
        handle_enemy_death(&mut data, skull_texture.clone(), &mut ecs, &death_events);
        update_player(&mut data, &mut ecs);
        update_weapon(&mut ecs, &mut data, bullet_texture.clone());
        update_enemies(&mut ecs);
        update_animated_sprites(&mut ecs);
        handle_door_collisions(&mut ecs);
        collisions = move_entities(&mut data, &mut ecs);

        flash_on_damage(&mut ecs);
        draw_animated_sprites(&mut ecs);
        data.current_map().draw_upper();

        if data.debug_collisions {
            draw_colliders(&data, &ecs);
            data.current_map().draw_colliders();
        }

        draw_hp(&data, &ecs);
        draw_aberration_meter(&data, &ecs);

        if data.show_fps {
            fps_counter.update_and_draw(&mut data);
        }

        if paused && pause_menu(&mut data) {
            break;
        }

        next_frame().await
    }
}

pub async fn pub_main() {
    main();
}
