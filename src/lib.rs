extern crate core;

mod quantizer;
mod ditherer;

use jni::JNIEnv;
use jni::sys::*;
use skia_bindings::*;
use skia_safe::*;
use skia_safe::wrapper::*;
use quantizer::quantizer::*;

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_quantizer_Quantizer_octtree(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, count: jint, sort: jboolean,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as *mut SkBitmap).expect("wrap SkBitmap");
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let colors = bitmap.pixmap().pixels().expect("quantizer read pixels");

    let mut arr = octtree_quantizer(colors, count as u32, sort == JNI_TRUE);

    println!("{:?}", arr);

    arr.as_mut_ptr() as jlong
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_quantizer_Quantizer_mediancut(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, count: jint, sort: jboolean,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as *mut SkBitmap).expect("wrap SkBitmap");
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let colors = bitmap.pixmap().pixels().expect("quantizer read pixels");

    let mut arr = mediancut_quantizer(colors, count as u32, sort == JNI_TRUE);

    println!("{:?}", arr);

    arr.as_mut_ptr() as jlong
}

#[no_mangle]
pub extern "system" fn Java_xyz_cssxsh_gif_quantizer_Quantizer_kmeans(
    _env: JNIEnv, _this: jclass, bitmap_ptr: jlong, count: jint, sort: jboolean,
) -> jlong {
    let sk_bitmap = RefHandle::wrap(bitmap_ptr as *mut SkBitmap).expect("wrap SkBitmap");
    let bitmap = Bitmap::wrap_ref(sk_bitmap.inner());
    let colors = bitmap.pixmap().pixels().expect("quantizer read pixels");

    let mut arr = kmeans_quantizer(colors, count as u32, sort == JNI_TRUE);

    println!("{:?}", arr);

    arr.as_mut_ptr() as jlong
}