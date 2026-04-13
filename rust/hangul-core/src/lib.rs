//! Hangul syllable composition engine — Dubeolsik (두벌식)
//!
//! Implements jamo-to-syllable composition using the Unicode Hangul Syllables
//! block (U+AC00..U+D7A3).
//!
//! Hangul syllable = (cho * 21 + jung) * 28 + jong + 0xAC00

const SYLLABLE_BASE: u32 = 0xAC00;
const JUNGSEONG_COUNT: u32 = 21;
const JONGSEONG_COUNT: u32 = 28;

// ---------------------------------------------------------------------------
// Mapping tables (all indices i32; -1 = "not applicable")
// ---------------------------------------------------------------------------

/// Compatibility jamo range: 0x3131..=0x3163
const COMPAT_BASE: u32 = 0x3131;
const COMPAT_LEN: usize = (0x3164 - 0x3131) as usize;

/// Choseong order (19 entries):
/// ㄱ ㄲ ㄴ ㄷ ㄸ ㄹ ㅁ ㅂ ㅃ ㅅ ㅆ ㅇ ㅈ ㅉ ㅊ ㅋ ㅌ ㅍ ㅎ
const CHO_COMPAT: [u32; 19] = [
    0x3131, 0x3132, 0x3134, 0x3137, 0x3138, 0x3139, 0x3141, 0x3142,
    0x3143, 0x3145, 0x3146, 0x3147, 0x3148, 0x3149, 0x314A, 0x314B,
    0x314C, 0x314D, 0x314E,
];

/// Jungseong order (21 entries):
/// ㅏ ㅐ ㅑ ㅒ ㅓ ㅔ ㅕ ㅖ ㅗ ㅘ ㅙ ㅚ ㅛ ㅜ ㅝ ㅞ ㅟ ㅠ ㅡ ㅢ ㅣ
const JUNG_COMPAT: [u32; 21] = [
    0x314F, 0x3150, 0x3151, 0x3152, 0x3153, 0x3154, 0x3155, 0x3156,
    0x3157, 0x3158, 0x3159, 0x315A, 0x315B, 0x315C, 0x315D, 0x315E,
    0x315F, 0x3160, 0x3161, 0x3162, 0x3163,
];

/// Jongseong order (27 entries, 1-based; index 0 = "no jongseong"):
/// ㄱ ㄲ ㄳ ㄴ ㄵ ㄶ ㄷ ㄹ ㄺ ㄻ ㄼ ㄽ ㄾ ㄿ ㅀ ㅁ ㅂ ㅄ ㅅ ㅆ ㅇ ㅈ ㅊ ㅋ ㅌ ㅍ ㅎ
const JONG_COMPAT: [u32; 27] = [
    0x3131, 0x3132, 0x3133, 0x3134, 0x3135, 0x3136, 0x3137, 0x3139,
    0x313A, 0x313B, 0x313C, 0x313D, 0x313E, 0x313F, 0x3140, 0x3141,
    0x3142, 0x3144, 0x3145, 0x3146, 0x3147, 0x3148, 0x314A, 0x314B,
    0x314C, 0x314D, 0x314E,
];

/// Build the three lookup arrays at compile time (const fn not flexible enough,
/// so we use lazy construction on first use via a helper).
fn build_compat_tables() -> ([i8; COMPAT_LEN], [i8; COMPAT_LEN], [i8; COMPAT_LEN]) {
    let mut cho  = [-1i8; COMPAT_LEN];
    let mut jung = [-1i8; COMPAT_LEN];
    let mut jong = [-1i8; COMPAT_LEN];

    for (i, &cp) in CHO_COMPAT.iter().enumerate() {
        cho[(cp - COMPAT_BASE) as usize] = i as i8;
    }
    for (i, &cp) in JUNG_COMPAT.iter().enumerate() {
        jung[(cp - COMPAT_BASE) as usize] = i as i8;
    }
    for (i, &cp) in JONG_COMPAT.iter().enumerate() {
        jong[(cp - COMPAT_BASE) as usize] = (i + 1) as i8; // 1-based
    }
    (cho, jung, jong)
}

// ---------------------------------------------------------------------------
// Choseong <-> Jongseong conversion
// ---------------------------------------------------------------------------

/// Choseong index → jongseong index (1-based), or -1 if no jongseong exists.
/// ㄸ(4), ㅃ(8), ㅉ(13) cannot be jongseong.
const CHO_TO_JONG: [i8; 19] = [
    1, 2, 4, 7, -1, 8, 16, 17, -1, 19, 20, 21, 22, -1, 23, 24, 25, 26, 27,
];

/// Jongseong index (1-based) → choseong index, or -1 for compounds/none.
const JONG_TO_CHO: [i8; 28] = [
    -1, // 0: none
     0, // 1: ㄱ
     1, // 2: ㄲ
    -1, // 3: ㄳ
     2, // 4: ㄴ
    -1, // 5: ㄵ
    -1, // 6: ㄶ
     3, // 7: ㄷ
     5, // 8: ㄹ
    -1, // 9: ㄺ
    -1, // 10: ㄻ
    -1, // 11: ㄼ
    -1, // 12: ㄽ
    -1, // 13: ㄾ
    -1, // 14: ㄿ
    -1, // 15: ㅀ
     6, // 16: ㅁ
     7, // 17: ㅂ
    -1, // 18: ㅄ
     9, // 19: ㅅ
    10, // 20: ㅆ
    11, // 21: ㅇ
    12, // 22: ㅈ
    14, // 23: ㅊ
    15, // 24: ㅋ
    16, // 25: ㅌ
    17, // 26: ㅍ
    18, // 27: ㅎ
];

/// Compound jongseong decomposition: jong_idx → (first_cho, second_cho)
/// Only compound jongseong entries are Some; simple ones are None.
const JONG_DECOMPOSE: [Option<(u8, u8)>; 28] = [
    None,           // 0
    None,           // 1: ㄱ
    None,           // 2: ㄲ
    Some((0, 9)),   // 3: ㄳ -> ㄱ,ㅅ
    None,           // 4: ㄴ
    Some((2, 12)),  // 5: ㄵ -> ㄴ,ㅈ
    Some((2, 18)),  // 6: ㄶ -> ㄴ,ㅎ
    None,           // 7: ㄷ
    None,           // 8: ㄹ
    Some((5, 0)),   // 9: ㄺ -> ㄹ,ㄱ
    Some((5, 6)),   // 10: ㄻ -> ㄹ,ㅁ
    Some((5, 7)),   // 11: ㄼ -> ㄹ,ㅂ
    Some((5, 9)),   // 12: ㄽ -> ㄹ,ㅅ
    Some((5, 16)),  // 13: ㄾ -> ㄹ,ㅌ
    Some((5, 17)),  // 14: ㄿ -> ㄹ,ㅍ
    Some((5, 18)),  // 15: ㅀ -> ㄹ,ㅎ
    None,           // 16: ㅁ
    None,           // 17: ㅂ
    Some((7, 9)),   // 18: ㅄ -> ㅂ,ㅅ
    None,           // 19: ㅅ
    None,           // 20: ㅆ
    None,           // 21: ㅇ
    None,           // 22: ㅈ
    None,           // 23: ㅊ
    None,           // 24: ㅋ
    None,           // 25: ㅌ
    None,           // 26: ㅍ
    None,           // 27: ㅎ
];

/// Compound jungseong decomposition: jung_idx → (first_jung, second_jung)
const JUNG_DECOMPOSE: [Option<(u8, u8)>; 21] = [
    None,           // 0: ㅏ
    None,           // 1: ㅐ
    None,           // 2: ㅑ
    None,           // 3: ㅒ
    None,           // 4: ㅓ
    None,           // 5: ㅔ
    None,           // 6: ㅕ
    None,           // 7: ㅖ
    None,           // 8: ㅗ
    Some((8, 0)),   // 9:  ㅘ -> ㅗ,ㅏ
    Some((8, 1)),   // 10: ㅙ -> ㅗ,ㅐ
    Some((8, 20)),  // 11: ㅚ -> ㅗ,ㅣ
    None,           // 12: ㅛ
    None,           // 13: ㅜ
    Some((13, 4)),  // 14: ㅝ -> ㅜ,ㅓ
    Some((13, 5)),  // 15: ㅞ -> ㅜ,ㅔ
    Some((13, 20)), // 16: ㅟ -> ㅜ,ㅣ
    None,           // 17: ㅠ
    None,           // 18: ㅡ
    Some((18, 20)), // 19: ㅢ -> ㅡ,ㅣ
    None,           // 20: ㅣ
];

/// Compound jongseong combine: JONG_COMBINE[existing_jong][new_cho] → combined_jong or -1
#[rustfmt::skip]
fn jong_combine(jong: usize, cho: usize) -> i8 {
    match (jong, cho) {
        (1, 9)  =>  3,  // ㄱ+ㅅ -> ㄳ
        (4, 12) =>  5,  // ㄴ+ㅈ -> ㄵ
        (4, 18) =>  6,  // ㄴ+ㅎ -> ㄶ
        (8, 0)  =>  9,  // ㄹ+ㄱ -> ㄺ
        (8, 6)  => 10,  // ㄹ+ㅁ -> ㄻ
        (8, 7)  => 11,  // ㄹ+ㅂ -> ㄼ
        (8, 9)  => 12,  // ㄹ+ㅅ -> ㄽ
        (8, 16) => 13,  // ㄹ+ㅌ -> ㄾ
        (8, 17) => 14,  // ㄹ+ㅍ -> ㄿ
        (8, 18) => 15,  // ㄹ+ㅎ -> ㅀ
        (17, 9) => 18,  // ㅂ+ㅅ -> ㅄ
        _ => -1,
    }
}

/// Compound jungseong combine: JUNG_COMBINE[existing_jung][new_jung] → combined or -1
#[rustfmt::skip]
fn jung_combine(existing: usize, new: usize) -> i8 {
    match (existing, new) {
        (8, 0)   =>  9,  // ㅗ+ㅏ -> ㅘ
        (8, 1)   => 10,  // ㅗ+ㅐ -> ㅙ
        (8, 20)  => 11,  // ㅗ+ㅣ -> ㅚ
        (13, 4)  => 14,  // ㅜ+ㅓ -> ㅝ
        (13, 5)  => 15,  // ㅜ+ㅔ -> ㅞ
        (13, 20) => 16,  // ㅜ+ㅣ -> ㅟ
        (18, 20) => 19,  // ㅡ+ㅣ -> ㅢ
        _ => -1,
    }
}

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    None,
    Cho,   // initial consonant entered
    Jung,  // vowel entered (may have cho)
    Jong,  // final consonant entered
}

/// Hangul composition engine. One instance per input field / IME session.
#[derive(Debug, Clone)]
pub struct HangulComposer {
    state: State,
    cho:  i8,  // choseong index 0-18, or -1
    jung: i8,  // jungseong index 0-20, or -1
    jong: i8,  // jongseong index 1-27, or -1
    // cached lookup tables
    compat_to_cho:  [i8; COMPAT_LEN],
    compat_to_jung: [i8; COMPAT_LEN],
    compat_to_jong: [i8; COMPAT_LEN],
}

impl Default for HangulComposer {
    fn default() -> Self {
        Self::new()
    }
}

impl HangulComposer {
    pub fn new() -> Self {
        let (c, j, n) = build_compat_tables();
        Self {
            state: State::None,
            cho:  -1,
            jung: -1,
            jong: -1,
            compat_to_cho:  c,
            compat_to_jung: j,
            compat_to_jong: n,
        }
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Process one Hangul compatibility jamo codepoint.
    ///
    /// Returns the text that must be **committed** before the new composing
    /// text is set (may be empty). Call [`get_composing`] after this to
    /// obtain the updated composing string.
    pub fn process(&mut self, code: u32) -> String {
        if Self::is_consonant(code) {
            let cho = self.cho_index(code);
            if cho >= 0 {
                return self.process_consonant(cho as usize);
            }
        } else if Self::is_vowel(code) {
            let jung = self.jung_index(code);
            if jung >= 0 {
                return self.process_vowel(jung as usize);
            }
        }
        // Not a jamo — commit and pass through
        let commit = self.get_composing();
        self.reset();
        commit
    }

    /// Current composing string (empty when not composing).
    pub fn get_composing(&self) -> String {
        match self.state {
            State::None => String::new(),
            State::Cho  => {
                let ch = self.cho_to_compat(self.cho as usize);
                ch.to_string()
            }
            State::Jung => {
                if self.cho >= 0 {
                    self.build_syllable(self.cho as u32, self.jung as u32, 0)
                        .to_string()
                } else {
                    self.jung_to_compat(self.jung as usize).to_string()
                }
            }
            State::Jong => {
                self.build_syllable(self.cho as u32, self.jung as u32, self.jong as u32)
                    .to_string()
            }
        }
    }

    /// Handle backspace inside composing state.
    ///
    /// Returns `true` if the backspace was consumed (caller must NOT delete an
    /// additional character). Returns `false` when there is nothing composing
    /// (caller should perform a normal delete).
    pub fn backspace(&mut self) -> bool {
        match self.state {
            State::Jong => {
                let jong = self.jong as usize;
                if let Some((first, _)) = JONG_DECOMPOSE[jong] {
                    // Remove second part of compound jongseong
                    self.jong = CHO_TO_JONG[first as usize];
                } else {
                    self.jong = -1;
                    self.state = State::Jung;
                }
                true
            }
            State::Jung => {
                let jung = self.jung as usize;
                if let Some((first, _)) = JUNG_DECOMPOSE[jung] {
                    self.jung = first as i8;
                } else if self.cho >= 0 {
                    self.jung = -1;
                    self.state = State::Cho;
                } else {
                    self.reset();
                }
                true
            }
            State::Cho => {
                self.reset();
                true
            }
            State::None => false,
        }
    }

    /// Commit current composing text and reset. Returns the committed string.
    pub fn commit(&mut self) -> String {
        let text = self.get_composing();
        self.reset();
        text
    }

    /// Reset all composing state.
    pub fn reset(&mut self) {
        self.state = State::None;
        self.cho  = -1;
        self.jung = -1;
        self.jong = -1;
    }

    /// Whether currently composing a syllable.
    pub fn is_composing(&self) -> bool {
        self.state != State::None
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn is_consonant(code: u32) -> bool { code >= 0x3131 && code <= 0x314E }
    fn is_vowel(code: u32)    -> bool { code >= 0x314F && code <= 0x3163 }

    fn cho_index(&self, code: u32) -> i8 {
        if code < COMPAT_BASE || code >= 0x3164 { return -1; }
        self.compat_to_cho[(code - COMPAT_BASE) as usize]
    }
    fn jung_index(&self, code: u32) -> i8 {
        if code < COMPAT_BASE || code >= 0x3164 { return -1; }
        self.compat_to_jung[(code - COMPAT_BASE) as usize]
    }
    fn jong_index(&self, code: u32) -> i8 {
        if code < COMPAT_BASE || code >= 0x3164 { return -1; }
        self.compat_to_jong[(code - COMPAT_BASE) as usize]
    }

    fn cho_to_jong(cho: usize) -> i8 {
        if cho >= CHO_TO_JONG.len() { return -1; }
        CHO_TO_JONG[cho]
    }
    fn jong_to_cho(jong: usize) -> i8 {
        if jong >= JONG_TO_CHO.len() { return -1; }
        JONG_TO_CHO[jong]
    }

    fn cho_to_compat(&self, cho: usize) -> char {
        char::from_u32(CHO_COMPAT[cho]).unwrap()
    }
    fn jung_to_compat(&self, jung: usize) -> char {
        char::from_u32(JUNG_COMPAT[jung]).unwrap()
    }

    fn build_syllable(&self, cho: u32, jung: u32, jong: u32) -> char {
        char::from_u32(SYLLABLE_BASE + (cho * JUNGSEONG_COUNT + jung) * JONGSEONG_COUNT + jong)
            .unwrap()
    }

    // -----------------------------------------------------------------------
    // State-machine transitions
    // -----------------------------------------------------------------------

    fn process_consonant(&mut self, cho: usize) -> String {
        match self.state {
            State::None => {
                self.cho   = cho as i8;
                self.state = State::Cho;
                String::new()
            }
            State::Cho => {
                let commit = self.get_composing();
                self.cho   = cho as i8;
                self.jung  = -1;
                self.jong  = -1;
                self.state = State::Cho;
                commit
            }
            State::Jung => {
                if self.cho >= 0 {
                    let jong = Self::cho_to_jong(cho);
                    if jong > 0 {
                        self.jong  = jong;
                        self.state = State::Jong;
                        String::new()
                    } else {
                        // ㄸ/ㅃ/ㅉ cannot be jongseong
                        let commit = self.get_composing();
                        self.cho   = cho as i8;
                        self.jung  = -1;
                        self.jong  = -1;
                        self.state = State::Cho;
                        commit
                    }
                } else {
                    // standalone vowel → commit, start new consonant
                    let commit = self.get_composing();
                    self.cho   = cho as i8;
                    self.jung  = -1;
                    self.jong  = -1;
                    self.state = State::Cho;
                    commit
                }
            }
            State::Jong => {
                let combined = jong_combine(self.jong as usize, cho);
                if combined >= 0 {
                    self.jong = combined;
                    String::new()
                } else {
                    let commit = self.get_composing();
                    self.cho   = cho as i8;
                    self.jung  = -1;
                    self.jong  = -1;
                    self.state = State::Cho;
                    commit
                }
            }
        }
    }

    fn process_vowel(&mut self, jung: usize) -> String {
        match self.state {
            State::None => {
                self.jung  = jung as i8;
                self.cho   = -1;
                self.state = State::Jung;
                String::new()
            }
            State::Cho => {
                self.jung  = jung as i8;
                self.state = State::Jung;
                String::new()
            }
            State::Jung => {
                if self.cho >= 0 {
                    let combined = jung_combine(self.jung as usize, jung);
                    if combined >= 0 {
                        self.jung = combined;
                        String::new()
                    } else {
                        let commit = self.get_composing();
                        self.cho   = -1;
                        self.jung  = jung as i8;
                        self.jong  = -1;
                        self.state = State::Jung;
                        commit
                    }
                } else {
                    let commit = self.get_composing();
                    self.cho   = -1;
                    self.jung  = jung as i8;
                    self.jong  = -1;
                    self.state = State::Jung;
                    commit
                }
            }
            State::Jong => {
                let jong = self.jong as usize;
                if let Some((first, second)) = JONG_DECOMPOSE[jong] {
                    // compound: first part stays as jong, second becomes new cho
                    self.jong = Self::cho_to_jong(first as usize);
                    let commit = self.get_composing();
                    self.cho   = second as i8;
                    self.jung  = jung as i8;
                    self.jong  = -1;
                    self.state = State::Jung;
                    commit
                } else {
                    // simple: commit current syllable WITH jongseong ("달"),
                    // then start new syllable with the jongseong as choseong ("라")
                    let new_cho = Self::jong_to_cho(jong);
                    let commit = self.get_composing(); // jong still set → "달"
                    self.cho   = new_cho;
                    self.jung  = jung as i8;
                    self.jong  = -1;
                    self.state = State::Jung;
                    commit
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Convenience: feed a string of compat jamo and collect (commits, composing)
    fn feed(c: &mut HangulComposer, jamo: &[u32]) -> (Vec<String>, String) {
        let mut commits = Vec::new();
        for &code in jamo {
            let commit = c.process(code);
            if !commit.is_empty() {
                commits.push(commit);
            }
        }
        let composing = c.get_composing();
        (commits, composing)
    }

    // 한 = ㅎ+ㅏ+ㄴ
    #[test]
    fn test_han() {
        let mut c = HangulComposer::new();
        let (commits, composing) = feed(&mut c, &[0x314E, 0x314F, 0x3134]);
        assert!(commits.is_empty(), "no commit mid-syllable");
        assert_eq!(composing, "한");
    }

    // 가 = ㄱ+ㅏ
    #[test]
    fn test_ga() {
        let mut c = HangulComposer::new();
        feed(&mut c, &[0x3131, 0x314F]);
        assert_eq!(c.get_composing(), "가");
    }

    // 국 = ㄱ+ㅜ+ㄱ
    #[test]
    fn test_guk() {
        let mut c = HangulComposer::new();
        feed(&mut c, &[0x3131, 0x315C, 0x3131]);
        assert_eq!(c.get_composing(), "국");
    }

    // 한글 = ㅎ+ㅏ+ㄴ then ㄱ+ㅡ+ㄹ
    #[test]
    fn test_hangeul() {
        let mut c = HangulComposer::new();
        // ㅎ ㅏ ㄴ
        c.process(0x314E);
        c.process(0x314F);
        c.process(0x3134);
        // ㄱ triggers commit of 한
        let commit = c.process(0x3131);
        assert_eq!(commit, "한");
        // ㅡ
        c.process(0x3161);
        // ㄹ
        c.process(0x3139);
        assert_eq!(c.get_composing(), "글");
        let last = c.commit();
        assert_eq!(last, "글");
        assert!(!c.is_composing());
    }

    // Compound jongseong: 닭 = ㄷ+ㅏ+ㄹ+ㄱ
    #[test]
    fn test_compound_jong() {
        let mut c = HangulComposer::new();
        feed(&mut c, &[0x3137, 0x314F, 0x3139, 0x3131]);
        assert_eq!(c.get_composing(), "닭");
    }

    // Compound jungseong: 봐 = ㅂ+ㅗ+ㅏ
    #[test]
    fn test_compound_jung() {
        let mut c = HangulComposer::new();
        feed(&mut c, &[0x3142, 0x3157, 0x314F]);
        assert_eq!(c.get_composing(), "봐");
    }

    // Backspace from Jong → Jung
    #[test]
    fn test_backspace_jong() {
        let mut c = HangulComposer::new();
        feed(&mut c, &[0x3131, 0x314F, 0x3131]); // 각
        assert_eq!(c.get_composing(), "각");
        assert!(c.backspace());
        assert_eq!(c.get_composing(), "가");
    }

    // Backspace from compound Jong → simple Jong
    #[test]
    fn test_backspace_compound_jong() {
        let mut c = HangulComposer::new();
        feed(&mut c, &[0x3137, 0x314F, 0x3139, 0x3131]); // 닭
        assert_eq!(c.get_composing(), "닭");
        assert!(c.backspace()); // remove ㄱ, leave ㄹ
        assert_eq!(c.get_composing(), "달");
    }

    // Backspace from Jung → Cho
    #[test]
    fn test_backspace_jung() {
        let mut c = HangulComposer::new();
        feed(&mut c, &[0x3131, 0x314F]); // 가
        assert!(c.backspace());
        assert_eq!(c.get_composing(), "ㄱ");
    }

    // Backspace from compound Jung → simple Jung
    #[test]
    fn test_backspace_compound_jung() {
        let mut c = HangulComposer::new();
        feed(&mut c, &[0x3142, 0x3157, 0x314F]); // 봐
        assert!(c.backspace());
        assert_eq!(c.get_composing(), "보");
    }

    // Backspace from Cho → None
    #[test]
    fn test_backspace_cho() {
        let mut c = HangulComposer::new();
        c.process(0x3131);
        assert!(c.backspace());
        assert!(!c.is_composing());
        assert_eq!(c.get_composing(), "");
    }

    // Backspace when not composing
    #[test]
    fn test_backspace_none() {
        let mut c = HangulComposer::new();
        assert!(!c.backspace());
    }

    // Jongseong splits when followed by vowel: 달아 = 달 + ㅏ
    #[test]
    fn test_jong_splits_on_vowel() {
        let mut c = HangulComposer::new();
        // 달
        c.process(0x3137); // ㄷ
        c.process(0x314F); // ㅏ
        let r = c.process(0x3139); // ㄹ → 달
        assert!(r.is_empty());
        // ㅏ → 달 commits, 라 starts composing
        let commit = c.process(0x314F);
        assert_eq!(commit, "달");
        assert_eq!(c.get_composing(), "라");
    }

    // Standalone vowel followed by consonant
    #[test]
    fn test_standalone_vowel() {
        let mut c = HangulComposer::new();
        c.process(0x314F); // ㅏ
        assert_eq!(c.get_composing(), "ㅏ");
        let commit = c.process(0x3131); // ㄱ
        assert_eq!(commit, "ㅏ");
        assert_eq!(c.get_composing(), "ㄱ");
    }

    // reset() clears state
    #[test]
    fn test_reset() {
        let mut c = HangulComposer::new();
        feed(&mut c, &[0x3131, 0x314F]);
        c.reset();
        assert!(!c.is_composing());
        assert_eq!(c.get_composing(), "");
    }

    // ㄸ, ㅃ, ㅉ cannot be jongseong
    #[test]
    fn test_no_jong_for_tense() {
        for &tense in &[0x3138u32, 0x3143, 0x3149] {
            // ㄷ, ㅂ, ㅈ (wait — these ARE valid; use ㄸ=0x3138, ㅃ=0x3143, ㅉ=0x3149)
            let mut c = HangulComposer::new();
            c.process(0x3131); // ㄱ
            c.process(0x314F); // ㅏ  → 가
            let commit = c.process(tense);
            assert_eq!(commit, "가", "tense consonant {:04X} should commit 가", tense);
        }
    }
}
