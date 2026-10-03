import importlib.util
from pathlib import Path


def load_checker(checker: Path, module_name: str, description: str):
    if not checker.is_file():
        raise AssertionError(f"{description} is missing: {checker}")
    spec = importlib.util.spec_from_file_location(module_name, checker)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module
