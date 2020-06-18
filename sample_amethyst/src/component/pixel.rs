use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage},
    prelude::*,
    renderer::{ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
};

use crate::nes_state::{ARENA_HEIGHT, ARENA_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH};
use nes_emulator_rs::nes::NES;

struct Pixel {
    x: usize,
    y: usize,
}

pub struct NesWrapper {
    pub nes: NES,
}

impl Component for NesWrapper {
    type Storage = DenseVecStorage<Self>;
}

impl Pixel {
    fn new(x: usize, y: usize) -> Self {
        Pixel { x, y }
    }
}

impl Component for Pixel {
    type Storage = DenseVecStorage<Self>;
}

pub fn initialise_pixel(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    for i in 0..240 {
        for j in 0..256 {
            let mut transform = Transform::default();
            // transform.set_translation_xyz(ARENA_WIDTH / 2., ARENA_HEIGHT / 2., 0.0);
            transform.set_translation_xyz(
                SCREEN_WIDTH / 256.0 * j as f32,
                (SCREEN_HEIGHT / 256.0 * i as f32) + (ARENA_HEIGHT - SCREEN_HEIGHT),
                0.0,
            );
            let mut scale = transform.scale().clone();
            scale[0] = 1.0 / 16.0;
            scale[1] = 1.0 / 16.0;
            transform.set_scale(scale);

            // Assign the sprites for the paddles
            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet_handle.clone(),
                sprite_number: if i > 234 || j > 250 || i < 5 || j < 5 { 0 } else { i % 64 }, // ball is the second sprite in the sprite_sheet
            };

            // Create a left plank entity.
            world
                .create_entity()
                .with(Pixel::new())
                .with(transform)
                .with(sprite_render.clone())
                .build();
        }
    }
}
