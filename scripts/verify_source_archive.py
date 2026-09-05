#!/usr/bin/env python3
"""양 OS release verifier가 공유하는 source archive 경계 검사기다."""

import argparse
import hashlib
import os
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess
import sys
import tarfile
import tempfile
import unicodedata
import zipfile


MAX_ARCHIVE_ENTRIES = 100_000
MAX_UNCOMPRESSED_BYTES = 512 * 1024 * 1024
WINDOWS_FORBIDDEN = set('<>:"\\|?*')
WINDOWS_DEVICES = {
    "con",
    "prn",
    "aux",
    "nul",
    "conin$",
    "conout$",
    "clock$",
    *(f"com{index}" for index in range(1, 10)),
    *(f"lpt{index}" for index in range(1, 10)),
}
EXCLUDED_ROOTS = {"legacy_nethack_port_reference", "target", "output"}
FULL_COMMIT = re.compile(r"[0-9a-f]{40}")


class ArchiveValidationError(Exception):
    pass


def fail(message):
    raise ArchiveValidationError(message)


def canonical_component(component, entry_name):
    if component in {"", ".", ".."}:
        fail(f"source archive contains a non-canonical component: {entry_name!r}")
    if component.endswith((".", " ")):
        fail(f"source archive contains a Windows trailing-name alias: {entry_name!r}")
    for character in component:
        value = ord(character)
        if value < 0x20 or value == 0x7F or 0x80 <= value <= 0x9F:
            fail(f"source archive contains a control character: {entry_name!r}")
        if character in WINDOWS_FORBIDDEN:
            fail(f"source archive contains a Windows-forbidden character: {entry_name!r}")
    canonical = unicodedata.normalize("NFKC", component).casefold()
    if canonical.endswith((".", " ")) or any(
        character in WINDOWS_FORBIDDEN for character in canonical
    ):
        fail(f"source archive normalization is not Windows-compatible: {entry_name!r}")
    if canonical.split(".", 1)[0] in WINDOWS_DEVICES:
        fail(f"source archive contains a Windows reserved device: {entry_name!r}")
    return canonical


def canonical_entry_name(name):
    if not name or "\\" in name or name.startswith("/") or name.startswith("//"):
        fail(f"source archive contains an unsafe path: {name!r}")
    stripped = name[:-1] if name.endswith("/") else name
    if not stripped:
        fail("source archive contains an empty path")
    path = PurePosixPath(stripped)
    components = stripped.split("/")
    canonical = tuple(canonical_component(component, name) for component in components)
    if path.is_absolute() or canonical[0] in EXCLUDED_ROOTS:
        fail(f"source archive contains an excluded or absolute path: {name!r}")
    return "/".join(canonical)


def zip_entries(path):
    with zipfile.ZipFile(path, "r") as archive:
        infos = archive.infolist()
        if not infos or len(infos) > MAX_ARCHIVE_ENTRIES:
            fail(f"source ZIP entry count is outside 1..={MAX_ARCHIVE_ENTRIES}")
        total_size = 0
        for info in infos:
            if info.flag_bits & 0x1:
                fail(f"source ZIP contains an encrypted entry: {info.filename!r}")
            mode = (info.external_attr >> 16) & 0xFFFF
            file_type = stat.S_IFMT(mode)
            if info.is_dir():
                kind = "directory"
                data = None
            elif file_type in {0, stat.S_IFREG}:
                kind = "file"
                total_size += info.file_size
                if total_size > MAX_UNCOMPRESSED_BYTES:
                    fail("source ZIP uncompressed size exceeds the validation budget")
                data = archive.read(info)
            else:
                fail(f"source ZIP contains a non-regular entry type: {info.filename!r}")
            yield info.filename, kind, data


def tar_entries(path):
    with tarfile.open(path, "r:gz") as archive:
        infos = archive.getmembers()
        if not infos or len(infos) > MAX_ARCHIVE_ENTRIES:
            fail(f"source TAR entry count is outside 1..={MAX_ARCHIVE_ENTRIES}")
        total_size = 0
        for info in infos:
            if info.isdir():
                kind = "directory"
                data = None
            elif info.isfile():
                kind = "file"
                total_size += info.size
                if total_size > MAX_UNCOMPRESSED_BYTES:
                    fail("source TAR uncompressed size exceeds the validation budget")
                stream = archive.extractfile(info)
                if stream is None:
                    fail(f"source TAR regular entry has no payload: {info.name!r}")
                data = stream.read()
            else:
                fail(
                    f"source TAR contains a link, device, FIFO, or unknown type: {info.name!r}"
                )
            yield info.name, kind, data


def inspect_archive(path, archive_format):
    entries = list(zip_entries(path) if archive_format == "zip" else tar_entries(path))

    canonical = {}
    for name, kind, data in entries:
        key = canonical_entry_name(name)
        if key in canonical:
            fail(f"source archive contains an extraction collision: {name!r}")
        canonical[key] = (name, kind, data)

    for key, (name, _, _) in canonical.items():
        components = key.split("/")
        for index in range(1, len(components)):
            prefix = "/".join(components[:index])
            if prefix in canonical and canonical[prefix][1] != "directory":
                fail(f"source archive contains a file/directory prefix conflict: {name!r}")
    return canonical


def safe_extraction_check(entries):
    expected_files = {}
    with tempfile.TemporaryDirectory(prefix="aihack-source-extract-") as directory:
        root = Path(directory).resolve()
        for key, (name, kind, data) in sorted(entries.items()):
            relative = PurePosixPath(name[:-1] if name.endswith("/") else name)
            target = root.joinpath(*relative.parts)
            resolved_parent = target.parent.resolve()
            try:
                resolved_parent.relative_to(root)
            except ValueError:
                fail(f"source archive extraction escaped the temporary root: {name!r}")
            if kind == "directory":
                target.mkdir(parents=True, exist_ok=True)
                continue
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open("xb") as stream:
                stream.write(data)
            expected_files[key] = hashlib.sha256(data).hexdigest()

        actual_files = {}
        for current, directories, files in os.walk(root):
            directories.sort()
            files.sort()
            for filename in files:
                path = Path(current, filename)
                relative = path.relative_to(root).as_posix()
                key = canonical_entry_name(relative)
                actual_files[key] = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual_files != expected_files:
            fail("source archive extracted path/content manifest does not match its raw entries")


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def verify_expected_commit(archive, archive_format, repository_root, expected_commit):
    if not FULL_COMMIT.fullmatch(expected_commit):
        fail("ExpectedCommit must be a full lowercase 40-hex commit ID")
    root = Path(repository_root).resolve()
    resolved = subprocess.run(
        ["git", "-C", str(root), "rev-parse", "--verify", f"{expected_commit}^{{commit}}"],
        check=False,
        capture_output=True,
        text=True,
    )
    if resolved.returncode != 0 or resolved.stdout.strip() != expected_commit:
        fail("ExpectedCommit does not resolve to the exact repository commit")

    with tempfile.TemporaryDirectory(prefix="aihack-expected-archive-") as directory:
        expected = Path(directory, "expected.archive")
        generated = subprocess.run(
            [
                "git",
                "-C",
                str(root),
                "archive",
                f"--format={archive_format}",
                f"--output={expected}",
                expected_commit,
            ],
            check=False,
            capture_output=True,
            text=True,
        )
        if generated.returncode != 0 or not expected.is_file():
            fail(f"ExpectedCommit archive regeneration failed: {generated.stderr.strip()}")
        if sha256(archive) != sha256(expected):
            fail("source archive is not byte-identical to git archive ExpectedCommit")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive", required=True)
    parser.add_argument("--format", choices=["zip", "tar.gz"], required=True)
    parser.add_argument("--repository-root")
    parser.add_argument("--expected-commit")
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()

    try:
        archive = Path(args.archive)
        if not archive.is_file():
            fail("source archive is missing")
        entries = inspect_archive(archive, args.format)
        if not args.validate_only:
            if not args.repository_root or not args.expected_commit:
                fail("complete identity validation requires repository root and ExpectedCommit")
            verify_expected_commit(
                archive,
                args.format,
                args.repository_root,
                args.expected_commit,
            )
        safe_extraction_check(entries)
    except (ArchiveValidationError, OSError, tarfile.TarError, zipfile.BadZipFile) as error:
        print(f"source archive verification failed: {error}", file=sys.stderr)
        return 1

    print(f"PASS source archive: format={args.format} entries={len(entries)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
