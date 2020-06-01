mod cpu;
mod cpu_bus;
pub mod nes;
mod ppu;
mod screen;
mod wram;

use std::f64;
use std::panic;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

use wasm_bindgen_futures::JsFuture;
use web_sys::{CanvasRenderingContext2d, ImageData, console};
use js_sys::{Function, Promise};
use console_error_panic_hook;

use screen::SCREEN_SIZE;

static program: &'static [u8] = include_bytes!("../sample1/sample1.nes");

async fn execute() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let mut nes = nes::NES::load(program.to_vec());
    let p = program.to_vec().iter().map(|c| *c as char).collect::<String>();
    console::log_1(&p.into());

    loop {
        nes.next();
        let mut screen = nes.ppu.borrow().ppu_bus.screen.screen.clone();
        let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut screen), SCREEN_SIZE.0 as u32, SCREEN_SIZE.1 as u32).unwrap();
        context.put_image_data(&data, 0.0, 0.0).unwrap();

        // sleep
        let promise = Promise::new(&mut |resolve, _| {
            window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 100).unwrap();
        });
        let js_fut = JsFuture::from(promise);
        js_fut.await?;
        console::log_1(&data);
    }
}

#[wasm_bindgen]
pub async fn start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    wasm_bindgen_futures::spawn_local(async {
        execute().await.unwrap();
    });
}
