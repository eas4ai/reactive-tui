# App wakeups review

## Specification review

Acceptance must observe the wait boundary and final rendered state, not
just a wake counter. Exercise notifications both before and during waits,
equal signal writes, timer reentrancy, earlier deadlines, coalescing and
shutdown. Existing polling roots retain behavior. Mutation demonstrations
and the final ownership review remain pending.
