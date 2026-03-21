# runtime.fast

runtime split:
- crates = engine/shared core
- apps = entrypoints/bootstrap
- game = game-facing runtime

default rule:
- inspect owning layer first
- do not collapse ownership
- keep engine/sdk/game boundaries explicit

when fast path is allowed:
- compile error
- import/path error
- local test fix
- small local refactor
- local bugfix with no architecture effect

