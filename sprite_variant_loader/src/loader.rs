use bevy::prelude::*;

use bevy::asset::{io::Reader, AssetLoader, LoadContext, RenderAssetUsages};
use image::{DynamicImage, Rgba};
use serde::{Deserialize, Serialize};
use shared::enum_map::{EnumIter, EnumMap};
use shared::PlayerColor;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Default, Serialize, Deserialize)]
pub struct SpriteVariantSettings;

#[derive(Asset, TypePath, Clone)]
pub struct SpriteVariants {
    pub variants: EnumMap<PlayerColor, Handle<Image>>,
}

pub trait SpriteVariantsAssetsExt {
    fn get_variant(&self, handle: &Handle<SpriteVariants>) -> Result<&SpriteVariants>;
}

impl SpriteVariantsAssetsExt for Assets<SpriteVariants> {
    fn get_variant(&self, handle: &Handle<SpriteVariants>) -> Result<&SpriteVariants> {
        self.get(handle)
            .ok_or("Sprite variant not generated yet".into())
    }
}

#[derive(Debug, Clone, Copy)]
struct ColorPalette {
    pub primary: Rgba<u8>,
    pub secondary: Rgba<u8>,
}

impl From<&PlayerColor> for ColorPalette {
    fn from(value: &PlayerColor) -> Self {
        match value {
            PlayerColor::Blue => ColorPalette {
                primary: Rgba([64, 88, 192, 255]),
                secondary: Rgba([96, 152, 232, 255]),
            },
            PlayerColor::Red => ColorPalette {
                primary: Rgba([172, 50, 50, 255]),
                secondary: Rgba([217, 87, 99, 255]),
            },
            PlayerColor::Green => ColorPalette {
                primary: Rgba([50, 172, 50, 255]),
                secondary: Rgba([99, 217, 99, 255]),
            },
            PlayerColor::Yellow => ColorPalette {
                primary: Rgba([255, 215, 0, 255]),
                secondary: Rgba([255, 235, 59, 255]),
            },
            PlayerColor::Purple => ColorPalette {
                primary: Rgba([155, 89, 182, 255]),
                secondary: Rgba([189, 142, 212, 255]),
            },
            PlayerColor::Orange => ColorPalette {
                primary: Rgba([255, 165, 0, 255]),
                secondary: Rgba([255, 193, 7, 255]),
            },
            PlayerColor::Cyan => ColorPalette {
                primary: Rgba([0, 180, 180, 255]),
                secondary: Rgba([87, 219, 219, 255]),
            },
            PlayerColor::Magenta => ColorPalette {
                primary: Rgba([180, 0, 180, 255]),
                secondary: Rgba([219, 87, 219, 255]),
            },
            PlayerColor::Pink => ColorPalette {
                primary: Rgba([255, 105, 180, 255]),
                secondary: Rgba([255, 182, 193, 255]),
            },
            PlayerColor::Brown => ColorPalette {
                primary: Rgba([150, 75, 0, 255]),
                secondary: Rgba([181, 101, 29, 255]),
            },
            PlayerColor::Teal => ColorPalette {
                primary: Rgba([0, 128, 128, 255]),
                secondary: Rgba([72, 209, 204, 255]),
            },
            PlayerColor::Gray => ColorPalette {
                primary: Rgba([128, 128, 128, 255]),
                secondary: Rgba([192, 192, 192, 255]),
            },
        }
    }
}

#[derive(Default)]
pub struct SpriteVariantLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SpriteVariantLoaderError {
    #[error("Could not load variant sprite: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not decode image: {0}")]
    Decode(#[from] image::ImageError),
}

impl AssetLoader for SpriteVariantLoader {
    type Asset = SpriteVariants;
    type Settings = SpriteVariantSettings;
    type Error = SpriteVariantLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &SpriteVariantSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)?;

        let mut variants = HashMap::new();

        for color in PlayerColor::all_variants() {
            let recolored_image = create_recolored_texture(&image, color);

            let bevy_image =
                Image::from_dynamic(recolored_image, true, RenderAssetUsages::default());

            let image_handle =
                load_context.add_labeled_asset(format!("variant_{color:?}"), bevy_image);
            variants.insert(color, image_handle);
        }
        let variants = EnumMap::new(|color: PlayerColor| variants.get(&color).unwrap().clone());

        Ok(SpriteVariants { variants })
    }

    fn extensions(&self) -> &[&str] {
        &["png"]
    }
}

fn create_recolored_texture(original: &DynamicImage, target_color: &PlayerColor) -> DynamicImage {
    let mut cloned = original.clone();

    if let Some(rgba_image) = cloned.as_mut_rgba8() {
        let blue_palette: ColorPalette = (&PlayerColor::Blue).into();
        let target_palette: ColorPalette = target_color.into();

        let mut color_map = HashMap::new();
        color_map.insert(blue_palette.primary, target_palette.primary);
        color_map.insert(blue_palette.secondary, target_palette.secondary);

        for (_x, _y, color) in rgba_image.enumerate_pixels_mut() {
            if let Some(new_color) = color_map.get(color) {
                *color = *new_color;
            }
        }
    }

    cloned
}
