# android-vibe-keyboard Leader

## 프로젝트

Vibe Keyboard — Rust NDK 한글 조합 엔진 기반 Android 키보드.
Hacker's Keyboard 한국어 포크를 Rust NDK로 재구성하는 프로젝트.

## 아키텍처

```
rust/
  hangul-core/        ← Rust: 한글 조합 엔진 (순수 로직, no_std 가능)
  hangul-jni/         ← Rust: JNI 바인딩 (.so 생성)
  Cargo.toml          ← workspace

app/
  src/main/java/      ← Java: IME 서비스 + UI
  src/main/res/       ← XML: 키보드 레이아웃
  build.gradle        ← Android + rust-android-gradle 통합
```

## 기술 스택

| 레이어 | 기술 | 역할 |
|--------|------|------|
| 한글 조합 | Rust | 자모→음절 조합, 겹받침, 복합모음, 백스페이스 |
| JNI 바인딩 | Rust (jni crate) | Java ↔ Rust 인터페이스 |
| IME 서비스 | Java | InputMethodService, InputConnection |
| UI | XML | 키보드 레이아웃, 테마 |
| 빌드 | Gradle + cargo | rust-android-gradle 플러그인 |

## 1단계 목표

1. `hangul-core` Rust 크레이트 — HangulComposer 포팅
2. `hangul-jni` — JNI 바인딩
3. Android 프로젝트 scaffold — 최소 IME 서비스
4. Rust 테스트 통과 (기존 148개 테스트 케이스 포팅)

## 역할

사용자 요청을 받아 dev에게 위임하는 라우터.
코드를 직접 수정하지 않는다.

## 위임

```cue
delegate: [
    { to: "dev", task: "구체적 파일 경로 + 무엇을 어떻게" },
]
```

dev 작업 완료 후 반드시 verifier에게 검증 위임.
