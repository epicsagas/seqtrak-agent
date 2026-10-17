# Contributing to seqtrak agent

English | [한국어](CONTRIBUTING.ko.md)

Thank you for helping improve `seqtrak agent`. This document covers local development, testing, hardware verification, and pull request conventions.

## Development setup

1. Install the Rust stable toolchain (`rustup`) and Node.js (`npm`). On Ubuntu or Debian Linux, install `pkg-config` and `libasound2-dev` for ALSA MIDI support.
2. Build and test the Rust workspace:
   ```bash
   cargo build
   cargo test
   cargo clippy --all-targets
   ```
3. Build and type-check the web GUI (if you touch `gui/`):
   ```bash
   cd gui
   npm install
   npm run check
   npm run build
   ```

## Adding a CLI command or MCP tool

When adding a new hardware control capability, keep the CLI, MCP server, and skill documentation in sync:

1. Define any new channel, CC, bank, or SysEx address constants in `src/model.rs`.
2. Add the subcommand to `Command` in `src/cli.rs` and implement it in `run()`.
3. Expose the matching tool schema and handler in `src/mcp.rs`.
4. Document the command and verified SysEx addresses in `.claude/skills/seqtrak/SKILL.md`, `README.md`, and `README.ko.md`.

## Hardware verification

- Whenever possible, verify SysEx addresses and MIDI behaviors on a physical Yamaha SEQTRAK (firmware OS 2.00) and cite the corresponding section or page in the SEQTRAK Data List D0.
- If a change has only been tested against unit tests without hardware verification, state that clearly in your pull request description.

## Pull request checklist

- `cargo test` and `cargo clippy --all-targets` pass with zero warnings.
- `cd gui && npm run check && npm run build` passes with zero errors or warnings if `gui/` was modified.
- Commit messages follow Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `ci:`, `chore:`).
- English (`.md`) and Korean (`.ko.md`) user-facing docs are updated together when behavior or flags change.

## License

By contributing to this repository, you agree that your contributions will be licensed under the [Apache License, Version 2.0](LICENSE).
