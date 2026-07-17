//! Composition-level enclosure: property-tested over whole computations rather
//! than single operations, over the f64 fixture.
//!
//! Each case draws a random expression program over a small pool of input forms,
//! from the op alphabet `add`, `sub`, `negate`, `scale`, `mul`, `sqr`, `recip`,
//! `sqrt`, `exp`, `ln`, and `condense` (a representation fold applied
//! mid-program so folded correlations flow through later ops). The program runs
//! once symbolically as affine forms and once, per sampled point, in plain f64;
//! the affine result's reduction must enclose the plain value.
//!
//! # Why fresh mid-program symbols do not enter the oracle
//!
//! The oracle assigns only the input forms' base noise symbols and evaluates the
//! program's true real value at that point in plain f64. Every symbol minted
//! mid-program (an operation's rounding error, a multiply's bilinear remainder,
//! a Chebyshev residual, a condensation tail) enters with an outward-rounded
//! coefficient on a fresh `[-1, 1]` symbol, so it can only widen the result's
//! value set, never narrow it. For every base-symbol assignment the true
//! pointwise value therefore stays enclosed regardless of how the minted symbols
//! are set, and the reduction, which is the value set over all symbols, encloses
//! it. That is why the oracle need not (and cannot) know the minted symbols:
//! assigning the base symbols alone pins a real value the widened form still
//! contains. Condensation is a no-op for the true value (it only refolds the
//! representation), so the oracle treats it as identity.
//!
//! # The two-tier arbiter
//!
//! The plain f64 oracle evaluates in round-to-nearest, so its value can drift
//! from the true real value by accumulated ulps. On a containment failure the
//! program is re-evaluated at the same point inputs through interval-1788 point
//! intervals, a guaranteed enclosure of the true value. A reduction *disjoint*
//! from that rigorous interval is a confirmed enclosure bug and panics with the
//! detail; an *overlap* is ulp-scale oracle noise, which is counted and reported
//! (a written note, not a fix), matching the ADR-0006 arbiter.

#![allow(clippy::cast_precision_loss)] // the mantissa bits fit f64 exactly
#![allow(clippy::cast_possible_truncation)] // modular reductions stay in range

use affine_arith::{with_source, AffineForm, SymbolSource, Term};
use interval_1788::Interval;

/// The number of random programs per run. Budgeted to keep this, the slowest
/// property lane, inside the crate's CI envelope: each program builds a pool of
/// input forms and applies up to `MAX_OPS` affine operations (every one minting
/// symbols and re-establishing the representation invariant), then replays a
/// short trace at `SAMPLES_PER_PROGRAM` points. At this count the lane runs in
/// the same order of wall time as `elementary_fixture`'s multi-symbol sweep
/// (1500 iterations of comparable per-iteration work), the reference budget.
/// Raising it buys diminishing coverage against linear cost.
const CASES: usize = 500;

/// Operations applied per program before the nonlinear-coverage backstop. Long
/// enough to compose several correlations, short enough that magnitudes stay
/// numeric under repeated squares and products.
const MAX_OPS: usize = 12;

/// Point samples per program: the two ADR-mandated corners (all `+1`, all `-1`,
/// where an enclosure bound binds hardest) plus random assignments.
const SAMPLES_PER_PROGRAM: usize = 12;

/// `SplitMix64`: a small, well-distributed deterministic generator, so the whole
/// lane is reproducible from the base seed.
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A float in `[0, 1)`.
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    /// An index in `[0, n)`.
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    /// An inclusive integer in `[lo, hi]`.
    fn range(&mut self, lo: usize, hi: usize) -> usize {
        lo + self.below(hi - lo + 1)
    }

    /// A sign, `+1` or `-1`.
    fn sign(&mut self) -> f64 {
        if self.next_u64() & 1 == 0 {
            1.0
        } else {
            -1.0
        }
    }
}

/// One recorded, replayable instruction. Operands are register indices into the
/// program's growing register file; constants (a scale factor, a condense
/// budget) travel with the instruction so the f64 and interval replays match the
/// affine execution exactly.
#[derive(Clone, Copy)]
enum Instr {
    Add(usize, usize),
    Sub(usize, usize),
    Negate(usize),
    Scale(usize, f64),
    Mul(usize, usize),
    Sqr(usize),
    Recip(usize),
    Sqrt(usize),
    Exp(usize),
    Ln(usize),
    Condense(usize, usize),
}

impl Instr {
    /// Whether this op introduces a nonlinear image (and so exercises the
    /// rigor-critical remainder or Chebyshev machinery). Condensation and the
    /// linear ops do not.
    fn is_nonlinear(self) -> bool {
        matches!(
            self,
            Instr::Mul(..)
                | Instr::Sqr(_)
                | Instr::Recip(_)
                | Instr::Sqrt(_)
                | Instr::Exp(_)
                | Instr::Ln(_)
        )
    }
}

/// A tag in the weighted op alphabet. The alphabet is nonlinear-heavy (half the
/// entries) so a random program nearly always applies at least one nonlinear op
/// without leaning on discards, per the ADR generation rule.
#[derive(Clone, Copy)]
enum Kind {
    Add,
    Sub,
    Negate,
    Scale,
    Mul,
    Sqr,
    Recip,
    Sqrt,
    Exp,
    Ln,
    Condense,
}

const ALPHABET: &[Kind] = &[
    Kind::Add,
    Kind::Add,
    Kind::Sub,
    Kind::Sub,
    Kind::Negate,
    Kind::Scale,
    Kind::Scale,
    Kind::Mul,
    Kind::Mul,
    Kind::Sqr,
    Kind::Sqr,
    Kind::Recip,
    Kind::Sqrt,
    Kind::Exp,
    Kind::Ln,
    Kind::Condense,
];

/// Draw one instruction over a register file of `len` values.
fn gen_instr(rng: &mut Rng, len: usize) -> Instr {
    match ALPHABET[rng.below(ALPHABET.len())] {
        Kind::Add => Instr::Add(rng.below(len), rng.below(len)),
        Kind::Sub => Instr::Sub(rng.below(len), rng.below(len)),
        Kind::Negate => Instr::Negate(rng.below(len)),
        Kind::Scale => {
            // A scale factor bounded away from zero and from blowing magnitudes.
            let s = rng.sign() * (0.5 + rng.unit() * 2.0);
            Instr::Scale(rng.below(len), s)
        }
        Kind::Mul => Instr::Mul(rng.below(len), rng.below(len)),
        Kind::Sqr => Instr::Sqr(rng.below(len)),
        Kind::Recip => Instr::Recip(rng.below(len)),
        Kind::Sqrt => Instr::Sqrt(rng.below(len)),
        Kind::Exp => Instr::Exp(rng.below(len)),
        Kind::Ln => Instr::Ln(rng.below(len)),
        Kind::Condense => Instr::Condense(rng.below(len), rng.range(1, 4)),
    }
}

/// The representation invariant: deviation terms strictly ascending by symbol id
/// with no zero coefficient. Asserted after every op, so a merge bug surfaces at
/// the op that introduced it rather than downstream as unexplained width.
fn assert_invariant(form: &AffineForm<'_, f64>) {
    let mut previous: Option<u64> = None;
    for t in form.terms() {
        assert!(
            t.coeff() != 0.0,
            "representation invariant: zero coefficient"
        );
        let id = t.symbol().id();
        if let Some(p) = previous {
            assert!(
                p < id,
                "representation invariant: terms not strictly ascending ({p} >= {id})"
            );
        }
        previous = Some(id);
    }
}

/// Execute one instruction affinely. Returns `None` when the op declines in
/// domain (`recip`/`sqrt`/`ln`/`exp` off their domain) or when the result would
/// leave the finite, bounded range this numeric lane keeps its programs inside;
/// either way the instruction is skipped and never recorded, so the concrete and
/// interval replays only ever see ops that ran and stay in domain.
fn apply_affine<'id>(
    instr: Instr,
    regs: &[AffineForm<'id, f64>],
    src: &mut SymbolSource<'id>,
) -> Option<AffineForm<'id, f64>> {
    let res = match instr {
        Instr::Add(i, j) => regs[i].add(&regs[j], src),
        Instr::Sub(i, j) => regs[i].sub(&regs[j], src),
        Instr::Negate(i) => regs[i].negate(),
        Instr::Scale(i, s) => regs[i].scale(s, src),
        Instr::Mul(i, j) => regs[i].mul(&regs[j], src),
        Instr::Sqr(i) => regs[i].sqr(src),
        Instr::Recip(i) => regs[i].recip(src)?,
        Instr::Sqrt(i) => regs[i].sqrt(src)?,
        Instr::Exp(i) => regs[i].exp(src)?,
        Instr::Ln(i) => regs[i].ln(src)?,
        Instr::Condense(i, budget) => regs[i].condense(budget, src),
    };
    // Keep the lane numeric: reject a result whose center or reduction has run
    // to a non-finite or unbounded value, so every recorded op leaves a bounded
    // form and the plain-f64 replay of an in-domain point stays finite.
    if !res.center().is_finite() {
        return None;
    }
    let red = res.reduce();
    if !red.inf().is_finite() || !red.sup().is_finite() {
        return None;
    }
    Some(res)
}

/// Replay a recorded trace in plain f64 from concrete input values. Condensation
/// is value-preserving, so it copies the operand.
fn replay_f64(trace: &[Instr], inputs: &[f64]) -> f64 {
    let mut regs: Vec<f64> = inputs.to_vec();
    for &instr in trace {
        let v = match instr {
            Instr::Add(i, j) => regs[i] + regs[j],
            Instr::Sub(i, j) => regs[i] - regs[j],
            Instr::Negate(i) => -regs[i],
            Instr::Scale(i, s) => regs[i] * s,
            Instr::Mul(i, j) => regs[i] * regs[j],
            Instr::Sqr(i) => regs[i] * regs[i],
            Instr::Recip(i) => 1.0 / regs[i],
            Instr::Sqrt(i) => regs[i].sqrt(),
            Instr::Exp(i) => regs[i].exp(),
            Instr::Ln(i) => regs[i].ln(),
            Instr::Condense(i, _) => regs[i],
        };
        regs.push(v);
    }
    *regs.last().expect("trace leaves at least the inputs")
}

/// Replay a recorded trace through interval-1788 point intervals: the rigorous
/// arbiter oracle. With point inputs there is no correlation to forget, so the
/// result is a guaranteed (only rounding-widened) enclosure of the true value.
fn replay_interval(trace: &[Instr], inputs: &[f64]) -> Interval<f64> {
    let mut regs: Vec<Interval<f64>> = inputs
        .iter()
        .map(|&v| Interval::point(v).unwrap_or_else(|_| Interval::entire()))
        .collect();
    for &instr in trace {
        let v = match instr {
            Instr::Add(i, j) => regs[i] + regs[j],
            Instr::Sub(i, j) => regs[i] - regs[j],
            Instr::Negate(i) => -regs[i],
            Instr::Scale(i, s) => {
                regs[i] * Interval::point(s).unwrap_or_else(|_| Interval::entire())
            }
            Instr::Mul(i, j) => regs[i] * regs[j],
            Instr::Sqr(i) => regs[i].sqr(),
            Instr::Recip(i) => regs[i].recip(),
            Instr::Sqrt(i) => regs[i].sqrt(),
            Instr::Exp(i) => regs[i].exp(),
            Instr::Ln(i) => regs[i].ln(),
            Instr::Condense(i, _) => regs[i],
        };
        regs.push(v);
    }
    *regs.last().expect("trace leaves at least the inputs")
}

/// Whether two nonempty intervals share a point. Empty is disjoint from
/// everything; unbounded endpoints participate through the ordinary comparison.
fn overlaps(a: &Interval<f64>, b: &Interval<f64>) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    a.inf() <= b.sup() && b.inf() <= a.sup()
}

/// A built program: the recorded trace, the affine result's reduction, the input
/// forms' `(center, coefficient)` (each input is a single-symbol lift, so its
/// value at symbol assignment `e` is `center + coefficient * e`), and how many
/// nonlinear ops actually ran.
struct Program {
    trace: Vec<Instr>,
    reduced: Interval<f64>,
    inputs: Vec<(f64, f64)>,
    nonlinear: u32,
}

/// Generate and affinely execute one program, asserting the representation
/// invariant after every op.
fn build_program(rng: &mut Rng) -> Program {
    let num_inputs = rng.range(2, 4);
    // Positive centers and moderate radii keep `recip`/`sqrt`/`ln` in domain by
    // construction, so declines are rare rather than the common case.
    let specs: Vec<(f64, f64)> = (0..num_inputs)
        .map(|_| {
            let center = 1.0 + rng.unit() * 4.0; // [1, 5]
            let radius = 0.02 + rng.unit() * 0.28; // [0.02, 0.3]
            (center, radius)
        })
        .collect();

    with_source(|mut src| {
        let mut regs: Vec<AffineForm<'_, f64>> = Vec::new();
        let mut inputs: Vec<(f64, f64)> = Vec::new();
        for &(center, radius) in &specs {
            let iv = Interval::new(center - radius, center + radius).expect("ordered endpoints");
            let form = AffineForm::from_interval(&iv, &mut src).expect("bounded nonempty");
            assert_invariant(&form);
            // A nondegenerate lift carries exactly one base symbol; a point lift
            // (radius rounded to zero) carries none, and its coefficient is zero.
            let coeff = form.terms().first().map_or(0.0, Term::coeff);
            inputs.push((form.center(), coeff));
            regs.push(form);
        }

        let mut trace: Vec<Instr> = Vec::new();
        let mut nonlinear = 0u32;
        for _ in 0..MAX_OPS {
            let instr = gen_instr(rng, regs.len());
            if let Some(res) = apply_affine(instr, &regs, &mut src) {
                assert_invariant(&res);
                regs.push(res);
                trace.push(instr);
                if instr.is_nonlinear() {
                    nonlinear += 1;
                }
            }
        }

        // Nonlinear-coverage backstop: if the random draw applied no nonlinear
        // op, square the last register (`sqr` is total, so it always runs), and
        // fall back to an input register if that somehow leaves the range. The
        // generator makes this nearly never fire; it exists so the coverage
        // assertion below can never flake on an unlucky all-linear seed.
        if nonlinear == 0 {
            let last = regs.len() - 1;
            let instr = if let Some(res) = apply_affine(Instr::Sqr(last), &regs, &mut src) {
                assert_invariant(&res);
                regs.push(res);
                Instr::Sqr(last)
            } else {
                let res = apply_affine(Instr::Sqr(0), &regs, &mut src)
                    .expect("input square stays in range");
                assert_invariant(&res);
                regs.push(res);
                Instr::Sqr(0)
            };
            trace.push(instr);
            nonlinear += 1;
        }

        let reduced = regs.last().expect("at least the inputs").reduce();
        Program {
            trace,
            reduced,
            inputs,
            nonlinear,
        }
    })
}

/// Map a base-symbol assignment to concrete input values.
fn input_values(inputs: &[(f64, f64)], eps: &[f64]) -> Vec<f64> {
    inputs
        .iter()
        .zip(eps)
        .map(|(&(center, coeff), &e)| center + coeff * e)
        .collect()
}

#[test]
fn random_programs_enclose_their_pointwise_value() {
    let mut rng = Rng::new(0x00C0_FFEE_5EED_u64);
    let mut overlap_notes = 0u64;
    let mut sample_total = 0u64;

    for _ in 0..CASES {
        let program = build_program(&mut rng);
        assert!(
            program.nonlinear >= 1,
            "coverage collapse: program applied no nonlinear op"
        );
        let n = program.inputs.len();

        for s in 0..SAMPLES_PER_PROGRAM {
            // The two ADR-mandated corners always; then random corners and
            // interiors so non-corner extrema and generic points are sampled too.
            let eps: Vec<f64> = match s {
                0 => vec![1.0; n],
                1 => vec![-1.0; n],
                _ => (0..n)
                    .map(|_| {
                        if rng.next_u64() & 1 == 0 {
                            rng.sign() // a random corner
                        } else {
                            2.0 * rng.unit() - 1.0 // an interior point
                        }
                    })
                    .collect(),
            };

            let values = input_values(&program.inputs, &eps);
            let oracle = replay_f64(&program.trace, &values);
            sample_total += 1;
            if !oracle.is_finite() {
                // A generation artifact (magnitude growth), not enclosure
                // evidence: the affine result stayed bounded by construction.
                continue;
            }
            if program.reduced.contains(oracle) {
                continue;
            }

            // Containment failed: consult the rigorous arbiter.
            let rigorous = replay_interval(&program.trace, &values);
            assert!(
                overlaps(&program.reduced, &rigorous),
                "confirmed enclosure bug: affine reduction [{}, {}] is disjoint from the \
                 rigorous point-interval oracle [{}, {}] at inputs {:?} (plain-f64 value {oracle})",
                program.reduced.inf(),
                program.reduced.sup(),
                rigorous.inf(),
                rigorous.sup(),
                values,
            );
            // Overlap: ulp-scale oracle noise. A written note, not a fix.
            overlap_notes += 1;
        }
    }

    // The arbiter's noise count is reported, not asserted: overlaps are oracle
    // noise the ADR expects, and only a disjoint reduction (panicked above) is a
    // bug. Visible under `--nocapture`.
    eprintln!(
        "composition lane: {CASES} programs, {sample_total} point samples, \
         {overlap_notes} ulp-scale oracle-noise (overlap) events"
    );
}
