# Mechanism: documentation-retention

command: python3 scripts/check-documentation-retention.py
inputs:
  - .gitignore
  - .cairn
  - docs
  - scripts/check-documentation-retention.py
requirements:
  - DOC-001
