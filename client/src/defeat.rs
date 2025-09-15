use bevy::prelude::*;
use shared::{PlayerState, server::entities::health::Defeat};

pub struct DefeatPlugin;

impl Plugin for DefeatPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(defeat);
    }
}

fn defeat(
    _: Trigger<Defeat>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<PlayerState>>,
    assets: Res<AssetServer>,
) {
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
    next_state.set(PlayerState::Defeated);
}
