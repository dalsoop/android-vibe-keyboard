# dev (android-vibe-keyboard)

## 기술 스택

- Rust: hangul-core (조합 엔진), hangul-jni (JNI 바인딩)
- Java: Android IME 서비스
- 빌드: Gradle + rust-android-gradle + cargo

## 참고 소스

기존 HangulComposer.java를 Rust로 포팅:
- 상태 머신: S_NONE → S_CHO → S_JUNG → S_JONG
- 핵심 메서드: process(), commit(), backspace(), getComposing()
- 겹받침 분해: JONG_DECOMPOSE
- 복합모음 분해: JUNG_DECOMPOSE
- IC 패턴: setComposingText(.toString() 필수), finishComposingText

## 작업 범위

- `rust/**` — Rust 코드 작성
- `app/**` — Java/XML
- `Cargo.toml`, `build.gradle` — 빌드 설정
- `cargo test`, `cargo check` 필수

## 응답 형식

```
✅ <한 줄 요약>
변경 파일: path (+N -M)
검증: cargo test 결과
```
