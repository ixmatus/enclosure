# The reference registry

This directory is the enclosure monorepo's citation registry: one markdown file per
external source the workspace cites or relies on. The crates implement external
standards, so the registry is part of the deliverable. A future maintainer should be
able to reconstruct where every algorithm, constant, and conformance claim came from
using the committed tree alone.

`INDEX.md` carries one line per entry and never carries content. Synthesis documents
(the kernel map, the registries inventory, the glossary, the verification map, the
failure museum) live beside the entries and cross reference them by slug.

## The accretion ritual

When a slice of work cites or relies on an external source, the same slice appends or
updates that source's entry here and saves the canonical URL to the Wayback Machine.
Never defer the entry to a future documentation pass. The convention is stated in the
repository `CLAUDE.md`; this file holds the mechanics.

## Entry format

Each entry is `<slug>.md` with YAML frontmatter followed by a short prose body saying
why this source and what it grounds. Required fields:

```yaml
---
slug: figueiredo-stolfi-2004        # matches the filename
category: algorithms                 # one of the seven categories below
citation: >-
  Author(s), "Title", venue, year, edition or revision.
canonical-url: https://...           # or a document number for paywalled standards
doi: 10.xxxx/...                     # "none" when the source has no DOI
archived-url: https://web.archive.org/web/...
archive-date: 2026-06-11             # when the Wayback save was made
retrieved: 2026-06-11                # when the canonical source was last fetched
license: ...                         # record BEFORE vendoring; "unstated" is a value
vendor-status: pointer-only          # vendored-at-path / pointer-only /
                                     # legally-cannot / paper-copy-owned
rot-risk: academic-personal          # died-once / single-maintainer / community-run /
                                     # academic-personal / stable-publisher / ephemeral
provenance-class: primary            # primary / secondary
consumers:
  - crates/affine-arith/src/elementary.rs
  - docs/decisions/0002-affine-elementary-functions.md
verification:
  - crates/affine-arith/tests/elementary_fixture.rs
sha256: none                         # required when vendor-status is vendored-at-path
notes: >-
  Why this source over alternatives, in one or two sentences.
---
```

Optional body sections may add anything the frontmatter cannot hold: coverage gaps of
a vector set, the clauses of a paywalled standard the code cites, errata, the story
of how the source was located.

Field rules:

- `vendor-status: vendored-at-path` requires a path under `vendor/<slug>/`, a
  recorded `license` that clearly permits the copy, and a `sha256` of each vendored
  file. The license is adjudicated before the copy lands, never after.
- Paywalled standards (IEEE) are `legally-cannot`: cite clause and document numbers
  and point at the free proxy literature (their entries name the proxies).
- Ephemeral URLs (distributor stock, vendor marketing pages) are never entries on
  their own; pin document numbers and revisions instead.
- `consumers` lists the code paths, decision records, and tests that lean on the
  source. An entry with no consumers is a candidate for removal, not a trophy.

## Categories

1. `spec-conformance`: the standards themselves, their open proxy literature,
   conformance vector sets (with license and coverage gaps), and the reference
   implementations used as behavioral oracles.
2. `algorithms`: per kernel sources. Directed rounding primitives, interval
   operations, elementary function enclosures, affine forms and linearization
   choices, condensation and order reduction, persistent forms.
3. `registries`: enumerable closed sets (decorations, rounding directions, exception
   behaviors) and where their in tree source of truth lives.
4. `glossary`: terminology sources where a term has a citable origin.
5. `verification-map`: sources that ground a verification artifact (a vector set a
   test derives from, a proof technique).
6. `history-philosophy`: the ancestry of the field and the commitments the work
   stands on.
7. `failure-museum`: post mortem sources, accreted at fix time.

## Validation

`scripts/check_references.py` (Python 3, stdlib only) validates the registry
structurally: required frontmatter fields present, slug matches filename, `INDEX.md`
in one to one sync with the entry files, vendored entries carry hashes that match the
files on disk. CI runs it on every push. It makes no network calls; archived URL
liveness is checked by a human at write time.
