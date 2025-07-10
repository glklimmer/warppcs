#[macro_export]
macro_rules! anim {
    ($row:expr, $last_col:expr) => {
        $crate::animations::SpriteSheetAnimation {
            first_sprite_index: $row * ATLAS_COLUMNS,
            last_sprite_index: $row * ATLAS_COLUMNS + $last_col,
            ..default()
        }
    };
}

#[macro_export]
macro_rules! anim_reverse {
    ($row:expr, $last_col:expr) => {
        $crate::animations::SpriteSheetAnimation {
            first_sprite_index: $row * ATLAS_COLUMNS + $last_col,
            last_sprite_index: $row * ATLAS_COLUMNS,
            direction: $crate::animations::AnimationDirection::Backward,
            ..default()
        }
    };
}
