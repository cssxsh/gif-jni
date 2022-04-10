extern crate core;

mod quantizer;
mod ditherer;

use jni::JNIEnv;
use jni::sys::*;
use skia_bindings::*;
use skia_safe::*;
use skia_safe::wrapper::RefWrapper;
use quantizer::quantizer::*;

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_quantizer_OctTreeQuantizer_native(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, count: jint, sort: jboolean
) -> jlong {
    let bitmap_ = unsafe { &SkBitmap::new1(bitmap_ptr as *const SkBitmap) };
    let bitmap = Bitmap::wrap_ref(&bitmap_);
    let colors = bitmap.pixmap().pixels().expect("quantizer read pixels");

    octtree_quantizer(colors, count as u32, sort == JNI_TRUE)
        .as_ptr() as jlong
}