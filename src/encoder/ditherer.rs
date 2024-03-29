type RGBA = [u8; 4];

type RGB = [u8; 3];

pub struct ErrorComponent {
    delta_x: i32,
    delta_y: i32,
    power: f64,
}

fn nearest(original: &RGBA, replacement: &RGB) -> u32 {
    let r = original[0] as i32 - replacement[0] as i32;
    let g = original[1] as i32 - replacement[1] as i32;
    let b = original[2] as i32 - replacement[2] as i32;

    (r * r + g * g + b * b) as u32
}

fn minus(original: &RGBA, replacement: &RGB) -> RGB {
    [
        original[0] - replacement[0],
        original[1] - replacement[1],
        original[2] - replacement[2],
    ]
}

fn add(original: &RGBA, offset: &RGB) -> RGBA {
    [
        original[0] + offset[0],
        original[1] + offset[1],
        original[2] + offset[2],
        original[3],
    ]
}

pub fn ditherer(colors: &[RGBA], width: i32, height: i32, palette: &[RGB], distribution: &[ErrorComponent]) -> Vec<RGBA> {
    let mut raw = Vec::from(colors);

    for x in 0..width {
        for y in 0..height {
            let index = (y * width + x) as usize;
            let original = colors.get(index)
                .expect("encoder get original color");
            let replacement = palette.iter()
                .min_by_key(|other| nearest(original, other))
                .expect("atkinson_ditherer get min replacement");

            let error = minus(original, replacement);
            for component in distribution.iter() {
                let sibling_x = x + component.delta_x;
                let sibling_y = y + component.delta_y;
                if sibling_x >= 0 && sibling_x < width && sibling_y >= 0 && sibling_y < height {
                    let index = (sibling_y * width + sibling_x) as usize;
                    let dest = colors.get(index).expect("encoder get dest color");
                    let offset = &error.map(|i| (i as f64 * component.power) as u8);

                    raw[index] = add(dest, offset);
                }
            }
        }
    }

    raw
}

pub fn atkinson_ditherer(colors: &[RGBA], width: i32, height: i32, palette: &[RGB]) -> Vec<RGBA> {
    let distribution = [
        ErrorComponent { delta_x: 1, delta_y: 0, power: 1.0 / 8.0 },
        ErrorComponent { delta_x: 2, delta_y: 0, power: 1.0 / 8.0 },
        //
        ErrorComponent { delta_x: -1, delta_y: 1, power: 1.0 / 8.0 },
        ErrorComponent { delta_x: 0, delta_y: 1, power: 1.0 / 8.0 },
        ErrorComponent { delta_x: 1, delta_y: 1, power: 1.0 / 8.0 },
        //
        ErrorComponent { delta_x: 0, delta_y: 2, power: 1.0 / 8.0 }
    ];

    ditherer(colors, width, height, palette, &distribution)
}

pub fn jjn_ditherer(colors: &[RGBA], width: i32, height: i32, palette: &[RGB]) -> Vec<RGBA> {
    let distribution = [
        ErrorComponent { delta_x: 1, delta_y: 0, power: 7.0 / 48.0 },
        ErrorComponent { delta_x: 2, delta_y: 0, power: 5.0 / 48.0 },
        //
        ErrorComponent { delta_x: -2, delta_y: 1, power: 3.0 / 48.0 },
        ErrorComponent { delta_x: 1, delta_y: 1, power: 5.0 / 48.0 },
        ErrorComponent { delta_x: 0, delta_y: 1, power: 7.0 / 48.0 },
        ErrorComponent { delta_x: 1, delta_y: 1, power: 5.0 / 48.0 },
        ErrorComponent { delta_x: 2, delta_y: 1, power: 3.0 / 48.0 },
        //
        ErrorComponent { delta_x: -2, delta_y: 2, power: 1.0 / 48.0 },
        ErrorComponent { delta_x: -1, delta_y: 2, power: 3.0 / 48.0 },
        ErrorComponent { delta_x: 0, delta_y: 2, power: 5.0 / 48.0 },
        ErrorComponent { delta_x: 1, delta_y: 2, power: 3.0 / 48.0 },
        ErrorComponent { delta_x: 2, delta_y: 2, power: 1.0 / 48.0 }
    ];

    ditherer(colors, width, height, palette, &distribution)
}

pub fn sierra_lite_ditherer(colors: &[RGBA], width: i32, height: i32, palette: &[RGB]) -> Vec<RGBA> {
    let distribution = [
        ErrorComponent { delta_x: 1, delta_y: 0, power: 2.0 / 4.0 },
        //
        ErrorComponent { delta_x: -1, delta_y: 1, power: 1.0 / 4.0 },
        ErrorComponent { delta_x: 0, delta_y: 1, power: 1.0 / 4.0 },
    ];

    ditherer(colors, width, height, palette, &distribution)
}

pub fn stucki_ditherer(colors: &[RGBA], width: i32, height: i32, palette: &[RGB]) -> Vec<RGBA> {
    let distribution = [
        ErrorComponent { delta_x: 1, delta_y: 0, power: 8.0 / 48.0 },
        ErrorComponent { delta_x: 2, delta_y: 0, power: 4.0 / 48.0 },
        //
        ErrorComponent { delta_x: -2, delta_y: 1, power: 2.0 / 48.0 },
        ErrorComponent { delta_x: 1, delta_y: 1, power: 4.0 / 48.0 },
        ErrorComponent { delta_x: 0, delta_y: 1, power: 8.0 / 48.0 },
        ErrorComponent { delta_x: 1, delta_y: 1, power: 4.0 / 48.0 },
        ErrorComponent { delta_x: 2, delta_y: 1, power: 2.0 / 48.0 },
        //
        ErrorComponent { delta_x: -2, delta_y: 2, power: 1.0 / 48.0 },
        ErrorComponent { delta_x: -1, delta_y: 2, power: 2.0 / 48.0 },
        ErrorComponent { delta_x: 0, delta_y: 2, power: 4.0 / 48.0 },
        ErrorComponent { delta_x: 1, delta_y: 2, power: 2.0 / 48.0 },
        ErrorComponent { delta_x: 2, delta_y: 2, power: 1.0 / 48.0 }
    ];

    ditherer(colors, width, height, palette, &distribution)
}
