use bevy::{
    app::Plugin,
    ecs::{system::In, world::World},
    remote::{BrpResult, RemotePlugin, http::RemoteHttpPlugin},
};
use bevy_replicon::shared::SERVER;
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
                .with_method(BRP_TRIGGER_RANDOM_ITEMS, spawn_random_items),
            RemoteHttpPlugin::default(),
        ));
    }
}

fn spawn_unit_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    match params {
        Some(event) => match serde_json::from_value::<BrpSpawnUnit>(event) {
            Ok(unit) => {
                let unit_type = match unit.unit.as_str() {
                    "archer" => UnitType::Archer,
                    "pikeman" => UnitType::Pikeman,
                    "shield" => UnitType::Shieldwarrior,
                    _ => UnitType::Archer,
                };
                let client_player_map = world.get_resource::<ClientPlayerMap>().unwrap();
                world.trigger(RecruitEvent {
                    player: *client_player_map.get(&SERVER).unwrap(),
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

fn spawn_random_items(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    match params {
        Some(_) => Ok(json!("succes")),
        None => {
            world.trigger(GiveRandomItem);
            Ok(json!("succes"))
        }
    }
}
