use bevy::prelude::*;
use shared::{Vec3LayerExt, server::buildings::item_assignment::ItemAssignment};

pub struct ItemAssignmentPlugin;

impl Plugin for ItemAssignmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(open_assignment_dialog);
    }
}

fn open_assignment_dialog(
    mut commands: Commands,
    building: Query<(&Transform, &ItemAssignment)>,
    asset_server: Res<AssetServer>,
) {
    let (transform, assignment) = building.get(event.interactable).unwrap();
    let translation = transform.translation;

    for maybe_slot in assignment.items.iter() {
        let slot_sprite = commands.spawn((
            Sprite {
                image: asset_server.load::<Image>("sprites/ui/slot.png"),
                ..Default::default()
            },
            translation.offset_x(50.).with_layer(Layers::UI),
        ));

        let Some(item) = maybe_slot else {
            continue;
        };

        match item.item_type {
            ItemType::Weapon(_) => todo!(),
            ItemType::Chest => todo!(),
            ItemType::Feet => todo!(),
            ItemType::Head => todo!(),
        }

        commands.spawn(
            (Sprite {
                image: todo!(),
                ..Default::default()
            }),
        );

        slot_sprite.add_child(child);
    }
}
