# Registry concurrency review

## Baseline investigation

The unchanged simple_performance_test binary hung on repetition 10 with two
test threads and a three-second deadline. Output stopped after initial metrics.
Source has opposite lock acquisition: register/unregister hold active_instances
then performance_stats; performance_metrics holds performance_stats then calls
active_count. Cleanup also drops user components under the active map lock.
Final mechanism demonstrations and independent review remain pending.
