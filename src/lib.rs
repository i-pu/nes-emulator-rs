mod cpu;
mod cpu_bus;
pub mod nes;
mod ppu;
mod screen;
mod wram;

use std::f64;
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;

use console_error_panic_hook;
use js_sys::{Function, Promise};
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, CanvasRenderingContext2d, ImageData};

use screen::{SCREEN_SIZE, DEBUG_SCREEN_SIZE};

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
    let p = program
        .to_vec()
        .iter()
        .map(|c| *c as char)
        .collect::<String>();
    console::log_1(&p.into());

    let mut cycles = 0;
    // let hz = 179_0000i32;
    let fps = 1;
    loop {
        // 1.79 MHz
        cycles += nes.next();

        // 1 frame
        if cycles % 1000 == 0 {
            // 60 fps
            let mut screen = nes.ppu.borrow().ppu_bus.screen.screen.clone();
            let mut debug_screen = nes.ppu.borrow().ppu_bus.screen.debug_screen.clone();
            let ppu = nes.ppu.borrow();
            let frames = ppu.frames;
            let data = ImageData::new_with_u8_clamped_array_and_sh(
                Clamped(&mut screen),
                SCREEN_SIZE.0 as u32,
                SCREEN_SIZE.1 as u32,
            )
            .unwrap();

            let debug_data = ImageData::new_with_u8_clamped_array_and_sh(
                Clamped(&mut debug_screen),
                DEBUG_SCREEN_SIZE.0 as u32,
                DEBUG_SCREEN_SIZE.1 as u32,
            )
            .unwrap();

            context.clear_rect(0f64, 0f64, 640 as f64, 480 as f64);

            // rendering
            context.put_image_data(&data, 0.0, 0.0).unwrap();

            // draw debug screen
            context.put_image_data(&debug_data, SCREEN_SIZE.0 as f64 + 10.0 , 0.0).unwrap();

            // debug info
            context.fill_text(&format!("{}", frames), SCREEN_SIZE.0 as f64 + 20., SCREEN_SIZE.1 as f64 + 20.);


            console::log_1(&"complete rendering".into());
            cycles = 0;

            // sleep
            let promise = Promise::new(&mut |resolve, _| {
                window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, (1000 / fps) as i32)
                    .unwrap();
            });
            let js_fut = JsFuture::from(promise);
            js_fut.await?;
        }
    }
}

#[wasm_bindgen]
pub async fn start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    wasm_bindgen_futures::spawn_local(async {
        execute().await.unwrap();
    });
}
