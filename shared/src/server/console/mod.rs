use bevy::{
    app::Plugin,
    ecs::{entity::Entity, system::In, world::World},
    reflect::Map,
    remote::{BrpResult, RemotePlugin, http::RemoteHttpPlugin},
};
use console_protocol::*;
use serde_json::{Value, json};

use crate::{ClientPlayerMap, networking::UnitType, player_loadout::GiveRandomItem};

use super::{
    buildings::recruiting::RecruitEvent,
    players::items::{ItemBuilder, ItemType, ProjectileWeapon, WeaponType},
};

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            RemotePlugin::default()
                .with_method(BRP_TRIGGER_SPAWN_UNIT, spawn_unit_handler)
                .with_method(BRP_TRIGGER_RANDOM_ITEM, spawn_random_item),
            RemoteHttpPlugin::default(),
        ));
    }
}

fn spawn_unit_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    match params {
        Some(event) => match serde_json::from_value::<BrpSpawnUnit>(event) {
            Ok(unit) => {
                let client_player_map = world.get_resource::<ClientPlayerMap>().unwrap();

                let unit_type = match unit.unit.as_str() {
                    "archer" => UnitType::Archer,
                    "pikemen" => UnitType::Pikeman,
                    "shield" => UnitType::Shieldwarrior,
                    _ => UnitType::Archer,
                };
                let player = client_player_map
                    .get_at(unit.player as usize)
                    .unwrap()
                    .1
                    .try_downcast_ref::<Entity>()
                    .unwrap();

                world.trigger(RecruitEvent {
                    player: *player,
                    unit_type,
                    items: Some(vec![
                        ItemBuilder::default()
                            .with_type(ItemType::Weapon(WeaponType::Projectile(
                                ProjectileWeapon::Bow,
                            )))
                            .build(),
                        ItemBuilder::default().with_type(ItemType::Head).build(),
                        ItemBuilder::default().with_type(ItemType::Chest).build(),
                        ItemBuilder::default().with_type(ItemType::Feet).build(),
                    ]),
                });
                Ok(json!("succes"))
            }
            Err(e) => {
                eprintln!("{}", e);
                Ok(json!("succes"))
            }
        },
        None => Ok(json!("succes")),
    }
}

fn spawn_random_item(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    match params {
        Some(_) => Ok(json!("succes")),
        None => {
            world.trigger(GiveRandomItem);
            Ok(json!("succes"))
        }
    }
}
