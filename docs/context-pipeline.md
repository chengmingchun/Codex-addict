# Context Pipeline

This document describes the target context pipeline.

1. Select files in Project Files.
2. Send selected paths to the runtime.
3. Runtime validates paths under the project root.
4. Runtime reads selected text files with size limits.
5. Runtime packs content into the prompt before launching the configured CLI agent.

Limits:

- Max files: 12
- Max bytes per file: 24 KiB
- Max total context: 96 KiB
