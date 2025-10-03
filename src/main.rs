use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component)]
struct Camera {
    speed: f32,
    margin: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            speed: 300.0,
            margin: 35.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_camera)
        .add_systems(Update, camera_edge_scroll)
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
    // Check bottom edge
    if cursor_pos.y < camera.margin {
        move_dir.y -= 1.0;
    }
    // Check top edge
    if cursor_pos.y > win_height - camera.margin {
        move_dir.y += 1.0;
    }

    // Normalize the diagonal movement so it doesn't move too fast
    if move_dir.length() > 0.0 {
        move_dir = move_dir.normalize();
        camera_transform.translation.x += move_dir.x * camera.speed * time.delta_secs();
        camera_transform.translation.y += move_dir.y * camera.speed * time.delta_secs();
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Camera::default()));
}
