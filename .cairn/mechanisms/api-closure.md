# Mechanism: api-closure

command: python3 -B .cairn/api-closure/check.py
inputs:
  - .cairn/api-closure
  - .cairn/reviews/api-020-image-capture-timeout
  - .cairn/reviews/rust-api-remediation.md
  - docs/api-audit.md
  - docs/spec/rust-api-remediation.md
  - tests/api_widget_behavior/image_host_capture.py
requirements:
  - API-020

Cairn requires current passing evidence from every inherited requirement before
this closure check can run. This independent check reconciles the complete audit
against the final review and proves that image-host processes are cleaned up after
both normal completion and a forced outer-runner timeout. It rejects the retained
timeout-leak control as acceptance evidence and exercises the corrected path.
