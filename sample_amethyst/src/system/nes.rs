use amethyst::{
    core::{Transform, SystemDesc},
    derive::SystemDesc,
    ecs::prelude::{Join, ReadStorage, System, SystemData, World, WriteStorage},
};
use nes_emulator_rs;
#[derive(SystemDesc)]
pub struct NesSystem;

impl<'s> System<'s> for NesSystem {
    type SystemData = (
        WriteStorage<'s, crate::component::pixel::NesWrapper>,
    );

    fn run(&mut self, (mut nes): Self::SystemData) {
        for n in nes {
            n.nes.next();
        }
    }
}
