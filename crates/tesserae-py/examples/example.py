import os
from pathlib import Path

import tesserae

# this is just the location of example.tess, pathlib isn't *necessary* for this.
# i believe ../../tesserae/file/example.tess works only if you're calling from the examples folder
file = str(Path(__file__).resolve().parents[2] / "tesserae" / "file" / "example.tess")

# tesserae.load loads the file from a path string or path with any environment
# tesserae.load("example.tess")
# tesserae.load("example.tess", "prod")
config = tesserae.load(file, os.environ.get("TESSERAE") or "prod")

print(f"serving {config['url']} on port {config['port']}")
print(f"request timeout: {config['timeout'].total_seconds():.0f}s")
print(f"cache size: {config['cache'] // 1024**2} MiB")
