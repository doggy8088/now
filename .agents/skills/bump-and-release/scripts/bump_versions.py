#!/usr/bin/env python3
"""Bump the release version in the now project's version sources."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


SEMVER_RE = re.compile(r"^(?P<major>0|[1-9][0-9]*)\.(?P<minor>0|[1-9][0-9]*)\.(?P<patch>0|[1-9][0-9]*)$")


class VersionError(ValueError):
    """Raised when a repository version source is invalid or inconsistent."""


def parse_version(value: str) -> tuple[int, int, int]:
    match = SEMVER_RE.fullmatch(value)
    if not match:
        raise VersionError(f"unsupported semantic version: {value}")
    return tuple(int(match.group(name)) for name in ("major", "minor", "patch"))


def next_version(current: str, bump: str) -> str:
    major, minor, patch = parse_version(current)
    if bump == "major":
        major, minor, patch = major + 1, 0, 0
    elif bump == "minor":
        minor, patch = minor + 1, 0
    elif bump == "patch":
        patch += 1
    else:
        raise VersionError(f"unsupported bump level: {bump}")
    return f"{major}.{minor}.{patch}"


def read_cargo_package(path: Path) -> tuple[str, str]:
    text = path.read_text(encoding="utf-8")
    match = re.search(
        r"(?ms)^\[package\]\n(?P<body>.*?)(?=^\[|\Z)",
        text,
    )
    if not match:
        raise VersionError(f"Cargo package section not found: {path}")
    name = re.search(r'^name\s*=\s*"([^"]+)"\s*$', match.group("body"), re.MULTILINE)
    version = re.search(r'^version\s*=\s*"([^"]+)"\s*$', match.group("body"), re.MULTILINE)
    if not name or not version:
        raise VersionError(f"Cargo package name/version not found: {path}")
    return name.group(1), version.group(1)


def replace_once(text: str, pattern: str, replacement: str, path: Path) -> str:
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.MULTILINE)
    if count != 1:
        raise VersionError(f"expected exactly one version entry in {path}")
    return updated


def update_cargo_toml(path: Path, version: str) -> None:
    text = path.read_text(encoding="utf-8")
    match = re.search(r"(?ms)^\[package\]\n(?P<body>.*?)(?=^\[|\Z)", text)
    if not match:
        raise VersionError(f"Cargo package section not found: {path}")
    body = replace_once(
        match.group("body"),
        r'^(version\s*=\s*")[^"]+("\s*$)',
        rf"\g<1>{version}\g<2>",
        path,
    )
    updated = text[: match.start("body")] + body + text[match.end("body") :]
    path.write_text(updated, encoding="utf-8")


def update_cargo_lock(path: Path, package_name: str, version: str) -> None:
    text = path.read_text(encoding="utf-8")
    package_pattern = (
        rf'(?ms)(^\[\[package\]\]\nname\s*=\s*"{re.escape(package_name)}"\n'
        r'version\s*=\s*")[^"]+("\s*$)'
    )
    updated = replace_once(text, package_pattern, rf"\g<1>{version}\g<2>", path)
    path.write_text(updated, encoding="utf-8")


def read_package_json(path: Path) -> str:
    data = json.loads(path.read_text(encoding="utf-8"))
    version = data.get("version")
    if not isinstance(version, str):
        raise VersionError(f"package.json version not found: {path}")
    return version


def update_package_json(path: Path, version: str) -> None:
    text = path.read_text(encoding="utf-8")
    updated = replace_once(
        text,
        r'^(\s{2}"version"\s*:\s*")[^"]+("\s*,?\s*)$',
        rf"\g<1>{version}\g<2>",
        path,
    )
    path.write_text(updated, encoding="utf-8")


def read_package_lock(path: Path) -> str:
    data = json.loads(path.read_text(encoding="utf-8"))
    version = data.get("version")
    if not isinstance(version, str):
        raise VersionError(f"package-lock.json version not found: {path}")
    root_package = data.get("packages", {}).get("")
    if isinstance(root_package, dict) and root_package.get("version") != version:
        raise VersionError(f"package-lock root version is inconsistent: {path}")
    return version


def update_package_lock(path: Path, version: str) -> None:
    data = json.loads(path.read_text(encoding="utf-8"))
    data["version"] = version
    packages = data.get("packages")
    if isinstance(packages, dict) and isinstance(packages.get(""), dict):
        packages[""]["version"] = version
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def collect_versions(root: Path) -> tuple[str, list[tuple[Path, str]]]:
    cargo_toml = root / "Cargo.toml"
    package_json = root / "package.json"
    if not cargo_toml.is_file() or not package_json.is_file():
        raise VersionError("expected Cargo.toml and package.json at repository root")

    package_name, cargo_version = read_cargo_package(cargo_toml)
    package_version = read_package_json(package_json)
    sources = [(cargo_toml, cargo_version), (package_json, package_version)]

    cargo_lock = root / "Cargo.lock"
    if cargo_lock.is_file():
        sources.append((cargo_lock, read_cargo_lock(cargo_lock, package_name)))

    package_lock = root / "package-lock.json"
    if package_lock.is_file():
        sources.append((package_lock, read_package_lock(package_lock)))

    versions = {version for _, version in sources}
    if len(versions) != 1:
        details = ", ".join(f"{path}: {version}" for path, version in sources)
        raise VersionError(f"version sources are inconsistent: {details}")
    return cargo_version, sources


def read_cargo_lock(path: Path, package_name: str) -> str:
    text = path.read_text(encoding="utf-8")
    match = re.search(
        rf'(?ms)^\[\[package\]\]\nname\s*=\s*"{re.escape(package_name)}"\nversion\s*=\s*"([^"]+)"',
        text,
    )
    if not match:
        raise VersionError(f"root package not found in Cargo.lock: {path}")
    return match.group(1)


def update(root: Path, version: str, sources: list[tuple[Path, str]]) -> None:
    package_name, _ = read_cargo_package(root / "Cargo.toml")
    for path, _ in sources:
        if path.name == "Cargo.toml":
            update_cargo_toml(path, version)
        elif path.name == "Cargo.lock":
            update_cargo_lock(path, package_name, version)
        elif path.name == "package.json":
            update_package_json(path, version)
        elif path.name == "package-lock.json":
            update_package_lock(path, version)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--bump", choices=("patch", "minor", "major"), default="patch")
    group.add_argument("--version")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    try:
        current, sources = collect_versions(args.root)
        target = args.version or next_version(current, args.bump)
        parse_version(target)
        if args.dry_run:
            print(f"{current} -> {target}")
            for path, version in sources:
                print(f"{path}: {version} -> {target}")
        else:
            update(args.root, target, sources)
            print(f"Updated version sources: {current} -> {target}")
    except (OSError, json.JSONDecodeError, VersionError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
