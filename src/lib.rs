extern crate core;

mod quantizer;
mod ditherer;

use std::fs::File;
use gif::*;
use jni::JNIEnv;
use jni::objects::JString;
use jni::sys::*;
use skia_safe::*;
use skia_safe::wrapper::*;
use quantizer::quantizer::*;

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_quantizer_Quantizer_octtree(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, count: jint, sort: jboolean,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _).expect("wrap SkBitmap");
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let colors = bitmap.pixmap().pixels().expect("quantizer read pixels");

    let mut arr = octtree_quantizer(colors, count as _, sort == JNI_TRUE);

    println!("{:?}", arr);

    arr.as_mut_ptr() as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_quantizer_Quantizer_mediancut(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, count: jint, sort: jboolean,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _).expect("wrap SkBitmap");
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let colors = bitmap.pixmap().pixels().expect("quantizer read pixels");

    let mut arr = mediancut_quantizer(colors, count as _, sort == JNI_TRUE);

    println!("{:?}", arr);

    arr.as_mut_ptr() as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_quantizer_Quantizer_kmeans(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, count: jint, sort: jboolean,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _).expect("wrap SkBitmap");
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let colors = bitmap.pixmap().pixels().expect("quantizer read pixels");

    let mut arr = kmeans_quantizer(colors, count as _, sort == JNI_TRUE);

    println!("{:?}", arr);

    arr.as_mut_ptr() as _
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

    if image.color_type() != ColorType::RGBA8888 && image.color_type() != ColorType::RGB888x {
        _env.fatal_error("color_type isn't RGBA8888")
    }

    let mut encoder: Box<Encoder<File>> = unsafe { Box::from_raw(encoder_ptr as _) };
    let pixmap = image.peek_pixels()
        .unwrap_or_else(|| _env.fatal_error("peek pixels fail."));
    let bytes = pixmap.bytes()
        .unwrap_or_else(|| _env.fatal_error("get pixels bytes fail."));
    let mut pixels = bytes.to_vec();
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

    if bitmap.color_type() != ColorType::RGBA8888 && bitmap.color_type() != ColorType::RGB888x {
        _env.fatal_error("color_type isn't RGBA8888")
    }

    let mut encoder: Box<Encoder<File>> = unsafe { Box::from_raw(encoder_ptr as _) };
    let pixmap = bitmap.peek_pixels()
        .unwrap_or_else(|| _env.fatal_error("peek pixels fail."));
    let bytes = pixmap.bytes()
        .unwrap_or_else(|| _env.fatal_error("get pixels bytes fail."));
    let mut pixels = bytes.to_vec();
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
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_default_00024gif(
    _env: JNIEnv, _this: jclass,
) -> jlong {
    let frame = Frame::default();

    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromIndexedPixels_00024gif(
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
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromPalettePixels_00024gif(
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
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromRGBSpeed_00024gif(
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
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromRGBASpeed_00024gif(
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
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromImage_00024gif(
    _env: JNIEnv, _this: jclass, image_ptr: jlong, speed: jint,
) -> jlong {
    if !(1..30).contains(&speed) {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let image = Image::wrap(image_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap image fail."));

    if image.color_type() != ColorType::RGBA8888 && image.color_type() != ColorType::RGB888x {
        _env.fatal_error("color_type isn't RGBA8888")
    }

    let pixmap = image.peek_pixels()
        .unwrap_or_else(|| _env.fatal_error("peek pixels fail."));
    let bytes = pixmap.bytes()
        .unwrap_or_else(|| _env.fatal_error("get pixels bytes fail."));
    let mut pixels = bytes.to_vec();

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
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromBitmap_00024gif(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, speed: jint,
) -> jlong {
    if !(1..30).contains(&speed) {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkBitmap"));
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());

    if bitmap.color_type() != ColorType::RGBA8888 && bitmap.color_type() != ColorType::RGB888x {
        _env.fatal_error("color_type isn't RGBA8888")
    }

    let pixmap = bitmap.peek_pixels()
        .unwrap_or_else(|| _env.fatal_error("peek pixels fail."));
    let bytes = pixmap.bytes()
        .unwrap_or_else(|| _env.fatal_error("get pixels bytes fail."));
    let mut pixels = bytes.to_vec();

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
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromPixmap_00024gif(
    _env: JNIEnv, _this: jclass, pixmap_ptr: jlong, speed: jint,
) -> jlong {
    if !(1..30).contains(&speed) {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let sk_pixmap = RefHandle::wrap(pixmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkPixmap"));
    let pixmap = Pixmap::wrap_ref(sk_pixmap.inner());

    if pixmap.color_type() != ColorType::RGBA8888 && pixmap.color_type() != ColorType::RGB888x {
        _env.fatal_error("color_type isn't RGBA8888")
    }

    let bytes = pixmap.bytes()
        .unwrap_or_else(|| _env.fatal_error("get pixels bytes fail."));
    let mut pixels = bytes.to_vec();

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
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_close_00024gif(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong,
) {
    unsafe { Box::from_raw(frame_ptr as *mut Frame) };
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_getDelay_00024gif(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong,
) -> jint {
    let frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
    let value = frame.delay;

    Box::into_raw(frame);

    value as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_setDelay_00024gif(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong, value: jint,
) {
    let mut frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
    frame.delay = value as _;

    Box::into_raw(frame);
}


#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_getDispose_00024gif(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong,
) -> jint {
    let frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
    let value = frame.dispose;

    Box::into_raw(frame);

    value as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_setDispose_00024gif(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong, value: jint,
) {
    let mut frame: Box<Frame> = unsafe { Box::from_raw(frame_ptr as _) };
    frame.dispose = DisposalMethod::from_u8(value as _)
        .unwrap_or_else(|| _env.fatal_error("get dispose method fail"));

    Box::into_raw(frame);
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_getRect_00024gif(
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
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_setRect_00024gif(
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
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_getPalette_00024gif(
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

    data.unwrap() as _
}