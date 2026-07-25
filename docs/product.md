# Product Overview

## Problem

Developers frequently need to understand codebases they have never seen before
— joining a new team, picking up an open-source project, or reviewing
AI-generated code. Existing tools either require a full IDE setup, send code to
remote servers, or provide shallow file-tree views without structural insight.

## Solution

CodeCompass is a local-first desktop application that:

1. **Opens** a local repository folder.
2. **Scans** its file structure and extracts symbols (functions, classes,
   imports).
3. **Visualizes** file and symbol relationships as an interactive graph.
4. **Identifies** likely entry points and high-coupling files.
5. **Answers** "what is affected if I change this file?"

Everything runs locally. No code leaves the user's machine.

## Target Users

| User                   | Need                                                     |
| ---------------------- | -------------------------------------------------------- |
| Students               | Understand assignment codebases and open-source projects |
| Junior developers      | Onboard into unfamiliar company repositories             |
| Senior developers      | Quickly assess a new dependency or inherited project     |
| AI-assisted developers | Understand auto-generated code before modifying it       |

## Principles

- **Local-first.** All analysis runs on the user's machine. No telemetry, no
  cloud calls.
- **Read-only by default.** CodeCompass analyzes code; it does not modify the
  repository.
- **Progressive disclosure.** Start with a high-level structure view; let users
  drill down to symbols and dependencies.
- **Fast feedback.** Scanning and incremental updates should feel snappy even on
  large repositories.

## Current Status

CodeCompass v1.0 provides the complete local workflow: register a repository,
scan and analyze supported source files, explore dependency and call graphs,
search symbols, read source in a guarded viewer, review structural insights,
estimate change impact, and inspect local Git health/evolution signals.

The verified release target is Windows. TypeScript, JavaScript, TSX, JSX, and
CSS analysis are supported. macOS/Linux distribution, auto-update, and cloud
features are not part of v1.0.

See [docs/roadmap.md](roadmap.md) for the planned milestones.
