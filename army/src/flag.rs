use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{AppRuleExt, Replicated, SyncRelatedAppExt};
use interaction::{InteractionTriggeredEvent, InteractionType};
use lobby::PlayerColor;
use physics::{attachment::AttachedTo, movement::BoxCollider};
use serde::{Deserialize, Serialize};
use shared::map::Layers;
use units::UnitType;

pub struct FlagPlugins;

impl Plugin for FlagPlugins {
    fn build(&self, app: &mut App) {
        app.replicate::<FlagHolder>()
            .replicate::<FlagDestroyed>()
            .replicate_bundle::<(Flag, Transform)>()
            .sync_related_entities::<FlagAssignment>();

        app.add_message::<DropFlagEvent>()
            .add_message::<PickFlagEvent>()
            .add_systems(
                FixedUpdate,
                (
                    flag_interact.run_if(on_message::<InteractionTriggeredEvent>),
                    drop_flag.run_if(on_message::<DropFlagEvent>),
                    pick_flag.run_if(on_message::<PickFlagEvent>),
                ),
            );
    }
}

#[derive(Component, Deserialize, Serialize, Debug)]
#[require(
    Replicated,
    Sprite,
    Anchor::BOTTOM_CENTER,
    BoxCollider = flag_collider(),
    Transform = Transform {translation: Vec3::new(0., 0., Layers::Flag.as_f32()), ..default()}
)]
pub struct Flag {
    #[entities]
    pub original_building: Entity,
    pub unit_type: UnitType,
    pub color: PlayerColor,
}

/// This component is added on Player. Tuple entity is flag.
#[derive(Component, Clone, Copy, Deref, DerefMut, Deserialize, Serialize)]
#[require(Replicated)]
pub struct FlagHolder(#[entities] pub Entity);

#[derive(Component, Deserialize, Serialize, Deref, DerefMut)]
#[relationship(relationship_target = FlagUnits)]
pub struct FlagAssignment(#[entities] pub Entity);

#[derive(Component, Deref, DerefMut, Serialize, Deserialize)]
#[relationship_target(relationship = FlagAssignment)]
pub struct FlagUnits(#[entities] Vec<Entity>);

fn flag_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(15., 20.),
        offset: Some(Vec2::new(0., 10.)),
    }
}

#[derive(Message)]
pub struct DropFlagEvent {
    player: Entity,
    pub flag: Entity,
}

#[derive(Message)]
pub struct PickFlagEvent {
    player: Entity,
    pub flag: Entity,
}

#[derive(Component, Serialize, Deserialize)]
pub struct FlagDestroyed;

fn flag_interact(
    mut interactions: MessageReader<InteractionTriggeredEvent>,
    mut drop_flag: MessageWriter<DropFlagEvent>,
    mut pick_flag: MessageWriter<PickFlagEvent>,
    flag_holder: Query<Option<&FlagHolder>>,
) -> Result {
    for event in interactions.read() {
        let InteractionType::Flag = &event.interaction else {
            continue;
        };

        let player = event.player;
        let has_flag = flag_holder.get(player)?;

        match has_flag {
            Some(_) => {
                drop_flag.write(DropFlagEvent {
                    player,
                    flag: event.interactable,
                });
            }
            None => {
                pick_flag.write(PickFlagEvent {
                    player,
                    flag: event.interactable,
                });
            }
        }
    }
    Ok(())
}

fn pick_flag(
    mut pick_flag: MessageReader<PickFlagEvent>,
    mut flag_query: Query<&mut Transform>,
    mut commands: Commands,
) -> Result {
    for event in pick_flag.read() {
        let mut transform = flag_query.get_mut(event.flag)?;

        transform.translation.y = 10.;

        commands.entity(event.flag).insert(AttachedTo(event.player));
        commands.entity(event.player).insert(FlagHolder(event.flag));
    }
    Ok(())
}

fn drop_flag(
    mut drop_flag: MessageReader<DropFlagEvent>,
    mut flag_query: Query<&mut Transform>,
    mut commands: Commands,
) -> Result {
    for event in drop_flag.read() {
        let mut transform = flag_query.get_mut(event.flag)?;

        transform.translation.y = 0.;

        commands.entity(event.flag).remove::<AttachedTo>();
        commands.entity(event.player).remove::<FlagHolder>();
    }
    Ok(())
}
