# unknown Wisdom

실패와 성공에서 얻은 교훈. 같은 실수를 반복하지 않기 위해 모든 dal 이
Pre-Flight 에서 반드시 읽는다. **직접 수정 금지** — 제안은
`.dal/wisdom-inbox/` 에 드롭.

---

## Anti-Patterns (하지 말 것)

### 예시: leader 가 파일 직접 수정

leader 는 라우터. `/workspace` 가 read-only 로 마운트되어 있으므로 시도해도
OS 가 거부한다. 수정은 반드시 멤버에게 위임.

(팀이 운영되며 실제 실수를 여기에 기록하세요.)

---

## Patterns (계속 할 것)

### 1. Pre-Flight ritual

모든 leader 세션 시작 시:
1. `.dal/decisions.md` 읽기
2. `.dal/wisdom.md` 읽기
3. Response Mode 판단 (Direct / Single / Multi)

### 2. cue block delegation

leader 는 응답 마지막에 cue block 으로만 위임:

```cue
delegate: [
    { to: "member-name", task: "구체적 한 줄 지시" },
]
```

---

## Inbox

`.dal/wisdom-inbox/{name}-{date}-{slug}.md` 형식으로 제안 드롭.
