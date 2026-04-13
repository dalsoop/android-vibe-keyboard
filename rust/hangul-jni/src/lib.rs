//! JNI bindings for hangul-core — exposes HangulComposer to Android Java.
//!
//! Java class: org.example.androidvibehangul.HangulEngine
//! Each method receives a `ptr: jlong` which is a raw pointer to a
//! heap-allocated `HangulComposer`.  The Java side must call `nativeCreate`
//! once and `nativeDestroy` when the IME session ends.

use hangul_core::HangulComposer;
use jni::objects::JClass;
use jni::sys::{jboolean, jint, jlong, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;

// ---------------------------------------------------------------------------
// Safety helpers
// ---------------------------------------------------------------------------

/// Obtain a mutable reference from the opaque pointer stored in Java.
///
/// # Safety
/// The pointer must have been returned by `nativeCreate` and not yet freed
/// by `nativeDestroy`.
unsafe fn composer(ptr: jlong) -> &'static mut HangulComposer {
    &mut *(ptr as *mut HangulComposer)
}

fn to_jstring(env: &mut JNIEnv, s: &str) -> jstring {
    env.new_string(s)
        .unwrap_or_else(|_| env.new_string("").unwrap())
        .into_raw()
}

// ---------------------------------------------------------------------------
// JNI exports
// ---------------------------------------------------------------------------

/// Allocate a new `HangulComposer` and return it as an opaque pointer.
#[no_mangle]
pub extern "system" fn Java_org_example_androidvibehangul_HangulEngine_nativeCreate(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    Box::into_raw(Box::new(HangulComposer::new())) as jlong
}

/// Free the `HangulComposer` previously created by `nativeCreate`.
///
/// After this call the pointer is invalid and must not be used again.
#[no_mangle]
pub extern "system" fn Java_org_example_androidvibehangul_HangulEngine_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut HangulComposer)) };
    }
}

/// Process one Hangul jamo codepoint.
///
/// Returns the text that must be committed to the input connection before
/// the composing region is updated (may be an empty string).
#[no_mangle]
pub extern "system" fn Java_org_example_androidvibehangul_HangulEngine_nativeProcess(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    code: jint,
) -> jstring {
    let commit = unsafe { composer(ptr) }.process(code as u32);
    to_jstring(&mut env, &commit)
}

/// Return the current composing text without changing state.
#[no_mangle]
pub extern "system" fn Java_org_example_androidvibehangul_HangulEngine_nativeGetComposing(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jstring {
    let s = unsafe { composer(ptr) }.get_composing();
    to_jstring(&mut env, &s)
}

/// Handle a backspace event inside a composing syllable.
///
/// Returns `true` if the backspace was consumed (the composing region
/// changed); `false` means the caller should forward a normal delete.
#[no_mangle]
pub extern "system" fn Java_org_example_androidvibehangul_HangulEngine_nativeBackspace(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jboolean {
    if unsafe { composer(ptr) }.backspace() { JNI_TRUE } else { JNI_FALSE }
}

/// Commit the current composing syllable and reset state.
///
/// Returns the committed text (empty if nothing was composing).
#[no_mangle]
pub extern "system" fn Java_org_example_androidvibehangul_HangulEngine_nativeCommit(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jstring {
    let s = unsafe { composer(ptr) }.commit();
    to_jstring(&mut env, &s)
}

/// Reset composition state without committing.
#[no_mangle]
pub extern "system" fn Java_org_example_androidvibehangul_HangulEngine_nativeReset(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    unsafe { composer(ptr) }.reset();
}

/// Returns `true` if a syllable is currently being composed.
#[no_mangle]
pub extern "system" fn Java_org_example_androidvibehangul_HangulEngine_nativeIsComposing(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jboolean {
    if unsafe { composer(ptr) }.is_composing() { JNI_TRUE } else { JNI_FALSE }
}
