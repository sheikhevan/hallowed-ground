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

impl Plugin for TilemapPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            tilemap_picking_backend.in_set(PickingSystems::Backend),
        )
        .add_systems(Update, highlight_hovered_tiles);
    }
}

fn tilemap_picking_backend(
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
    mut pointer_hits: MessageWriter<PointerHits>,
) {
    for (pointer_id, pointer_location) in pointers.iter() {
        let Some(location) = pointer_location.location() else {
            continue;
        };

        // Find the active camera for this pointer
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

        // Now convert cursor position to world coords
        let Ok(cursor_world_pos) = camera.viewport_to_world_2d(camera_transform, location.position)
        else {
            continue;
        };

        // Check all the tilemaps for hits
        let mut hits: Vec<(Entity, HitData)> = Vec::new();
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
            // Skip invisible tilemaps
            if !visibility.get() {
                continue;
            }

            // Transform cursor position to tilemap local space
            let inverse_transform = tilemap_transform.affine().inverse();
            let local_cursor_pos = inverse_transform.transform_point3(cursor_world_pos.extend(0.0));
            let local_pos = local_cursor_pos.truncate();

            // Convert to tile position
            if let Some(tile_pos) = TilePos::from_world_pos(
                &local_pos, map_size, grid_size, tile_size, map_type, anchor,
            ) {
                // Check if tile exists and is visible
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    if let Ok(tile_visible) = tiles.get(tile_entity) {
                        if tile_visible.0 {
                            // Calculate depth for sorting
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

        // Send the hits to the picking system
        let order = camera.order as f32;
        pointer_hits.write(PointerHits::new(*pointer_id, hits, order));
    }
}

/// Highlights the tiles when hovered over
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
