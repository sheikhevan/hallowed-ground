use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

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
            margin: 35.0,

            zoom_speed: 0.1,
            max_zoom: 3.0,
            min_zoom: 0.5,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, temp_setup_shapes))
        .add_systems(Update, (camera_edge_scroll, camera_zoom, camera_drag))
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

fn camera_drag(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut msg_motion: MessageReader<MouseMotion>,
    mut q_camera: Query<(&mut Transform, &Camera)>,
) {
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let Ok((mut camera_transform, _)) = q_camera.single_mut() else {
        return;
    };

    for msg in msg_motion.read() {
        camera_transform.translation.x -= msg.delta.x;
        camera_transform.translation.y += msg.delta.y;
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Camera::default()));
}

const X_EXTENT: f32 = 900.;

fn temp_setup_shapes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shapes = [
        meshes.add(Circle::new(50.0)),
        meshes.add(CircularSector::new(50.0, 1.0)),
        meshes.add(CircularSegment::new(50.0, 1.25)),
        meshes.add(Ellipse::new(25.0, 50.0)),
        meshes.add(Annulus::new(25.0, 50.0)),
        meshes.add(Capsule2d::new(25.0, 50.0)),
        meshes.add(Rhombus::new(75.0, 100.0)),
        meshes.add(Rectangle::new(50.0, 100.0)),
        meshes.add(RegularPolygon::new(50.0, 6)),
        meshes.add(Triangle2d::new(
            Vec2::Y * 50.0,
            Vec2::new(-50.0, -50.0),
            Vec2::new(50.0, -50.0),
        )),
        meshes.add(Segment2d::new(
            Vec2::new(-50.0, 50.0),
            Vec2::new(50.0, -50.0),
        )),
        meshes.add(Polyline2d::new(vec![
            Vec2::new(-50.0, 50.0),
            Vec2::new(0.0, -50.0),
            Vec2::new(50.0, 50.0),
        ])),
    ];
    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(
                // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                0.0,
                0.0,
            ),
        ));
    }
}
