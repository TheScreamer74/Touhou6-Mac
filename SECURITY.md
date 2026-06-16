# Security Policy

## Supported versions

This is a hobby fan project under active development. Only the latest commit on
`main` is supported. Please reproduce any issue against the current `main`
before reporting.

## Reporting a vulnerability

**Do not open a public issue for security reports.**

Report privately via a
[GitHub security advisory](https://github.com/TheScreamer74/Touhou6/security/advisories/new).
If you cannot use that, contact the maintainer through their GitHub profile
([@TheScreamer74](https://github.com/TheScreamer74)).

Please include:

- A description of the issue and its impact.
- Steps to reproduce, against the current `main` (commit hash).
- Your environment (native macOS or web/WASM build, OS/browser version).

You can expect an initial response within a reasonable time for a volunteer
project. We will confirm the report, work on a fix, and credit you in the
release notes unless you prefer to stay anonymous.

## Scope

In scope — issues in this engine's code, such as:

- Memory-safety bugs reachable while parsing untrusted input (PBG3 `.DAT`,
  ANM, ECL, STD, MSG files), including crafted or corrupted game data.
- Crashes or undefined behavior triggered by malformed input files.
- Vulnerabilities in the WebAssembly build (the page processes user-supplied
  game files locally in the browser).

Out of scope:

- Bugs in the original game's data or logic.
- Vulnerabilities in third-party dependencies — report those upstream
  (mention them here if they affect this project).
- Anything requiring already-compromised local access.

## A note on game files

Do **not** attach copyrighted game data files to a security report. If a sample
input is needed to reproduce a parser bug, describe how to construct it, or
provide a minimized non-copyrighted file that triggers the same code path.
