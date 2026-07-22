# Index

One line per entry; content lives in the entry files. See `README.md` for the schema
and the accretion ritual.

## Entries

- [Kashiwagi, kv C++ verified computation library](kv-kashiwagi.md) — oracle candidate for interval outputs; MIT
- [Rump & Kashiwagi 2015, implementation of affine arithmetic](rump-kashiwagi-2015.md) — literature anchor for ADR-0004 condensation follow-ups
- [pfloat-libm, the correctly rounded oracle](pfloat-libm-oracle.md) — truth source of the nightly exp/ln and arithmetic certification lanes
- [astro-float, the reduction oracle](astro-float-oracle.md) — pure Rust arbitrary-precision cross-check behind the Kulisch reduction lanes, with the two 0.9.5 quirks every consumer must work around
- [de Figueiredo & Stolfi 2004, the AA survey](figueiredo-stolfi-2004.md) — Chebyshev linearization and condensation; the workspace's most cited source
- [Stolfi & de Figueiredo 1997, the IMPA monograph](stolfi-figueiredo-1997.md) — the approximation theory under the survey; the "course notes" of folklore
- [The Affine Arithmetic Project page (Stolfi)](stolfi-affine-arith-project.md) — the AA line's hub; publication list and the C library prior art
- [Messine 2002, extensions of affine arithmetic](messine-2002-extensions-affine.md) — AF1/AF2 accumulator forms; the fixed memory alternative to condensation
- [Messine & Touhami 2006, reliable quadratic forms](messine-touhami-2006.md) — the heavier sequel; paywalled road continues marker
- [IEEE Std 754-2019](ieee-754-2019.md) — the floating point standard; the two directed rounding attributes round-float consumes
- [IEEE Std 1788-2015](ieee-1788-2015.md) — the interval standard interval-1788 implements; pointer plus three free proxies
- [IEEE Std 1788.1-2017](ieee-1788-1-2017.md) — the simplified profile; the declined v1.0 anchor, kept as an interim progress marker
- [IEEE P1788.1/D9.8 draft (March 2017)](p1788-1-d98-draft.md) — free clause-level proxy for the profile; Annex A settled the v1.0 required-op boundary
- [Goldberg 1991](goldberg-1991.md) — the free floating point exposition; first proxy for 754
- [Muller et al, Handbook of Floating-Point Arithmetic](muller-handbook-fp.md) — the modern professional reference; second proxy for 754
- [Ogita, Rump & Oishi 2005, accurate sum and dot product](ogita-rump-oishi-2005.md) — the error-free transforms behind the tight f64 backend and the future reductions
- [Boldo & Daumas 2003, representable correcting terms](boldo-daumas-2003.md) — where the EFT premise holds under underflow; source dead, Wayback survives
- [Revol 2017, introduction to IEEE 1788-2015](revol-1788-introduction.md) — the open exposition spec.rs cites; PDF freshly archived
- [musl libm accuracy statement](musl-libm-accuracy.md) — the external claim the f64 transcendental margin rests on, quoted verbatim
- [Comba & Stolfi 1993, affine arithmetic](comba-stolfi-1993.md) — the founding AA paper; SIBGRAPI'93, freshly archived
- [rust-lang libm crate](rust-libm-crate.md) — the fixture's host math; repository archived into compiler-builtins
- [MPFI (Revol & Rouillier)](mpfi.md) — arbitrary precision intervals over MPFR; ADR-0001 ecosystem precedent
- [filib++ (Lerch et al)](filib-pp.md) — the fast C++ interval library; precedent, and its vectors live in the vendored corpus
- [libaffa](libaffa.md) — affine C++ library, dormant; packaging precedent
- [aaflib](aaflib.md) — affine C++ library, dormant; packaging precedent
- [INTLAB (Rump)](intlab-rump.md) — the verified computing toolbox; restricted license, pointer and outputs only
- [Johansson 2017, Arb](johansson-2017-arb.md) — the mid-rad case; half of the three representation comparison
- [Rump 2010, Acta Numerica](rump-2010-acta-numerica.md) — the verification methods survey; the representation trade space
- [Girard 2005, zonotope reachability](girard-2005-zonotope-reachability.md) — origin of the box fold reduction; ADR-0004 ladder rung one
- [Combastel 2003, zonotope state bounding](combastel-2003.md) — the second baseline reduction; ECC not CDC; first ever snapshot
- [Kopetzki, Schurmann & Althoff 2017](kopetzki-schurmann-althoff-2017.md) — the order reduction survey with computable inflation bounds
- [Yang & Scott 2018](yang-scott-2018.md) — the Automatica comparison; paywalled boundary marker
- [Makino & Berz 2003, Taylor models](makino-berz-2003-taylor-models.md) — the road not taken; MSU host soft dead, mirror freshly archived
- [Muller, iRRAM](muller-irram.md) — the re-evaluation paradigm the downstream recovery design borrows
- [JCGM 100:2008, the GUM](gum-jcgm-100.md) — measurement uncertainty semantics; the metrology connection originates here
- [Sunaga 1958](sunaga-1958.md) — the forgotten ancestor; interval enclosure eight years before Moore
- [Moore 1966, Interval Analysis](moore-1966.md) — the founding book; bibliographic pointer, paper copy wanted
- [Kulisch, Computer Arithmetic and Validity](kulisch-computer-arithmetic.md) — the axiomatic arithmetic program; publisher page unarchivable
- [ITF1788, the interval conformance suite](itf1788-framework.md) — vendored vector corpus with per file licenses and the coverage gap analysis
- [GNU Octave interval package (Heimlich)](octave-interval-heimlich.md) — most complete open 1788 implementation; named behavioral oracle; GPL pointer
- [libieeep1788 (Nehmeier)](libieeep1788-nehmeier.md) — the draft era C++ reference; oracle with named caveats
- [Kearfott 2013, the P-1788 overview](kearfott-2013-p1788-overview.md) — the working group chair's own account; free proxy for the standard's rationale
- [Pryce 2016, the forthcoming standard 1788](pryce-2016-forthcoming-1788.md) — the technical editor's design rationale; free proxy
- [Kulisch 2008 P1788 draft, complete arithmetic](kulisch-2008-p1788-draft.md) — the exact-dot-product lineage behind the clause 12.2.12 reductions
- [P1788/D8.4 full standard draft (June 2014)](p1788-d8-4-draft.md) — the clause 12.12.4 two output division decoration rule the reverse doctrine rests on
- [Motion divPair (Wolff von Gudenberg 2013)](p1788-motion-divpair.md) — the working group motion behind the split decoration doctrine's design lineage
- [P1788 mailing list threads](p1788-mailing-list-threads.md) — the post ballot clause numbering and the reverse mode merge witness
- [Interval Computations site and reliable_computing list](interval-computations-site.md) — the community hub and the standardization record
- [Reliable Computing, the journal](reliable-computing-journal.md) — the field's venue of record, community run since 2009

## Synthesis documents

- [Kernel map](kernel-map.md) — per kernel: algorithm, source, decision record, verification artifact
- [Registries](registries.md) — the enumerable closed sets and their in tree sources of truth
- [Verification map](verification-map.md) — claim to artifact, with the named gaps
- [Glossary](glossary.md) — working vocabulary with citable origins
- [Failure museum](failure-museum.md) — post mortems at fix time
