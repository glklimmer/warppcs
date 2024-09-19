use bevy::prelude::*;

use super::animation::UnitAnimation;

pub struct KingPlugin;

impl Plugin for KingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaladinSpriteSheet>();
        app.init_resource::<WarriorSpriteSheet>();
    }
}

pub struct AnimationsAtlasLayout {
    pub idle: Handle<TextureAtlasLayout>,
    pub walk: Handle<TextureAtlasLayout>,
    pub attack: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Debug)]
pub struct AnimationSetting {
    pub state: UnitAnimation,
    pub config: AnimationConfig,
}

#[derive(Component, Debug, Clone)]
pub struct AnimationReferences {
    pub idle: AnimationConfig,
    pub walk: AnimationConfig,
    pub attack: AnimationConfig,
}

#[derive(Component, Debug, Clone)]
pub struct AnimationConfig {
    pub layout_handle: Handle<TextureAtlasLayout>,
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub frame_timer: Timer,
}

// ------  Paladin ----- //
#[derive(Resource)]
pub struct PaladinSpriteSheet {
    pub texture: Handle<Image>,
    pub animations_atlas_layout: AnimationsAtlasLayout,
}

impl FromWorld for PaladinSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture_handle: Handle<Image> = asset_server.load("f1_general.png");
        let layout_walk =
            TextureAtlasLayout::from_grid(UVec2::splat(100), 1, 8, Some(UVec2::new(1, 1)), None);
        let layout_idle = TextureAtlasLayout::from_grid(
            UVec2::splat(100),
            1,
            8,
            Some(UVec2::new(1, 1)),
            Some(UVec2::new(100, 1)),
        );

        let layout_attack = TextureAtlasLayout::from_grid(
            UVec2::splat(100),
            1,
            8,
            Some(UVec2::new(1, 1)),
            Some(UVec2::new(700, 1)),
        );

        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        PaladinSpriteSheet {
            texture: texture_handle,
            animations_atlas_layout: AnimationsAtlasLayout {
                idle: texture_atlas_layouts.add(layout_idle),
                walk: texture_atlas_layouts.add(layout_walk),
                attack: texture_atlas_layouts.add(layout_attack),
            },
        }
    }
}

#[derive(Component, Debug)]
pub struct Paladin;

#[derive(Bundle, Debug)]
pub struct PaladinBundle {
    pub paladin: Paladin,
    pub sprite_sheet: SpriteBundle,
    pub texture_atlas: TextureAtlas,
    pub state: UnitAnimation,
    pub current_animation: AnimationSetting,
    pub animations: AnimationReferences,
}

impl PaladinBundle {
    pub fn new(
        paladin_sprite_sheet: &Res<PaladinSpriteSheet>,
        translation: [f32; 3],
        initial_state: UnitAnimation,
    ) -> Self {
        let idle_handle = paladin_sprite_sheet.animations_atlas_layout.idle.clone();
        let walk_handle = paladin_sprite_sheet.animations_atlas_layout.walk.clone();
        let attack_handle = paladin_sprite_sheet.animations_atlas_layout.attack.clone();

        let layout_handle = match initial_state {
            UnitAnimation::Idle => &idle_handle,
            UnitAnimation::Walk => &walk_handle,
            UnitAnimation::Attack => &attack_handle,
        };

        PaladinBundle {
            paladin: Paladin,
            sprite_sheet: SpriteBundle {
                texture: paladin_sprite_sheet.texture.clone(),
                transform: Transform {
                    translation: Vec3::new(translation[0], translation[1], translation[2]),
                    scale: Vec3::splat(2.),
                    ..Default::default()
                },
                ..Default::default()
            },
            state: initial_state,
            texture_atlas: TextureAtlas {
                layout: layout_handle.clone(),
                index: 7,
            },
            animations: AnimationReferences {
                idle: AnimationConfig {
                    layout_handle: idle_handle.clone(),
                    first_sprite_index: 7,
                    last_sprite_index: 0,
                    frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
                },
                walk: AnimationConfig {
                    layout_handle: walk_handle.clone(),
                    first_sprite_index: 7,
                    last_sprite_index: 0,
                    frame_timer: Timer::from_seconds(1. / 15., TimerMode::Repeating),
                },
                attack: AnimationConfig {
                    layout_handle: attack_handle.clone(),
                    first_sprite_index: 7,
                    last_sprite_index: 0,
                    frame_timer: Timer::from_seconds(1. / 15., TimerMode::Repeating),
                },
            },
            current_animation: AnimationSetting {
                state: UnitAnimation::Idle,
                config: AnimationConfig {
                    layout_handle: idle_handle.clone(),
                    first_sprite_index: 7,
                    last_sprite_index: 0,
                    frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
                },
            },
        }
    }
}

// ------  Warrior ----- //

#[derive(Resource)]
pub struct WarriorSpriteSheet {
    pub texture: Handle<Image>,
    pub animations_atlas_layout: AnimationsAtlasLayout,
}

impl FromWorld for WarriorSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture_handle: Handle<Image> = asset_server.load("f5_general.png");

        let layout_walk = TextureAtlasLayout::from_grid(
            UVec2::splat(80),
            1,
            10,
            Some(UVec2::new(1, 1)),
            Some(UVec2::new(81 * 4, 1)),
        );

        let layout_idle = TextureAtlasLayout::from_grid(
            UVec2::splat(80),
            1,
            10,
            Some(UVec2::new(1, 1)),
            Some(UVec2::new(81 * 3, 1)),
        );

        let layout_attack = TextureAtlasLayout::from_grid(
            UVec2::splat(80),
            1,
            10,
            Some(UVec2::new(1, 1)),
            Some(UVec2::new(81, 1)),
        );

        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        WarriorSpriteSheet {
            texture: texture_handle,
            animations_atlas_layout: AnimationsAtlasLayout {
                idle: texture_atlas_layouts.add(layout_idle),
                walk: texture_atlas_layouts.add(layout_walk),
                attack: texture_atlas_layouts.add(layout_attack),
            },
        }
    }
}

#[derive(Component, Debug)]
pub struct Warrior;

#[derive(Bundle, Debug)]
pub struct WarriorBundle {
    pub warrior: Warrior,
    pub sprite_sheet: SpriteBundle,
    pub texture_atlas: TextureAtlas,
    pub state: UnitAnimation,
    pub current_animation: AnimationSetting,
    pub animations: AnimationReferences,
}

impl WarriorBundle {
    pub fn new(
        paladin_sprite_sheet: &Res<WarriorSpriteSheet>,
        translation: [f32; 3],
        initial_state: UnitAnimation,
    ) -> Self {
        let idle_handle = paladin_sprite_sheet.animations_atlas_layout.idle.clone();
        let walk_handle = paladin_sprite_sheet.animations_atlas_layout.walk.clone();
        let attack_handle = paladin_sprite_sheet.animations_atlas_layout.attack.clone();

        let layout_handle = match initial_state {
            UnitAnimation::Idle => &idle_handle,
            UnitAnimation::Walk => &walk_handle,
            UnitAnimation::Attack => &attack_handle,
        };

        WarriorBundle {
            warrior: Warrior,
            sprite_sheet: SpriteBundle {
                texture: paladin_sprite_sheet.texture.clone(),
                transform: Transform {
                    translation: Vec3::new(translation[0], translation[1], translation[2]),
                    scale: Vec3::splat(2.),
                    ..Default::default()
                },
                ..Default::default()
            },
            state: initial_state,
            texture_atlas: TextureAtlas {
                layout: layout_handle.clone(),
                index: 9,
            },
            animations: AnimationReferences {
                idle: AnimationConfig {
                    layout_handle: idle_handle.clone(),
                    first_sprite_index: 7,
                    last_sprite_index: 0,
                    frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
                },
                walk: AnimationConfig {
                    layout_handle: walk_handle.clone(),
                    first_sprite_index: 7,
                    last_sprite_index: 0,
                    frame_timer: Timer::from_seconds(1. / 20., TimerMode::Repeating),
                },
                attack: AnimationConfig {
                    layout_handle: attack_handle.clone(),
                    first_sprite_index: 7,
                    last_sprite_index: 0,
                    frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
                },
            },
            current_animation: AnimationSetting {
                state: UnitAnimation::Idle,
                config: AnimationConfig {
                    layout_handle: idle_handle.clone(),
                    first_sprite_index: 7,
                    last_sprite_index: 0,
                    frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
                },
            },
        }
    }
}
