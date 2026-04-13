/*
 * HangulComposer - Korean Hangul syllable composition engine for Hacker's Keyboard.
 *
 * Implements Dubeolsik (두벌식) jamo-to-syllable composition using Unicode
 * Hangul Syllables block (U+AC00..U+D7A3).
 *
 * Hangul syllable = (initial * 21 + medial) * 28 + final + 0xAC00
 *
 * Licensed under the Apache License, Version 2.0
 */
package org.pocketworkstation.pckeyboard;

public class HangulComposer {

    // Unicode Hangul Jamo ranges
    private static final int CHOSEONG_BASE = 0x1100;  // ㄱ
    private static final int JUNGSEONG_BASE = 0x1161;  // ㅏ
    private static final int JONGSEONG_BASE = 0x11A7;  // (none) - 0x11A8 = ㄱ
    private static final int SYLLABLE_BASE = 0xAC00;
    private static final int JUNGSEONG_COUNT = 21;
    private static final int JONGSEONG_COUNT = 28;  // including "no jongseong"

    // Composition states
    private static final int S_NONE = 0;
    private static final int S_CHO = 1;       // initial consonant entered
    private static final int S_JUNG = 2;      // vowel entered (may have cho)
    private static final int S_JONG = 3;      // final consonant entered

    private int mState = S_NONE;
    private int mCho = -1;    // choseong index (0-18)
    private int mJung = -1;   // jungseong index (0-20)
    private int mJong = -1;   // jongseong index (1-27, 0 = none)

    // Compatibility Jamo (ㄱ=0x3131) to Choseong index mapping
    // ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ
    private static final int[] COMPAT_TO_CHO = new int[0x3164 - 0x3131];
    // Compatibility Jamo to Jungseong index mapping
    // ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ
    private static final int[] COMPAT_TO_JUNG = new int[0x3164 - 0x3131];
    // Compatibility Jamo to Jongseong index mapping
    private static final int[] COMPAT_TO_JONG = new int[0x3164 - 0x3131];

    // Compound jongseong decomposition: jongIdx -> [first, second] cho indices
    // e.g., ㄳ(3) -> ㄱ(0), ㅅ(9)
    private static final int[][] JONG_DECOMPOSE = new int[28][];

    // Compound jungseong decomposition: jungIdx -> [first, second] jung indices
    // e.g., ㅘ(9) -> ㅗ(8), ㅏ(0)
    private static final int[][] JUNG_DECOMPOSE = new int[21][];

    static {
        java.util.Arrays.fill(COMPAT_TO_CHO, -1);
        java.util.Arrays.fill(COMPAT_TO_JUNG, -1);
        java.util.Arrays.fill(COMPAT_TO_JONG, -1);

        // Map compatibility jamo consonants to choseong indices
        // Choseong order: ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ (19)
        int[] choCompat = {
            0x3131, 0x3132, 0x3134, 0x3137, 0x3138, 0x3139, 0x3141, 0x3142,
            0x3143, 0x3145, 0x3146, 0x3147, 0x3148, 0x3149, 0x314A, 0x314B,
            0x314C, 0x314D, 0x314E
        };
        for (int i = 0; i < choCompat.length; i++) {
            COMPAT_TO_CHO[choCompat[i] - 0x3131] = i;
        }

        // Map compatibility jamo vowels to jungseong indices
        // Jungseong order: ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ (21)
        int[] jungCompat = {
            0x314F, 0x3150, 0x3151, 0x3152, 0x3153, 0x3154, 0x3155, 0x3156,
            0x3157, 0x3158, 0x3159, 0x315A, 0x315B, 0x315C, 0x315D, 0x315E,
            0x315F, 0x3160, 0x3161, 0x3162, 0x3163
        };
        for (int i = 0; i < jungCompat.length; i++) {
            COMPAT_TO_JUNG[jungCompat[i] - 0x3131] = i;
        }

        // Map compatibility jamo consonants to jongseong indices
        // Jongseong order (1-based): ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ (27)
        int[] jongCompat = {
            0x3131, 0x3132, 0x3133, 0x3134, 0x3135, 0x3136, 0x3137, 0x3139,
            0x313A, 0x313B, 0x313C, 0x313D, 0x313E, 0x313F, 0x3140, 0x3141,
            0x3142, 0x3144, 0x3145, 0x3146, 0x3147, 0x3148, 0x314A, 0x314B,
            0x314C, 0x314D, 0x314E
        };
        for (int i = 0; i < jongCompat.length; i++) {
            COMPAT_TO_JONG[jongCompat[i] - 0x3131] = i + 1;
        }

        // Compound jongseong decomposition
        // index -> [cho of first, cho of second]
        JONG_DECOMPOSE[3]  = new int[]{0, 9};   // ㄳ -> ㄱ,ㅅ
        JONG_DECOMPOSE[5]  = new int[]{2, 12};   // ㄵ -> ㄴ,ㅈ
        JONG_DECOMPOSE[6]  = new int[]{2, 18};   // ㄶ -> ㄴ,ㅎ
        JONG_DECOMPOSE[9]  = new int[]{5, 0};    // ㄺ -> ㄹ,ㄱ
        JONG_DECOMPOSE[10] = new int[]{5, 6};    // ㄻ -> ㄹ,ㅁ
        JONG_DECOMPOSE[11] = new int[]{5, 7};    // ㄼ -> ㄹ,ㅂ
        JONG_DECOMPOSE[12] = new int[]{5, 9};    // ㄽ -> ㄹ,ㅅ
        JONG_DECOMPOSE[13] = new int[]{5, 16};   // ㄾ -> ㄹ,ㅌ
        JONG_DECOMPOSE[14] = new int[]{5, 17};   // ㄿ -> ㄹ,ㅍ
        JONG_DECOMPOSE[15] = new int[]{5, 18};   // ㅀ -> ㄹ,ㅎ
        JONG_DECOMPOSE[18] = new int[]{7, 9};    // ㅄ -> ㅂ,ㅅ

        // Compound jungseong decomposition
        JUNG_DECOMPOSE[9]  = new int[]{8, 0};   // ㅘ -> ㅗ,ㅏ
        JUNG_DECOMPOSE[10] = new int[]{8, 1};   // ㅙ -> ㅗ,ㅐ
        JUNG_DECOMPOSE[11] = new int[]{8, 20};  // ㅚ -> ㅗ,ㅣ
        JUNG_DECOMPOSE[14] = new int[]{13, 4};  // ㅝ -> ㅜ,ㅓ
        JUNG_DECOMPOSE[15] = new int[]{13, 5};  // ㅞ -> ㅜ,ㅔ
        JUNG_DECOMPOSE[16] = new int[]{13, 20}; // ㅟ -> ㅜ,ㅣ
        JUNG_DECOMPOSE[19] = new int[]{18, 20}; // ㅢ -> ㅡ,ㅣ
    }

    // Compound jongseong table: [existingJongIdx][newChoIdx] -> combinedJongIdx or -1
    private static final int[][] JONG_COMBINE = new int[28][19];
    // Compound jungseong table: [existingJungIdx][newJungIdx] -> combinedJungIdx or -1
    private static final int[][] JUNG_COMBINE = new int[21][21];

    static {
        for (int[] row : JONG_COMBINE) java.util.Arrays.fill(row, -1);
        for (int[] row : JUNG_COMBINE) java.util.Arrays.fill(row, -1);

        // Compound jongseong: existing jong + new consonant -> compound jong
        JONG_COMBINE[1][9]   = 3;   // ㄱ+ㅅ -> ㄳ
        JONG_COMBINE[4][12]  = 5;   // ㄴ+ㅈ -> ㄵ
        JONG_COMBINE[4][18]  = 6;   // ㄴ+ㅎ -> ㄶ
        JONG_COMBINE[8][0]   = 9;   // ㄹ+ㄱ -> ㄺ
        JONG_COMBINE[8][6]   = 10;  // ㄹ+ㅁ -> ㄻ
        JONG_COMBINE[8][7]   = 11;  // ㄹ+ㅂ -> ㄼ
        JONG_COMBINE[8][9]   = 12;  // ㄹ+ㅅ -> ㄽ
        JONG_COMBINE[8][16]  = 13;  // ㄹ+ㅌ -> ㄾ
        JONG_COMBINE[8][17]  = 14;  // ㄹ+ㅍ -> ㄿ
        JONG_COMBINE[8][18]  = 15;  // ㄹ+ㅎ -> ㅀ
        JONG_COMBINE[17][9]  = 18;  // ㅂ+ㅅ -> ㅄ

        // Compound jungseong: existing jung + new vowel -> compound jung
        JUNG_COMBINE[8][0]   = 9;   // ㅗ+ㅏ -> ㅘ
        JUNG_COMBINE[8][1]   = 10;  // ㅗ+ㅐ -> ㅙ
        JUNG_COMBINE[8][20]  = 11;  // ㅗ+ㅣ -> ㅚ
        JUNG_COMBINE[13][4]  = 14;  // ㅜ+ㅓ -> ㅝ
        JUNG_COMBINE[13][5]  = 15;  // ㅜ+ㅔ -> ㅞ
        JUNG_COMBINE[13][20] = 16;  // ㅜ+ㅣ -> ㅟ
        JUNG_COMBINE[18][20] = 19;  // ㅡ+ㅣ -> ㅢ
    }

    /** Check if a Unicode codepoint is a Hangul compatibility jamo */
    public static boolean isHangulJamo(int code) {
        return code >= 0x3131 && code < 0x3164;
    }

    /** Check if jamo is a consonant */
    private static boolean isConsonant(int code) {
        return code >= 0x3131 && code <= 0x314E;
    }

    /** Check if jamo is a vowel */
    private static boolean isVowel(int code) {
        return code >= 0x314F && code <= 0x3163;
    }

    private int getChoIndex(int code) {
        if (code < 0x3131 || code >= 0x3164) return -1;
        return COMPAT_TO_CHO[code - 0x3131];
    }

    private int getJungIndex(int code) {
        if (code < 0x3131 || code >= 0x3164) return -1;
        return COMPAT_TO_JUNG[code - 0x3131];
    }

    private int getJongIndex(int code) {
        if (code < 0x3131 || code >= 0x3164) return -1;
        return COMPAT_TO_JONG[code - 0x3131];
    }

    /** Convert choseong index to jongseong index, or -1 */
    private int choToJong(int choIdx) {
        // Map choseong index -> jongseong index
        // Choseong: ㄱ0 ㄲ1 ㄴ2 ㄷ3 ㄸ4 ㄹ5 ㅁ6 ㅂ7 ㅃ8 ㅅ9 ㅆ10 ㅇ11 ㅈ12 ㅉ13 ㅊ14 ㅋ15 ㅌ16 ㅍ17 ㅎ18
        // Jongseong (1-based): ㄱ1 ㄲ2 ㄴ4 ㄷ7 ㄹ8 ㅁ16 ㅂ17 ㅅ19 ㅆ20 ㅇ21 ㅈ22 ㅊ23 ㅋ24 ㅌ25 ㅍ26 ㅎ27
        // ㄸ(4), ㅃ(8), ㅉ(13) have no jongseong
        int[] map = {1, 2, 4, 7, -1, 8, 16, 17, -1, 19, 20, 21, 22, -1, 23, 24, 25, 26, 27};
        if (choIdx < 0 || choIdx >= map.length) return -1;
        return map[choIdx];
    }

    /** Convert jongseong index to choseong index, or -1 */
    private int jongToCho(int jongIdx) {
        int[] map = {
            -1,  // 0: none
             0,  // 1: ㄱ
             1,  // 2: ㄲ
            -1,  // 3: ㄳ (compound)
             2,  // 4: ㄴ
            -1,  // 5: ㄵ
            -1,  // 6: ㄶ
             3,  // 7: ㄷ
             5,  // 8: ㄹ
            -1,  // 9: ㄺ
            -1,  // 10: ㄻ
            -1,  // 11: ㄼ
            -1,  // 12: ㄽ
            -1,  // 13: ㄾ
            -1,  // 14: ㄿ
            -1,  // 15: ㅀ
             6,  // 16: ㅁ
             7,  // 17: ㅂ
            -1,  // 18: ㅄ
             9,  // 19: ㅅ
            10,  // 20: ㅆ
            11,  // 21: ㅇ
            12,  // 22: ㅈ
            14,  // 23: ㅊ
            15,  // 24: ㅋ
            16,  // 25: ㅌ
            17,  // 26: ㅍ
            18,  // 27: ㅎ
        };
        if (jongIdx < 0 || jongIdx >= map.length) return -1;
        return map[jongIdx];
    }

    /** Build a Hangul syllable from cho/jung/jong indices */
    private char buildSyllable(int cho, int jung, int jong) {
        return (char) (SYLLABLE_BASE + (cho * JUNGSEONG_COUNT + jung) * JONGSEONG_COUNT + jong);
    }

    /** Get the current composing text */
    public String getComposing() {
        switch (mState) {
            case S_CHO:
                return Character.toString(choToCompat(mCho));
            case S_JUNG:
                if (mCho >= 0) {
                    return Character.toString(buildSyllable(mCho, mJung, 0));
                } else {
                    return Character.toString(jungToCompat(mJung));
                }
            case S_JONG:
                return Character.toString(buildSyllable(mCho, mJung, mJong));
            default:
                return "";
        }
    }

    /** Convert choseong index to compatibility jamo char */
    private char choToCompat(int choIdx) {
        int[] map = {
            0x3131, 0x3132, 0x3134, 0x3137, 0x3138, 0x3139, 0x3141, 0x3142,
            0x3143, 0x3145, 0x3146, 0x3147, 0x3148, 0x3149, 0x314A, 0x314B,
            0x314C, 0x314D, 0x314E
        };
        return (char) map[choIdx];
    }

    /** Convert jungseong index to compatibility jamo char */
    private char jungToCompat(int jungIdx) {
        return (char) (0x314F + jungIdx);
    }

    /**
     * Process a Hangul jamo input and update composing state.
     * Returns the text to commit (if any) before the new composing text.
     */
    public String process(int code) {
        int oldState = mState;
        String commit = null;
        int cho = getChoIndex(code);
        int jung = getJungIndex(code);

        if (isConsonant(code) && cho >= 0) {
            commit = processConsonant(cho);
        } else if (isVowel(code) && jung >= 0) {
            commit = processVowel(jung);
        } else {
            // Not a jamo - commit current and pass through
            commit = getComposing();
            reset();
        }
        HangulDebugLog.state(oldState, mState, code, getComposing());
        if (commit != null && !commit.isEmpty()) {
            HangulDebugLog.commit(commit);
        }
        return commit;
    }

    private String processConsonant(int cho) {
        String commit = null;

        switch (mState) {
            case S_NONE:
                // Start new composition with initial consonant
                mCho = cho;
                mState = S_CHO;
                break;

            case S_CHO:
                // Already have initial consonant, commit it and start new
                commit = getComposing();
                mCho = cho;
                mJung = -1;
                mJong = -1;
                mState = S_CHO;
                break;

            case S_JUNG:
                if (mCho >= 0) {
                    // Have cho+jung, try adding as jongseong
                    int jong = choToJong(cho);
                    if (jong > 0) {
                        mJong = jong;
                        mState = S_JONG;
                    } else {
                        // This consonant can't be jongseong (ㄸ,ㅃ,ㅉ)
                        commit = getComposing();
                        mCho = cho;
                        mJung = -1;
                        mJong = -1;
                        mState = S_CHO;
                    }
                } else {
                    // Had standalone vowel, commit it and start consonant
                    commit = getComposing();
                    mCho = cho;
                    mJung = -1;
                    mJong = -1;
                    mState = S_CHO;
                }
                break;

            case S_JONG:
                // Try compound jongseong
                int combined = JONG_COMBINE[mJong][cho];
                if (combined >= 0) {
                    mJong = combined;
                    // Stay in S_JONG
                } else {
                    // Can't combine - commit current syllable and start new
                    commit = getComposing();
                    mCho = cho;
                    mJung = -1;
                    mJong = -1;
                    mState = S_CHO;
                }
                break;
        }
        return commit;
    }

    private String processVowel(int jung) {
        String commit = null;

        switch (mState) {
            case S_NONE:
                // Standalone vowel
                mJung = jung;
                mCho = -1;
                mState = S_JUNG;
                break;

            case S_CHO:
                // cho + vowel -> syllable
                mJung = jung;
                mState = S_JUNG;
                break;

            case S_JUNG:
                if (mCho >= 0) {
                    // Try compound jungseong (e.g., ㅗ+ㅏ -> ㅘ)
                    int combined = JUNG_COMBINE[mJung][jung];
                    if (combined >= 0) {
                        mJung = combined;
                        // Stay in S_JUNG
                    } else {
                        // Can't combine - commit and start new standalone vowel
                        commit = getComposing();
                        mCho = -1;
                        mJung = jung;
                        mJong = -1;
                        mState = S_JUNG;
                    }
                } else {
                    // Had standalone vowel, commit and start new
                    commit = getComposing();
                    mCho = -1;
                    mJung = jung;
                    mJong = -1;
                    mState = S_JUNG;
                }
                break;

            case S_JONG:
                // Vowel after jongseong: split jongseong
                // The jongseong (or part of it) becomes choseong of new syllable
                if (JONG_DECOMPOSE[mJong] != null) {
                    // Compound jongseong: split into two
                    int[] parts = JONG_DECOMPOSE[mJong];
                    mJong = choToJong(parts[0]);
                    // Commit current syllable with first part as jong
                    commit = getComposing();
                    // Start new syllable with second part as cho
                    mCho = parts[1];
                    mJung = jung;
                    mJong = -1;
                    mState = S_JUNG;
                } else {
                    // Simple jongseong: it becomes choseong of new syllable
                    int newCho = jongToCho(mJong);
                    mJong = 0;
                    commit = getComposing();
                    mCho = newCho;
                    mJung = jung;
                    mJong = -1;
                    mState = S_JUNG;
                }
                break;
        }
        return commit;
    }

    /**
     * Handle backspace in composing state.
     * Returns true if backspace was consumed (composing was modified).
     */
    public boolean backspace() {
        switch (mState) {
            case S_JONG:
                if (JONG_DECOMPOSE[mJong] != null) {
                    // Compound jongseong: remove last part
                    int[] parts = JONG_DECOMPOSE[mJong];
                    mJong = choToJong(parts[0]);
                } else {
                    mJong = -1;
                    mState = S_JUNG;
                }
                return true;

            case S_JUNG:
                if (JUNG_DECOMPOSE[mJung] != null) {
                    // Compound jungseong: remove last part
                    int[] parts = JUNG_DECOMPOSE[mJung];
                    mJung = parts[0];
                } else {
                    if (mCho >= 0) {
                        mJung = -1;
                        mState = S_CHO;
                    } else {
                        reset();
                    }
                }
                return true;

            case S_CHO:
                reset();
                return true;

            default:
                return false;
        }
    }

    /** Commit current composing text and reset */
    public String commit() {
        String text = getComposing();
        reset();
        return text;
    }

    /** Reset composition state */
    public void reset() {
        mState = S_NONE;
        mCho = -1;
        mJung = -1;
        mJong = -1;
    }

    /** Check if currently composing */
    public boolean isComposing() {
        return mState != S_NONE;
    }
}
