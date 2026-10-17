# SEQTRAK GUI 디자인 명세 (GUI Design)

[English](DESIGN.md) | 한국어

대상: 로컬 `st serve` 백엔드가 서빙하는 Yamaha SEQTRAK 웹 컨트롤러.

사용자: 책상 위에서 SEQTRAK을 USB로 연결해 두고 브라우저 한 화면에서 트랜스포트, 패턴, 트랙 볼륨, 뮤트/솔로, 아르페지에이터, 이펙트를 바로 조작하려는 연주자 및 개발자.

## 디자인 콘셉트: "두 번째 전면 패널 (Second Faceplate)"

SEQTRAK 본체는 따뜻한 차콜 색상의 섀시 위에 밝은 회색 몰드 패널을 얹고, 촘촘한 실크스크린 인쇄와 노란색·시안색 패턴 LED, 그리고 시그니처 주황색 포인트로 구성된 투톤 악기입니다. 웹 GUI 역시 일반적인 어두운 DAW나 대시보드 형태를 피하고 실제 악기의 두 번째 상판처럼 보이도록 설계했습니다. 각 컨트롤은 실제 물리 노브, 키, 터치 스트립의 형태와 조작감을 그대로 따릅니다.

## 디자인 토큰

| 토큰 | Hex | 역할 |
|---|---|---|
| `chassis` | `#23211E` | 페이지 배경(차콜 본체 색상) |
| `panel` | `#D8D5CE` | 밝은 회색 몰드 패널 영역 |
| `panel-deep` | `#C7C4BC` | 패널 내부의 오목한 컨트롤 슬롯 |
| `ink` | `#211F1B` | 패널 위 기본 실크스크린 인쇄 색상 |
| `ink-soft` | `#6E6A62` | 보조 레이블 및 비활성 표시 |
| `orange` | `#E8590C` | 재생, REC, 활성 탭 등 핵심 강조 색상 |
| `led-yellow` | `#FFC400` | 드럼 트랙군(트랙 1-7) 및 뱅크 1-3 표시 |
| `led-cyan` | `#12C7D6` | 신스·샘플러 트랙군(트랙 8-11) 및 뱅크 4-6 표시 |
| `led-red` | `#E03A2F` | 연결 끊김 및 오류 배너 표시 |

서체는 전 영역에서 영문과 한글을 함께 지원하는 `Noto Sans KR`(`@fontsource/noto-sans-kr`)을 사용하며, 숫자가 흔들리지 않도록 고정폭 숫자(`tabular-nums`)를 적용합니다. 레이블은 굵기 600, 값 표시는 굵기 400을 사용합니다.

## 화면 레이아웃과 뷰 구성

상단 바에서 두 가지 메인 뷰(`DEVICE`, `MIXER`)와 화면별 매뉴얼(`HELP`)을 전환할 수 있습니다:

- **DEVICE 뷰** (`DeviceFace.svelte`): 실제 하드웨어 상판과 측면 버튼을 그대로 옮긴 반응형 SVG(`1560x430` viewBox) 화면입니다. 측면 버튼, 11개 트랙 패턴 노브와 `[ALL]` 노브, 장비 MIDI 클럭에 동기화되는 2x8 드럼 스텝 그리드, `[REC]` 및 7개 스케일 키, 사운드 디자인 노브 4개, `[FX]` 노브, 고정식 `MASTER/SINGLE` 스위치, 터치 슬라이더 3개로 구성됩니다.
- **MIXER 뷰**: 본체의 구역 구분선을 따라 좌측 `DRUMS`(7개 트랙 스트립 + 7개 드럼 패드), 중앙 `SYNTH / SAMPLER`(4개 트랙 스트립 + 아르페지오 블록), 우측 `MASTER`(`ALL` 볼륨 노브, 트랜스포트, 프로젝트 로더 `P1`-`P8`, 수신 MIDI 이벤트 로그)의 3단 구역으로 배치됩니다.

ASCII 와이어프레임 (`MIXER` 뷰):

```text
+-----------------------------------------------------------------------+
| SEQTRAK  OS 2.0  connected  link up            [DEVICE][MIXER][HELP]  |
+----------------------+---------------------------+--------------------+
| DRUMS                | SYNTH / SAMPLER           | MASTER             |
| [kick][snare][clap]  | [synth1][synth2]          |      _____         |
| [hat1][hat2]         | [dx    ][sampler]         |    /  ALL  \       |
| [perc1][perc2]       |  arp: template gate speed |   |  knob   |      |
| 7-pad drum triggers  |  mono/poly  portamento    |    \_______/       |
|                      |                           | [PLAY/PAUSE][STOP] |
|                      |                           | project P1-P8      |
|                      |                           | event log          |
+----------------------+---------------------------+--------------------+
```

## 핵심 디자인 원칙

1. 하드웨어에 없는 장식을 덧붙이지 않습니다. 본체에 노브가 있으면 노브를 그리고, 키가 있으면 키를 그립니다.
2. 수치 표시는 장비 LCD처럼 간결한 고정폭 숫자로 통일합니다.
3. 색상은 장식이 아니라 구역과 상태를 나타냅니다. 노란색은 드럼, 시안색은 신스/샘플러, 주황색은 실시간 변경 및 재생/녹음 상태입니다.
4. 데스크톱(`1440x900`)에서는 스크롤 없이 한 화면에 모두 들어오며, 좁은 화면에서는 구역이 세로로 쌓입니다.
5. 애니메이션은 재생 중 버튼 점멸이나 노트 발음 시 LED 점등처럼 실제 하드웨어 LED 동작에 대응하는 곳에만 사용합니다.

## 컴포넌트 구성

- `DeviceFace` (`src/components/device/`): SVG 하드웨어 판면(`DeviceKnob`, `DeviceKey`, `TouchSlider`), 실제 SysEx 패턴 선택(1-6), MIDI 클럭 스텝 그리드, `REC` + 스케일 키, FX 제어.
- `AllKnob`: 40개 LED 눈금을 가진 SVG 로터리 노브. 드래그하거나 방향키를 누르면 11개 전체 트랙에 볼륨(CC 7)을 브로드캐스트합니다.
- `TrackStrip`: 트랙 이름, 채널 번호, 볼륨/팬 슬라이더, `M`(뮤트), `S`(솔로), `HIT`(청음) 버튼.
- `PadGrid`: 7개 드럼 트랙(`kick`부터 `perc2`)을 즉시 발음하는 7개 패드.
- `Transport`: 시퀀서 재생(`0xFA`)과 정지(`0xFC`)를 제어하는 `PLAY`/`PAUSE` 및 `STOP` 버튼.
- `ArpBlock`: 대상 트랙(`synth1`, `synth2`, `dx`) 선택, 16가지 아르페지오 템플릿, 게이트, 스피드, `mono`/`poly`/`chord` 모드, 포르타멘토 제어.
- `EventLog`: `/api/monitor`에서 받아온 최근 수신 MIDI 메시지 8줄 표시.
- `HelpPanel`: 상단 `HELP` 버튼으로 여는 영/한(`EN` / `KO`) 전환 지원 화면 매뉴얼.

## 문구 규칙

버튼 이름은 본체 실크스크린 인쇄(`PATTERN`, `MUTE`, `SOLO`, `PLAY`, `STOP`)를 그대로 따릅니다. 오류 메시지는 해결 방법을 직접 안내합니다: `"SEQTRAK not connected. Plug in USB and power on; st serve reconnects automatically."`
