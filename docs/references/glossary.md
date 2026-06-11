# Glossary

The working vocabulary of the workspace, with the citable origin of each term where
one exists. Downstream consumers (the SMIL calculator's `Enclosed` number type)
coin their own user facing names; this glossary is upstream of those coinages and
deliberately does not import them.

**Enclosure / containment.** The one directional rigor property: a computed set
contains every value the true operation can reach. A wider result is still correct;
a narrower one is the defect that falsifies everything downstream. The Fundamental
Theorem of Interval Arithmetic, from Moore's founding line
([moore-1966](moore-1966.md), anticipated by [sunaga-1958](sunaga-1958.md)); Law 2
in `crates/interval-1788/src/spec.rs`.

**Bare interval.** An interval with no decoration: the set, nothing about the
history of the computation that produced it. IEEE 1788 vocabulary
([ieee-1788-2015](ieee-1788-2015.md)).

**Decorated interval.** A bare interval paired with a decoration recording what is
known about the operation history (defined, continuous, bounded). The pair travels
together so the history cannot be separated from the set.

**Decoration propagation.** The rule that an operation's output decoration is the
meet (lattice minimum) of the input decorations and the operation's own local
decoration. Weakest knowledge wins; `Ill` poisons; `Com` is the identity.

**Inf sup representation.** An interval stored as its two endpoints. The alternative
is mid rad (midpoint and radius), the ball representation; the workspace choice and
the comparison literature are recorded in interval-1788 ADR-0001 and the entries
[rump-2010-acta-numerica](rump-2010-acta-numerica.md) and
[johansson-2017-arb](johansson-2017-arb.md).

**Noise symbol.** A formal variable `ε_i` ranging over `[-1, 1]` in an affine form.
Two forms sharing a symbol are correlated through it; that sharing is the entire
advantage of affine over interval arithmetic. Vocabulary of Comba and Stolfi
([comba-stolfi-1993](comba-stolfi-1993.md)).

**Deviation term.** One `x_i ε_i` summand of an affine form: the symbol and its
coefficient, the magnitude of the form's dependence on that noise source.

**Affine form.** `x_0 + Σ x_i ε_i`: a center plus deviation terms. Geometrically the
image of the unit cube under an affine map, which is why the zonotope literature
applies.

**Linearization.** Replacing a nonlinear function over a form's range with an affine
approximant plus a fresh noise symbol bounding the residual. The workspace uses the
Chebyshev (min max) construction everywhere; see the linearization section of the
[kernel map](kernel-map.md).

**Condensation.** Folding a form's smallest deviation terms into a single fresh
symbol bounding their summed magnitude, to cap the term count under a memory budget.
Width is preserved up to rounding; correlation through the folded symbols is
permanently forgotten (workspace ADR-0004). The same operation appears in the
reachability literature as zonotope order reduction.

**Zonotope correspondence.** An affine form with `n` deviation terms is a zonotope
with `n` generators; condensation is order reduction. The correspondence imports
twenty years of "which generators to merge" results, surveyed in
[kopetzki-schurmann-althoff-2017](kopetzki-schurmann-althoff-2017.md) and
[yang-scott-2018](yang-scott-2018.md).

**Directed rounding.** Computing a result rounded toward a chosen direction
(downward for lower bounds, upward for upper bounds) so every representation error
widens rather than narrows. The two attributes of [ieee-754-2019](ieee-754-2019.md)
that rigor consumes; the whole surface of `round-float`.

**Outward rounding.** Directed rounding applied per endpoint of an interval: lower
endpoint down, upper endpoint up. Law 3 in `spec.rs`.

**Tightness.** How close an enclosure is to the exact range. Always subordinate to
enclosure: the verification lanes weight soundness above tightness, and a tightness
regression is a quality bug while an enclosure violation is a falsehood.
