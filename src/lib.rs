extern crate core;

mod quantizer;
mod ditherer;

use std::fs::File;
use std::slice::from_raw_parts;
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
    _env: JNIEnv, _this: jclass, path: JString, width: jint, height: jint, palette: jbyteArray,
) -> jlong {
    let str = _env.get_string(path)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
    let text = str.to_str()
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
    let file = File::create(text)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
    let global_palette = _env.convert_byte_array(palette)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
    let encoder = Encoder::new(file, width as _, height as _, global_palette.as_slice())
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    Box::into_raw(Box::new(encoder)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Encoder_setRepeat(
    _env: JNIEnv, _this: jclass, encoder_ptr: jlong, value: jint,
) -> jlong {
    let mut encoder = unsafe { Box::from_raw(encoder_ptr as *mut Encoder<File>) };
    let repeat = if value > 0 {
        Repeat::Finite(value as _)
    } else {
        Repeat::Infinite
    };

    encoder.set_repeat(repeat)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    Box::into_raw(encoder) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Encoder_writeFrame(
    _env: JNIEnv, _this: jclass, encoder_ptr: jlong, frame_ptr: jlong,
) -> jlong {
    let mut encoder = unsafe { Box::from_raw(encoder_ptr as *mut Encoder<File>) };
    let frame = unsafe { Box::from_raw(frame_ptr as *mut Frame) };

    encoder.write_frame(frame.as_ref())
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    Box::into_raw(frame);

    Box::into_raw(encoder) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Encoder_writeImage(
    _env: JNIEnv, _this: jclass, encoder_ptr: jlong, image_ptr: jlong, delay: jint, dispose: jint, speed: jint,
) -> jlong {
    if speed < 1 || speed > 30 {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let image = Image::wrap(image_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap image fail."));

    if image.color_type() != ColorType::RGBA8888 && image.color_type() != ColorType::RGB888x {
        _env.fatal_error("color_type isn't RGBA8888")
    }

    let mut encoder = unsafe { Box::from_raw(encoder_ptr as *mut Encoder<File>) };
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

    Box::into_raw(encoder) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Encoder_writeBitmap(
    _env: JNIEnv, _this: jclass, encoder_ptr: jlong, bitmap_ptr: jlong, delay: jint, dispose: jint, speed: jint,
) -> jlong {
    if speed < 1 || speed > 30 {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as _)
        .unwrap_or_else(|| _env.fatal_error("wrap SkBitmap"));
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());

    if bitmap.color_type() != ColorType::RGBA8888 && bitmap.color_type() != ColorType::RGB888x {
        _env.fatal_error("color_type isn't RGBA8888")
    }

    let mut encoder = unsafe { Box::from_raw(encoder_ptr as *mut Encoder<File>) };
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

    Box::into_raw(encoder) as _
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
    _env: JNIEnv, _this: jclass, width: jint, height: jint, pixels: jbyteArray, transparent: jint,
) -> jlong {
    let pixels = _env.convert_byte_array(pixels)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
    let transparent = if (0..255).contains(&transparent) {
        Some(transparent as u8)
    } else {
        None
    };

    let frame = Frame::from_indexed_pixels(width as _, height as _, pixels.as_slice(), transparent);

    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromPalettePixels_00024gif(
    _env: JNIEnv, _this: jclass, width: jint, height: jint, pixels: jbyteArray, palette: jbyteArray, transparent: jint,
) -> jlong {
    let pixels = _env.convert_byte_array(pixels)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
    let palette = _env.convert_byte_array(palette)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
    let transparent = if (0..255).contains(&transparent) {
        Some(transparent as u8)
    } else {
        None
    };

    let frame = Frame::from_palette_pixels(width as _, height as _, pixels.as_slice(), palette.as_slice(), transparent);

    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromRGBSpeed_00024gif(
    _env: JNIEnv, _this: jclass, width: jint, height: jint, pixels: jbyteArray, speed: jint,
) -> jlong {
    if speed < 1 || speed > 30 {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let pixels = _env.convert_byte_array(pixels)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    let frame = Frame::from_rgb_speed(width as _, height as _, pixels.as_slice(), speed as _);

    Box::into_raw(Box::from(frame)) as _
}


#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromRGBASpeed_00024gif(
    _env: JNIEnv, _this: jclass, width: jint, height: jint, pixels: jbyteArray, speed: jint,
) -> jlong {
    if speed < 1 || speed > 30 {
        _env.fatal_error("speed needs to be in the range [1, 30]")
    }
    let mut pixels = _env.convert_byte_array(pixels)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    let frame = Frame::from_rgba_speed(width as _, height as _, pixels.as_mut_slice(), speed as _);

    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromImage_00024gif(
    _env: JNIEnv, _this: jclass, image_ptr: jlong, speed: jint,
) -> jlong {
    if speed < 1 || speed > 30 {
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

    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromBitmap_00024gif(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, speed: jint,
) -> jlong {
    if speed < 1 || speed > 30 {
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

    Box::into_raw(Box::from(frame)) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_fromPixmap_00024gif(
    _env: JNIEnv, _this: jclass, pixmap_ptr: jlong, speed: jint,
) -> jlong {
    if speed < 1 || speed > 30 {
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
    let frame = unsafe { Box::from_raw(frame_ptr as *mut Frame) };
    let value = frame.delay;

    Box::into_raw(frame);

    value as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_setDelay_00024gif(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong, value: jint,
) -> jlong {
    let mut frame = unsafe { Box::from_raw(frame_ptr as *mut Frame) };
    frame.delay = value as _;

    Box::into_raw(frame) as _
}


#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_getDispose_00024gif(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong,
) -> jint {
    let frame = unsafe { Box::from_raw(frame_ptr as *mut Frame) };
    let value = frame.dispose;

    Box::into_raw(frame);

    value as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_setDispose_00024gif(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong, value: jint,
) -> jlong {
    let mut frame = unsafe { Box::from_raw(frame_ptr as *mut Frame) };
    frame.dispose = DisposalMethod::from_u8(value as _)
        .unwrap_or_else(|| _env.fatal_error("get dispose method fail"));

    Box::into_raw(frame) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_getRect_00024gif(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong,
) -> jintArray {
    let frame = unsafe { Box::from_raw(frame_ptr as *mut Frame) };
    let arr = _env.new_int_array(4)
        .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    _env.set_int_array_region(arr, 0, &[
        frame.top as jint,
        frame.left as jint,
        frame.width as jint,
        frame.height as jint
    ]).unwrap_or_else(|error| _env.fatal_error(error.to_string()));

    Box::into_raw(frame);

    arr
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_setRect_00024gif(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong, top: jint, left: jint, width: jint, height: jint,
) -> jlong {
    let mut frame = unsafe { Box::from_raw(frame_ptr as *mut Frame) };
    frame.top = top as _;
    frame.left = left as _;
    frame.width = width as _;
    frame.height = height as _;

    Box::into_raw(frame) as _
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_Frame_getPalette_00024gif(
    _env: JNIEnv, _this: jclass, frame_ptr: jlong,
) -> jbyteArray {
    let frame = unsafe { Box::from_raw(frame_ptr as *mut Frame) };
    let arr = match &frame.palette {
        None => {
            _env.new_byte_array(0)
                .unwrap_or_else(|error| _env.fatal_error(error.to_string()))
        }
        Some(vec) => {
            let arr = _env.new_byte_array(vec.len() as jsize)
                .unwrap_or_else(|error| _env.fatal_error(error.to_string()));
            let buf = unsafe { from_raw_parts(vec.as_ptr() as *const jbyte, vec.len()) };

            _env.set_byte_array_region(arr, 0, buf)
                .unwrap_or_else(|error| _env.fatal_error(error.to_string()));

            arr
        }
    };

    Box::into_raw(frame);

    arr
}