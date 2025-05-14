use bevy::prelude::*;

use bevy::remote::BrpError;
use bevy::{
    app::Plugin,
    ecs::{entity::Entity, system::In, world::World},
    reflect::Map,
    remote::{BrpResult, RemotePlugin, http::RemoteHttpPlugin},
};
use console_protocol::*;
use serde_json::{Value, json};

use crate::{ClientPlayerMap, Vec3LayerExt, map::Layers, networking::UnitType};

use super::{
    buildings::recruiting::RecruitEvent,
    physics::movement::Velocity,
    players::items::{Item, ItemBuilder, ItemType, ProjectileWeapon, Rarity, WeaponType},
};

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            RemotePlugin::default()
                .with_method(BRP_SPAWN_UNIT, spawn_unit_handler)
                .with_method(BRP_SPAWN_RANDOM_ITEM, spawn_random_items),
            RemoteHttpPlugin::default(),
        ));
    }
}

trait PlayerCommand {
    fn player(&self) -> u8;

    fn player_entity(&self, world: &mut World) -> BrpResult<Entity> {
        let client_player_map = world
            .get_resource::<ClientPlayerMap>()
            .ok_or_else(|| BrpError::internal("Missing ClientPlayerMap resource"))?;
        let (_, player) = client_player_map
            .get_at(self.player() as usize)
            .ok_or_else(|| BrpError::internal("Player index out of bounds"))?;
        let entity = player
            .try_downcast_ref::<Entity>()
            .ok_or_else(|| BrpError::internal("Value in ClientPlayerMap wasn’t an Entity"))?;
        Ok(*entity)
    }
}

impl PlayerCommand for BrpSpawnItems {
    fn player(&self) -> u8 {
        self.player
    }
}
impl PlayerCommand for BrpSpawnUnit {
    fn player(&self) -> u8 {
        self.player
    }
}

fn spawn_unit_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult<Value> {
    let value = params.ok_or_else(|| BrpError::internal("spawn-units requires parameters"))?;

    let unit_req: BrpSpawnUnit = serde_json::from_value(value)
        .map_err(|e| BrpError::internal(format!("invalid spawn parameters: {}", e)))?;

    let unit_type = match unit_req.unit.as_str() {
        "archer" => UnitType::Archer,
        "pikemen" => UnitType::Pikeman,
        "shield" => UnitType::Shieldwarrior,
        other => {
            return Err(BrpError::internal(format!("unknown unit type `{}`", other)));
        }
    };

    let player = unit_req.player_entity(world)?;

    world.trigger(RecruitEvent::new(
        player,
        unit_type,
        Some(vec![
            ItemBuilder::default()
                .with_type(ItemType::Weapon(WeaponType::Projectile(
                    ProjectileWeapon::Bow,
                )))
                .build(),
            ItemBuilder::default().with_type(ItemType::Head).build(),
            ItemBuilder::default().with_type(ItemType::Chest).build(),
            ItemBuilder::default().with_type(ItemType::Feet).build(),
        ]),
    ));

    Ok(json!("success"))
}

fn spawn_random_items(In(params): In<Option<serde_json::Value>>, world: &mut World) -> BrpResult {
    if let Some(value) = params {
        if let Ok(brp) = serde_json::from_value::<BrpSpawnItems>(value) {
            let player_entity = brp.player_entity(world)?;

            let player_pos = {
                let mut query: QueryState<&Transform> = QueryState::new(world);
                let transform = query.get(world, player_entity).unwrap();
                transform.translation
            };

            for item_type in ItemType::all_variants() {
                let item = Item::builder()
                    .with_rarity(Rarity::Common)
                    .with_type(item_type)
                    .build();

                world.spawn((
                    item.collider(),
                    item,
                    player_pos.with_y(12.5).with_layer(Layers::Item),
                    Velocity(Vec2::new((fastrand::f32() - 0.5) * 100., 100.)),
                ));
            }
        }
    }

    Ok(json!("success"))
}
