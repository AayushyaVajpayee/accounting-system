use typst::foundations::Bytes;
use typst::text::Font;

const F1: &[u8] = include_bytes!("../fonts/DejaVuSansMono.ttf");
const F2: &[u8] = include_bytes!("../fonts/LinLibertine_R.ttf");
const F3: &[u8] = include_bytes!("../fonts/LinLibertine_RB.ttf");
const F4: &[u8] = include_bytes!("../fonts/LinLibertine_RBI.ttf");
const F5: &[u8] = include_bytes!("../fonts/LinLibertine_RI.ttf");
const F6:&[u8]=include_bytes!("../fonts/NewCMMath-Book.otf");
const F7:&[u8]=include_bytes!("../fonts/NewCMMath-Regular.otf");
const F8:&[u8]=include_bytes!("../fonts/NewCM10-Regular.otf");
const F9:&[u8]=include_bytes!("../fonts/NewCM10-Bold.otf");
const F10:&[u8]=include_bytes!("../fonts/NewCM10-Italic.otf");
const F11:&[u8]=include_bytes!("../fonts/NewCM10-BoldItalic.otf");
const F12:&[u8]=include_bytes!("../fonts/DejaVuSansMono-Bold.ttf");
const F13:&[u8]=include_bytes!("../fonts/DejaVuSansMono-Oblique.ttf");
const F14:&[u8]=include_bytes!("../fonts/DejaVuSansMono-BoldOblique.ttf");

fn register_fonts() -> Vec<Font> {
    let p = vec![
        F1, F2, F3, F4, F5,F6,F7,F8,F9,F10,F11,F12,F13,F14
    ];
    p.into_iter()
        .flat_map(|a| {
            let buffer = Bytes::from(a);
            let face_count = ttf_parser::fonts_in_collection(&buffer).unwrap_or(1);
            (0..face_count).map(move |face| {
                Font::new(buffer.clone(), face).unwrap()
            })
        }).collect::<Vec<_>>()
}