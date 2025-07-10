#[macro_export]
macro_rules! anim {
    // from column 0 to the specified column
    ($row:expr, $last_col:expr) => {
        $crate::animations::SpriteSheetAnimation {
            first_sprite_index: $row * ATLAS_COLUMNS,
            last_sprite_index: $row * ATLAS_COLUMNS + $last_col,
            ..default()
        }
    };

    ($row:expr, $last_col:expr, $direction:expr) => {
        $crate::animations::SpriteSheetAnimation {
            first_sprite_index: $row * ATLAS_COLUMNS,
            last_sprite_index: $row * ATLAS_COLUMNS + $last_col,
            direction: $direction,
            ..default()
        }
    };
}

