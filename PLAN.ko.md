# SEQTRAK 제어 프로젝트 계획 (Project Plan)

[English](PLAN.md) | 한국어

목표: USB MIDI와 SysEx를 통해 Yamaha SEQTRAK의 모든 기능을 제어하고 장비 내부 패턴 메모리에 작곡까지 수행하는 Rust CLI, MCP 서버, 에이전트 스킬, 웹 GUI를 구축합니다. 아래의 모든 마일스톤은 구현 및 실기 검증이 완료되었습니다.

## 하드웨어 검증 결과

### 장비 기본 특성
- USB: Yamaha VID `0x0499`, PID `0x174C`, 표준 USB MIDI 장치(별도 드라이버 불필요).
- 펌웨어 기준: Yamaha가 배포하는 공식 SEQTRAK Data List for OS V2.00 (YJ-D0, 01/2026)입니다. 매뉴얼은 이 저장소에 포함되어 있지 않습니다.
- MIDI 클럭: 재생 중 96 PPQ 타이밍 클럭을 전송합니다(16분음표당 6클럭, MIDI Clock Out 기본값 On).
- 전원 동작: 자동 전원 꺼짐이 작동하면 USB 장치가 완전히 사라지며, `st serve`는 장치가 다시 켜질 때 자동으로 재연결합니다.

### 고정 MIDI 채널
채널 1부터 7은 드럼(`kick`, `snare`, `clap`, `hat1`, `hat2`, `perc1`, `perc2`), 채널 8과 9는 AWM2 신스(`synth1`, `synth2`), 채널 10은 4오퍼레이터 FM 신스(`dx`), 채널 11은 샘플러(`sampler`)로 고정되어 있습니다.

### SysEx 메시지 구조 (Model ID `1C 0C`)
- 파라미터 변경(Parameter Change): `F0 43 1n 7F 1C 0C ah am al dd.. F7`
- 파라미터 조회(Parameter Request): `F0 43 3n 7F 1C 0C ah am al F7`
- 덤프 요청(Dump Request): `F0 43 2n 7F 1C 0C ah am al F7`
- 벌크 덤프(Bulk Dump): `F0 43 0n 7F 1C bh bl 0C ah am al dd.. cc F7` (체크섬 범위는 Model ID부터 마지막 데이터 바이트까지이며 디바이스 번호 `n = 0`).

### 주요 SysEx 주소 (전체 표는 `.claude/skills/seqtrak/SKILL.md` 참고)
- 시스템(`00 00 00`): 오프셋 `10` = SysEx 송신 스위치, `2D` = Clock Out, `2F` = 패턴 선택 감도.
- 프로젝트 공통(`30 40 00`): SysEx 조회 시 항상 0을 반환하며 템포는 SysEx로 읽거나 쓸 수 없습니다.
- 센드 및 마스터 이펙트: Send Reverb `30 41`, Send Delay `30 42`, Master FX 1-4 `30 43`-`30 46`, Master EQ `30 47`.
- 오디오 입력 및 스케일: A/D `30 49`-`30 4B`, USB Audio `30 4C`, Scale `30 4D`.
- 프로젝트 트랙(`30 5p 00`): `00` 볼륨, `01` 팬, `0F` 패턴 선택(0부터 5가 패턴 1부터 6에 대응), `16..` 패턴 길이.
- 사운드(`31 0p` 이름, `31 1p` 일반, `31 2p`/`31 3p` 인서션, `31 4p` 공통) 및 아르페지에이터(`31 5p`, 신스 파트 전용).
- AWM 엘리먼트(`41 ep`, `42 ep`) 및 샘플러(`48 09 00` 공통, `50 eA 00` 7개 샘플 슬롯).

### 주요 Control Change 번호 (채널 1-11)
볼륨 7, 팬 10, 뮤트 23, 솔로 24, 드럼 피치 25, 모노/폴리/코드 26, 아르페지오 템플릿 27, 게이트 28, 스피드 29, EQ 하이/로우 20/21, 컷오프 74, 레조넌스 71, 어택 73, 디케이 75, 리버브/딜레이 센드 91/94, 할당 FX 102-109, FM 116-119.

### 장비 패턴 메모리에 작곡하는 경로
문서화된 스텝 노트 쓰기용 SysEx는 존재하지 않습니다. 대신 본체 패널에서 `REC`를 켜 두면 실시간 녹음기가 외부 USB MIDI 노트를 받아 패턴에 기록합니다. `st rec-steps`(및 MCP `st_rec_steps`)는 트랜스포트를 시작하고 장비의 96 PPQ 클럭 그리드에 맞춰 노트를 발사합니다. 트랜스포트가 멈추면 녹음 세션이 끝나므로 여러 트랙을 연속 녹음할 때는 마지막 테이크 전까지 `--keep-playing`을 붙여서 실행합니다.

## 아키텍처

```text
src/model.rs     채널, CC, 뱅크, SysEx 주소 테이블
src/sysex.rs     SysEx 조립/파싱, 체크섬, 버전 포맷
src/midi.rs      midir 래퍼, 벌크 덤프 스티칭, 클럭 포트, MIDIRestart 복구
src/recsteps.rs  클럭 동기식 스텝 녹음
src/serve.rs     axum 웹 서버 및 GUI용 MIDI 클럭 스텝 시퀀서
src/cli.rs       st 명령 (list, identity, note, cc, pc, sysex, param, dump,
                 monitor, transport, load, get, snapshot, serve, rec-steps)
src/mcp.rs       st-mcp 표준 입출력 JSON-RPC 서버 (14개 도구)
gui/             Svelte 5 + TS 웹 컨트롤러 (DEVICE 뷰, MIXER 뷰, HELP 매뉴얼)
```

## 완료된 마일스톤
- [x] 프로토콜 조사: USB 식별자, MIDI 구현 차트, CC/뱅크/SysEx 주소 표
- [x] 코어 라이브러리(`midi`, `sysex`, `model`) 및 단위 테스트
- [x] CLI 명령 구현 및 실기 테스트
- [x] `st-mcp` 서버 및 JSON-RPC 스모크 테스트
- [x] 에이전트 스킬, 문서화, 로컬 설치 구성
- [x] `st serve` 기반 웹 GUI 믹서 뷰 및 브라우저 E2E 검증
- [x] 하드웨어 판면을 재현한 DEVICE 뷰(패턴 노브, 2x8 스텝 그리드, REC + 스케일 키, FX 노브, 고정식 스위치, 스프링 슬라이더)
- [x] 장비 MIDI 클럭에 동기화되는 백엔드 스텝 시퀀서(`/api/seq`)
- [x] `rec-steps` 연속 테이크를 통한 장비 패턴 메모리 작곡 검증
- [x] 오프라인 기동, 백그라운드 자동 재연결, CoreMIDI `MIDIRestart` 복구
- [x] 영/한 문서 지원(영문 기본 + `.ko.md` 확장) 및 인앱 매뉴얼 탑재

## 프로토콜상 한계 (문서화 완료)
- 스텝 노트는 SysEx로 직접 쓸 수 없으며 장비 녹음기와 `st rec-steps` 조합으로 기록합니다.
- 템포는 SysEx로 읽거나 쓸 수 없고, `REC` 활성화는 본체 물리 버튼으로만 가능합니다.
- 프로젝트는 전원 종료나 프로젝트 전환 시 자동 저장됩니다(별도 수동 WRITE 버튼 없음).
- Song Position Pointer와 송 모드는 장비에서 지원하지 않습니다.
