---
slug: boldo-daumas-2003
category: algorithms
citation: >-
  Boldo, S., Daumas, M., "Representable correcting terms for possibly
  underflowing floating point operations", 16th IEEE Symposium on Computer
  Arithmetic (ARITH-16), Santiago de Compostela, pp. 79-86, 2003.
canonical-url: http://perso.ens-lyon.fr/marc.daumas/SoftArith/BolDau03a.pdf (dead at source, 404 on 2026-07-16)
doi: none recorded (IEEE ARITH-16 proceedings; dblp conf/arith/2003 carries the record)
archived-url: http://web.archive.org/web/20040123065641/http://perso.ens-lyon.fr:80/marc.daumas/SoftArith/BolDau03a.pdf
archive-date: 2004-01-23
retrieved: 2026-07-16
license: unstated (author preprint; IEEE holds the proceedings copyright)
vendor-status: pointer-only
rot-risk: died-once
provenance-class: secondary
consumers:
  - crates/round-float/docs/decisions/0002-tight-f64-backend.md (the underflow representability conditions the subnormal case analysis rests on)
verification:
  - none yet (the tight backend's subnormal hard-case vectors will exercise the conditions)
sha256: none
notes: >-
  The precise answer to "when is the rounding error of an operation itself a
  representable float": always for addition (subnormals included), and for
  multiplication, division, and square root under stated exponent
  conditions that fail near underflow. Those conditions draw the boundary
  of the tight backend's rescaling path, so this source is load-bearing for
  the one region where an EFT silently stops being error-free. The author
  page hosting the preprint is already dead; the 2004 Wayback snapshot is
  the surviving copy, which is the registry's died-once case in its purest
  form. Formalized machine-checked versions of these theorems exist in
  Boldo's later Coq work if the conditions ever need re-derivation.
---

Companion of [ogita-rump-oishi-2005](ogita-rump-oishi-2005.md): that paper
supplies the error-free transforms, this one says exactly where their
"error-free" premise holds. The Handbook
([muller-handbook-fp](muller-handbook-fp.md)) restates both in consolidated
form and is the accessible modern text for a reader without the paper.
