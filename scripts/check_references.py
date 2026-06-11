#!/usr/bin/env python3
"""Structural validation for docs/references/.

Checks, with no network access:
  * every entry file has the required frontmatter fields;
  * each entry's slug matches its filename;
  * INDEX.md's Entries section is in one to one sync with the entry files;
  * entries with vendor-status vendored-at-path record sha256 lines that match
    the files on disk under docs/references/vendor/<slug>/.

An entry file is any *.md under docs/references/ that opens with a `---`
frontmatter fence. README.md, INDEX.md, and the synthesis documents carry no
frontmatter and are exempt.
"""

import hashlib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REFS = ROOT / "docs" / "references"
INDEX = REFS / "INDEX.md"

REQUIRED = [
    "slug",
    "category",
    "citation",
    "canonical-url",
    "doi",
    "archived-url",
    "archive-date",
    "retrieved",
    "license",
    "vendor-status",
    "rot-risk",
    "provenance-class",
    "consumers",
    "verification",
    "sha256",
    "notes",
]

CATEGORIES = {
    "spec-conformance",
    "algorithms",
    "registries",
    "glossary",
    "verification-map",
    "history-philosophy",
    "failure-museum",
}

VENDOR_STATUSES = {
    "vendored-at-path",
    "pointer-only",
    "legally-cannot",
    "paper-copy-owned",
}

ROT_RISKS = {
    "died-once",
    "single-maintainer",
    "community-run",
    "academic-personal",
    "stable-publisher",
    "ephemeral",
}


def parse_frontmatter(text):
    """Parse the constrained YAML subset the schema uses.

    Supports `key: value`, `key: >-` folded scalars, and `key:` followed by
    `- item` lists. Returns None when the file has no frontmatter fence.
    """
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    fields = {}
    key = None
    mode = None  # None, "folded", or "list"
    for line in lines[1:]:
        if line.strip() == "---":
            return fields
        if line[:1] not in ("", " ", "\t") and ":" in line:
            key, _, rest = line.partition(":")
            key = key.strip()
            rest = rest.split("#", 1)[0].strip()
            if rest == ">-" or rest == ">":
                fields[key] = ""
                mode = "folded"
            elif rest == "":
                fields[key] = []
                mode = "list"
            else:
                fields[key] = rest
                mode = None
        elif mode == "folded" and key is not None:
            fields[key] = (fields[key] + " " + line.strip()).strip()
        elif mode == "list" and key is not None and line.strip().startswith("- "):
            fields[key].append(line.strip()[2:].strip())
    return None  # unterminated fence


def main():
    errors = []
    if not REFS.is_dir():
        print(f"missing directory: {REFS}", file=sys.stderr)
        return 1

    entries = {}
    for path in sorted(REFS.glob("*.md")):
        fm = parse_frontmatter(path.read_text(encoding="utf-8"))
        if fm is None:
            continue  # README, INDEX, synthesis documents
        entries[path.stem] = (path, fm)

    for stem, (path, fm) in entries.items():
        where = path.relative_to(ROOT)
        for field in REQUIRED:
            if field not in fm or fm[field] in ("", []):
                errors.append(f"{where}: missing or empty field `{field}`")
        if fm.get("slug") != stem:
            errors.append(f"{where}: slug `{fm.get('slug')}` != filename `{stem}`")
        if fm.get("category") not in CATEGORIES:
            errors.append(f"{where}: unknown category `{fm.get('category')}`")
        if fm.get("vendor-status") not in VENDOR_STATUSES:
            errors.append(f"{where}: unknown vendor-status `{fm.get('vendor-status')}`")
        if fm.get("rot-risk") not in ROT_RISKS:
            errors.append(f"{where}: unknown rot-risk `{fm.get('rot-risk')}`")

        if fm.get("vendor-status") == "vendored-at-path":
            sha = fm.get("sha256")
            sha_lines = sha if isinstance(sha, list) else [sha] if sha else []
            if not sha_lines or sha_lines == ["none"]:
                errors.append(f"{where}: vendored entry without sha256 lines")
            if fm.get("license") in ("", "unstated", "none", None):
                errors.append(f"{where}: vendored entry without an explicit license")
            for line in sha_lines:
                m = re.match(r"^([0-9a-f]{64})\s+(\S+)$", line or "")
                if not m:
                    errors.append(f"{where}: malformed sha256 line `{line}`")
                    continue
                digest, rel = m.groups()
                target = REFS / rel
                if not target.is_file():
                    errors.append(f"{where}: vendored file missing: {rel}")
                    continue
                actual = hashlib.sha256(target.read_bytes()).hexdigest()
                if actual != digest:
                    errors.append(f"{where}: sha256 mismatch for {rel}")
        else:
            sha = fm.get("sha256")
            if isinstance(sha, str) and sha not in ("none",) and not re.match(
                r"^[0-9a-f]{64}$", sha
            ):
                errors.append(f"{where}: sha256 should be `none` or a hex digest")

    # INDEX sync: every entry linked exactly once in the Entries section, no
    # dead links.
    index_text = INDEX.read_text(encoding="utf-8") if INDEX.is_file() else ""
    entries_section = index_text.split("## Entries", 1)[-1].split("## ", 1)[0]
    linked = re.findall(r"\]\(([A-Za-z0-9._-]+\.md)\)", entries_section)
    linked_stems = [Path(name).stem for name in linked]
    for stem in entries:
        n = linked_stems.count(stem)
        if n == 0:
            errors.append(f"INDEX.md: entry `{stem}` not listed in Entries section")
        elif n > 1:
            errors.append(f"INDEX.md: entry `{stem}` listed {n} times")
    for stem in linked_stems:
        if stem not in entries:
            errors.append(f"INDEX.md: links to `{stem}.md` which is not an entry")

    if errors:
        for e in errors:
            print(e, file=sys.stderr)
        print(f"\n{len(errors)} problem(s).", file=sys.stderr)
        return 1
    print(f"references registry OK: {len(entries)} entries, INDEX in sync.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
