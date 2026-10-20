# 개발자 퀵스타트 (Developer Quickstart)

[English](QUICKSTART.md) | 한국어

CLI, AI 에이전트, 웹 GUI에서 Yamaha SEQTRAK을 설정하고 제어하며 장비 패턴 메모리에 작곡하는 가장 빠른 가이드입니다. 상세 프로토콜 사양은 `.claude/skills/seqtrak/SKILL.md`를, 하드웨어 판면 매핑은 `docs/DEVICE_UX.ko.md` ([English](DEVICE_UX.md))를 참고하세요.

## 0. 준비물과 장비 체크

- 필수 환경: Yamaha SEQTRAK(펌웨어 **OS 2.00 이상**) 및 데이터 전송용 USB 케이블(소스에서 직접 빌드할 경우 Rust 툴체인과 Node.js 추가 필요). 모든 기능은 **OS 2.00** 기준으로 구현 및 테스트되었으므로 다른 버전에서는 일부 또는 전체가 동작하지 않을 수 있습니다. 다른 펌웨어 버전에서 문제를 겪으시면 정확한 증상이 포함된 [이슈](https://github.com/epicsagas/seqtrak-agent/issues)나 직접 수정한 [PR](../CONTRIBUTING.ko.md)을 남겨 주시면 검토하여 반영하겠습니다.
- 장비 전원을 켜고 USB로 연결합니다. `st list`를 실행했을 때 출력과 입력 양쪽에 `SEQTRAK` 포트가 보여야 합니다.
  - 포트가 보이지 않으면 자동 전원 꺼짐 기능이 작동한 상태일 수 있습니다. 전원을 다시 켜고 재시도하세요.
- 최초 1회, 장비가 USB로 SysEx 응답을 보내도록 설정합니다:
  ```bash
  st param 00:00:10 01      # SYSTEM General: USB MIDI Transmit SysEx = On
  ```
- 장비의 MIDI Sync 설정을 `External` 또는 `Auto`로 맞춥니다. `Internal`로 되어 있으면 트랜스포트 제어와 `rec-steps` 클럭 동기가 동작하지 않습니다.
- 본체 녹음 대기(`REC` 활성화)는 장비 상판의 물리 `REC` 버튼으로만 켤 수 있습니다(SysEx 명령 없음).

## 1. 설치 및 빌드

### 릴리즈 바이너리 바로 설치 (Rust 설치 불필요)

```bash
# macOS 및 Linux
curl -fsSL https://raw.githubusercontent.com/epicsagas/seqtrak-agent/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/epicsagas/seqtrak-agent/main/install.ps1 | iex
```

### 소스에서 직접 빌드 (로컬 개발 시)

```bash
cargo build --release
cargo install --path . --locked                   # st, st-mcp를 PATH에 설치
cd gui && npm install && npm run build && cd ..   # GUI 정적 번들 생성 (gui/dist)
st serve &                                        # http://127.0.0.1:8787 서빙
```

- `st serve`는 장비가 꺼져 있거나 연결되지 않아도 먼저 실행되며, 장비가 켜지면 자동으로 연결합니다. 케이블 재연결 후 macOS CoreMIDI 클라이언트가 응답하지 않으면 `MIDIRestart`를 호출해 복구합니다.
- CLI나 MCP 코드를 수정한 뒤에는 `cargo install --path . --locked`를 다시 실행해 설치된 바이너리를 갱신하세요.

## 2. AI 에이전트 연결 (MCP + 스킬)

### 에이전트에게 바로 붙여 넣는 설치 프롬프트

사용 중인 AI 에이전트(Claude Code, Codex CLI, Grok, Antigravity, Cursor 등)에 아래 문장을 복사해 붙여 넣으면 에이전트가 저장소를 읽고 자신의 환경에 맞게 알아서 설치합니다:

```text
https://github.com/epicsagas/seqtrak-agent 저장소를 읽고, 지금 내가 쓰는 에이전트 환경에 맞게 CLI(st), MCP 서버(st-mcp), 스킬(.claude/skills/seqtrak)을 설치해 줘.
```

### Claude Code 및 Codex CLI 명령어

```bash
# Claude Code
claude mcp add seqtrak --scope user -- "$(command -v st-mcp)"
mkdir -p ~/.claude/skills && cp -r .claude/skills/seqtrak ~/.claude/skills/

# OpenAI Codex CLI
codex mcp add seqtrak -- "$(command -v st-mcp)"
mkdir -p ~/.codex/skills && cp -r .claude/skills/seqtrak ~/.codex/skills/
```

### MCP 설정 파일 직접 추가 (JSON / TOML)

JSON (`.mcp.json`, Claude Desktop, Grok, Antigravity, Cursor 등):

```json
{
  "mcpServers": {
    "seqtrak": {
      "command": "st-mcp",
      "args": []
    }
  }
}
```

TOML (Codex CLI `~/.codex/config.toml` 등):

```toml
[mcp_servers.seqtrak]
command = "st-mcp"
args = []
```

- `st-mcp`는 작곡용 `st_rec_steps`를 포함해 총 15개 도구를 노출합니다.
- 셸에서 MCP 서버 응답을 바로 확인할 수 있습니다:
  ```bash
  printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | st-mcp
  ```

## 3. 작곡 루프 (장비 패턴 메모리에 기록)

SysEx로는 스텝 노트를 직접 쓸 수 없으므로 장비의 실시간 녹음기를 통해 패턴을 기록합니다:

```bash
st load 1                                  # 대상 프로젝트 로드
# 장비 본체 패널에서 물리 REC 버튼 누르기
st rec-steps kick   --steps 1,5,9,13 --keep-playing
st rec-steps snare  --steps 5,13 --vel 90 --keep-playing
st rec-steps synth1 --steps 1,5,9,13 --note 60 --gate 2    # 마지막 테이크만 정지
```

- 노트는 장비가 내보내는 MIDI 클럭(96 PPQ, 16분음표 = 6클럭)에 맞춰 발사되며, 채널별로 해당 트랙의 패턴에 기록됩니다(`kick` = 채널 1).
- 트랜스포트 정지(`0xFC`)가 전송되면 녹음 세션도 함께 종료됩니다. 여러 트랙을 이어서 녹음할 때는 마지막 테이크 전까지 반드시 `--keep-playing`을 붙여야 합니다.
- 녹음 결과 검증: `st transport start` 실행 후 `st monitor --timeout 3`으로 장비가 재생하며 내보내는 `note_on` 이벤트를 확인합니다.
- 프로젝트 저장: 전원을 끄거나 다른 프로젝트로 전환할 때 자동으로 저장됩니다(유저 가이드 4.2, 수동 WRITE 키 없음).

## 4. 트랙, 사운드, 이펙트 제어 요약

- 11개 고정 트랙(채널 1부터 11): `kick`, `snare`, `clap`, `hat1`, `hat2`, `perc1`, `perc2`, `synth1`, `synth2`, `dx`, `sampler`.
- 트랙별 패턴 선택(1부터 6): `st param 30:5p:0F <0-5>` (`p` = 파트 0부터 10, `kick`은 `30:50:0F`).
- 패턴 길이 변경(1부터 128스텝): `st param 30:5p:16 <MSB> <LSB>` (16스텝 = `00 10`).
- 주요 CC 이름: `volume`, `pan`, `mute`, `solo`, `filter_cutoff`, `filter_resonance`, `eg_attack`, `eg_decay_release`, `reverb_send`, `delay_send`, `fx_master_1..3`, `fx_single_1..3`, `arp_template`, `arp_gate`, `arp_speed`, `drum_pitch`, `fm_algorithm`, `fm_mod_amount`, `fm_mod_frequency`, `fm_mod_feedback`.
- 전체 블록 주소 지도: `.claude/skills/seqtrak/SKILL.md`의 "SysEx address map" 섹션 참고.
- 프로젝트 템포: `st get` / `st param 30:40:76` (2바이트 MSB/LSB, `00 78` = 120 BPM). 읽기·쓰기 실기 검증 완료.

## 5. 웹 GUI 개발 (프로토타입)

> **참고:** 웹 GUI는 현재 프로토타입 상태이므로 일부 동작이 정확하지 않을 수 있습니다.

```bash
cd gui && npm run check && npm run build   # svelte-check + vite build (gui/dist 생성)
st serve                                   # 정적 파일 서빙 + /api/*
```

- 2개 화면 제공: `DEVICE`(하드웨어 판면 재현 및 클럭 동기 스텝 시퀀서)와 `MIXER`(실시간 공연 및 믹싱용 스트립 뷰).
- 스텝 레이어 API: `GET /api/seq`, `POST /api/seq {op: step|rec|clear|note}`. GUI 레이어에서 동작하며 장비 내부 패턴 메모리를 덮어쓰지 않습니다.
- 상단 바의 `HELP` 버튼을 누르면 화면별 사용 설명서를 영문과 한국어로 볼 수 있습니다.

### 브라우저 E2E 테스트 (`browser-harness`)

1. `st serve &` 실행 후 Chrome에서 `http://127.0.0.1:8787`을 엽니다.
2. SVG 컨트롤 조작은 합성 `PointerEvent`(`pointerdown` 후 `pointerup`) 디스패치 방식이 안정적입니다.
3. 상태 검증은 `/api/seq`, `/api/param?addr=...`, `/api/health` API 응답으로 확인합니다.
4. 발음 검증은 `st monitor --timeout 3`으로 수신 노트를 카운트합니다.

## 6. 코드 지도와 기능 확장

```text
src/model.rs     채널, CC, 뱅크, 주소 상수
src/sysex.rs     SysEx 조립/파싱, 체크섬, 버전 포맷
src/midi.rs      midir 래퍼, 덤프 스티칭, 클럭 포트, MIDIRestart
src/recsteps.rs  클럭 동기 스텝 기록 (rec-steps)
src/serve.rs     axum 백엔드 및 스텝 시퀀서
src/cli.rs       st 서브커맨드
src/mcp.rs       st-mcp JSON-RPC 서버
gui/             Svelte 5 + TypeScript 프론트엔드
```

새 명령 추가 순서: `src/cli.rs`의 `Command` 열거형에 추가한 뒤 `run()`에서 분기 처리하고, `src/mcp.rs`에 `Tool` 정의와 디스패치를 추가한 다음, `.claude/skills/seqtrak/SKILL.md`에 반영하고 마지막으로 `cargo install --path . --locked`를 실행합니다.

## 7. 자주 겪는 문제

- `st get`이나 `st dump`가 빈 값을 반환함: USB SysEx 송신 스위치(`st param 00:00:10 01`)가 켜져 있는지 확인하세요.
- `rec-steps`에서 `no MIDI clock` 오류 발생: 장비의 MIDI Sync 설정이 `Internal`입니다. `External` 또는 `Auto`로 변경하세요.
- `st list`에서 장비가 사라짐: 자동 전원 꺼짐이 작동한 상태이므로 본체 전원을 다시 켜세요.
- 프로젝트 전환 시 미완성 테이크 저장 주의: `st load N`으로 프로젝트를 바꾸면 현재 라이브 버퍼 상태가 자동 저장됩니다. 패턴 녹음을 마친 뒤 프로젝트를 전환하세요.
