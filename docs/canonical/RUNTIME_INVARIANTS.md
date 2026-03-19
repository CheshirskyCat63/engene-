# RUNTIME_INVARIANTS

## These are engine nerves, not decoration

1. **phase order cannot change silently**
2. **editor mutation must propagate spatial dirty**
3. **audio update cannot depend on render completion**
4. **tools mode must provide required resources or fail loudly**
5. **full rebuild fallback must be observable and testable**
6. **streaming unload/load must invalidate spatial state**
7. **origin shift must trigger documented rebuild behavior**
8. **docs may not advertise a non-existent runtime product**
9. **CI must cover the active architecture branch**
10. **root shell growth must be treated as debt, not convenience**

## Enforcement expectation

Each invariant should end up protected by at least one of:

- compile-time boundary,
- contract test,
- doctor/audit check,
- CI gate,
- explicit migration ledger entry.
