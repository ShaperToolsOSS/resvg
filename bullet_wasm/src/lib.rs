mod utils;
use js_sys::{Uint8Array};

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen]
pub fn js_process_svg_str_to_usvg_str(s: &str) -> String{
  bullet_svg::process_svg_str_to_usvg_str(s).unwrap()
}


#[wasm_bindgen]
pub fn js_init_svg_parser(){
  //TODO - move this to initialization
  utils::set_panic_hook();
  bullet_svg::init_fontdb();
}

#[wasm_bindgen]
pub fn js_add_font(font_data: &Uint8Array){
  bullet_svg::add_font_to_fontdb(font_data.to_vec());
}

#[wasm_bindgen]
pub fn js_set_render_dpi(render_dpi: f64){
  bullet_svg::set_render_dpi(render_dpi);
}

// #[wasm_bindgen]
// extern {
//     fn alert(s: &str);
// }

// #[wasm_bindgen]
// pub fn greet() {
//     alert("Hello, bullet-wasm!");
// }

