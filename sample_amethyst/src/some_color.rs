use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage},
    prelude::*,
    renderer::{ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
};

use crate::nes_state::{ARENA_HEIGHT, ARENA_WIDTH};

pub struct SomeColor{}
impl SomeColor {
    fn new() -> SomeColor {
        SomeColor {}
    }
}

impl Component for SomeColor {
    type Storage = DenseVecStorage<Self>;
}

pub fn initialise_some_color(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    let mut transform = Transform::default();
    // Correctly position the paddles.
    transform.set_translation_xyz(ARENA_WIDTH / 2., ARENA_HEIGHT / 2., 0.0);

    // Assign the sprites for the paddles
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 1, // ball is the second sprite in the sprite_sheet
    };

    // Create a left plank entity.
    world
        .create_entity()
        .with(SomeColor::new())
        .with(transform)
        .with(sprite_render.clone())
        .build();
}