[35mdocs/canonical/DOC_CONSISTENCY_MATRIX.md[m[36m:[m[32m51[m[36m:[m2. **[1;31mQUARANTINE[m terminology eliminated** - Replaced with accurate physical reality
[35mdocs/canonical/ENGINE_AUDIT_FINDINGS.md[m[36m:[m[32m3[m[36m:[m## [1;31mQUARANTINE[m LIST - FROZEN
[35mdocs/canonical/ENGINE_AUDIT_FINDINGS.md[m[36m:[m[32m73[m[36m:[m1. **FREEZE** - Mark non-engine crates as [1;31mquarantine[m
[35mdocs/canonical/ENGINE_ONLY_RECOVERY_SCOPE.md[m[36m:[m[32m3[m[36m:[m## 🚨 [1;31mQUARANTINE[m ORDER - IMMEDIATE
[35mdocs/canonical/ENGINE_ONLY_RECOVERY_SCOPE.md[m[36m:[m[32m5[m[36m:[m**All consumer/tooling crates are now [1;31mQUARANTINED[m from engine recovery.**
[35mdocs/canonical/ENGINE_ONLY_RECOVERY_SCOPE.md[m[36m:[m[32m7[m[36m:[m### ❌ [1;31mQUARANTINED[m (DO NOT TOUCH)
[35mdocs/canonical/ENGINE_ONLY_RECOVERY_SCOPE.md[m[36m:[m[32m81[m[36m:[m**[1;31mQuarantined[m validation:**
[35mdocs/canonical/ENGINE_ONLY_RECOVERY_SCOPE.md[m[36m:[m[32m84[m[36m:[m# cargo check -p sdk_app          # ❌ [1;31mQUARANTINED[m
[35mdocs/canonical/ENGINE_ONLY_RECOVERY_SCOPE.md[m[36m:[m[32m85[m[36m:[m# cargo check -p game_framework    # ❌ [1;31mQUARANTINED[m  
[35mdocs/canonical/ENGINE_ONLY_RECOVERY_SCOPE.md[m[36m:[m[32m86[m[36m:[m# cargo check -p engine_tools      # ❌ [1;31mQUARANTINED[m
[35mdocs/canonical/ENGINE_ONLY_RECOVERY_SCOPE.md[m[36m:[m[32m87[m[36m:[m# cargo test -p engene_sdk       # ❌ [1;31mQUARANTINED[m
[35mdocs/canonical/ENGINE_ONLY_RECOVERY_SCOPE.md[m[36m:[m[32m100[m[36m:[m### Tier 2 - [1;31mQuarantined[m (Consumer Layer)
[35mdocs/canonical/ENGINE_ONLY_RECOVERY_SCOPE.md[m[36m:[m[32m111[m[36m:[m### 1. NO FIXES FOR [1;31mQUARANTINED[m CRATES
[35mdocs/canonical/ENGINE_ONLY_RECOVERY_SCOPE.md[m[36m:[m[32m158[m[36m:[m"[1;31mQUARANTINED[m during engine-only recovery. Not part of current scope."
[35mdocs/canonical/ENGINE_ONLY_SCOPE.md[m[36m:[m[32m12[m[36m:[m## [1;31mQUARANTINE[m LIST - FORBIDDEN IN ENGINE PATH
[35mdocs/canonical/ENGINE_ONLY_SCOPE.md[m[36m:[m[32m14[m[36m:[m**Immediately [1;31mquarantined[m:**
[35mdocs/canonical/ENGINE_ONLY_SCOPE.md[m[36m:[m[32m26[m[36m:[m**[1;31mQuarantine[m rules:**
[35mdocs/canonical/ENGINE_ONLY_SCOPE.md[m[36m:[m[32m27[m[36m:[m1. No engine decisions influenced by [1;31mquarantined[m crates
[35mdocs/canonical/ENGINE_PATH_FINAL_AUDIT.md[m[36m:[m[32m47[m[36m:[m**ACTION: [1;31mQUARANTINE[m UNTIL ENGINE STABLE**
[35mdocs/canonical/ENGINE_PATH_FINAL_AUDIT.md[m[36m:[m[32m74[m[36m:[m- sdk_app, game_framework, engine_tools → [1;31mQUARANTINE[m
[35mdocs/canonical/ENGINE_RECOVERY_COMPLETE.md[m[36m:[m[32m45[m[36m:[m### 5. ✅ Consumer Crates - [1;31mQUARANTINED[m
[35mdocs/canonical/ENGINE_RECOVERY_COMPLETE.md[m[36m:[m[32m104[m[36m:[m3. ❌ Handle consumer applications ([1;31mquarantined[m)
[35mdocs/canonical/ENGINE_RECOVERY_FINAL_COMPLETE.md[m[36m:[m[32m5[m[36m:[m**Consumer crates [1;31mquarantined[m. Engine core restored. Minimal runtime verified.**
[35mdocs/canonical/ENGINE_RECOVERY_FINAL_COMPLETE.md[m[36m:[m[32m38[m[36m:[m- **Zero framework bloat:** All fat modules removed to [1;31mquarantine[m
[35mdocs/canonical/ENGINE_RECOVERY_FINAL_COMPLETE.md[m[36m:[m[32m56[m[36m:[m### 🚫 [1;31mQUARANTINE[m STATUS: ENFORCED
[35mdocs/canonical/ENGINE_RECOVERY_FINAL_COMPLETE.md[m[36m:[m[32m88[m[36m:[m1. ✅ **Scope Definition** - Engine-only vs [1;31mquarantine[m clearly separated
[35mdocs/canonical/ENGINE_RECOVERY_FINAL_COMPLETE.md[m[36m:[m[32m118[m[36m:[m- ✅ **Protected scope** - Consumer noise [1;31mquarantined[m
[35mdocs/canonical/ENGINE_RECOVERY_FINAL_COMPLETE.md[m[36m:[m[32m142[m[36m:[m**[1;31mQUARANTINE[m: ✅ ENFORCED**  
[35mdocs/canonical/INFRASTRUCTURE_CLEANUP_REPORT.md[m[36m:[m[32m22[m[36m:[m  - Contents: [1;31mquarantine[m/, phase0/, crash data
[35mdocs/canonical/MIGRATION_LEDGER.md[m[36m:[m[32m169[m[36m:[m| world_streaming tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | [1;31mQUARANTINE[m |
[35mdocs/canonical/MIGRATION_LEDGER.md[m[36m:[m[32m170[m[36m:[m| physics_body_combat tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | [1;31mQUARANTINE[m |
[35mdocs/canonical/MIGRATION_LEDGER.md[m[36m:[m[32m171[m[36m:[m| content_pipeline tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | [1;31mQUARANTINE[m |
[35mdocs/canonical/MIGRATION_LEDGER.md[m[36m:[m[32m172[m[36m:[m| persistence_full tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | [1;31mQUARANTINE[m |
[35mdocs/canonical/MIGRATION_LEDGER.md[m[36m:[m[32m173[m[36m:[m| runtime_systems tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | [1;31mQUARANTINE[m |
[35mdocs/canonical/MIGRATION_LEDGER.md[m[36m:[m[32m174[m[36m:[m| gameplay_and_ai tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | [1;31mQUARANTINE[m |
[35mdocs/canonical/MIGRATION_LEDGER.md[m[36m:[m[32m200[m[36m:[m3. Status must be: ACTIVE, PENDING, [1;31mQUARANTINE[m, or COMPLETE.
