use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiTextureHandle, EguiUserTextures, egui};
use std::collections::HashMap;

#[derive(Resource)]
pub struct Images {
    pub basic_chapel: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct EguiTextureCache {
    pub cache: HashMap<String, egui::TextureId>,
}

#[derive(Message)]
pub struct DebugSpawnBuildingMsg {
    pub name: String,
    pub image_handle: Handle<Image>,
}

#[derive(Component)]
pub struct Building;

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            basic_chapel: asset_server.load("basic_chapel.png"),
        }
    }
}

pub fn register_textures(
    images: Res<Images>,
    mut texture_cache: ResMut<EguiTextureCache>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
) {
    // You (the developer) need to add entries here as you add more placements
    let image_list = [("Basic Chapel", &images.basic_chapel)];

    for (name, handle) in image_list.iter() {
        texture_cache
            .cache
            .entry(name.to_string())
            .or_insert_with(|| {
                egui_user_textures.add_image(EguiTextureHandle::Strong((*handle).clone()))
            });
    }
}

pub fn debug_ui(
    mut contexts: EguiContexts,
    texture_cache: Res<EguiTextureCache>,
    images: Res<Images>,
    mut spawn_msgs: MessageWriter<DebugSpawnBuildingMsg>,
) -> Result {
    // You (the developer) need to add the names of entries here. They must match names in
    // register_textures
    let image_list = [("Basic Chapel", &images.basic_chapel)];

    egui::Window::new("DEBUG").show(contexts.ctx_mut()?, |ui| {
        ui.heading("Buildings");

        for (name, handle) in image_list.iter() {
            if let Some(texture_id) = texture_cache.cache.get(*name) {
                ui.horizontal(|ui| {
                    ui.collapsing(*name, |ui| {
                        ui.image((*texture_id, egui::vec2(192.0, 192.0)));
                        if ui.button("Spawn").clicked() {
                            spawn_msgs.write(DebugSpawnBuildingMsg {
                                name: name.to_string(),
                                image_handle: (*handle).clone(),
                            });
                        }
                    });
                });
            }
        }
    });
    Ok(())
}

pub fn debug_handle_spawn_building(
    mut commands: Commands,
    mut spawn_msgs: MessageReader<DebugSpawnBuildingMsg>,
) {
    for msg in spawn_msgs.read() {
        info!("DEBUG: Spawning building: {}", msg.name);

        commands.spawn((
            Sprite {
                image: msg.image_handle.clone(),
                color: Color::WHITE,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
            Building,
            Pickable::default(),
            Name::new(msg.name.clone()),
        ));
    }
}
