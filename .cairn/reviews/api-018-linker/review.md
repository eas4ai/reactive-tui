# API-018 linker timeout investigation

The full check at 20260913T035930407Z failed only because the minimal-feature
crate doctest command exceeded its existing 900-second deadline. Default and
all-feature doctests each passed 74 tests with 10 ignored. Other documentation
checks passed. The stalled child was rust-lld linking the TabsBuilder example;
all 17 observed OS threads waited in futex_do_wait and no CPU progress was seen.
This identifies the stalled stage, not its underlying cause.

The identical minimal-feature command, source, toolchain and 12-job/test settings
passed on an isolated retry (74 passed, 10 ignored), retained in
20260913T040013489880Z. No source repair or relaxed assertion is justified by
this non-reproducing failure. Preserve the failed receipt and rerun the complete
formal check once. Do not count the development retry as formal acceptance.
