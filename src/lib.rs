extern crate core;

mod quantizer;
mod ditherer;

use std::fs::File;
use std::slice;
use gif::*;
use jni::JNIEnv;
use jni::objects::JString;
use jni::sys::*;
use skia_safe::*;
use skia_safe::image::*;
use skia_safe::wrapper::*;
use quantizer::quantizer::*;
use ditherer::ditherer::*;

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Quantizer_00024OctTree_native(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, count: jint, sort: jboolean,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkBitmap"));
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let mut vec: Vec<[u8; 4]>;
    let pixels = match bitmap.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            bitmap.pixmap().pixels()
                .unwrap_or_else(|| _env.fatal_error("get pixels fail."))
        }
        _ => {
            let image_info = bitmap.info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size() / 4;
            vec = Vec::with_capacity(capacity);
            vec.resize(capacity, [0; 4]);

            bitmap.pixmap().read_pixels(
                &image_info,
                vec.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
            );

            vec.as_slice()
        }
    };

    let palette = octtree_quantizer(pixels, count as _, sort == JNI_TRUE);
    let bytes = unsafe { slice::from_raw_parts(palette.as_ptr() as _, palette.len() * 3) };
    let data = Data::new_copy(bytes);

    sk_bitmap.unwrap();
    data.unwrap() as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Quantizer_00024MedianCut_native(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, count: jint, sort: jboolean,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkBitmap"));
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let mut vec: Vec<[u8; 4]>;
    let pixels = match bitmap.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            bitmap.pixmap().pixels()
                .unwrap_or_else(|| _env.fatal_error("get pixels fail."))
        }
        _ => {
            let image_info = bitmap.info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size() / 4;
            vec = Vec::with_capacity(capacity);
            vec.resize(capacity, [0; 4]);

            bitmap.pixmap().read_pixels(
                &image_info,
                vec.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
            );

            vec.as_slice()
        }
    };

    let palette = mediancut_quantizer(pixels, count as _, sort == JNI_TRUE);
    let bytes = unsafe { slice::from_raw_parts(palette.as_ptr() as _, palette.len() * 3) };
    let data = Data::new_copy(bytes);

    sk_bitmap.unwrap();
    data.unwrap() as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Quantizer_00024KMeans_native(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, count: jint, sort: jboolean,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkBitmap"));
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let mut vec: Vec<[u8; 4]>;
    let pixels = match bitmap.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            bitmap.pixmap().pixels()
                .unwrap_or_else(|| _env.fatal_error("get pixels fail."))
        }
        _ => {
            let image_info = bitmap.info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size() / 4;
            vec = Vec::with_capacity(capacity);
            vec.resize(capacity, [0; 4]);

            bitmap.pixmap().read_pixels(
                &image_info,
                vec.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
            );

            vec.as_slice()
        }
    };

    let palette = kmeans_quantizer(pixels, count as _, sort == JNI_TRUE);
    let bytes = unsafe { slice::from_raw_parts(palette.as_ptr() as _, palette.len() * 3) };
    let data = Data::new_copy(bytes);

    sk_bitmap.unwrap();
    data.unwrap() as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Ditherer_00024Atkinson_native(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, palette_ptr: jlong,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkBitmap"));
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let data = Data::wrap(palette_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap palette fail."));
    let mut pixels: Vec<[u8; 4]>;
    let colors: &[[u8; 4]] = match bitmap.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            bitmap.pixmap().pixels()
                .unwrap_or_else(|| _env.fatal_error("get pixels fail."))
        }
        _ => {
            let image_info = bitmap.info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size() / 4;
            pixels = Vec::with_capacity(capacity);
            pixels.resize(capacity, [0; 4]);

            bitmap.pixmap().read_pixels(
                &image_info,
                pixels.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
            );

            pixels.as_slice()
        }
    };
    let palette = unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len() / 3) };

    let temp = atkinson_ditherer(colors, bitmap.width(), bitmap.height(), palette);
    let bytes = unsafe { slice::from_raw_parts(temp.as_ptr() as _, temp.len() * 3) };
    let result = Data::new_copy(bytes);

    sk_bitmap.unwrap();
    data.unwrap();
    result.unwrap() as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Ditherer_00024JJN_native(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, palette_ptr: jlong,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkBitmap"));
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let data = Data::wrap(palette_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap palette fail."));
    let mut pixels: Vec<[u8; 4]>;
    let colors: &[[u8; 4]] = match bitmap.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            bitmap.pixmap().pixels()
                .unwrap_or_else(|| _env.fatal_error("get pixels fail."))
        }
        _ => {
            let image_info = bitmap.info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size() / 4;
            pixels = Vec::with_capacity(capacity);
            pixels.resize(capacity, [0; 4]);

            bitmap.pixmap().read_pixels(
                &image_info,
                pixels.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
            );

            pixels.as_slice()
        }
    };
    let palette = unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len() / 3) };

    let temp = jjn_ditherer(colors, bitmap.width(), bitmap.height(), palette);
    let bytes = unsafe { slice::from_raw_parts(temp.as_ptr() as _, temp.len() * 3) };
    let result = Data::new_copy(bytes);

    sk_bitmap.unwrap();
    data.unwrap();
    result.unwrap() as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Ditherer_00024SierraLite_native(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, palette_ptr: jlong,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkBitmap"));
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let data = Data::wrap(palette_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap palette fail."));
    let mut pixels: Vec<[u8; 4]>;
    let colors: &[[u8; 4]] = match bitmap.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            bitmap.pixmap().pixels()
                .unwrap_or_else(|| _env.fatal_error("get pixels fail."))
        }
        _ => {
            let image_info = bitmap.info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size() / 4;
            pixels = Vec::with_capacity(capacity);
            pixels.resize(capacity, [0; 4]);

            bitmap.pixmap().read_pixels(
                &image_info,
                pixels.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
            );

            pixels.as_slice()
        }
    };
    let palette = unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len() / 3) };

    let temp = sierra_lite_ditherer(colors, bitmap.width(), bitmap.height(), palette);
    let bytes = unsafe { slice::from_raw_parts(temp.as_ptr() as _, temp.len() * 3) };
    let result = Data::new_copy(bytes);

    sk_bitmap.unwrap();
    data.unwrap();
    result.unwrap() as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Ditherer_00024Stucki_native(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, palette_ptr: jlong,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkBitmap"));
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let data = Data::wrap(palette_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap palette fail."));
    let mut pixels: Vec<[u8; 4]>;
    let colors: &[[u8; 4]] = match bitmap.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            bitmap.pixmap().pixels()
                .unwrap_or_else(|| _env.fatal_error("get pixels fail."))
        }
        _ => {
            let image_info = bitmap.info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size() / 4;
            pixels = Vec::with_capacity(capacity);
            pixels.resize(capacity, [0; 4]);

            bitmap.pixmap().read_pixels(
                &image_info,
                pixels.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
            );

            pixels.as_slice()
        }
    };
    let palette = unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len() / 3) };

    let temp = stucki_ditherer(colors, bitmap.width(), bitmap.height(), palette);
    let bytes = unsafe { slice::from_raw_parts(temp.as_ptr() as _, temp.len() * 3) };
    let result = Data::new_copy(bytes);

    sk_bitmap.unwrap();
    data.unwrap();
    result.unwrap() as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Encoder_file(
    _env: JNIEnv, _this: jclass, path: JString, width: jint, height: jint, palette: jlong,
) -> jlong {
    let str = _env.get_string(path)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
    let text = str.to_str()
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
    let file = File::create(text)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
    let global_palette = Data::wrap(palette as _)
        .unwrap_or_else(|| _env.fatal_error("wrap palette fail."));
    let encoder = Encoder::new(file, width as _, height as _, global_palette.as_bytes())
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    global_palette.unwrap();
    Box::into_raw(Box::new(encoder)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Encoder_setRepeat(
    _env: JNIEnv, _this: jclass, encoder_ptr: jlong, value: jint,
) {
    let mut encoder: Box<Encoder<File>> = unsafe { Box::from_raw(encoder_ptr as _) };
    let repeat = if (0..65535).contains(&value) {
        Repeat::Finite(value as _)
    } else {
        Repeat::Infinite
    };

    encoder.set_repeat(repeat)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    Box::into_raw(encoder);
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Encoder_writeFrame(
    _env: JNIEnv, _this: jclass, encoder_ptr: jlong, frame_ptr: jlong,
) {
    let mut encoder: Box<Encoder<File>> = unsafe { Box::from_raw(encoder_ptr as _) };
    let frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };

    encoder.write_frame(frame.as_ref())
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    Box::into_raw(frame);
    Box::into_raw(encoder);
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Encoder_writeImage(
    _env: JNIEnv, _this: jclass, encoder_ptr: jlong, image_ptr: jlong, delay: jint, dispose: jint, speed: jint,
) {
    if !(1..30).contains(&speed) {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let image = Image::wrap(image_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap image fail."));
    let mut encoder: Box<Encoder<File>> = unsafe { Box::from_raw(encoder_ptr as _) };

    let mut pixels = match image.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            let pixmap = image.peek_pixels()
                .unwrap_or_else(|| _env.fatal_error("peek pixels fail."));
            let bytes = pixmap.bytes()
                .unwrap_or_else(|| _env.fatal_error("get pixels bytes fail."));
            bytes.to_vec()
        }
        _ => {
            let image_info = image.image_info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size();
            let mut pixels = Vec::with_capacity(capacity);
            pixels.resize(capacity, 0);

            image.read_pixels(
                &image_info,
                pixels.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
                CachingHint::Allow,
            );

            pixels
        }
    };
    let mut frame = Frame::from_rgba_speed(
        image.width() as _,
        image.height() as _,
        pixels.as_mut_slice(),
        speed,
    );

    frame.delay = delay as _;
    frame.dispose = DisposalMethod::from_u8(dispose as _)
        .unwrap_or_else(|| _env.fatal_error("get dispose method fail"));

    encoder.write_frame(&frame)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    image.unwrap();
    Box::into_raw(encoder);
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Encoder_writeBitmap(
    _env: JNIEnv, _this: jclass, encoder_ptr: jlong, bitmap_ptr: jlong, delay: jint, dispose: jint, speed: jint,
) {
    if !(1..30).contains(&speed) {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkBitmap"));
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());

    let mut encoder: Box<Encoder<File>> = unsafe { Box::from_raw(encoder_ptr as _) };
    let mut pixels = match bitmap.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            let bytes = bitmap.pixmap().bytes()
                .unwrap_or_else(|| _env.fatal_error("get pixels bytes fail."));
            bytes.to_vec()
        }
        _ => {
            let image_info = bitmap.info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size();
            let mut pixels = Vec::with_capacity(capacity);
            pixels.resize(capacity, 0);

            bitmap.pixmap().read_pixels(
                &image_info,
                pixels.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
            );

            pixels
        }
    };
    let mut frame = Frame::from_rgba_speed(
        bitmap.width() as _,
        bitmap.height() as _,
        pixels.as_mut_slice(),
        speed,
    );

    frame.delay = delay as _;
    frame.dispose = DisposalMethod::from_u8(dispose as _)
        .unwrap_or_else(|| _env.fatal_error("get dispose method fail"));

    encoder.write_frame(&frame)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    sk_bitmap.unwrap();
    Box::into_raw(encoder);
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Encoder_close(
    _env: JNIEnv, _this: jclass, encoder_ptr: jlong,
) {
    unsafe { Box::from_raw(encoder_ptr as *mut Encoder<File>) };
}


#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_default_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass,
) -> jlong {
    let frame = Frame::default();

    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromIndexedPixels_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, width: jint, height: jint, pixels: jlong, transparent: jint,
) -> jlong {
    let pixels = Data::wrap(pixels as _)
        .unwrap_or_else(|| _env.fatal_error("wrap pixels fail."));
    let transparent = if (0..255).contains(&transparent) {
        Some(transparent as u8)
    } else {
        None
    };

    let frame = Frame::from_indexed_pixels(width as _, height as _, pixels.as_bytes(), transparent);

    pixels.unwrap();
    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromPalettePixels_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, width: jint, height: jint, pixels: jlong, palette: jlong, transparent: jint,
) -> jlong {
    let pixels = Data::wrap(pixels as _)
        .unwrap_or_else(|| _env.fatal_error("wrap pixels fail."));
    let palette = Data::wrap(palette as _)
        .unwrap_or_else(|| _env.fatal_error("wrap palette fail."));
    let transparent = if (0..255).contains(&transparent) {
        Some(transparent as u8)
    } else {
        None
    };

    let frame = Frame::from_palette_pixels(width as _, height as _, pixels.as_bytes(), palette.as_bytes(), transparent);

    pixels.unwrap();
    palette.unwrap();
    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromRGBSpeed_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, width: jint, height: jint, pixels: jlong, speed: jint,
) -> jlong {
    if !(1..30).contains(&speed) {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let pixels = Data::wrap(pixels as _)
        .unwrap_or_else(|| _env.fatal_error("wrap pixels fail."));

    let frame = Frame::from_rgb_speed(width as _, height as _, pixels.as_bytes(), speed as _);

    pixels.unwrap();
    Box::into_raw(Box::from(frame)) as _
}


#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromRGBASpeed_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, width: jint, height: jint, pixels: jlong, speed: jint,
) -> jlong {
    if !(1..30).contains(&speed) {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let pixels = Data::wrap(pixels as _)
        .unwrap_or_else(|| _env.fatal_error("wrap pixels fail."));

    let frame = Frame::from_rgba_speed(width as _, height as _, pixels.to_vec().as_mut_slice(), speed as _);

    pixels.unwrap();
    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromImage_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, image_ptr: jlong, speed: jint,
) -> jlong {
    if !(1..30).contains(&speed) {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let image = Image::wrap(image_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap image fail."));

    let mut pixels = match image.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            let pixmap = image.peek_pixels()
                .unwrap_or_else(|| _env.fatal_error("peek pixels fail."));
            let bytes = pixmap.bytes()
                .unwrap_or_else(|| _env.fatal_error("get pixels bytes fail."));
            bytes.to_vec()
        }
        _ => {
            let image_info = image.image_info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size();
            let mut pixels = Vec::with_capacity(capacity);
            pixels.resize(capacity, 0);

            image.read_pixels(
                &image_info,
                pixels.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
                CachingHint::Allow,
            );

            pixels
        }
    };

    let frame = Frame::from_rgba_speed(
        image.width() as _,
        image.height() as _,
        pixels.as_mut_slice(),
        speed,
    );

    image.unwrap();
    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromBitmap_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, speed: jint,
) -> jlong {
    if !(1..30).contains(&speed) {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkBitmap"));
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());

    let mut pixels = match bitmap.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            let bytes = bitmap.pixmap().bytes()
                .unwrap_or_else(|| _env.fatal_error("get pixels bytes fail."));
            bytes.to_vec()
        }
        _ => {
            let image_info = bitmap.info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size();
            let mut pixels = Vec::with_capacity(capacity);
            pixels.resize(capacity, 0);

            bitmap.pixmap().read_pixels(
                &image_info,
                pixels.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
            );

            pixels
        }
    };

    let frame = Frame::from_rgba_speed(
        bitmap.width() as _,
        bitmap.height() as _,
        pixels.as_mut_slice(),
        speed,
    );

    sk_bitmap.unwrap();
    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromPixmap_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, pixmap_ptr: jlong, speed: jint,
) -> jlong {
    if !(1..30).contains(&speed) {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let sk_pixmap = RefHandle::wrap(pixmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkPixmap"));
    let pixmap = Pixmap::wrap_ref(sk_pixmap.inner());

    let mut pixels = match pixmap.color_type() {
        ColorType::RGBA8888 | ColorType::RGB888x => {
            let bytes = pixmap.bytes()
                .unwrap_or_else(|| _env.fatal_error("get pixels bytes fail."));
            bytes.to_vec()
        }
        _ => {
            let image_info = pixmap.info()
                .with_color_type(ColorType::RGBA8888);
            let capacity = image_info.compute_min_byte_size();
            let mut pixels = Vec::with_capacity(capacity);
            pixels.resize(capacity, 0);

            pixmap.read_pixels(
                &image_info,
                pixels.as_mut_slice(),
                image_info.min_row_bytes(),
                IPoint { x: 0, y: 0 },
            );

            pixels
        }
    };

    let frame = Frame::from_rgba_speed(
        pixmap.width() as _,
        pixmap.height() as _,
        pixels.as_mut_slice(),
        speed,
    );

    sk_pixmap.unwrap();
    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_close_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong,
) {
    let _: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_getDelay_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong,
) -> jint {
    let frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
    let value = frame.delay;

    Box::into_raw(frame);

    value as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_setDelay_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong, value: jint,
) {
    let mut frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
    frame.delay = value as _;

    Box::into_raw(frame);
}


#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_getDispose_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong,
) -> jint {
    let frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
    let value = frame.dispose;

    Box::into_raw(frame);

    value as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_setDispose_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong, value: jint,
) {
    let mut frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
    frame.dispose = DisposalMethod::from_u8(value as _)
        .unwrap_or_else(|| _env.fatal_error("get dispose method fail"));

    Box::into_raw(frame);
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_getRect_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong,
) -> jintArray {
    let frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
    let arr = _env.new_int_array(4)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
    let buf = [
        frame.top as jint,
        frame.left as jint,
        frame.width as jint,
        frame.height as jint
    ];

    _env.set_int_array_region(arr, 0, &buf)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    Box::into_raw(frame);

    arr
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_setRect_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong, top: jint, left: jint, width: jint, height: jint,
) {
    let mut frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
    frame.top = top as _;
    frame.left = left as _;
    frame.width = width as _;
    frame.height = height as _;

    Box::into_raw(frame);
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_getPalette_00024mirai_1skia_1plugin(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong,
) -> jlong {
    let frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
    let data = match &frame.palette {
        None => {
            Data::new_empty()
        }
        Some(vec) => {
            Data::new_copy(vec.as_slice())
        }
    };

    Box::into_raw(frame);

    data.unwrap() as _
}