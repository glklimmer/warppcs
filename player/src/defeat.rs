use bevy::prelude::*;

use bevy::ecs::entity::MapEntities;
use bevy_replicon::prelude::{Channel, ServerEventAppExt};
use lobby::ControlledPlayer;
use serde::{Deserialize, Serialize};
use shared::PlayerState;

pub struct DefeatPlugin;

impl Plugin for DefeatPlugin {
    fn build(&self, app: &mut App) {
        app.add_mapped_server_event::<PlayerDefeated>(Channel::Ordered)
            .add_observer(set_defeated)
            .add_systems(OnEnter(PlayerState::Defeated), defeat);
    }
}

#[derive(Event, Clone, Copy, Deserialize, Serialize, Deref)]
pub struct PlayerDefeated(Entity);

impl MapEntities for PlayerDefeated {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.get_mapped(self.0);
    }
}

fn set_defeated(
    trigger: On<PlayerDefeated>,
    player: Query<Entity, With<ControlledPlayer>>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    if let Ok(player) = player.single()
        && player == **trigger
    {
        next_state.set(PlayerState::Defeated);
    }
}

fn defeat(mut commands: Commands, assets: Res<AssetServer>) {
    let defeat_texture = assets.load::<Image>("sprites/ui/defeat.png");

    commands.spawn((
        Node {
            display: Display::Flex,
            width: Val::Px(500.0),
            height: Val::Px(350.0),
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Percent(30.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ImageNode::new(defeat_texture),
        children![(
            Text::new("Defeat"),
            TextColor(Color::BLACK),
            TextFont::from_font_size(60.)
        )],
    ));
}
