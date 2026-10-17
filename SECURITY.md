# Security Policy

English | [한국어](SECURITY.ko.md)

## Supported versions

Security fixes are applied to the latest release on the `main` branch.

| Version | Supported |
|---|---|
| `0.1.x` | Yes |

## Scope

`seqtrak agent` runs locally on your machine:

- `st` and `st-mcp` communicate with a USB-connected Yamaha SEQTRAK over local OS MIDI ports and standard input/output.
- `st serve` binds to `127.0.0.1:8787` by default so the web GUI HTTP endpoints are only reachable from the local host. Do not expose `st serve` directly to untrusted networks without authentication.

## Reporting a vulnerability

If you discover a security vulnerability (for example, command injection, path traversal, or unintended remote exposure in `st`, `st-mcp`, or `st serve`), please report it privately rather than opening a public issue:

1. Use [GitHub Private Vulnerability Reporting](https://github.com/epicsagas/seqtrak-agent/security/advisories/new) on this repository.
2. Include steps to reproduce, affected binary or endpoint, and potential impact.

We will review the report, confirm the issue, and publish a fix and advisory as quickly as possible.
