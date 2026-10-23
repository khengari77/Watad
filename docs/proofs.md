# Watad: mathematical specification and machine-checked proofs

This guide explains the Rust/Verus contracts at code checkpoint `3ed3ad6`,
their mathematical meaning, and their premises.

Verification of this checkpoint passed:

- 294 Verus obligations: 36 in `watad-types`, 1 in `watad-daira`, and 257 in
  `watad-checker`, with zero errors;
- all 31 targeted mutation controls;
- 149 workspace Rust tests/examples and eight Python classifier tests;
- formatting, strict Clippy, strict rustdoc, and whitespace checks.

The workspace properties use `PROPTEST_CASES=512`; the compact source-rule
property explicitly fixes 64 cases. The usage catalog contains 147 source rows,
including eight Wafir rows. Verification is relative to declared profiles;
historical completeness and source interpretation remain separate questions.
See [verification commands and toolchain pins](../formal/README.md).

## 1. How to read the claims

Three kinds of statement appear below.

1. **Machine-checked contract.** A Verus `requires`/`ensures` clause or a
   `proof fn` lemma. These are the only statements that the verifier checks.
2. **Semantic corollary.** An ordinary mathematical reading of a checked
   contract, for example “recognition accepts exactly the declared finite
   language.” The corollary is not a separate theorem unless the source proves
   it explicitly.
3. **Ordinary Rust or test evidence.** Catalog deduplication, source-table
   construction, CLI assembly, reference-oracle comparisons and fingerprint
   tests are not inside the Verus proof boundary. They are labelled as such.

This distinction matters especially for Wafir deduplication. The generic
`remove_exact_duplicates` helper is ordinary Rust. The verified lowering starts
after its output has been supplied as profile data.

## 2. Domains and notation

Let `B = {0,1}`. A **beat word** is a nonempty finite sequence `w ∈ B+`, where
`1` is moving and `0` is still. A compiled **slot** is a finite nonempty list of
alternative words:

$$
S = (w_0,\ldots,w_{m-1}), \qquad m>0,\quad w_j\in B+.
$$

A **half-form** is a finite sequence of slots `H = (S_0,\ldots,S_{n-1})`.
A **form** contains a context label, an initial-only flag, a darb class, a sadr
half, an ajuz half (possibly absent), and two lists of adjacency rules:

$$
F = (\mathit{context},\mathit{initial},c,H_{\mathrm{sadr}},H_{\mathrm{ajuz}},
     R_{\mathrm{sadr}},R_{\mathrm{ajuz}}).
$$

A **profile** is

$$
P = (\mathit{id},\mathit{binding},(F_0,\ldots,F_{k-1})),
\qquad k>0,
$$

with nonzero family and revision in `id`. A **requirement system** is a finite
list of finite index groups:

$$
\mathit{Req} = (G_0,\ldots,G_{r-1}), \qquad G_g\subseteq\{0,\ldots,k-1\}.
$$

A **certificate** for one verse is

$$
C = (\mathit{profile},\mathit{context},f,\mathit{sadrSteps},\mathit{ajuzSteps}),
$$

where each step is a triple `(choice,start,end)`. The profile and context in the
certificate are claims to be checked, not authorities chosen by the certificate.

A **poem** is a nonempty finite sequence of verse inputs
`I_0,\ldots,I_{q-1}`. Each input supplies a context, sadr beats and ajuz beats.

## 3. Typed units, circle origins and layout

This section records the foundations that the compact-language layer assumes.
They are not a second model: each item is a contract on the actual shared
`Unit`/`FootId` types or on the checker's layout, profile and morphology
modules.

### 3.1 Typed units and primitive words

A `Beat` is `Mutaharrik` (`/`) or `Sakin` (`0`). A `Sabab` is `Khafif` (`/0`)
or `Thaqil` (`//`). A `Watad` is `Majmu` (`//0`) or `Mafruq` (`/0/`). These are
distinct Rust/Verus enums, not tags on a beat string. `Unit` is exactly
`Sabab(Sabab)` or `Watad(Watad)`.

The four primitive words are declared once in `unit_atoms!`, which generates
both the executable `beats` method and the specification `beat_spec`:

$$
\begin{aligned}
\mathsf{Sabab.Khafif} &= \texttt{/0},&
\mathsf{Sabab.Thaqil} &= \texttt{//},\\
\mathsf{Watad.Majmu} &= \texttt{//0},&
\mathsf{Watad.Mafruq} &= \texttt{/0/}.
\end{aligned}
$$

The checked contract on `Sabab::beats`, `Watad::beats` and `Unit::beats` is
that the returned slice equals the corresponding `beat_spec`, with length
between 2 and 3. The literal strings above are also checked by ordinary Rust
tests. There is no separate Verus theorem asserting pairwise distinctness of the
four rendered strings; the type structure supplies four constructors, and the
tests check the intended renderings.

The typed unit word of a unit is `beat_spec`. `watad_count` counts
`Unit::Watad` values, and `has_exactly_one_watad(units)` has the contract

$$
\mathsf{hasExactlyOneWatad}(u)
\iff
\mathsf{watadCount}(u,|u|)=1.
$$

This counts typed watads, not occurrences of the substring `//0` in a beat
string.

### 3.2 Named feet as derived circle windows

The five circle seeds are declared in `circle_seeds!`; `circle_word_spec(i)`
returns seed $i$ for $i<5$, and `circle_word(i)` has the contract that its
slice equals that specification and has length 2 to 9. `FOOT_AT` is the only
name-to-position table. For an attestation tuple
$a=(c,r,o,\ell)$, the derived typed window is

$$
\mathsf{windowWord}(a)[j]
=
\mathsf{circleWordSpec}(c)[(o+j+r)\bmod |\mathsf{circleWordSpec}(c)|]
\quad(0\le j<\ell).
$$

`foot_window(a)` requires $c<5$, $r<9$, $o\le 6$, $\ell\le 3$, and
returns exactly this window with true length $\ell$. `derive_foot_units`
requires the same bounds for every entry of `FOOT_AT` and ensures, for every
$k<11$,

$$
\mathsf{table}[k].1=\mathsf{FOOT\_AT}[k].3
\land
\mathsf{table}[k].0[0..\mathsf{table}[k].1]
=\mathsf{windowWord}(\mathsf{FOOT\_AT}[k]).
$$

`FootId::word_spec` is defined as
$\mathsf{windowWord}(\mathsf{FOOT\_AT}[f])$; it is not a second table.
`FootId::one_watad` proves, separately for all eleven variants, that
`watad_count(word_spec, |word_spec|) = 1`. `FootId::pattern` returns the same
window with length in $2..3$, the same `word_spec`, and exactly one watad.

The ordinary const function `foot_derivation_sane` additionally checks that the
eleven derived typed windows are pairwise distinct, have length 2 or 3, and
carry exactly one watad. It is a compile-time assertion, not a Verus theorem.
`foot_window_at` is an ordinary-Rust inverse used by the untrusted daira
segmentation generator; it has no Verus contract. `FOOT_AT` is attestation data:
the proofs derive the pattern from the chosen position, not that the position is
historically correct.

### 3.3 Rotation, permutation, composition and repetition

For a typed word $w$ of length $n$, define
$$
\mathsf{index}(n,r,i)=
\begin{cases}
r+i & i<n-r,\\
i-(n-r) & \text{otherwise},
\end{cases}
\qquad
\mathsf{rotated}(w,r)[i]=w[\mathsf{index}(|w|,r,i)].
$$
`rotate` requires no precondition and returns `None` unless $w$ is nonempty,
$r<|w|$ and $|w|\le\mathit{CAP}$; on success it returns exactly
`rotated(w,r)` and the true length. Invalid bounds or capacity are explicit
`None`, not truncation.

The checked rotation facts are:

- `permutation(n,r,i,j)` requires $n>0$, $0\le r<n$, $0\le i,j<n$. It
  ensures $0\le\mathsf{index}(n,r,i)<n$,
  $\mathsf{index}(n,r,i)=\mathsf{index}(n,r,j)\iff i=j$, and the inverse
  identity $\mathsf{index}(n,n-r,\mathsf{index}(n,r,i))=i$ (with $n-r$
  normalized to 0 when $r=0$).
- `identity(w)` ensures `rotated(w,0) = w`.
- `composition(w,a,b)` requires $|w|>0$ and $0\le a,b<|w|$. It ensures
  \[
  \mathsf{rotated}(\mathsf{rotated}(w,a),b)
  =
  \mathsf{rotated}(w,(a+b)\bmod |w|).
  \]
  This is two-fold cyclic addition; general associativity or multi-fold
  composition is not a separate theorem.
- `repeated(w,k)` is empty when $w$ is empty or $k=0$; otherwise it has
  length $|w|k$ and repeats $w$ by index modulo $|w|$. `repeat` returns
  `None` exactly when $|w|k>\mathit{CAP}$ (including checked multiplication
  failure), and on success returns exactly `repeated(w,k)`.

The total specification `rotated` has no range precondition; the theorems above
supply the needed bounds. `rotate` is the checked executable boundary.

### 3.4 Primitive and disjoint typed rotations

`circle_theorems::primitive(c,a,b)` requires $c<5$ and
$a,b<|\mathsf{circleWordSpec}(c)|$. It ensures
$$
\mathsf{rotated}(\mathsf{circleWordSpec}(c),a)
=
\mathsf{rotated}(\mathsf{circleWordSpec}(c),b)
\Longrightarrow a=b.
$$
Thus each circle has exactly its period many distinct typed rotations.

`circle_theorems::disjoint(c,d,a,b)` requires $c<5$, $d<5$, $c\ne d$,
$a<|\mathsf{circleWordSpec}(c)|$ and
$b<|\mathsf{circleWordSpec}(d)|$. It ensures the two typed rotations are
unequal. `circle_theorems::twenty_two()` ensures
$$
\sum_{c=0}^{4}|\mathsf{circleWordSpec}(c)|=22.
$$
Together these three Verus lemmas give 22 pairwise distinct **typed**
rotations. There is no single theorem with that name, and the proofs compare
positions up to index 8; extending the seed lengths would require revisiting
that bound.

The daira crate also has const-evaluated `theorem1_primitivity`,
`theorem2_disjointness`, `theorem3_names`, `theorem4_counts` and
`theorem5_segmentation`, each guarded by `const _: () = assert!(...)`. These are
compile-time checks over the actual tables, not Verus proofs. `theorem2` compares
`Unit::same`, i.e. typed unit identity.

Typed disjointness does **not** imply disjointness of rendered beat strings.
For example, the typed sequences
$$
[\mathsf{S},\mathsf{W}] \quad\text{and}\quad [\mathsf{M},\mathsf{S}]
$$
both render as `/0//0`. Likewise `MustafIlun` (`SSW`) and
`MustafIlunMafruq` (`SMS`) share `/0/0//0`, and `Failatun` (`SWS`) and
`FailatunMafruq` (`MSS`) share `/0//0/0`. The circle theorems deliberately
compare `Seq<Unit>`, not `Seq<Beat>`. For the current 22 raw rotation words a
direct enumeration finds 22 distinct beat strings, but that is not a proved or
const-asserted consequence of the disjointness theorems, so this document does
not claim it as one.

### 3.5 Layout tiling and basis binding

For a circle word $w$, rotation $r$, repetitions $k$, and foot sequence
$F$, define
$$
\mathsf{expected}(w,r,k)=\mathsf{repeated}(\mathsf{rotated}(w,r),k).
$$
`layout::tiling(input,feet,i,at)` requires the remaining foot words to cover the
suffix exactly:
$$
\begin{cases}
\bot & i<0 \lor i>|feet| \lor at<0 \lor at>|input|,\\
at=|input| & i=|feet|,\\
\mathsf{prefix}(input,at,feet[i].word\_spec)
\land
\mathsf{tiling}(input,feet,i+1,at+|feet[i].word\_spec|)
& i<|feet|.
\end{cases}
$$
`layout::valid(circle,rot,reps,feet)` requires $circle<5$,
$rot<|\mathsf{circleWordSpec}(circle)|$,
$reps>0$, $|\mathsf{circleWordSpec}(circle)|\cdot reps\le
\mathtt{usize::MAX}$, and
$\mathsf{tiling}(\mathsf{expected}(\mathsf{circleWordSpec}(circle),rot,reps),
feet,0,0)$. `layout::check` returns exactly this predicate and uses checked
multiplication. This is exact typed-unit tiling: full coverage, no gaps, ending
at the input length. It does not require distinct foot ids or a unique tiling;
the daira generator is the untrusted chooser of a tiling.

A circle-bound profile additionally binds the source slots to the chosen feet.
For a slot sequence $S$ and feet $F$,
$$
\mathsf{halfBound}(S,F)
\iff
|S|\le |F|
\land
\forall i<|S|:\;S[i].\mathit{units}=F[i].\mathit{word\_spec}.
$$
$$
\mathsf{basisBound}(P,F)
\iff
\forall i<|P.\mathit{forms}|:\;
\mathsf{halfBound}(P.\mathit{forms}[i].\mathit{sadr},F)
\land
\mathsf{halfBound}(P.\mathit{forms}[i].\mathit{ajuz},F).
$$
`check_half` and `check_basis` return exactly these predicates. The binding is
prefix-only ($|S|\le|F|$) and vacuous for empty slots; trailing feet beyond
the retained source slots are not constrained by `basisBound`. `Profile::new`
sets `origin=None`; `new_bound` requires both `layout::valid` and
`basisBound`, and the `Profile` type invariant permanently records them for a
circle-bound profile. `origin()` re-exposes the checked origin. The binding is
typed units, not surface beats, so equal-surface feet cannot be substituted for
the attested base.

### 3.6 Morphology: original coordinates and watad protection

Expansion assigns every letter its original unit owner, offset and a
`Letter.sabab` flag:
$$
\mathsf{unitLetters}(u,o)[j]
=
(\mathsf{unitWord}(u)[j],o,j,\mathsf{isSabab}(u)).
$$
`expanded(units,n)` concatenates these letter sequences in original unit order.
It is not a beat-string split: equal surface strings with different unit
structure have different ownership.

`change_valid(letter,action,expected,scope)` is:
$$
\begin{aligned}
&\mathsf{action}=\mathsf{Keep} \lor \bigl(
\mathsf{expected}=\mathsf{letter.beat}
\land(\mathsf{action}\ne\mathsf{Silence}\lor\mathsf{expected}=1)\\
&\qquad\land
(\mathsf{scope}\ne\mathsf{Zihaf}
\lor(\mathsf{letter.sabab}\land\mathsf{letter.offset}=1))\bigr).
\end{aligned}
$$
`eligible(program,position)` is the separate scope gate: `Zihaf` requires an
empty suffix; `Terminal` requires a terminal position; `Initial` requires an
initial position and empty suffix; `Licensed` adds no further restriction.
`valid(units,program,position)` additionally requires nonempty units, the
required final unit if any, actions and expected arrays of exactly the expanded
letter length, every `change_valid`, and suffix entries at most 1.

`changed(letters,actions,n)` applies the actions in original order: `Keep` emits
the original beat, `Delete` emits nothing, `Silence` emits 0. `apply` returns
`Err` only when `valid` fails; on `Ok` it returns exactly
$$
\mathsf{changed}\bigl(\mathsf{expanded}(units,|units|),actions,|actions|\bigr)
+\mathit{suffix}.
$$
Earlier deletions never shift the coordinate of a later action. This is why a
normalizer cannot implement a later edit by indexing the already-shortened
surface string.

`watad_is_protected(units,p,position,i)` requires `valid(units,p,position)`,
`p.scope == Zihaf`, $0\le i<|\mathsf{expanded}(units,|units|)|$, and
$\neg\mathsf{expanded}(units,|units|)[i].\mathit{sabab}$. It ensures
`p.actions[i] == Keep`. Under the current `Unit` type, a non-sabab letter is
exactly a letter owned by a `Watad`; the theorem does not extend to `Licensed`
or other scopes. It is a statement about the checked program, not about a
historical permission.

## 4. Adjacency and selected-path semantics

### 4.1 Source compatibility

A source program `p` has a sequence of actions, one per original letter. The
primitive compatibility relation is

$$
\mathsf{compatible}(p,q,l,r)
\iff
l<|p.\mathit{actions}| \;\land\; r<|q.\mathit{actions}|
\;\land\;
\neg\bigl(p.\mathit{actions}[l]=\mathsf{Delete}
\land q.\mathit{actions}[r]=\mathsf{Delete}\bigr).
$$

This is deliberately directional. It says only that the named deletion in the
left program and the named deletion in the right program do not coexist. It does
not invent a converse rule.

A rule is `Rule { left, right, left_letter, right_letter }`. Its scope condition
is

$$
\mathsf{ruleScopeOk}(H,R)
\iff
R.\mathit{left}<|H| \land R.\mathit{right}<|H|
\land \bigl(\forall i<|H[R.\mathit{left}].\mathit{programs}|:\;
  R.\mathit{leftLetter}<|H[R.\mathit{left}].\mathit{programs}[i].\mathit{actions}|\bigr)
\land \bigl(\forall j<|H[R.\mathit{right}].\mathit{programs}|:\;
  R.\mathit{rightLetter}<|H[R.\mathit{right}].\mathit{programs}[j].\mathit{actions}|\bigr).
$$

The all-pairs source predicate is

$$
\mathsf{validRule}(H,R)
\iff
\mathsf{ruleScopeOk}(H,R)
\land \forall i<|H[R.\mathit{left}].\mathit{programs}|:\;
\forall j<|H[R.\mathit{right}].\mathit{programs}|:\;
\mathsf{compatible}\bigl(H[R.\mathit{left}].\mathit{programs}[i],
H[R.\mathit{right}].\mathit{programs}[j],
R.\mathit{leftLetter},R.\mathit{rightLetter}\bigr).
$$

`adjacency::valid` is the conjunction of `validRule` over a rule list. This is
the reference-expander check: every alternative in the left slot is compatible
with every alternative in the right slot.

### 4.2 Compact flags

Production does not expand all alternatives. For a program list `programs` and
an original-letter index `letter`, the compact flag vector is

$$
\begin{aligned}
\mathsf{deletionFlags}(\mathit{programs},\mathit{letter})[i]
\iff{}&
\mathit{letter}<|\mathit{programs}[i].\mathit{actions}|\\
&\land\mathit{programs}[i].\mathit{actions}[\mathit{letter}]
=\mathsf{Delete}.
\end{aligned}
$$

A compact rule stores the two flag vectors. The compiled correspondence is

$$
\mathsf{compiledRule}(H,R,C)
\iff
R.\mathit{left}<|H|\land R.\mathit{right}<|H|
\land C.\mathit{left}=R.\mathit{left}
\land C.\mathit{right}=R.\mathit{right}
\land C.\mathit{leftDeleted}
=\mathsf{deletionFlags}(H[R.\mathit{left}].\mathit{programs},R.\mathit{leftLetter})
\land C.\mathit{rightDeleted}
=\mathsf{deletionFlags}(H[R.\mathit{right}].\mathit{programs},R.\mathit{rightLetter}).
$$

`structured::compiled_rules` additionally requires the source and compact rule
lists to have the same length:
$$
\mathsf{compiledRules}(H,Rs,Cs)
\iff
|Rs|=|Cs|
\land\forall k<|Rs|:\;\mathsf{compiledRule}(H,Rs[k],Cs[k]).
$$
The key point is that the flags are indexed by **source alternative**, not by
surface word. Two alternatives with the same beat word remain distinct.

A selected step sequence `steps` satisfies one compact rule exactly when

$$
\mathsf{permitsRule}(C,\mathit{steps})
\iff
C.\mathit{left}<|\mathit{steps}|
\land C.\mathit{right}<|\mathit{steps}|
\land \mathit{steps}[C.\mathit{left}].\mathit{choice}<|C.\mathit{leftDeleted}|
\land \mathit{steps}[C.\mathit{right}].\mathit{choice}<|C.\mathit{rightDeleted}|
\land
\neg\bigl(
C.\mathit{leftDeleted}[\mathit{steps}[C.\mathit{left}].\mathit{choice}]
\land
C.\mathit{rightDeleted}[\mathit{steps}[C.\mathit{right}].\mathit{choice}]
\bigr).
$$

`choice::permits` conjoins this over all rules. This is the selected-path
semantics: conflicting unselected alternatives may remain in the same slot.

### 4.3 The checked bridge

The source-to-compact bridge is not an axiom. `structured::compile_deletion_flags`
has the contract

$$
\mathsf{out}=\mathsf{deletionFlags}(\mathit{programs},\mathit{letter}),
$$

and `structured::compile_rule` returns a compact rule only when
`ruleScopeOk` holds, with the exact flag vectors. `structured::compile_rules`
conjoins `compiledRule` over the rule list. The proof
`structured::rules_projection_valid` then establishes `choice::valid` for the
compiled profile from source scope and the compiled correspondence.

The theorem `structured::projection_equivalent` states

$$
\mathsf{valid}_{\mathrm{structured}}(P_{\mathrm{source}})
\iff
\mathsf{valid}_{\mathrm{verse}}(P_{\mathrm{lowered}})
$$

under `projected(P_source,P_lowered)`. Here the source is the **unexpanded**
`structured::Profile` of source forms, source slots, source programs and source
rules; the target is the lowered `verse::Profile`. The theorem does not mention
`expand::assignments`, `reference_expand_forms`, or the expanded reference
representation.

`projected` requires the same profile id and binding policy, the same number of
forms, and, for every form, the same context, initial-only flag and darb class;
it requires source rule scope to hold; it requires every source rule to be
compiled to the exact deletion flags (`compiled_rules`); and it requires every
source slot to be lowered index-for-index with the exact output word
(`projected_slots`). The proof composes two facts:

- `slots_equivalent`: source slot validity is equivalent to lowered slot
  validity, because every output word is exactly the normalized source program
  output and the alternatives are preserved index-for-index;
- `rules_projection_valid`: compiled rule flags are well-formed whenever source
  rule scope holds.

This is a machine-checked contract about representation compaction. It does
**not** say that a particular historical rule is correct, and it is not a proof
that the compact selected-path language equals the reference-expanded language.

The latter equivalence is a semantic corollary supported by finite oracle
evidence, not a Verus theorem. The test
`compact_and_reference_languages_agree_exhaustively` constructs an independent
reference-expanded profile for the four constrained built-ins and compares
recognition for every enumerated sadr/ajuz half-path. The property test
`arbitrary_source_rules_lower_to_the_same_selected_language` compares the
compact recognizer with an independent `source_oracle` for generated source
rules. `reference_expand_forms` is retained as an independent test oracle, not
used by production recognition. These tests support language equivalence for
their finite cases; they do not replace a general theorem.

## 5. Normalization and lowering

The named normalizer is separate from the compact language. Its key contract is

$$
\mathsf{normalizeSpec}(\mathit{base},\mathit{changes})
=\mathsf{Ok}(\mathsf{programView}(p))
\land
\mathsf{eligible}(p,\mathit{position})
\Longrightarrow
\mathsf{valid}(\mathit{base}.\mathit{word},p,\mathit{position}).
$$

The executable normalizer constructs simultaneous actions in **original
coordinates**. An action is `Keep`, `Delete` or `Silence`, and an optional suffix
is recorded separately. Applying a program walks the original word once; a
deletion at letter 2 does not shift the target of a later edit at letter 4.

The lowering used by compact profiles is `structured::compile`:

$$
\mathsf{output}(S,i)
=
\mathsf{changed}\bigl(
\mathsf{expanded}(S.\mathit{units},|S.\mathit{units}|),
p_i.\mathit{actions},|p_i.\mathit{actions}|\bigr)
+\;p_i.\mathit{suffix}.
$$

The contract says that a successful compile produces one output word per source
program, in source order, with exactly this output; an invalid program rejects
the slot. `projected_slots` requires the same for every slot. This is how
distinct alternatives with equal surfaces survive lowering.

## 6. Prefix, span certificates and exact recognition

For an input sequence `x` and a word `w`, the prefix relation is

$$
\mathsf{prefix}(x,\mathit{at},w)
\iff
0\le \mathit{at}\le |x|
\land \mathit{at}+|w|\le |x|
\land \forall j<|w|:\; w[j]=x[\mathit{at}+j].
$$

A span step is `(choice,start,end)`. All guards are stated before the cases,
so a trace cannot carry surplus or missing steps:

$$
\mathsf{trace}(H,x,i,\mathit{at},\mathit{steps})
\iff
\begin{cases}
\bot
& \begin{aligned}
&i<0 \lor i>|H| \lor \mathit{at}<0 \lor \mathit{at}>|x|\\
&\quad\lor |\mathit{steps}|\ne |H|-i,
\end{aligned}\\
\mathit{at}=|x|
& i=|H| \quad(\text{then } \mathit{steps}=\epsilon),\\
\begin{aligned}
&\exists s,t,w:\;
\mathit{steps}=s::t\\
&\quad\land 0\le s.\mathit{choice}<|H[i].\mathit{words}|\\
&\quad\land w=H[i].\mathit{words}[s.\mathit{choice}]\\
&\quad\land s.\mathit{start}=\mathit{at}\\
&\quad\land \mathsf{prefix}(x,\mathit{at},w)\\
&\quad\land s.\mathit{end}=\mathit{at}+|w|\\
&\quad\land \mathsf{trace}(H,x,i+1,s.\mathit{end},t)
\end{aligned}
& i<|H|.
\end{cases}
$$

At termination the length guard already forces `steps` to be empty; the base
case checks the input end. In the recursive case the choice is in range, the
start is the current offset, the selected word matches the input there, the end
is exactly the word's end, and the tail continues from that end. The step count
is exactly the number of remaining slots.

The certificate checker `language::check` returns exactly this relation. It does
not repair intervals, skip a slot, accept a prefix of the input, or ignore extra
steps. The recursive recognizer `language::recognize_from` has the contract

$$
\mathsf{answer}\ne\mathsf{None}
\iff
\mathsf{derives}(H,x,i,\mathit{at}),
$$

and on success returns a trace. The loop invariant records that every earlier
alternative was tried and failed; the recursive call covers the remaining slots.
Thus the recognizer is sound and complete for the declared finite language, with
no search budget or truncation.

The compact rule layer adds `choice::permits` to the trace:

$$
\mathsf{choiceTrace}(H,R,x,\mathit{steps})
\iff
\mathsf{trace}(H,x,0,0,\mathit{steps})
\land \mathsf{permits}(R,\mathit{steps}).
$$

`choice::check` returns exactly this predicate. The independent derivation
relation `choice::derives_from` existentially chooses one alternative per slot,
requires the prefix at each step, and requires `permits` on the whole selected
sequence. `choice::trace_is_derivation` proves trace implies derivation;
`choice::recognize` returns `Some` exactly when the derivation relation holds.
This is the formal core of “certificate replay is separate from acceptance.”

## 7. Verse semantics and certificate identity

A scope is `(context, initial)`. A certificate is legal for a profile `P`, scope
`scope`, and input halves when

$$
\mathsf{legal}(P,\mathit{scope},\mathit{sadr},\mathit{ajuz},C)
\iff
\begin{aligned}
&\mathsf{valid}(P)
\land C.\mathit{profile}=P.\mathit{id}
\land C.\mathit{context}=\mathit{scope}.\mathit{context}\\
&\land C.\mathit{form}<|P.\mathit{forms}|
\land P.\mathit{forms}[C.\mathit{form}].\mathit{context}=\mathit{scope}.\mathit{context}\\
&\land (\neg P.\mathit{forms}[C.\mathit{form}].\mathit{initialOnly}
      \lor \mathit{scope}.\mathit{initial})\\
&\land \mathsf{choiceTrace}(P.\mathit{forms}[C.\mathit{form}].\mathit{sadr},
       P.\mathit{forms}[C.\mathit{form}].\mathit{sadrRules},\mathit{sadr},C.\mathit{sadr})\\
&\land \mathsf{choiceTrace}(P.\mathit{forms}[C.\mathit{form}].\mathit{ajuz},
       P.\mathit{forms}[C.\mathit{form}].\mathit{ajuzRules},\mathit{ajuz},C.\mathit{ajuz}).
\end{aligned}
$$

The certificate carries its own profile id and context, but `verse::check` does
not trust them: it compares them with the externally selected profile and scope,
checks the form index, the form context, the initial-only condition, and both
half traces under the actual rules.

The verse derivation relation is

$$
\mathsf{verseDerives}(P,\mathit{scope},\mathit{sadr},\mathit{ajuz},\mathit{filter})
\iff
\mathsf{valid}(P)
\land \exists f<|P.\mathit{forms}|:\;
P.\mathit{forms}[f].\mathit{context}=\mathit{scope}.\mathit{context}
\land (\neg P.\mathit{forms}[f].\mathit{initialOnly}\lor\mathit{scope}.\mathit{initial})
\land \mathsf{classMatches}(P.\mathit{forms}[f].\mathit{class},\mathit{filter})
\land \mathsf{choiceDerives}(P.\mathit{forms}[f].\mathit{sadr},\ldots,\mathit{sadr})
\land \mathsf{choiceDerives}(P.\mathit{forms}[f].\mathit{ajuz},\ldots,\mathit{ajuz}).
$$

`verse::recognize_form` returns `Some` exactly when the form-specific derivation
holds, and every returned certificate is legal. `verse::certificate_derives`
proves that legality implies the form-specific derivation. `verse::recognize`
adds the optional darb-class filter and returns an error for an invalid profile;
an invalid profile is never reported as a negative recognition result.

## 8. Poem semantics

A poem input is a finite nonempty sequence of verse inputs. For a fixed form
index $f$, define

$$
\begin{aligned}
&\mathsf{formDerives}(P,\mathit{scope},\mathit{sadr},\mathit{ajuz},f)
\iff
\mathsf{valid}(P)
\land 0\le f<|P.\mathit{forms}|\\
&\quad\land P.\mathit{forms}[f].\mathit{context}=\mathit{scope}.\mathit{context}
\land(\neg P.\mathit{forms}[f].\mathit{initialOnly}
\lor\mathit{scope}.\mathit{initial})\\
&\quad\land \mathsf{choiceDerives}(P.\mathit{forms}[f].\mathit{sadr},\ldots,\mathit{sadr})
\land \mathsf{choiceDerives}(P.\mathit{forms}[f].\mathit{ajuz},\ldots,\mathit{ajuz}).
\end{aligned}
$$

This is `verse::form_derives`. It fixes the form and has **no** class-filter
parameter. `verse::derives(...,filter)` is a different existential relation that
chooses the form itself and applies `class_matches` inside that existential; it
is not used with a separately recorded $f$.

The poem derivation relation is recursive over the input sequence and carries
the whole prefix history:

$$
\mathsf{poemDerives}(P,I,\mathit{Req},\mathit{at},\mathit{filter},\mathit{selected})
\iff
\begin{cases}
\bot & \mathit{at}<0 \lor \mathit{at}>|I|,\\
\mathsf{satisfied}(\mathit{Req},\mathit{selected}) & \mathit{at}=|I|,\\
\exists f<|P.\mathit{forms}|:\;
\mathsf{classMatches}(P.\mathit{forms}[f].\mathit{class},\mathit{filter})\\
\quad\land \mathsf{formDerives}(P,\{\mathit{context}=I[\mathit{at}].\mathit{context},
\mathit{initial}=(\mathit{at}=0)\},I[\mathit{at}].\mathit{sadr},
I[\mathit{at}].\mathit{ajuz},f)\\
\quad\land \mathsf{poemDerives}(P,I,\mathit{Req},\mathit{at}+1,
\mathsf{nextFilter}(P,\mathit{filter},f),\mathit{selected}\cdot f)
& \mathit{at}<|I|.
\end{cases}
$$

The class filter is the mechanism for whole-poem darb-class binding. For
`FixedDarbClass`, the first selected form sets the filter and every later form
must match it. For `Unconstrained`, the filter remains `None`. This is captured
by

$$
\mathsf{nextFilter}(P,\mathit{filter},f)
=
\begin{cases}
\mathsf{Some}(P.\mathit{forms}[f].\mathit{class})
& P.\mathit{binding}=\mathsf{FixedDarbClass}\land \mathit{filter}=\mathsf{None},\\
\mathit{filter} & \text{otherwise}.
\end{cases}
$$

A requirement system is satisfied when every group contains at least one selected
form index:

$$
\mathsf{satisfied}(\mathit{Req},\mathit{selected})
\iff
\forall g<|\mathit{Req}|:\;
\exists i<|\mathit{Req}[g]|:\;
\mathit{Req}[g][i]\in\mathit{selected}.
$$

The poem certificate trace requires one legal certificate per verse, the same
class discipline, and the same requirement satisfaction. The search function has
the contract

$$
\mathsf{valid}(P)\land 0\le \mathit{at}\le|I|
\Longrightarrow
\Bigl(
\mathsf{search}(P,I,\mathit{Req},\mathit{at},\mathit{filter},\mathit{selected})
.\mathsf{isSome}()
\iff
\mathsf{poemDerives}(P,I,\mathit{Req},\mathit{at},\mathit{filter},\mathit{selected})
\Bigr).
$$
Separately, under the same preconditions, every returned certificate satisfies
the trace relation:
$$
\forall C:\;
\mathsf{search}(P,I,\mathit{Req},\mathit{at},\mathit{filter},\mathit{selected})
=\mathsf{Some}(C)
\Longrightarrow
\mathsf{trace}(P,I,\mathit{Req},\mathit{at},\mathit{filter},\mathit{selected},C).
$$

The loop invariant records that every earlier form index was tried and failed;
on success the result is a trace. `obligations::trace_is_derivation` proves that
a certificate trace implies the derivation relation. For nonempty input,
`obligations::recognize` returns `Some` exactly when the derivation relation
holds; empty input returns `Err(EmptyPoem)` even though the terminal derivation
relation is satisfied, and an invalid profile returns `Err(InvalidProfile)`.

The poem layer is therefore an existential joint assignment problem: choose one
form per verse, one alternative per slot in each half, satisfy every selected
rule, keep one darb class where required, and satisfy every witness group
somewhere in the poem. Backtracking is exact; there is no greedy commitment to
the first equal-surface reading.

## 9. Compact catalog storage, deduplication and witness membership

### 9.1 What is verified

The verified compact layer starts from a `structured::Profile`: source forms,
source slots, source programs, and source rules. It proves:

- `compile` preserves every source alternative index and its output word;
- `projected_slots` relates every source slot to every lowered slot;
- `compiled_rule` equates each source rule to its deletion flags;
- `projection_equivalent` equates source and lowered validity.

This is the verified lowering. It is independent of how the source forms were
constructed.

### 9.2 What is ordinary Rust

The built-in catalog is assembled in `crates/lugha/src/catalog.rs`. This is
ordinary Rust, outside the Verus boundary. It constructs the source forms,
applies licences, removes exact duplicate rows, rebuilds Wafir's requirement, and
passes the result to `MeterProfile::new`. The deduplication helper is not proved;
it is tested and reviewed.

For exact duplicate removal, two rows may merge only when all of the following
hold:

- every semantic form field is equal (`feet`, `context`, `initial_only`, class,
  constraints, sadr recipes, ajuz recipes);
- their membership in **every** requirement group is identical.

The second condition is part of semantic identity. Suppose rows `A` and `B` are
identical as forms but the requirement groups are `[[0],[1]]`. If the rows were
merged without considering membership, the groups would become `[[0],[0]]`. A
one-verse poem selecting the merged row would then satisfy both groups, although
the original profile required two distinct witnesses. The helper therefore keeps
`A` and `B` distinct when their membership signatures differ. A focused ordinary
Rust regression demonstrates one-verse rejection and two-witness acceptance for
the different-membership case; a same-membership duplicate still merges.

Wafir's two duplicate tasri rows were exact duplicates in the built-in data and
were removed. Wafir's final requirement is rebuilt after deduplication, so the
built-in Wafir behavior does not depend on the removed rows' membership. The
generic helper remains membership-safe for future source data.

### 9.3 Built-in satisfiability versus external construction

The built-in catalog's requirement groups are satisfiable, and the compact
language is exhaustively compared with the reference expander for the
constrained meters. That is test/oracle evidence, not a Verus theorem about the
source tables.

For an **external** profile, `MeterProfile::new` validates layout, requirement
group nonemptiness and index ranges, but it deliberately does not enumerate all
paths at construction. Consequently, an external profile whose requirement group
has no compatible selected path can be accepted as `Ok(profile)`, and
`recognize_poem` can then return `Ok(None)`. This is a construction-time
validation boundary, not a soundness failure: no certificate is accepted. For a
constrained meter, the reference expander rejects the same profile with
`Requirement` when the affected group's remapped output union is empty. If a
group still contains a satisfiable form, the expander does not reject; the
unsatisfiable form simply contributes no branches. The built-in catalog does not
exhibit the all-empty case. This distinction must not be hidden behind a blanket
claim that every accepted profile has a satisfiable required path.

## 10. Worked derivations

### 10.1 A legal Madid path and a mu'aqabah-violating path

Consider Madid form 0, context 0, three retained feet in the sadr. Its first
source rule is

$$
R = (\mathit{left}=0,\mathit{leftLetter}=6,\mathit{right}=1,\mathit{rightLetter}=1),
$$

using zero-based letter coordinates: original letter 7 of foot 0 and original
letter 2 of foot 1. In the traditional terminology, this is the within-hemistich
mu'aqabah direction “Kaf of the preceding foot cannot coexist with Khabn of the
following foot.”

A legal unchanged path is choices `[0,0,0]`. Its sadr beats are

$$
\underbrace{/0//0/0}_{7}\;\underbrace{/0//0}_{5}\;\underbrace{/0//0/0}_{7},
$$

with spans `[0,7)`, `[7,12)`, `[12,19)`. The same form's unchanged ajuz path is
also `/0//0/0 /0//0 /0//0/0` (the CLI prints the concatenated sadr
`/0//0/0/0//0/0//0/0`), so the pair is accepted by `verse::check`.

A source path that violates only `R` is choices `[2,1,0]`:

- foot 0, choice 2 applies `Kaf`, deleting original letter 7;
- foot 1, choice 1 applies `Khabn`, deleting original letter 2;
- foot 2, choice 0 is unchanged.

Its sadr beats are

$$
\underbrace{/0//0/}_{6}\;\underbrace{///0}_{4}\;\underbrace{/0//0/0}_{7},
$$

with spans `[0,6)`, `[6,10)`, `[10,17)`. This is a well-formed source path, but
it is not a permitted derivation because both deletions named by `R` occur on
the selected path.

### 10.2 A matching-input certificate rejected solely by the rule

Take the illegal path above and its exact spans. Let `slots` be the lowered sadr
slots, `x` the 17-beat input, and `steps` the three spans. The unconstrained
checker accepts the certificate:

$$
\mathsf{choice.check}(\mathit{slots},[],x,\mathit{steps})=\mathsf{true}.
$$

The rule-aware checker rejects it:

$$
\mathsf{choice.check}(\mathit{slots},[C_R],x,\mathit{steps})=\mathsf{false}.
$$

The input, words and spans all match; the only failing premise is
`permitsRule(C_R,steps)`, because

$$
C_R.\mathit{leftDeleted}[2]=\mathsf{true}
\quad\text{and}\quad
C_R.\mathit{rightDeleted}[1]=\mathsf{true}.
$$

If the legal ajuz path is supplied, the whole verse certificate still fails
`verse::check` solely because the sadr half violates `R`. This is the intended
separation: a certificate can be structurally and lexically correct while being
illegal under the selected source rule. The production tests
`production_replay_rejects_a_forbidden_sadr_correlation` and
`production_replay_rejects_a_forbidden_ajuz_correlation` exercise both halves.

### 10.3 A witness-membership derivation

Consider a synthetic profile with two forms `0` and `1` and requirements
`[[0],[1]]`. A one-verse poem selecting form `0` satisfies group `[0]` but not
group `[1]`, so the poem derivation fails at the end. A two-verse poem selecting
forms `0` and `1` satisfies both groups, so it succeeds. This is exactly

$$
\forall g<2:\;\exists i<|\mathit{Req}[g]|:\;
\mathit{Req}[g][i]\in\mathit{selected}.
$$

Now suppose forms `0` and `1` are identical as forms but have different
membership. Merging them would collapse the selected indices and could let one
verse satisfy both groups. The ordinary Rust dedup helper prevents this by
including the membership signature in identity. The verified lowering does not
prove the helper correct; the helper is tested, and the verified layer only
receives the resulting distinct forms and remapped groups.

### 10.4 Wafir's shortened-arud witness

After exact duplicate removal the Wafir profile has eight forms and one
requirement group `[0,1,2,5,6]`. Rows 0 and 5 are separate three-foot
fixed-class forms (contexts 0 and 1, both class 0); they belong to the group but
are not evidence that a full Wafir arud is unaltered. For a two-foot poem the
group requires at least one selected shortened arud without Asb.

The final two-foot rows show exactly which half supplies that witness. Row 1
(context 0, class 1) has unaltered sadr and ajuz endings `//0///0` and
`//0///0`. Row 2 (context 0, class 2) has the same unaltered arud `//0///0` but
an altered darb `//0/0/0`; it is in the group. Row 3 (context 0, class 1) has an
altered arud `//0/0/0` but sound body alternatives and the sound darb
`//0///0`; it is not in the group. Rows 4 and 7 are also excluded. Thus a sound
body foot or a sound darb cannot substitute for the shortened arud: row 3 has
both and still fails, while row 2 passes with an altered darb because its arud
is unaltered.

The fixture in `crates/lugha/tests/catalog.rs` exercises these final indices. A
single row-3 verse is rejected; two row-3 verses are also rejected, because
every selected form remains outside the group. A poem containing row 1 is
accepted, and a single row-2 verse is accepted, confirming that the witness is
the selected shortened sadr ending, not the body or the darb.

## 11. Loops, recursion and proof erasure

The executable algorithms are written with explicit loop invariants and
recursive decreases clauses. Some representative invariants:

- `language::recognize_from`: `c <= |words|` and, for every earlier index `k`,
  there is no prefix match followed by a successful recursive derivation. This
  is the completeness invariant: the loop cannot skip a viable alternative.
- `language::check_from`: its recursive contract fixes the step count to the
  number of remaining slots, and each step's start/end intervals are consecutive
  and exact. This is the no-repair certificate invariant.
- `choice::recognize_from`: the same prefix invariant plus the selected-step
  history; the recursive success branch proves `trace` for the whole sequence.
- `obligations::search`: for every earlier form index `j`, there is no
  class-compatible form derivation followed by a successful recursive poem
  derivation. This is the poem-level no-skip invariant.
- `structured::compile_deletion_flags`: every output flag equals the action test
  at the original letter index.
- `profile::Profile::invariant`: the lowered profile is valid, the source and
  lowered profiles are related by `projected`, and a circle-bound profile has a
  valid layout and basis. `new_bound` establishes this invariant; public methods
  use `use_type_invariant` before exposing `kernel` or recognition. `source()`
  returns the authority directly and needs no invariant call.

The `verus!` macro is the official proof-erasure boundary. Ghost/proof code is
removed from the executable Rust build, while the executable control flow and
data are the same code the verifier reasons about. The trusted base includes the
pinned Verus/Z3 toolchain, the preverified `vstd` specifications and the proof
erasure macro. The report does not claim that ordinary Rust adapters are verified
by this mechanism.

## 12. Verification evidence and mutation sensitivity

The pinned driver verifies and natively compiles the annotated types, daira and
checker sources, then runs the lugha integration suites. At this checkpoint,
all 294 obligations passed, the driver integration stage passed 103 tests, and
the complete workspace passed 149 tests/examples. Eight Python tests exercise
the mutation-diagnostic classifier. These are distinct checks, not additive
counts of independent mathematical theorems.

All 31 controls passed: each mutation caused a proof-obligation failure at its
declared diagnostic kind, source file and obligation fragment. Compilation
failures, timeouts and warnings do not satisfy the control gate. The controls
cover authority, input bounds, ownership, normalization, expansion, catalog
preservation, requirement validity, and selected-path recognition and replay.

Three concrete examples are:

- `source-rule-flags` fails the exact deletion-flag loop invariant in
  `structured::compile_deletion_flags`;
- `verse-sadr-replay` fails the legality postcondition of `verse::check` when
  its executable sadr rule guard is bypassed;
- `verse-ajuz-replay` fails the same legality postcondition when the ajuz guard
  is independently bypassed.

Mutation controls demonstrate sensitivity to particular implementation defects.
They do not establish that a specification covers every intended property.
The exhaustive compact/reference comparisons and source fixtures provide
separate ordinary-Rust evidence about the selected catalog. The Wafir
row-identity fingerprint reflects its reviewed duplicate removal; it is not a
claim that certificate coordinates stayed unchanged.

## 13. Trust boundaries and non-claims

The following are outside the machine-checked proof boundary:

- historical permission, source adequacy and the philological interpretation of
  a named rule;
- meter-specific catalog table construction and licences in ordinary Rust;
- exact duplicate removal, requirement rebuilding and witness-membership
  signatures in ordinary Rust;
- high-level adapter assembly and CLI display;
- caller-supplied claims about tasri, ridf or rhyme evidence;
- Arabic orthography conversion and reading ambiguity (Layer 2);
- performance limits of exact search on long, ambiguous or unsatisfiable inputs;
- hosted CI execution.

The compact language proof is relative to declared source programs, alternatives
and rules. The poem proof is relative to the declared requirement groups and
binding policy. The built-in catalog is a versioned reading, not a universal
historical oracle.

## 14. Source map

- [`../crates/types/src/lib.rs`](../crates/types/src/lib.rs): typed units,
  primitive words, circle seeds, foot windows and the one-watad contracts.
- [`../crates/types/src/rotation.rs`](../crates/types/src/rotation.rs):
  rotation, permutation, composition and bounded repetition.
- [`../crates/types/src/circle_theorems.rs`](../crates/types/src/circle_theorems.rs):
  primitive periods, disjoint typed rotations and the 22-rotation count.
- [`../crates/daira/src/lib.rs`](../crates/daira/src/lib.rs): const-evaluated
  circle/name/segmentation assertions and the untrusted tiling generator.
- [`../crates/checker/src/layout.rs`](../crates/checker/src/layout.rs): typed
  tiling against the rotated/repeated circle.
- [`../crates/checker/src/profile.rs`](../crates/checker/src/profile.rs):
  circle-basis binding, immutable authority and the type invariant.
- [`../crates/checker/src/morphology.rs`](../crates/checker/src/morphology.rs):
  original-coordinate edits, scope/position gates and watad protection.
- [`../crates/checker/src/language.rs`](../crates/checker/src/language.rs):
  prefix, exact trace and finite recognition.
- [`../crates/checker/src/choice.rs`](../crates/checker/src/choice.rs):
  selected-path flags, permits, trace and derivations.
- [`../crates/checker/src/adjacency.rs`](../crates/checker/src/adjacency.rs):
  source compatibility, rule scope and all-pairs validation.
- [`../crates/checker/src/structured.rs`](../crates/checker/src/structured.rs):
  lowering, deletion flags, compiled rules and projection equivalence.
- [`../crates/checker/src/verse.rs`](../crates/checker/src/verse.rs): profile
  validation, legal certificates, form and verse recognition.
- [`../crates/checker/src/obligations.rs`](../crates/checker/src/obligations.rs):
  poem derivation, class filter, witness groups and exact search.
- [`../crates/checker/src/normalize.rs`](../crates/checker/src/normalize.rs):
  original-coordinate normalization and conditional validity.
- [`../crates/lugha/src/catalog.rs`](../crates/lugha/src/catalog.rs):
  ordinary-Rust catalog construction and exact deduplication.
- [`../crates/lugha/tests/compact_profiles.rs`](../crates/lugha/tests/compact_profiles.rs):
  compact/reference agreement, revision rejection and forbidden replay.
- [`../crates/lugha/tests/baseline_fingerprint.rs`](../crates/lugha/tests/baseline_fingerprint.rs):
  reviewed row-identity hashes, including the post-dedup Wafir hash.
- [`../formal/check-verus.py`](../formal/check-verus.py): pinned proof driver
  and 31 mutation controls.
- [`../formal/test_check_verus.py`](../formal/test_check_verus.py): classifier
  regression tests.
- [`../formal/README.md`](../formal/README.md): toolchain pin, commands and
  checked-scope inventory.
- [`TYPES.md`](TYPES.md), [`API.md`](API.md),
  [`../spec/catalog-compilation.md`](../spec/catalog-compilation.md),
  [`../spec/profiles/classical-v1.md`](../spec/profiles/classical-v1.md):
  technical type, layer, catalog-compilation and profile specifications.
