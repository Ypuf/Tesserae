import os
from datetime import timedelta
from typing import Any, TypeAlias, final

__all__ = ["Config", "TesseraeError", "load", "loads"]

_Value: TypeAlias = bool | int | float | str | timedelta | list[_Value]

class TesseraeError(Exception): ...

@final
class Config:
    def keys(self) -> list[str]: ...
    def __getitem__(self, key: str, /) -> _Value | dict[str, Any]: ...
    def __contains__(self, key: str, /) -> bool: ...
    def get(self, key: str) -> _Value: ...
    def line(self, key: str) -> int: ...
    def to_dict(self) -> dict[str, Any]: ...

def load(path: str | os.PathLike[str], env: str | None = None) -> Config:
    """Load a .tess file. Pass your env (e.g. "prod") to use that @environment"""

def loads(src: str, env: str | None = None) -> Config:
    """Parse a .tess source from a string"""
