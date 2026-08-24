import argparse
import io
import stat
import tarfile
import zipfile


BASE_ENTRIES = {
    "Cargo.toml": b"[package]\nname='fixture'\n",
    "LICENSE": b"license\n",
    "NOTICE": b"notice\n",
    "MODIFICATIONS.md": b"modifications\n",
    "PROJECT_OWNER_LICENSE_APPROVAL.md": b"approval\n",
    "RELEASE-METADATA": b"commit=fixture\n",
    "src/lib.rs": b"pub fn fixture() {}\n",
}


def fault_entries(case):
    entries = dict(BASE_ENTRIES)
    if case == "normal" or case == "docs_only":
        if case == "docs_only":
            entries.pop("src/lib.rs")
        return entries, None
    if case.startswith("forbidden_"):
        forbidden = {
            "forbidden_question": "?",
            "forbidden_pipe": "|",
            "forbidden_quote": '"',
            "forbidden_angle": "<",
        }[case]
        entries[f"bad{forbidden}name/probe.txt"] = b"bad\n"
    elif case == "superscript_com":
        entries["COM¹/probe.txt"] = b"bad\n"
    elif case == "superscript_lpt":
        entries["LPT².log/probe.txt"] = b"bad\n"
    elif case == "console_in":
        entries["CONIN$/probe.txt"] = b"bad\n"
    elif case == "console_out":
        entries["CONOUT$/probe.txt"] = b"bad\n"
    elif case == "raw_c0_control":
        entries["bad\x01/probe.txt"] = b"bad\n"
    elif case == "raw_c1_control":
        entries["bad\x85/probe.txt"] = b"bad\n"
    elif case == "unicode_collision":
        entries["Straße.txt"] = b"one\n"
        entries["STRASSE.txt"] = b"two\n"
    elif case == "prefix_conflict":
        entries["prefix"] = b"file\n"
        entries["prefix/child.txt"] = b"child\n"
    elif case in {"symlink", "hardlink", "device"}:
        return entries, case
    else:
        raise ValueError(case)
    return entries, None


def write_zip(output, case):
    entries, special = fault_entries(case)
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in entries.items():
            archive.writestr(name, data)
        if special == "symlink":
            info = zipfile.ZipInfo("link")
            info.create_system = 3
            info.external_attr = (stat.S_IFLNK | 0o777) << 16
            archive.writestr(info, "../../outside")
        elif special is not None:
            raise ValueError(f"ZIP does not support fixture {special}")


def add_tar_file(archive, name, data):
    info = tarfile.TarInfo(name)
    info.size = len(data)
    info.mode = 0o644
    archive.addfile(info, io.BytesIO(data))


def write_tar(output, case):
    entries, special = fault_entries(case)
    with tarfile.open(output, "w:gz") as archive:
        for name, data in entries.items():
            add_tar_file(archive, name, data)
        if special == "symlink":
            info = tarfile.TarInfo("link")
            info.type = tarfile.SYMTYPE
            info.linkname = "../../outside"
            archive.addfile(info)
        elif special == "hardlink":
            info = tarfile.TarInfo("hard")
            info.type = tarfile.LNKTYPE
            info.linkname = "src/lib.rs"
            archive.addfile(info)
        elif special == "device":
            info = tarfile.TarInfo("device")
            info.type = tarfile.CHRTYPE
            info.devmajor = 1
            info.devminor = 3
            archive.addfile(info)


def copy_mutated(source, output, archive_format, case):
    if archive_format == "zip":
        with zipfile.ZipFile(source, "r") as original:
            members = [(info, original.read(info)) for info in original.infolist()]
        changed = False
        with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for info, data in members:
                if case == "omission" and not changed and info.filename.endswith(".rs"):
                    changed = True
                    continue
                if case == "blob_changed" and not changed and info.filename.endswith(".rs"):
                    data += b"\nmutated\n"
                    changed = True
                if case == "type_mutation" and not changed and info.filename.endswith(".rs"):
                    replacement = zipfile.ZipInfo(info.filename)
                    replacement.create_system = 3
                    replacement.external_attr = (stat.S_IFLNK | 0o777) << 16
                    archive.writestr(replacement, "../../outside")
                    changed = True
                    continue
                archive.writestr(info, data)
            if case == "safe_extra":
                archive.writestr("safe-extra.txt", b"extra\n")
                changed = True
        if not changed:
            raise RuntimeError(f"mutation did not apply: {case}")
        return

    with tarfile.open(source, "r:gz") as original:
        members = []
        for info in original.getmembers():
            stream = original.extractfile(info) if info.isfile() else None
            members.append((info, stream.read() if stream is not None else None))
    changed = False
    with tarfile.open(output, "w:gz") as archive:
        for info, data in members:
            if case == "omission" and not changed and info.name.endswith(".rs"):
                changed = True
                continue
            if case == "blob_changed" and not changed and info.name.endswith(".rs"):
                data = (data or b"") + b"\nmutated\n"
                info.size = len(data)
                changed = True
            if case == "type_mutation" and not changed and info.name.endswith(".rs"):
                info.type = tarfile.SYMTYPE
                info.linkname = "../../outside"
                info.size = 0
                data = None
                changed = True
            archive.addfile(info, io.BytesIO(data) if data is not None else None)
        if case == "safe_extra":
            add_tar_file(archive, "safe-extra.txt", b"extra\n")
            changed = True
    if not changed:
        raise RuntimeError(f"mutation did not apply: {case}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--format", choices=["zip", "tar.gz"], required=True)
    parser.add_argument("--case", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--source")
    args = parser.parse_args()

    if args.source:
        copy_mutated(args.source, args.output, args.format, args.case)
    elif args.format == "zip":
        write_zip(args.output, args.case)
    else:
        write_tar(args.output, args.case)


if __name__ == "__main__":
    main()
