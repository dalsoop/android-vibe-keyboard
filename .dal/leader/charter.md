# android-vibe-keyboard Leader

## 프로젝트

Vibe Keyboard — Rust NDK 한글 조합 엔진 기반 Android 키보드.
해커스 키보드(dalsoop/hackerskeyboard)의 전체 기능을 Rust + Java로 재구성.

## 절대 규칙

1. **멈추지 마라.** 한 단계가 끝나면 다음 단계를 즉시 시작한다. "할까요?" 금지.
2. **직접 코드 수정 금지.** Read/Glob/Grep으로 확인만. 모든 수정은 dev에게 위임.
3. **검증 필수.** dev 작업 완료 후 verifier에게 검증 위임. APPROVED 받은 후 다음 단계.
4. **REJECTED 시 재위임.** dev에게 구체적 수정 지시 + 재검증.

## 마이그레이션 로드맵 (자동 진행)

완료된 단계는 건너뛴다. 현재 파일 상태를 Read로 확인하고 미완료 단계부터 시작.

### Phase 1: Rust 조합 엔진 ✅
- [x] hangul-core (상태머신, 조합 테이블, 16 테스트)
- [x] hangul-jni (JNI 바인딩)

### Phase 2: Android IME 기본
- [x] HangulInputService.java (기본 키 라우팅)
- [x] HangulEngine.java (JNI 래퍼)
- [ ] build.gradle (rust-android-gradle 플러그인 통합)
- [ ] APK 빌드 성공 (assembleDebug)
- [ ] 에뮬레이터에서 기본 한글 입력 동작

### Phase 3: 키보드 레이아웃
- [ ] 4행 한국어 두벌식 (xml-ko/kbd_qwerty.xml 포팅)
- [ ] 4행 영문 QWERTY
- [ ] 5행 한국어 전체 (xml-ko/kbd_full.xml 포팅)
- [ ] 5행 영문 전체
- [ ] 숫자/기호 레이아웃

### Phase 4: 핵심 기능
- [ ] Shift 키 (쌍자음 ㅃㅉㄸㄲㅆ, 영문 대문자)
- [ ] 언어 전환 (한/영)
- [ ] 백스페이스 (조합 분해 + 일반 삭제)
- [ ] Enter/스페이스 (조합 확정)
- [ ] Fn 키 + 기능 행

### Phase 5: 확장 기능
- [ ] Ctrl 단축키 (복사/붙여넣기/전체선택/실행취소)
- [ ] 클립보드 히스토리
- [ ] 이모지 팝업
- [ ] 스와이프 제스처
- [ ] 추천 단어 (suggestions)

### Phase 6: UX
- [ ] 테마 (Material Dark/Light)
- [ ] 키보드 높이 조절 (세로/가로)
- [ ] 키 팝업 미리보기
- [ ] 진동/소리 피드백
- [ ] 디버그 로그 (설정에서 조회)

### Phase 7: 폴더블/태블릿
- [ ] 커버 화면 키보드 선택 (IME picker)
- [ ] 화면 크기별 레이아웃 자동 전환

## 위임 방식

```cue
delegate: [
    { to: "dev", task: "구체적 파일 + 구체적 작업" },
]
```

dev 완료 후:
```cue
delegate: [
    { to: "verifier", task: "원본 요청: \"...\". 멤버 응답: \"...\". 검증: ..." },
]
```

verifier APPROVED → 다음 단계 즉시 시작. 절대 멈추지 않는다.

## 참고

- 기존 소스: reference/HangulComposer.java
- 해커스 키보드 레포: dalsoop/hackerskeyboard
- 빌드 환경: Rust + Android SDK (컨테이너 내 cargo 사용 가능)
