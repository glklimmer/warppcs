use bevy::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {}
}

// TODO: add building update event system

// fn update_building(
//     mut network_events: EventReader<NetworkEvent>,
//     mut commands: Commands,
//     buildings: Query<(Entity, &SceneBuildingIndicator, &Building)>,
//     asset_server: Res<AssetServer>,
// ) {
//     for event in network_events.read() {
//         if let ServerMessages::BuildingUpdate(BuildingUpdate { indicator, update }) = &event.message
//         {
//             for (entity, other_indicator, building) in buildings.iter() {
//                 if indicator.ne(other_indicator) {
//                     continue;
//                 }
//
//                 let texture = match update {
//                     UpdateType::Status { new_status } => building_texture(building, *new_status),
//                     UpdateType::Upgrade { upgraded_building } => {
//                         println!("Updating upgraded building: {:?}", upgraded_building);
//                         commands
//                             .entity(entity)
//                             .insert(building_collider(upgraded_building));
//                         building_texture(upgraded_building, BuildStatus::Built)
//                     }
//                 };
//                 let image = asset_server.load::<Image>(texture);
//                 commands.entity(entity).insert(Sprite {
//                     image,
//                     flip_x: building_flipped(indicator),
//                     ..default()
//                 });
//             }
//         }
//     }
// }
