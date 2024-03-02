use std::path::PathBuf;

use fps_counter::FPSCounter;
use game_state::GameState;
use gamepads::Gamepads;
use macroquad::{audio, miniquad::window::set_mouse_cursor, prelude::*};
use macroquad_tiled::load_map;
use map::{tiled_macroquad::TiledMap, Map};
use settings::{GameSettings, WindowSize};
use ui::{pause_menu::pause_menu, ui_data::UIData};

use crate::{
    game_data::GameData,
    pawn::{entity::Entity, player::Player},
};

mod fps_counter;
mod game_data;
mod game_state;
mod map;
mod pawn;
mod settings;
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

    // Map
    // let tileset = load_texture("map/mockup_01.png").await.unwrap();
    // tileset.set_filter(FilterMode::Nearest);
    // let tiled_map_json = load_string("map/map_01.json").await.unwrap();
    // let tiled_map = load_map(tiled_map_json.as_str(), &[("mockup_01.png", tileset)], &[]).unwrap();

    // let map = Map { tiled_map };

    // Sfx
    let button_click_sfx = audio::load_sound("audio/ui/bookClose.ogg").await.unwrap();

    let ui_data = UIData {
        button_texture: button_texture,
        button_texture_hover: button_texture_hover,
        button_texture_pressed: button_texture_pressed,
        button_click_sfx: button_click_sfx,
        frame_texture: frame_texture,
        font: font.clone(),
        icon_font: icon_font.clone(),
        text_color: Color::from_hex(0xe4d2aa),
        text_shadow_color: Color::from_hex(0xb4202a),
    };
    let mut settings = GameSettings::default();

    let mut fullscreen = false;

    let mut camera = Camera2D::from_display_rect(Rect {
        x: 0.,
        y: 240.,
        w: 360.,
        h: -240.,
    });
    camera.render_target = Some(render_target.clone());

    let mut fps_counter = FPSCounter::default();

    let mut game_state = GameState::default();
    let mut paused = false;

    let map_path =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets/map/map_01.tmx");
    println!("{:?}", map_path);
    // let tiled_map = TiledMap::load_map(&map_path);
    let tiled_map = TiledMap::new(&map_path);

    let mut entities: Vec<Box<dyn Entity>> = vec![];

    let player_texture: Texture2D = load_texture("entities/player_01.png").await.unwrap();
    player_texture.set_filter(FilterMode::Nearest);
    let player = Player::new(player_texture);
    entities.push(Box::new(player));

    let mut gamepads = Gamepads::new();

    let mut data = GameData {
        gamepads: gamepads,
        settings,
        state: game_state,
        gamepad: None,
    };

    loop {
        data.update();
        set_mouse_cursor(miniquad::CursorIcon::Default);
        set_camera(&camera);

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

        if is_key_pressed(KeyCode::Escape)
            || (data.gamepad.is_some()
                && data
                    .gamepad
                    .unwrap()
                    .is_just_pressed(gamepads::Button::RightCenterCluster))
        {
            paused = !paused;
        }

        // map.draw();
        for entity in &mut entities {
            if !paused {
                entity.update(&mut data);
            }
            entity.draw(&mut data);
        }

        fps_counter.update_and_draw(&ui_data);

        if paused && pause_menu(&ui_data, &mut data) {
            break;
        }

        set_default_camera();
        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        next_frame().await
    }
}
