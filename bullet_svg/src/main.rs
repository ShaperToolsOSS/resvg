use bullet_svg;
use std::fs::File;
use std::io::Read; 

fn main() -> std::io::Result<()> {
  println!("starting!");
  let mut file = File::open("Affinity_72dpi_export_test_copy.svg")?;
  let mut svg_str = String::new();
  file.read_to_string(&mut svg_str)?;
  let guess = bullet_svg::guess_svg_generator(&svg_str);
  println!("{}",guess);
  Ok(())
  // let db = bullet_svg::create_fontdb();
  // println!("{} font faces", db.len());
  // println!("Hello foo!");
}