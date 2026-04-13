package org.example.androidvibehangul;

/**
 * Java wrapper around the Rust hangul-core library via JNI.
 *
 * Lifecycle: call {@link #create()} once, use the instance, call
 * {@link #destroy()} when the IME session ends (e.g. onFinishInput).
 */
public final class HangulEngine {

    static {
        System.loadLibrary("hangul_jni");
    }

    private long mPtr;

    /** Allocate the native composer.  Must be called before any other method. */
    public void create() {
        mPtr = nativeCreate();
    }

    /** Free the native composer.  Do not call any method after this. */
    public void destroy() {
        if (mPtr != 0) {
            nativeDestroy(mPtr);
            mPtr = 0;
        }
    }

    /**
     * Process one Hangul compatibility jamo codepoint.
     *
     * @param code Unicode codepoint (0x3131–0x3163)
     * @return text to commit to the input connection before updating the
     *         composing region; empty string if nothing to commit
     */
    public String process(int code) {
        return nativeProcess(mPtr, code);
    }

    /** @return current composing text; empty if not composing */
    public String getComposing() {
        return nativeGetComposing(mPtr);
    }

    /**
     * Handle a backspace key inside a composing syllable.
     *
     * @return {@code true} if the backspace was consumed (composing state
     *         changed); {@code false} means caller should forward a normal
     *         delete to the input connection
     */
    public boolean backspace() {
        return nativeBackspace(mPtr);
    }

    /**
     * Commit the current composing syllable and reset state.
     *
     * @return the committed text, or empty string if nothing was composing
     */
    public String commit() {
        return nativeCommit(mPtr);
    }

    /** Reset composition state without committing. */
    public void reset() {
        nativeReset(mPtr);
    }

    /** @return {@code true} if a syllable is currently being composed */
    public boolean isComposing() {
        return nativeIsComposing(mPtr);
    }

    // ------------------------------------------------------------------
    // Native declarations
    // ------------------------------------------------------------------

    private static native long    nativeCreate();
    private static native void    nativeDestroy(long ptr);
    private static native String  nativeProcess(long ptr, int code);
    private static native String  nativeGetComposing(long ptr);
    private static native boolean nativeBackspace(long ptr);
    private static native String  nativeCommit(long ptr);
    private static native void    nativeReset(long ptr);
    private static native boolean nativeIsComposing(long ptr);
}
