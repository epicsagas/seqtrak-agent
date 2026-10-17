# seqtrak agent 기여 가이드

[English](CONTRIBUTING.md) | 한국어

`seqtrak agent` 개선에 참여해 주셔서 감사합니다. 이 문서는 로컬 개발 환경 설정, 테스트, 실기 검증 기준, 그리고 PR 작성 규칙을 안내합니다.

## 로컬 개발 환경

1. Rust 안정 버전 툴체인(`rustup`)과 Node.js(`npm`)를 설치합니다. Ubuntu/Debian 리눅스에서는 ALSA MIDI 빌드를 위해 `pkg-config`와 `libasound2-dev` 패키지가 필요합니다.
2. Rust 빌드 및 테스트:
   ```bash
   cargo build
   cargo test
   cargo clippy --all-targets
   ```
3. 웹 GUI 타입 검사 및 번들 빌드(`gui/` 수정 시):
   ```bash
   cd gui
   npm install
   npm run check
   npm run build
   ```

## 새 CLI 명령 또는 MCP 도구 추가 순서

장비 제어 기능을 새로 추가할 때는 CLI, MCP 서버, 스킬 문서를 함께 맞춰 주세요:

1. `src/model.rs`에 채널, CC, 뱅크, 또는 SysEx 주소 상수를 정의합니다.
2. `src/cli.rs`의 `Command` 열거형에 서브커맨드를 추가하고 `run()`에서 처리합니다.
3. `src/mcp.rs`에 대응하는 MCP 도구 스키마와 핸들러를 등록합니다.
4. `.claude/skills/seqtrak/SKILL.md`, `README.md`, `README.ko.md`에 사용법과 검증된 주소를 기록합니다.

## 하드웨어 실기 검증

- 새로운 SysEx 주소나 MIDI 동작은 가능한 한 실제 Yamaha SEQTRAK(펌웨어 OS 2.00)에서 검증하고, SEQTRAK Data List D0의 해당 페이지나 절을 함께 명시해 주세요.
- 실제 장비 없이 단위 테스트로만 검증한 변경 사항이라면 PR 본문에 그 사실을 명확히 적어 주세요.

## PR 체크리스트

- `cargo test` 및 `cargo clippy --all-targets`가 경고 없이 통과해야 합니다.
- `gui/`를 수정한 경우 `cd gui && npm run check && npm run build`가 경고 및 오류 없이 통과해야 합니다.
- 커밋 메시지는 Conventional Commits(`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `ci:`, `chore:`) 형식을 따릅니다.
- 사용자용 문서에 영향을 주는 변경은 영문(`.md`)과 한국어(`.ko.md`) 문서를 함께 갱신합니다.

## 라이선스

이 저장소에 기여된 코드는 [Apache License, Version 2.0](LICENSE)에 따라 배포되는 것에 동의한 것으로 간주됩니다.
