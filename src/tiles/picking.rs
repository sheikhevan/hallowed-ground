use bevy::picking::PickingSystems;
use bevy::picking::backend::{HitData, PointerHits};
use bevy::picking::hover::PickingInteraction;
use bevy::picking::pointer::{PointerId, PointerLocation};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_tilemap::prelude::*;

// Special thanks to dpogorzelski on GitHub, whose code I adapted from. Original can be found
// here: https://github.com/StarArawn/bevy_ecs_tilemap/issues/572

// This plugin integrates bevy_ecs_tilemap with native bevy picking
pub struct TilemapPickingPlugin;

#[derive(Component)]
struct DragState {
    offset: Vec2,
}

impl Plugin for TilemapPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, picking_backend.in_set(PickingSystems::Backend))
            .add_systems(Update, (highlight_hovered_tiles, manage_hovered_buildings));
    }
}

fn picking_backend(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    tilemaps: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TilemapTileSize,
        &TilemapAnchor,
        &TileStorage,
        &GlobalTransform,
        &ViewVisibility,
    )>,
    tiles: Query<&TileVisible>,
    buildings: Query<
        (Entity, &GlobalTransform, &Sprite, &ViewVisibility),
        With<crate::ui::Building>,
    >,
    mut pointer_hits: MessageWriter<PointerHits>,
) {
    for (pointer_id, pointer_location) in pointers.iter() {
        let Some(location) = pointer_location.location() else {
            continue;
        };

        let Some((camera_entity, camera, camera_transform)) = cameras
            .iter()
            .filter(|(_, cam, _)| cam.is_active)
            .find(|(_, cam, _)| {
                let Ok(primary_window_entity) = primary_window.single() else {
                    return false;
                };
                cam.target.normalize(Some(primary_window_entity)).unwrap() == location.target
            })
        else {
            continue;
        };

        let Ok(cursor_world_pos) = camera.viewport_to_world_2d(camera_transform, location.position)
        else {
            continue;
        };

        let mut hits: Vec<(Entity, HitData)> = Vec::new();

        // Check tilemaps
        for (
            map_size,
            grid_size,
            map_type,
            tile_size,
            anchor,
            tile_storage,
            tilemap_transform,
            visibility,
        ) in tilemaps.iter()
        {
            if !visibility.get() {
                continue;
            }

            let inverse_transform = tilemap_transform.affine().inverse();
            let local_cursor_pos = inverse_transform.transform_point3(cursor_world_pos.extend(0.0));
            let local_pos = local_cursor_pos.truncate();

            if let Some(tile_pos) = TilePos::from_world_pos(
                &local_pos, map_size, grid_size, tile_size, map_type, anchor,
            ) {
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    if let Ok(tile_visible) = tiles.get(tile_entity) {
                        if tile_visible.0 {
                            let depth = tilemap_transform.translation().z;
                            hits.push((
                                tile_entity,
                                HitData::new(camera_entity, depth, None, None),
                            ));
                        }
                    }
                }
            }
        }

        // Check buildings
        for (entity, transform, sprite, visibility) in buildings.iter() {
            if !visibility.get() {
                continue;
            }

            let size = sprite.custom_size.unwrap_or(Vec2::new(192.0, 192.0));
            let half_size = size / 2.0;

            let building_pos = transform.translation().truncate();
            let relative_pos = cursor_world_pos - building_pos;

            if relative_pos.x.abs() <= half_size.x && relative_pos.y.abs() <= half_size.y {
                let depth = -transform.translation().z; // NEGATIVE depth!
                hits.push((entity, HitData::new(camera_entity, depth, None, None)));
            }
        }

        let order = camera.order as f32;
        pointer_hits.write(PointerHits::new(*pointer_id, hits, order));
    }
}

fn highlight_hovered_tiles(mut tiles: Query<(&PickingInteraction, &mut TileColor)>) {
    for (interaction, mut tile_color) in tiles.iter_mut() {
        match interaction {
            PickingInteraction::Pressed => {
                // Tile is being clicked
                tile_color.0 = Color::srgb(1.0, 0.5, 0.5); // Reddish tint
            }
            PickingInteraction::Hovered => {
                // Tile is being hovered
                tile_color.0 = Color::srgb(1.3, 1.3, 1.0); // Bright yellow tint
            }
            PickingInteraction::None => {
                // Reset to default
                tile_color.0 = Color::WHITE;
            }
        }
    }
}

fn manage_hovered_buildings(
    mut commands: Commands,
    mut q_buildings: Query<
        (Entity, &PickingInteraction, &mut Sprite, &mut Transform),
        With<crate::ui::Building>,
    >,
    q_dragging: Query<(Entity, &DragState)>,
    q_pointers: Query<&PointerLocation>,
    q_cameras: Query<(&Camera, &GlobalTransform)>,
    q_primary_window: Query<Entity, With<PrimaryWindow>>,
    q_tilemaps: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TilemapTileSize,
        &TilemapAnchor,
        &GlobalTransform,
    )>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    let cursor_world_pos = q_pointers
        .iter()
        .filter_map(|pointer_location| {
            let location = pointer_location.location()?;

            q_cameras
                .iter()
                .filter(|(cam, _)| cam.is_active)
                .find(|(cam, _)| {
                    let Ok(primary_window_entity) = q_primary_window.single() else {
                        return false;
                    };
                    cam.target.normalize(Some(primary_window_entity)).unwrap() == location.target
                })
                .and_then(|(camera, camera_transform)| {
                    camera
                        .viewport_to_world_2d(camera_transform, location.position)
                        .ok()
                })
        })
        .next();

    // Had to add this bc the dragging never stopped
    if !mouse_button.pressed(MouseButton::Left) {
        for (entity, _) in q_dragging.iter() {
            commands.entity(entity).remove::<DragState>();
        }
    }

    // Handle dragging for buildings that are already being dragged
    if let Some(cursor_pos) = cursor_world_pos {
        for (entity, drag_state) in q_dragging.iter() {
            if let Ok((_, _, _, mut transform)) = q_buildings.get_mut(entity) {
                // Calculate desired position
                let desired_pos = Vec2::new(
                    cursor_pos.x - drag_state.offset.x,
                    cursor_pos.y - drag_state.offset.y,
                );

                // Snap to grid if we have a tilemap
                let snapped_pos = if let Some((
                    map_size,
                    grid_size,
                    map_type,
                    tile_size,
                    anchor,
                    tilemap_transform,
                )) = q_tilemaps.iter().next()
                {
                    // Convert world position to tilemap local space
                    let inverse_transform = tilemap_transform.affine().inverse();
                    let local_pos = inverse_transform
                        .transform_point3(desired_pos.extend(0.0))
                        .truncate();

                    // Get tile position
                    if let Some(tile_pos) = TilePos::from_world_pos(
                        &local_pos, map_size, grid_size, tile_size, map_type, anchor,
                    ) {
                        // Convert tile position back to world position (center of tile)
                        let tile_center = tile_pos
                            .center_in_world(map_size, grid_size, tile_size, map_type, anchor);
                        let world_pos = tilemap_transform.transform_point(tile_center.extend(0.0));
                        world_pos.truncate()
                    } else {
                        desired_pos
                    }
                } else {
                    desired_pos
                };

                transform.translation.x = snapped_pos.x;
                transform.translation.y = snapped_pos.y;
            }
        }
    }

    for (entity, interaction, mut sprite, transform) in q_buildings.iter_mut() {
        let is_dragging = q_dragging.get(entity).is_ok();

        match interaction {
            PickingInteraction::Pressed => {
                if !is_dragging && mouse_button.pressed(MouseButton::Left) {
                    if let Some(cursor_pos) = cursor_world_pos {
                        let building_pos = transform.translation.truncate();
                        let offset = cursor_pos - building_pos;
                        commands.entity(entity).insert(DragState { offset });
                    }
                }
                sprite.color = Color::srgb(1.0, 0.5, 0.5); // Reddish tint
            }
            PickingInteraction::Hovered => {
                // Only highlight if not being dragged
                if !is_dragging {
                    sprite.color = Color::srgb(1.3, 1.3, 1.0); // Bright yellow tint
                } else {
                    sprite.color = Color::srgb(1.0, 0.5, 0.5);
                }
            }
            PickingInteraction::None => {
                // Reset color if not dragging
                if !is_dragging {
                    sprite.color = Color::WHITE;
                } else {
                    sprite.color = Color::srgb(1.0, 0.5, 0.5);
                }
            }
        }
    }
}
