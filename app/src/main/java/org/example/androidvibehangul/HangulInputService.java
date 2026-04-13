package org.example.androidvibehangul;

import android.inputmethodservice.InputMethodService;
import android.inputmethodservice.Keyboard;
import android.inputmethodservice.KeyboardView;
import android.view.KeyEvent;
import android.view.View;
import android.view.inputmethod.InputConnection;

/**
 * Dubeolsik Hangul IME service.
 *
 * Key routing:
 *   KEYCODE_DELETE      → handleBackspace()
 *   KEYCODE_DONE        → commitComposing() + ENTER
 *   KEYCODE_MODE_CHANGE → commitComposing() + 한/영 전환
 *   Hangul jamo         → engine.process() → commitText / setComposingText
 *   Other               → commitComposing() + passthrough
 */
public class HangulInputService extends InputMethodService
        implements KeyboardView.OnKeyboardActionListener {

    private HangulEngine mEngine;
    private KeyboardView mKeyboardView;
    private Keyboard mHangulKeyboard;
    private Keyboard mQwertyKeyboard;
    private boolean mHangulMode = true;

    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    @Override
    public void onCreate() {
        super.onCreate();
        mEngine = new HangulEngine();
        mEngine.create();
    }

    @Override
    public void onDestroy() {
        mEngine.destroy();
        super.onDestroy();
    }

    @Override
    public View onCreateInputView() {
        mKeyboardView = (KeyboardView) getLayoutInflater()
                .inflate(R.layout.keyboard_view, null);
        mHangulKeyboard = new Keyboard(this, R.xml.keyboard_hangul);
        mQwertyKeyboard = new Keyboard(this, R.xml.keyboard_qwerty);
        mKeyboardView.setKeyboard(mHangulKeyboard);
        mKeyboardView.setOnKeyboardActionListener(this);
        return mKeyboardView;
    }

    @Override
    public void onFinishInput() {
        super.onFinishInput();
        commitComposing();
        mEngine.reset();
    }

    // -----------------------------------------------------------------------
    // KeyboardView.OnKeyboardActionListener
    // -----------------------------------------------------------------------

    @Override
    public void onKey(int primaryCode, int[] keyCodes) {
        InputConnection ic = getCurrentInputConnection();
        if (ic == null) return;

        switch (primaryCode) {
            case Keyboard.KEYCODE_DELETE:
                handleBackspace(ic);
                break;

            case Keyboard.KEYCODE_DONE:
                commitComposing(ic);
                sendDownUpKeyEvents(KeyEvent.KEYCODE_ENTER);
                break;

            case Keyboard.KEYCODE_MODE_CHANGE:
                commitComposing(ic);
                toggleHangulMode();
                break;

            case Keyboard.KEYCODE_SHIFT:
                // TODO: 경음(ㄲ/ㄸ/ㅃ/ㅆ/ㅉ) 처리
                mKeyboardView.setShifted(!mKeyboardView.isShifted());
                break;

            default:
                if (mHangulMode && isHangulJamo(primaryCode)) {
                    handleHangulKey(ic, primaryCode);
                } else {
                    commitComposing(ic);
                    ic.commitText(String.valueOf((char) primaryCode), 1);
                }
                break;
        }
    }

    // -----------------------------------------------------------------------
    // Hangul input
    // -----------------------------------------------------------------------

    private void handleHangulKey(InputConnection ic, int code) {
        String commit = mEngine.process(code);

        // Flush any syllable that was completed by this keypress
        if (commit != null && !commit.isEmpty()) {
            ic.commitText(commit, 1);
        }

        // Update (or clear) the composing region
        String composing = mEngine.getComposing();
        if (composing.isEmpty()) {
            ic.finishComposingText();
        } else {
            ic.setComposingText(composing, 1);
        }
    }

    private void handleBackspace(InputConnection ic) {
        if (mEngine.isComposing()) {
            boolean consumed = mEngine.backspace();
            if (consumed) {
                String composing = mEngine.getComposing();
                if (composing.isEmpty()) {
                    ic.finishComposingText();
                } else {
                    ic.setComposingText(composing, 1);
                }
            } else {
                // Shouldn't happen (backspace returns false only when not composing)
                ic.finishComposingText();
            }
        } else {
            ic.deleteSurroundingText(1, 0);
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /** Commit any in-progress syllable to the input connection. */
    private void commitComposing() {
        InputConnection ic = getCurrentInputConnection();
        if (ic != null) commitComposing(ic);
    }

    private void commitComposing(InputConnection ic) {
        if (mEngine.isComposing()) {
            String text = mEngine.commit();
            if (!text.isEmpty()) {
                ic.commitText(text, 1);
            }
            ic.finishComposingText();
        }
    }

    private void toggleHangulMode() {
        mHangulMode = !mHangulMode;
        mKeyboardView.setKeyboard(mHangulMode ? mHangulKeyboard : mQwertyKeyboard);
        mKeyboardView.invalidateAllKeys();
    }

    /** Returns true for Hangul compatibility jamo codepoints (0x3131–0x3163). */
    private static boolean isHangulJamo(int code) {
        return code >= 0x3131 && code <= 0x3163;
    }

    // -----------------------------------------------------------------------
    // Unused OnKeyboardActionListener callbacks
    // -----------------------------------------------------------------------

    @Override public void onPress(int primaryCode) {}
    @Override public void onRelease(int primaryCode) {}
    @Override public void onText(CharSequence text) {}
    @Override public void swipeLeft() {}
    @Override public void swipeRight() {}
    @Override public void swipeDown() {}
    @Override public void swipeUp() {}
}
