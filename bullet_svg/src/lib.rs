use std::cell::RefCell;
// use usvg::Error;

use svgtypes::LengthUnit;

// use include_dir_macro::include_dir;

thread_local!(static BULLET_SVG_OPT : RefCell<usvg::Options> = RefCell::new(usvg::Options::default()));


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}




fn load_static_font_data(db: &mut usvg::fontdb::Database){
    
    //Fonts have been omitted to avoid potential licensing issues, but code is left here as an example.

    //Static font loading

    //Sans Serif fonts
    // db.load_font_data(include_bytes!("../fonts/Roboto/Roboto-Regular.ttf").to_vec());

    // db.set_sans_serif_family("Roboto");

    // //Cursive
    // db.load_font_data(include_bytes!("../fonts/ComicNeue/ComicNeue-Bold.ttf").to_vec());

    // db.set_cursive_family("Comic Neue, Bold");

    // //Fantasy
    // db.load_font_data(include_bytes!("../fonts/Oswald/Oswald-VariableFont_wght.ttf").to_vec());

    // db.set_fantasy_family("Oswald");

    // //Monospace
    // db.load_font_data(include_bytes!("../fonts/Roboto_Mono/RobotoMono-VariableFont_wght.ttf").to_vec());

    // db.set_monospace_family("Roboto Mono");

    // //IBM Plex Sans
    // db.load_font_data(include_bytes!("../fonts/IBM Plex Sans/IBMPlexSans-Regular.ttf").to_vec());

    // //Serif fonts
    // db.load_font_data(include_bytes!("../fonts/Roboto_Slab/RobotoSlab-Regular.ttf").to_vec());
    // db.set_serif_family("Roboto Slab");

    // db.load_font_data(include_bytes!("../fonts/Studio Fonts/SansSerif/NationalPark/NationalPark-Regular.otf").to_vec());

    // Static load all fonts in directory
    //Specify directory relative to Cargo.toml
    // let sherpa_font_files = include_dir!("./fonts/Studio Fonts/");

    // for (_path, font_data) in &sherpa_font_files{
    //     db.load_font_data(font_data.to_vec());
    // }
}

pub fn init_fontdb(){
    BULLET_SVG_OPT.with(|bullet_svg_opt_cell| {
        //Get static parser options
        let mut re_opt = bullet_svg_opt_cell.borrow_mut();

        //Load statically bundled fonts
        let mut db = usvg::fontdb::Database::new();
        load_static_font_data(&mut db);
        re_opt.fontdb = db;

        //Set fallback font to a statically loaded font to ensure it is always available.
        re_opt.font_family = std::string::String::from("National Park");
    });
}

// Load font dynamically via bullet_wasm API
pub fn add_font_to_fontdb(font_data: std::vec::Vec<u8>){
     BULLET_SVG_OPT.with(|bullet_svg_opt_cell| {
        //Get static parser options
        let mut re_opt = bullet_svg_opt_cell.borrow_mut();
        re_opt.fontdb.load_font_data(font_data);
    });
}

pub fn set_render_dpi(dpi_render: f64){
    BULLET_SVG_OPT.with(|bullet_svg_opt_cell| {
        //Get static parser options
        let mut re_opt = bullet_svg_opt_cell.borrow_mut();
        re_opt.dpi_render = dpi_render;
    });
}

pub fn set_units_dpi(dpi_units: f64){
    BULLET_SVG_OPT.with(|bullet_svg_opt_cell| {
        //Get static parser options
        let mut re_opt = bullet_svg_opt_cell.borrow_mut();
        re_opt.dpi_units = dpi_units;
    });
}

pub enum SvgGenerator {
    SmartRouter,
    Illustrator,
    Inkscape,
    Vectr,
    Affinity,
    Ambiguous,
}

use std::fmt;
impl fmt::Display for SvgGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
           SvgGenerator::SmartRouter => write!(f, "SmartRouter"),
           SvgGenerator::Illustrator => write!(f, "Illustrator"),
           SvgGenerator::Inkscape => write!(f, "Inkscape"),
           SvgGenerator::Vectr => write!(f, "Vectr"),
           SvgGenerator::Affinity => write!(f, "Affinity"),
           SvgGenerator::Ambiguous => write!(f, "Ambiguous"),
       }
    }
}



impl SvgGenerator {
    fn get_dpi_value(&self) -> f64 {
        match *self {
            SvgGenerator::SmartRouter => 72.0,
            SvgGenerator::Illustrator => 72.0,
            SvgGenerator::Inkscape => 96.0,
            SvgGenerator::Vectr => 96.0,
            SvgGenerator::Affinity => 72.0,
            SvgGenerator::Ambiguous => 96.0,
        }
    }
}

pub fn guess_svg_generator(svg_str: &str) -> SvgGenerator{
    if svg_str.contains("Illustrator") || svg_str.contains("illustrator"){
        return SvgGenerator::Illustrator;
    }

    if svg_str.contains("Inkscape") || svg_str.contains("inkscape"){
        return SvgGenerator::Inkscape;
    }

    if svg_str.contains("SmartRouter") || svg_str.contains("smartrouter") || svg_str.contains("Shaper Tools") {
        return SvgGenerator::SmartRouter;
    }

    // Really, having <use> isn't enough to make it a Vectr file, but if it's got a <use> and none of the other tags we can call it a Vectr file.
    if svg_str.contains("<use ") {
        return SvgGenerator::Vectr;
    }

    //
    if svg_str.contains("xmlns:serif") {
        return SvgGenerator::Affinity;
    }

    return SvgGenerator::Ambiguous
}


fn get_svg_dpi_units(svg_str: &str) -> f64 {
    guess_svg_generator(svg_str).get_dpi_value()
}

//TODO add error bounds
pub fn process_svg_str_to_usvg_str(svg_str: &str) -> Result<String, String>{
    let dpi_unit = get_svg_dpi_units(svg_str);
    set_units_dpi(dpi_unit);

    BULLET_SVG_OPT.with(|bullet_svg_opt_cell| {
        //Get static parser options
        let re_opt = bullet_svg_opt_cell.borrow();

        let tree_res = usvg::Tree::from_str(&svg_str, &re_opt);

        let tree = match tree_res {
            Ok(t) => t,
            Err(e) => return Err(e.to_string()),
        };

        let xml_opt = usvg::XmlOptions::default();
        Ok(tree.to_string_with_unit(xml_opt,  LengthUnit::Mm, dpi_unit))
    })
}
