use harfbuzz_wasm::{Font, Glyph, GlyphBuffer};

#[export_name = "shape"]
pub fn shape(
    _shape_plan: u32,
    font_ref: u32,
    buf_ref: u32,
    _features: u32,
    _num_features: u32,
) -> i32 {
    let font = Font::from_ref(font_ref);
    let mut buffer = GlyphBuffer::from_ref(buf_ref);

    let buf_u8: Vec<u8> = buffer.glyphs.iter().map(|g| g.codepoint as u8).collect();
    let str_buf = String::from_utf8_lossy(&buf_u8);

    let (program, input) = match str_buf.split_once("#") {
        Some((program, input)) => (program, format!("{}\n", input)),
        _ => (str_buf.as_ref(), String::new()),
    };

    let res_str = brainfuck::execute_program(program, &input).unwrap_or(String::from("error"));
    let chars: Vec<_> = res_str.chars().collect();

    buffer.glyphs.clear();

    let mut i = 0;
    let mut x_advance_sum = 0;
    while i < chars.len() {
        if chars[i] == '\n' {
            i += 1;
            continue;
        }

        let mut item = Glyph {
            codepoint: font.get_glyph(chars[i] as u32, 0),
            flags: 0,
            x_advance: 0,
            y_advance: 0,
            cluster: i as u32,
            x_offset: 0,
            y_offset: 0,
        };

        if chars.get(i + 1) == Some(&'\n') {
            item.x_advance = -x_advance_sum;
            item.y_advance = font.get_glyph_v_advance(item.codepoint);
            x_advance_sum = 0;
        } else {
            item.x_advance = font.get_glyph_h_advance(item.codepoint);
            x_advance_sum += item.x_advance;
        }

        buffer.glyphs.push(item);
        i += 1;
    }
    1
}
