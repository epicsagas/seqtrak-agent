# seqtrak agent

[English](README.md) | 한국어

터미널, AI 에이전트, 웹 브라우저에서 Yamaha SEQTRAK을 제어하기 위한 도구 모음입니다. Rust CLI(`st`), MCP 서버(`st-mcp`, 15개 도구), Claude 스킬, 그리고 실제 하드웨어 상판을 그대로 옮긴 웹 GUI(`st serve`)로 구성되어 있습니다. 장비의 MIDI 클럭에 맞춰 돌아가는 스텝 시퀀서와 장비 내부 패턴 메모리에 직접 노트를 새기는 녹음 기능을 포함합니다.

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

### Homebrew (macOS 및 Linux)

```bash
brew tap epicsagas/tap
brew install seqtrak-agent
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

## MCP 활용: 한 음에서 전체 편곡까지

MCP 서버(`st-mcp`)는 `st` CLI와 동일한 15개 도구를 제공하므로, 에이전트는 한 줄 연결 테스트에서 시작해 대화를 벗어나지 않고 여러 마디·여러 트랙짜리 편곡까지 진행할 수 있습니다. 아래 예시는 모두 실제 도구 호출이며, 에이전트가 이를 계획으로 조합하면 장비가 그 결과를 녹음합니다.

### 사용자는 말로, 에이전트가 연주

실제로는 도구를 직접 호출하지 않습니다. 원하는 결과를 자연어로 말하면 에이전트가 도구를 고르고, 테이크를 계획하고, 장비를 구동하고, 실제로 착지한 것까지 검증합니다:

- "트랙 재생해줘", "킥 한 대 맞아줘": `st_note` 한 번
- "키트 바꿔줘", "synth2에 따뜻한 패드 얹어줘": `st_pc`, 필요하면 `st_cc`
- "리버브 좀 줄여줘": `reverb_send`로 `st_cc` 한 번
- "비트 하나 만들어줘": 프로젝트 로드, 패턴 길이 설정, `st_rec_steps` 테이크, 재생 검증
- "Thriller 느낌으로 3마디 작곡해줘": 아래 레벨 5와 같은 전체 세션. 에이전트가 트랙별로 계획하고, 녹음하고, 피치별 재생 이벤트 수를 세어 부족분만 재발사합니다

[seqtrak 스킬](.claude/skills/seqtrak/SKILL.md)이 에이전트에게 기기 모델, 검증된 SysEx 주소 지도, 펌웨어 특이사항을 알려주므로 자연어 경로가 안전하게 유지됩니다: 에이전트가 계획하고, 사용자가 듣고, 마음에 드는 것만 남깁니다.

### 15개 도구

| 도구 | 하는 일 |
|------|---------|
| `st_list_ports` | MIDI 포트 목록. SEQTRAK 포트명에는 `SEQTRAK` 포함 |
| `st_identity` | 기기 패밀리·펌웨어 버전 조회 |
| `st_note` | 트랙에 한 음 재생 |
| `st_cc` | 이름 또는 번호로 컨트롤 체인지: 볼륨, 팬, 뮤트, 솔로, 필터, EG, FX 센드, 드럼 피치, 아르페지에이터 |
| `st_pc` | 뱅크 셀렉트 + 프로그램 체인지: 프로젝트, 프리셋·유저 사운드, 드럼 키트, 샘플러 엘리먼트 |
| `st_load_project` | 프로젝트 1~8 로드 |
| `st_get` | SysEx 파라미터 하나 읽기 |
| `st_param` | SysEx 파라미터 하나 쓰기, 체크섬 자동 |
| `st_sysex` | 원시 SysEx 헥스 문자열 전송 |
| `st_dump` | 메모리 블록 벌크 덤프 |
| `st_bulk` | 벌크 덤프 블록을 장비로 재전송(복원; 시스템 블록으로 실기 검증) |
| `st_snapshot` | 프로젝트 로드 후 전체 파라미터 에코를 TSV로 캡처 |
| `st_monitor` | N초간 수신 MIDI 청취, 클럭 플러드 필터링 |
| `st_transport` | 시퀀서 시작·정지 |
| `st_rec_steps` | 실시간 레코더를 통한 트랙 패턴 스텝 녹음 |

### 레벨 1: 장비 확인, 한 음 재생

연결과 펌웨어를 확인하고 단일 노트를 트리거한 뒤 MIDI 스트림을 관찰합니다:

```text
st_identity    {}
st_note        {"track": "kick", "note": 36, "vel": 110, "dur_ms": 150}
st_monitor     {"seconds": 5}
```

### 레벨 2: 사운드와 믹서

프로젝트를 로드하고, 트랙에 프리셋 사운드를 얹고, 믹싱합니다:

```text
st_load_project {"project": 4}
st_pc           {"msb": 63, "lsb": 0, "prog": 5, "track": "synth1"}
st_cc           {"track": "synth1", "cc": "volume", "val": 100}
st_cc           {"track": "synth1", "cc": "reverb_send", "val": 90}
```

### 레벨 3: SysEx 딥 파라미터

문서화된 모든 파라미터를 읽고 쓸 수 있습니다: 패턴 길이, 이펙트 타입, 센드 레벨, 아르페지에이터, 스케일. 킥 패턴을 48스텝으로 늘리고 확인:

```text
st_get         {"address": "30:50:16"}                  # 패턴 1 길이 읽기
st_param       {"address": "30:50:16", "data": [0, 48]} # MSB+LSB = 48스텝 쓰기
```

### 레벨 4: 트랜스포트와 스텝 녹음

본체 패널에서 REC를 무장합니다(이것만 SysEx가 없음). 그러면 에이전트가 장비 자체 클럭 그리드 위에 스텝을 녹음합니다:

```text
st_rec_steps   {"track": "kick", "steps": "1,7,9,13", "vel": 110, "keep_playing": true}
st_rec_steps   {"track": "snare", "steps": "5,13", "vel": 100}
st_transport   {"action": "start"}
```

재생 중 `st_monitor`로 녹음된 노트가 돌아오는 것을 보면 테이크 착지 여부를 검증할 수 있습니다.

### 레벨 5: 통편곡

실제 세션 사례: 프로젝트 5에 녹음한 3마디 7트랙 그루브(드럼, 크로매틱 베이스 라인, 코드 스탭, FM 리드)입니다. 드럼은 트랙마다 단일 피치이므로 호출 1번으로 끝납니다:

```text
st_rec_steps   {"track": "kick",   "bars": 3, "steps": "1,7,9,13,17,23,25,29,33,39,41,45", "vel": 110, "keep_playing": true}
st_rec_steps   {"track": "snare",  "bars": 3, "steps": "5,13,21,29,37,45,46,47", "vel": 100, "keep_playing": true}
st_rec_steps   {"track": "hat1",   "bars": 3, "steps": "1,3,5,7,9,11,13,15,17,19,21,23,25,27,29,31,33,35,37,39,41,43,45,47", "vel": 70, "keep_playing": true}
st_rec_steps   {"track": "dx",     "bars": 3, "steps": "33", "note": 73, "gate": 2, "vel": 90, "keep_playing": true}
```

멜로디 트랙은 여러 피치가 필요하므로, CLI가 한 줄 전체를 클럭 동기 스윕 한 번으로 쏩니다(멀티 피치 라인은 `st_rec_steps` 호출 하나로 표현할 수 없음):

```bash
st seq synth1 --bars 3 --gate 2 --follow --anchor-ch 10 --anchor-note 73 --anchor-step 33 --loop-steps 48 \
  --events "1:42,3:42,5:47,7:47,9:46,11:46,13:45,15:44,17:42,19:42,21:40,23:40,25:42,27:42,29:44,31:42,33:49,35:49,37:47,39:47,41:46,43:46,45:44,47:42"
```

`--follow`는 장비가 스스로 재생하는 노트(여기서는 스텝 33의 DX C#5 하나뿐인 피치)에 스윕을 고정하므로, 녹음 중 에이전트가 트랜스포트를 건드리지 않습니다. 이후 플랜 대비 피치별 재생 이벤트 수를 세어 결과를 검증하고, 부족한 노트만 재발사합니다. 전체 워크플로우, 검증된 SysEx 주소 지도, 펌웨어 특이사항은 [`.claude/skills/seqtrak/SKILL.md`](.claude/skills/seqtrak/SKILL.md)에 있습니다.

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
