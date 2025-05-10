use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::{
    Faction, Owner, Player, Vec3LayerExt,
    map::{
        Layers,
        buildings::{BuildStatus, Building, MainBuildingLevels, RecruitBuilding, WallLevels},
    },
    networking::{LobbyEvent, MountType, UnitType},
    server::{
        buildings::item_assignment::ItemAssignment,
        entities::{Unit, health::Health},
        players::{
            chest::Chest,
            interaction::{Interactable, InteractionType},
            mount::Mount,
        },
    },
};
use std::collections::VecDeque;

use super::{
    map::{LoadMap, Scene},
    travel::{Portal, TravelDestination},
};

pub struct StartGamePlugin;

impl Plugin for StartGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(start_game);
    }
}

fn start_game(
    trigger: Trigger<LoadMap>,
    mut players: Query<&mut Transform, With<Player>>,
    mut commands: Commands,
) {
    let map = &**trigger.event();
    for i in map.node_indices() {
        let node = &map[i];
        let offset = Vec3::new(10000. * i.index() as f32, 0., 0.);

        match node.scene {
            Scene::Player {
                player,
                left,
                right,
            } => {
                player_base(commands.reborrow(), offset, player, left, right);
                let mut transform = players.get_mut(player).unwrap();
                transform.translation = offset.with_z(Layers::Player.as_f32());
            }
            Scene::Bandit { left, right } => camp(commands.reborrow(), offset, left, right),
        };
    }

    for i in map.edge_indices() {
        // TODO: add portals
    }

    let len = map.len();
    for (i, (base_left, base_right, camp_left, _)) in map.iter().enumerate() {
        // base_left <-> previous_base.camp_right (3)
        let previous_index = if i == 0 { len - 1 } else { i - 1 };
        let previous_camp_right = map[previous_index].3;
        connect_portals(commands.reborrow(), *base_left, previous_camp_right);

        // base_right <-> camp_left
        connect_portals(commands.reborrow(), *base_right, *camp_left);
    }
}

fn connect_portals(mut commands: Commands, left: Entity, right: Entity) {
    commands.entity(left).insert(TravelDestination::new(right));
    commands.entity(right).insert(TravelDestination::new(left));
}

fn camp(mut commands: Commands, offset: Vec3, camp_left_portal: Entity, camp_right_portal: Entity) {
    for i in 1..10 {
        commands.spawn((
            Owner(Faction::Bandits),
            Unit {
                unit_type: UnitType::Bandit,
                swing_timer: Timer::default(),
            },
            Health { hitpoints: 20. },
            offset
                .offset_x(50. - 10. * i as f32)
                .with_layer(Layers::Unit),
        ));
    }
    commands
        .entity(camp_left_portal)
        .insert((Portal, offset.offset_x(-150.).with_layer(Layers::Building)));
    commands
        .entity(camp_right_portal)
        .insert((Portal, offset.offset_x(150.).with_layer(Layers::Building)));
}

fn player_base(
    mut commands: Commands,
    offset: Vec3,
    player: Entity,
    left_portal: Entity,
    right_portal: Entity,
) {
    let owner = Owner(Faction::Player(player));
    println!("onwer {:?}", player);
    commands.spawn((
        Building::MainBuilding {
            level: MainBuildingLevels::Tent,
        },
        Building::MainBuilding {
            level: MainBuildingLevels::Tent,
        }
        .collider(),
        BuildStatus::Built,
        offset.with_layer(Layers::Building),
        owner,
        RecruitBuilding,
        Interactable {
            kind: InteractionType::Recruit,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        Mount {
            mount_type: MountType::Horse,
        },
        offset.offset_x(50.).with_layer(Layers::Mount),
    ));
    commands.spawn((
        Chest::Normal,
        offset.offset_x(-50.).with_layer(Layers::Chest),
    ));
    commands.spawn((
        RecruitBuilding,
        ItemAssignment::default(),
        offset.offset_x(135.).with_layer(Layers::Building),
        owner,
        Interactable {
            kind: InteractionType::ItemAssignment,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        RecruitBuilding,
        ItemAssignment::default(),
        offset.offset_x(-135.).with_layer(Layers::Building),
        owner,
        Interactable {
            kind: InteractionType::ItemAssignment,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        RecruitBuilding,
        ItemAssignment::default(),
        offset.offset_x(235.).with_layer(Layers::Building),
        owner,
        Interactable {
            kind: InteractionType::ItemAssignment,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        Building::Wall {
            level: WallLevels::Basic,
        },
        offset.offset_x(390.).with_layer(Layers::Wall),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        Building::Wall {
            level: WallLevels::Basic,
        },
        offset.offset_x(-345.).with_layer(Layers::Wall),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        Building::GoldFarm,
        offset.offset_x(320.).with_layer(Layers::Building),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        Building::GoldFarm,
        offset.offset_x(-265.).with_layer(Layers::Building),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(owner),
        },
    ));

    commands
        .entity(left_portal)
        .insert((Portal, offset.offset_x(-450.).with_layer(Layers::Building)));
    commands
        .entity(right_portal)
        .insert((Portal, offset.offset_x(450.).with_layer(Layers::Building)));
}
