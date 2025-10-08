use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

mod tiles;
mod ui;

#[derive(Component)]
struct Camera {
    speed: f32,
    margin: f32,

    zoom_speed: f32,
    max_zoom: f32,
    min_zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            speed: 300.0,
            margin: 50.0,

            zoom_speed: 0.03,
            max_zoom: 3.0,
            min_zoom: 0.5,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Hallowed Ground"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((TilemapPlugin, tiles::picking::TilemapPickingPlugin))
        .add_plugins(EguiPlugin::default())
        .init_resource::<ui::Images>()
        .init_resource::<ui::EguiTextureCache>()
        .add_message::<ui::DebugSpawnBuildingMsg>()
        .add_systems(PreUpdate, ui::register_textures)
        .add_systems(
            EguiPrimaryContextPass,
            (ui::debug_ui, ui::debug_handle_spawn_building),
        )
        .add_systems(Startup, tiles::setup_tiles)
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (camera_edge_scroll, camera_zoom, camera_wasd))
        .run();
}

fn camera_edge_scroll(
    time: Res<Time>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(&mut Transform, &Camera)>,
) {
    let window = q_window.single().unwrap();

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok((mut camera_transform, camera)) = q_camera.single_mut() else {
        return;
    };

    let win_width = window.width();
    let win_height = window.height();

    let mut move_dir = Vec2::ZERO;

    // Check left edge
    if cursor_pos.x < camera.margin {
        move_dir.x -= 1.0;
    }
    // Check right edge
    if cursor_pos.x > win_width - camera.margin {
        move_dir.x += 1.0;
    }
    // Check top edge
    if cursor_pos.y > win_height - camera.margin {
        move_dir.y -= 1.0;
    }
    // Check bottom edge
    if cursor_pos.y < camera.margin {
        move_dir.y += 1.0;
    }

    // Normalize the diagonal movement so it doesn't move too fast
    if move_dir.length() > 0.0 {
        move_dir = move_dir.normalize();
        camera_transform.translation.x += move_dir.x * camera.speed * time.delta_secs();
        camera_transform.translation.y += move_dir.y * camera.speed * time.delta_secs();
    }
}

fn camera_zoom(
    mut msg_scroll: MessageReader<MouseWheel>,
    mut q_camera: Query<(&mut Transform, &Camera)>,
) {
    let Ok((mut camera_transform, camera)) = q_camera.single_mut() else {
        return;
    };

    for msg in msg_scroll.read() {
        // Gets the scroll amount (+ is zoom in, - is zoom out)
        let zoom_delta = msg.y * camera.zoom_speed;

        // Calculate new scale
        let current_scale = camera_transform.scale.x;
        let new_scale = (current_scale - zoom_delta).clamp(camera.min_zoom, camera.max_zoom);

        camera_transform.scale = Vec3::splat(new_scale);
    }
}

fn camera_wasd(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q_camera: Query<(&mut Transform, &Camera)>,
) {
    let mut move_dir = Vec2::ZERO;

    let Ok((mut camera_transform, camera)) = q_camera.single_mut() else {
        return;
    };

    if keys.pressed(KeyCode::KeyW) {
        move_dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        move_dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        move_dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        move_dir.x += 1.0;
    }

    if move_dir.length() > 0.0 {
        move_dir = move_dir.normalize();
        camera_transform.translation.x += move_dir.x * camera.speed * time.delta_secs();
        camera_transform.translation.y += move_dir.y * camera.speed * time.delta_secs();
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Camera::default()));
}
