# Use one typed allocation for native signal handles

Level: Judged
Decided by: Codex
Rests on: API-001
Would be wrong if: A working signal ABI changes, legacy 64-bit integer values are narrowed, mismatched live signal types are dereferenced, or shared hook signals lose their lifetime guarantees.

## Decision

Use the existing type-erased FFISignal allocation for both legacy and improved RTuiSignal constructors, adding a distinct 64-bit integer type while retaining the improved C-int type. Both signal destructors release the same allocation. Typed access checks the stored type before reading it; same-type string, float and bool access can interoperate across function families. Null and wrong-type errors retain existing ABI conventions. Handles remain owned, thread-confined and invalid after destruction; arbitrary pointers, double destruction and racing access are not newly promised. Thread-safe signal handles remain a separate type. Verify real Rust and C lifecycles and run the Rust cases under Miri. A static guard rejects the known direct Signal cast before any unsafe baseline lifecycle runs.

## Realized by

- 7fc976546f807527e666e6b16244adb5f64823a4 Repair native signal allocation types and destructor ownership
