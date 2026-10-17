# SEQTRAK 웹 GUI (`gui/`)

[English](README.md) | 한국어

`st serve`(`http://127.0.0.1:8787`)가 서빙하는 Svelte 5, TypeScript, Vite 기반 웹 컨트롤러입니다.

> **참고:** 이 웹 GUI는 현재 프로토타입 상태이며 일부 기능이 정확히 동작하지 않을 수 있습니다. 정확한 장비 제어는 `st` CLI 또는 `st-mcp`를 사용하세요.

## 화면 구성

- **DEVICE** (`src/components/device/DeviceFace.svelte`): 실제 하드웨어 판면을 재현한 SVG 화면입니다. 트랙 및 전체 패턴 선택 노브(SysEx `30:5p:0F`), 장비 MIDI 클럭에 동기화되는 2x8 드럼 스텝 그리드, `[REC]` 및 7개 스케일 키, 사운드 디자인 노브, `[FX]` 타입 로터리, 고정식 `MASTER/SINGLE` 스위치, 순간 작동식 `HIGH PASS` 및 `REPEATER` 터치 슬라이더를 제공합니다.
- **MIXER** (`src/App.svelte`): 11개 트랙 스트립(`TrackStrip.svelte`), 7개 드럼 패드(`PadGrid.svelte`), 아르페지오 및 포르타멘토 블록(`ArpBlock.svelte`), 마스터 볼륨 `AllKnob.svelte`, `Transport.svelte`, 프로젝트 로더(P1-P8), 실시간 MIDI 수신 로그(`EventLog.svelte`)를 한 화면에 배치했습니다.
- **HELP** (`src/components/HelpPanel.svelte`): 화면별 사용 설명서를 영문(`EN`)과 한국어(`KO`)로 전환하며 볼 수 있는 오버레이 패널입니다.

## 개발 및 빌드

```bash
npm install
npm run check   # svelte-check 및 tsc 타입 검사
npm run build   # gui/dist/에 정적 번들 생성
```

빌드 후 저장소 루트에서 서버를 실행합니다:

```bash
st serve        # http://127.0.0.1:8787 에서 gui/dist/ 및 /api/* 서빙
```

## 관련 문서

- 하드웨어 판면 매핑 및 UX: `../docs/DEVICE_UX.ko.md` ([English](../docs/DEVICE_UX.md))
- 디자인 토큰 및 레이아웃: `../docs/DESIGN.ko.md` ([English](../docs/DESIGN.md))
- 백엔드 API 명세: `../docs/GUI_PLAN.ko.md` ([English](../docs/GUI_PLAN.md))
