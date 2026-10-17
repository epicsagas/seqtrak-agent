# seqtrak agent

[English](README.md) | 한국어

터미널, AI 에이전트, 웹 브라우저에서 Yamaha SEQTRAK을 제어하기 위한 도구 모음입니다. Rust CLI(`st`), MCP 서버(`st-mcp`, 14개 도구), Claude 스킬, 그리고 실제 하드웨어 상판을 그대로 옮긴 웹 GUI(`st serve`)로 구성되어 있습니다. 장비의 MIDI 클럭에 맞춰 돌아가는 스텝 시퀀서와 장비 내부 패턴 메모리에 직접 노트를 새기는 녹음 기능을 포함합니다.

별도 드라이버 없이 표준 USB MIDI(노트, CC, 뱅크/프로그램 체인지, Yamaha SysEx 파라미터 변경 및 벌크 덤프)로 동작합니다.

> **펌웨어 호환성 안내 (OS 2.00 이상):** 이 프로젝트는 Yamaha SEQTRAK 펌웨어 **OS 2.00 이상**에서 동작하며, **OS 2.00**(SEQTRAK Data List D0) 기준으로 구현하고 실제 장비에서 테스트했습니다. 다른 펌웨어 버전에서는 일부 또는 전체 기능이 정상 동작하지 않을 수 있습니다. 버전 차이로 인한 문제를 발견하시면 정확한 증상(`st identity` 출력 포함)을 적어 [이슈](https://github.com/epicsagas/seqtrak-agent/issues)로 제보해 주시거나, 직접 해결한 [PR](CONTRIBUTING.ko.md)을 올려 주시면 검토 후 반영하겠습니다.

## 설치

### 릴리즈 바이너리 바로 설치 (macOS, Linux, Windows)

Rust나 Cargo를 설치하지 않고 최신 릴리즈(`st`, `st-mcp`, `seqtrak` 스킬)를 OS에 맞게 바로 설치할 수 있습니다:

```bash
# macOS 및 Linux
curl -fsSL https://raw.githubusercontent.com/epicsagas/seqtrak-agent/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/epicsagas/seqtrak-agent/main/install.ps1 | iex
```

### 소스에서 직접 빌드

```bash
cargo build --release            # target/release/st, target/release/st-mcp 빌드
cargo install --path . --locked  # st, st-mcp를 PATH에 설치
```

## AI 에이전트 연결

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

JSON 설정 파일(`.mcp.json`, Claude Desktop, Grok, Antigravity, Cursor 등)을 사용하는 경우:

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

TOML 설정 파일(Codex CLI `~/.codex/config.toml` 등)을 사용하는 경우:

```toml
[mcp_servers.seqtrak]
command = "st-mcp"
args = []
```

## 장비 패턴 메모리에 작곡하기

SEQTRAK은 SysEx로 스텝 노트를 직접 쓰는 명령을 제공하지 않습니다. 대신 본체 패널에서 `REC` 버튼을 켜 두면 USB MIDI로 들어오는 노트를 실시간 녹음기가 받아 해당 트랙 패턴에 기록합니다. 패널에서 `REC`를 켠 뒤 아래처럼 실행합니다:

```bash
st rec-steps kick   --steps 1,5,9,13 --keep-playing
st rec-steps snare  --steps 5,13 --vel 90 --keep-playing
st rec-steps synth1 --steps 1,5,9,13 --note 60 --gate 2   # 마지막 테이크에서 트랜스포트 정지
```

`st rec-steps`는 장비가 내보내는 96 PPQ MIDI 클럭(6클럭 = 16분음표)에 맞춰 정확한 타이밍에 채널별 노트를 쏩니다. 녹음된 프로젝트는 전원을 끄거나 다른 프로젝트로 전환할 때 자동으로 저장됩니다(유저 가이드 4.2).

전체 워크플로우와 SysEx 주소 지도는 [`.claude/skills/seqtrak/SKILL.md`](.claude/skills/seqtrak/SKILL.md)를, 초기 장비 설정과 단계별 가이드는 [개발자 퀵스타트 (`docs/QUICKSTART.ko.md`)](docs/QUICKSTART.ko.md) ([English](docs/QUICKSTART.md))를 참고하세요.

## 웹 GUI (프로토타입)

> **참고:** 웹 GUI는 현재 프로토타입 상태이므로 일부 조작이나 상태 동기화가 정확히 동작하지 않을 수 있습니다. 안정적인 제어와 패턴 녹음은 `st` CLI 또는 `st-mcp` 사용을 권장합니다.

```bash
st serve        # http://127.0.0.1:8787 에서 빌드된 웹 GUI 서빙
```

`st serve`는 GUI 파일을 `$ST_GUI_DIR`, `./gui/dist`(소스 체크아웃), `st` 바이너리 옆의 `gui/`(릴리스 압축 해제), `~/.local/share/seqtrak-agent/gui`(`install.sh` / `install.ps1` 설치 위치) 순서로 찾습니다.

웹 GUI는 실행 시 Google Fonts에서 Noto Sans KR 폰트를 불러옵니다(폰트 파일은 포함하지 않습니다). 오프라인에서는 시스템 폰트로 대체됩니다.

- **DEVICE 뷰**: 실제 하드웨어 판면을 재현한 화면입니다. 트랙 노브와 `[ALL]` 노브는 SysEx(`30:5p:0F`)를 통해 실제로 패턴 1부터 6을 전환하고, 드럼 키는 2x8 그리드에서 스텝을 켜고 끕니다. 재생 중 `[REC]`와 7개 스케일 키로 실시간 입력을 할 수 있고, `[FX]` 노브로 이펙트 종류를 바꾸며, `MASTER/SINGLE`은 고정식 슬라이드 스위치로, `HIGH PASS`와 `REPEATER`는 손을 떼면 0으로 돌아가는 스프링 슬라이더로 동작합니다.
- **MIXER 뷰**: 11개 트랙 스트립(볼륨, 팬, 뮤트, 솔로, 청음), 드럼 패드, 아르페지오 블록, 트랜스포트, 프로젝트 로더(P1부터 P8), 수신 MIDI 이벤트 로그를 한 화면에 모았습니다.
- **자동 재연결**: 장비가 연결되지 않은 상태에서도 서버가 먼저 뜨며, 백그라운드에서 연결을 감지합니다. 케이블 재연결 후 macOS CoreMIDI 클라이언트가 멈추면 `MIDIRestart`를 호출해 복구합니다.

GUI 스텝 시퀀서 API는 `GET /api/seq`, `POST /api/seq`(`step`, `rec`, `clear`, `note`)입니다. 자세한 내용은 [하드웨어 판면 UX (`docs/DEVICE_UX.ko.md`)](docs/DEVICE_UX.ko.md) ([English](docs/DEVICE_UX.md)), [디자인 명세 (`docs/DESIGN.ko.md`)](docs/DESIGN.ko.md) ([English](docs/DESIGN.md)), [GUI 아키텍처 (`docs/GUI_PLAN.ko.md`)](docs/GUI_PLAN.ko.md) ([English](docs/GUI_PLAN.md))를 참고하세요.

## 빠른 실행 예시 및 CLI 상세 가이드

장비 초기 설정과 전체 작곡 흐름은 [개발자 퀵스타트 (`docs/QUICKSTART.ko.md`)](docs/QUICKSTART.ko.md) ([English](docs/QUICKSTART.md))에서, 모든 서브커맨드와 옵션, CC 이름 표, 뱅크 및 SysEx 주소는 [CLI 상세 가이드 (`docs/CLI.ko.md`)](docs/CLI.ko.md) ([English](docs/CLI.md))에서 확인할 수 있습니다.

```bash
st identity                    # SEQTRAK 연결 및 펌웨어 버전 확인
st note -c kick -n 36          # 킥 1회 발음
st cc -c synth1 -n volume -v 100
st load 2                      # 프로젝트 2 로드
st param 30:50:0F 02           # 킥 트랙을 패턴 3으로 변경
st transport start             # 시퀀서 재생 (장비 MIDI Sync = External/Auto)
st snapshot --project 2        # 전체 파라미터 상태를 TSV로 캡처
st dump 11:00:00 -o project.hex
```

## 디렉터리 구조

```text
src/model.rs     트랙 채널, CC 테이블, 뱅크, SysEx 주소 상수
src/sysex.rs     SysEx 메시지 조립/파싱, 체크섬, 버전 포맷
src/midi.rs      midir 래퍼, 벌크 덤프 스티칭, 클럭 포트, MIDIRestart
src/recsteps.rs  장비 클럭 동기식 스텝 녹음
src/serve.rs     axum 웹 서버 및 MIDI 클럭 스텝 시퀀서
src/cli.rs       st CLI 서브커맨드
src/mcp.rs       st-mcp 표준 입출력 JSON-RPC MCP 서버
gui/             Svelte 5 + TypeScript 웹 컨트롤러
docs/            퀵스타트, CLI 레퍼런스, UX/디자인 문서, 리뷰 보고서
```

## 테스트

```bash
cargo test && (cd gui && npm run check && npm run build)
```

## 상표 고지

Yamaha 및 SEQTRAK은 Yamaha Corporation의 상표 또는 등록상표입니다. 이 프로젝트는 독립적인 비공식 오픈소스 도구이며, Yamaha Corporation과 제휴하거나 Yamaha Corporation의 승인 또는 후원을 받지 않았습니다.
