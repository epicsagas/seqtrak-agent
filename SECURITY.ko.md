# 보안 정책 (Security Policy)

[English](SECURITY.md) | 한국어

## 지원 버전

보안 패치는 `main` 브랜치의 최신 릴리즈 버전을 기준으로 제공됩니다.

| 버전 | 지원 여부 |
|---|---|
| `0.1.x` | 지원 |

## 동작 범위 및 주의 사항

`seqtrak agent`는 사용자의 로컬 머신에서 실행됩니다:

- `st`와 `st-mcp`는 로컬 OS MIDI 포트 및 표준 입출력을 통해 USB로 연결된 Yamaha SEQTRAK과 통신합니다.
- `st serve`는 기본적으로 로컬 루프백(`127.0.0.1:8787`)에만 바인딩됩니다. 별도의 인증 계층 없이 `st serve`를 외부 네트워크에 직접 노출하지 마세요.

## 취약점 제보 방법

`st`, `st-mcp`, 또는 `st serve`에서 명령어 주입, 경로 탐색, 비인가 외부 노출 등의 보안 취약점을 발견하신 경우 공개 이슈 대신 비공개 채널로 제보해 주세요:

1. 저장소의 [GitHub 비공개 취약점 제보(Private Vulnerability Reporting)](https://github.com/epicsagas/seqtrak-agent/security/advisories/new)에서 리포트를 작성합니다.
2. 재현 절차, 영향받는 바이너리 또는 API 경로, 예상 영향을 함께 적어 주시면 빠르게 확인하고 패치를 배포하겠습니다.
