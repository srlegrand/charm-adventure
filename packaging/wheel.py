#!/usr/bin/env python3
"""Packs a built game folder (the binary with its assets folder beside it) into a pip wheel.

    python3 packaging/wheel.py dist/charm-adventure manylinux_2_35_x86_64 0.1.42 out/

After `pip install charm-adventure`, `charm-adventure` starts the game and `charm-story` the story editor.
"""
import base64, hashlib, os, stat, sys, zipfile

NAME = "charm_adventure"
LAUNCHER = '''"""Charm Adventure in Tomato Land. `charm-adventure` on the command line starts the game."""
import os, subprocess, sys


def main(name="charm_adventure"):
    here = os.path.join(os.path.dirname(os.path.abspath(__file__)), "game")
    exe = os.path.join(here, name + (".exe" if os.name == "nt" else ""))
    if os.name != "nt":
        os.chmod(exe, 0o755)
        os.execv(exe, [exe] + sys.argv[1:])
    sys.exit(subprocess.call([exe] + sys.argv[1:]))


def story():
    """`charm-story` on the command line starts the story editor."""
    main("charm_story")


if __name__ == "__main__":
    main()
'''


def build(folder, platform, version, out):
    info = f"{NAME}-{version}.dist-info"
    files = {f"{NAME}/__init__.py": (LAUNCHER.encode(), 0o644), f"{NAME}/__main__.py": (b"from . import main\n\nmain()\n", 0o644)}
    for root, _, names in os.walk(folder):
        for n in sorted(names):
            path = os.path.join(root, n)
            rel = os.path.relpath(path, folder).replace(os.sep, "/")
            mode = 0o755 if "/" not in rel else 0o644
            files[f"{NAME}/game/{rel}"] = (open(path, "rb").read(), mode)
    readme = open("README.md", encoding="utf-8").read() if os.path.exists("README.md") else ""
    files[f"{info}/METADATA"] = ((
        f"Metadata-Version: 2.1\nName: charm-adventure\nVersion: {version}\n"
        "Summary: Charm Adventure in Tomato Land, a two-player co-op action platformer\n"
        "Home-page: https://github.com/srlegrand/charm-adventure\nRequires-Python: >=3.8\n"
        "Description-Content-Type: text/markdown\n\n" + readme).encode(), 0o644)
    files[f"{info}/WHEEL"] = (f"Wheel-Version: 1.0\nGenerator: packaging/wheel.py\nRoot-Is-Purelib: false\nTag: py3-none-{platform}\n".encode(), 0o644)
    files[f"{info}/entry_points.txt"] = (b"[console_scripts]\ncharm-adventure = charm_adventure:main\ncharm-story = charm_adventure:story\n", 0o644)
    record = []
    os.makedirs(out, exist_ok=True)
    target = os.path.join(out, f"{NAME}-{version}-py3-none-{platform}.whl")
    with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as z:
        for name, (data, mode) in files.items():
            zi = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            zi.external_attr = (stat.S_IFREG | mode) << 16
            zi.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(zi, data)
            digest = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()
            record.append(f"{name},sha256={digest},{len(data)}")
        record.append(f"{info}/RECORD,,")
        z.writestr(f"{info}/RECORD", "\n".join(record) + "\n")
    print(target)


if __name__ == "__main__":
    build(*sys.argv[1:5])
