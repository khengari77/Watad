//! Generic catalog compilation: lower source recipes, select alternatives by
//! source index, copy metadata and enumerate source-form products. Meter-specific
//! recipe data stays in the `lugha` adapter.
use crate::adjacency::Rule;
use crate::named::Change;
use crate::normalize;
use crate::structured::FootSlot;
use crate::verse::DarbClass;
use alloc::vec;
use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;
use watad_types::FootId;

verus! {

/// A source form in raw recipe coordinates. This is the common data boundary
/// shared by the catalog adapter and the verified expansion core.
#[derive(Debug, Clone)]
#[verifier::allow(autoderive_clone_without_spec)]
pub struct Form {
    pub constraints: Vec<Rule>,
    /// Retained prefix of the circle-generated feet; no implicit truncation.
    pub feet: usize,
    pub context: u16,
    pub initial_only: bool,
    pub class: DarbClass,
    /// One set of allowed recipes per retained original foot.
    pub sadr: Vec<Vec<Vec<Change>>>,
    pub ajuz: Vec<Vec<Vec<Change>>>,
}

pub type BranchGroups = Vec<Vec<Form>>;
pub type BranchRanges = Vec<(usize, usize)>;
pub type CompiledGroups = (Vec<Form>, BranchRanges, Vec<Vec<usize>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Layout,
    Requirement,
    Recipe(normalize::Error),
}

pub open spec fn lowers(
    feet: Seq<FootId>,
    recipes: Seq<Vec<Vec<Change>>>,
    slots: Seq<FootSlot>,
) -> bool {
    slots.len() == recipes.len()
        && forall|i: int| 0 <= i < slots.len() ==>
            slots[i].units@ == feet[i].word_spec()
            && slots[i].programs@.len() == recipes[i]@.len()
            && forall|j: int| 0 <= j < slots[i].programs@.len() ==>
                normalize::normalize_spec(feet[i], recipes[i]@[j]@)
                    == Ok(crate::normalize::program_view(&slots[i].programs[j]))
}

/// Lower every retained source recipe against its original foot. Recipe order is
/// preserved exactly; no alternatives are dropped or deduplicated.
pub fn lower_slots(
    feet: &[FootId],
    recipes: &[Vec<Vec<Change>>],
) -> (answer: Result<Vec<FootSlot>, Error>)
    requires recipes.len() <= feet.len(),
    ensures match answer {
        Err(_) => true,
        Ok(slots) => lowers(feet@, recipes@, slots@),
    },
{
    let mut slots: Vec<FootSlot> = Vec::new();
    let mut i = 0;
    while i < recipes.len()
        invariant i <= recipes.len(), recipes.len() <= feet.len(),
            slots.len() == i,
            forall|a: int| 0 <= a < i ==>
                slots[a].units@ == feet[a].word_spec()
                && slots[a].programs@.len() == recipes[a]@.len()
                && forall|j: int| 0 <= j < slots[a].programs@.len() ==>
                    normalize::normalize_spec(feet[a], recipes[a]@[j]@)
                        == Ok(crate::normalize::program_view(&slots[a].programs[j])),
        decreases recipes.len() - i,
    {
        let base = feet[i];
        let mut programs = Vec::new();
        let mut j = 0;
        while j < recipes[i].len()
            invariant j <= recipes@[i as int]@.len(), i < recipes.len(),
                programs@.len() == j,
                forall|b: int| 0 <= b < j ==>
                    normalize::normalize_spec(base, recipes@[i as int]@[b]@)
                        == Ok(crate::normalize::program_view(&programs[b])),
            decreases recipes@[i as int]@.len() - j,
        {
            match normalize::compile(base, &recipes[i][j]) {
                Ok(program) => {
                    assert(normalize::normalize_spec(base, recipes@[i as int]@[j as int]@)
                        == Ok(crate::normalize::program_view(&program)));
                    programs.push(program);
                }
                Err(error) => return Err(Error::Recipe(error)),
            }
            j += 1;
        }
        let units = crate::adapter::encode_units(base);
        slots.push(FootSlot { units, programs });
        i += 1;
    }
    Ok(slots)
}

pub open spec fn selects(
    source: Seq<Vec<Vec<Change>>>,
    choice: Seq<usize>,
    selected: Seq<Vec<Vec<Change>>>,
) -> bool {
    source.len() == choice.len()
        && selected.len() == choice.len()
        && forall|i: int| 0 <= i < selected.len() ==>
            selected[i]@.len() == 1
            && selected[i]@[0]@ == source[i]@[choice[i] as int]@
}

/// Map selected source indices back to their exact raw recipes. Each output foot
/// carries one recipe chosen from the corresponding source foot.
pub fn select_recipes(
    source: &[Vec<Vec<Change>>],
    choice: &[usize],
) -> (answer: Result<Vec<Vec<Vec<Change>>>, Error>)
    ensures match answer {
        Err(_) => true,
        Ok(selected) => selects(source@, choice@, selected@),
    },
{
    if choice.len() != source.len() {
        return Err(Error::Layout);
    }
    let mut selected: Vec<Vec<Vec<Change>>> = Vec::new();
    let mut i = 0;
    while i < source.len()
        invariant i <= source.len(), choice.len() == source.len(),
            source.len() == source@.len(), choice.len() == choice@.len(),
            selected.len() == i,
            forall|a: int| 0 <= a < i ==>
                selected[a]@.len() == 1
                && selected[a]@[0]@ == source@[a]@[choice[a] as int]@,
        decreases source.len() - i,
    {
        let row = &source[i];
        let chosen = choice[i];
        if chosen >= row.len() {
            return Err(Error::Layout);
        }
        assert(row@ == source@[i as int]@);
        let one = vec![row[chosen].clone()];
        selected.push(one);
        i += 1;
    }
    Ok(selected)
}

pub open spec fn same_meta(a: &Form, b: &Form) -> bool {
    a.constraints@ == b.constraints@
        && a.feet == b.feet
        && a.context == b.context
        && a.initial_only == b.initial_only
        && a.class == b.class
}

/// One output form is the source form with each foot replaced by the selected
/// raw recipe. An absent ajuz stays absent.
pub open spec fn expanded_form(
    source: &Form,
    sadr_choice: Seq<usize>,
    ajuz_choice: Seq<usize>,
    out: &Form,
) -> bool {
    same_meta(source, out)
        && selects(source.sadr@, sadr_choice, out.sadr@)
        && (if source.ajuz@.len() == 0 {
            out.ajuz@.len() == 0
        } else {
            selects(source.ajuz@, ajuz_choice, out.ajuz@)
        })
}

/// For each source choice index `s` and ajuz choice index `a`, the unique output
/// position is `s * A + a`. This is the exact Cartesian product of the verified
/// half enumerations, preserving duplicate recipe values as separate branches.
pub fn expand_form_choices(
    source: &Form,
    sadr_choices: &[Vec<usize>],
    ajuz_choices: &[Vec<usize>],
) -> (answer: Result<Vec<Form>, Error>)
    ensures match answer {
        Err(_) => true,
        Ok(out) =>
            out@.len() == sadr_choices@.len() * ajuz_choices@.len()
            && forall|s: int, a: int|
                0 <= s < sadr_choices@.len() && 0 <= a < ajuz_choices@.len()
                ==> expanded_form(
                    source,
                    sadr_choices@[s as int]@,
                    ajuz_choices@[a]@,
                    &out[(s as int) * ajuz_choices@.len() + a as int],
                ),
    },
{
    let mut out = Vec::new();
    let mut s = 0;
    while s < sadr_choices.len()
        invariant s <= sadr_choices.len(),
            sadr_choices.len() == sadr_choices@.len(),
            ajuz_choices.len() == ajuz_choices@.len(),
            out@.len() == (s as int) * ajuz_choices@.len(),
            forall|sp: int, a: int|
                0 <= sp < s && 0 <= a < ajuz_choices@.len()
                ==> expanded_form(
                    source,
                    sadr_choices@[sp]@,
                    ajuz_choices@[a]@,
                    &out[sp * ajuz_choices@.len() + a],
                ),
        decreases sadr_choices.len() - s,
    {
        let mut a = 0;
        while a < ajuz_choices.len()
            invariant a <= ajuz_choices.len(),
                s < sadr_choices.len(),
                sadr_choices.len() == sadr_choices@.len(),
                ajuz_choices.len() == ajuz_choices@.len(),
                out@.len() == (s as int) * ajuz_choices@.len() + a,
                forall|sp: int, ap: int|
                    0 <= sp < s && 0 <= ap < ajuz_choices@.len()
                    ==> expanded_form(
                        source,
                        sadr_choices@[sp]@,
                        ajuz_choices@[ap]@,
                        &out[sp * ajuz_choices@.len() + ap],
                    ),
                forall|ap: int| 0 <= ap < a ==>
                    expanded_form(
                        source,
                        sadr_choices@[s as int]@,
                        ajuz_choices@[ap]@,
                        &out[(s as int) * ajuz_choices@.len() + ap],
                    ),
            decreases ajuz_choices.len() - a,
        {
            assert((s as int) < sadr_choices@.len());
            let sadr_selected = select_recipes(&source.sadr, &sadr_choices[s])?;
            let ajuz_selected = if source.ajuz.is_empty() {
                Vec::new()
            } else {
                select_recipes(&source.ajuz, &ajuz_choices[a])?
            };
            let candidate = Form {
                constraints: clone_rules(&source.constraints),
                feet: source.feet,
                context: source.context,
                initial_only: source.initial_only,
                class: source.class,
                sadr: sadr_selected,
                ajuz: ajuz_selected,
            };
            assert(same_meta(source, &candidate));
            assert(selects(source.sadr@, sadr_choices@[s as int]@, candidate.sadr@));
            let ghost before_push = out@;
            out.push(candidate);
            assert(out@ =~= before_push.push(candidate));
            assert(out@.len() == before_push.len() + 1);
            assert(before_push.len() == (s as int) * ajuz_choices@.len() + (a as int));
            assert(out@.len() == (s as int) * ajuz_choices@.len() + (a as int) + 1);
            assert(ajuz_choices@.len() == ajuz_choices.len());
            assert(a < ajuz_choices@.len());
            assert forall|sp: int, ap: int|
                0 <= sp < s && 0 <= ap < ajuz_choices@.len()
                implies #[trigger] (sp * ajuz_choices@.len() + ap) < before_push.len() by {
                assert(sp * ajuz_choices@.len() + ap < s * ajuz_choices@.len())
                    by (nonlinear_arith)
                    requires 0 <= sp < s, 0 <= ap < ajuz_choices@.len();
                assert(s * ajuz_choices@.len() <= before_push.len());
            }
            assert(((s as int) * ajuz_choices@.len() + (a as int))
                < ((s as int) * ajuz_choices@.len() + (a as int)) + 1);
            assert(((s as int) * ajuz_choices@.len() + (a as int)) < out@.len());
            assert(expanded_form(
                source,
                sadr_choices[s as int]@,
                ajuz_choices[a as int]@,
                &out[(s as int) * ajuz_choices@.len() + a as int],
            ));
            a += 1;
        }
        assert(a == ajuz_choices.len());
        assert(ajuz_choices@.len() == ajuz_choices.len());
        assert(a as int == ajuz_choices@.len());
        assert(out@.len() == (s as int) * ajuz_choices@.len() + (a as int));
        assert((s as int) * ajuz_choices@.len() + (a as int)
            == (s as int) * ajuz_choices@.len() + ajuz_choices@.len());
        assert(out@.len() == (s as int) * ajuz_choices@.len() + ajuz_choices@.len());
        assert(((s + 1) as int) * ajuz_choices@.len()
            == (s as int) * ajuz_choices@.len() + ajuz_choices@.len())
            by (nonlinear_arith);
        assert(out@.len() == (((s + 1) as int) * ajuz_choices@.len()));
        s += 1;
    }
    Ok(out)
}

pub open spec fn same_form(a: &Form, b: &Form) -> bool {
    same_rules(a.constraints@, b.constraints@)
        && a.feet == b.feet
        && a.context == b.context
        && a.initial_only == b.initial_only
        && a.class == b.class
        && same_recipe_groups(a.sadr@, b.sadr@)
        && same_recipe_groups(a.ajuz@, b.ajuz@)
}

pub open spec fn same_recipe(a: Seq<Change>, b: Seq<Change>) -> bool {
    a == b
}

pub open spec fn same_choices(a: Seq<Vec<Change>>, b: Seq<Vec<Change>>) -> bool {
    a.len() == b.len()
        && forall|i: int| 0 <= i < a.len() ==> same_recipe(a[i]@, b[i]@)
}

pub open spec fn same_recipe_groups(
    a: Seq<Vec<Vec<Change>>>,
    b: Seq<Vec<Vec<Change>>>,
) -> bool {
    a.len() == b.len()
        && forall|i: int| 0 <= i < a.len() ==> same_choices(a[i]@, b[i]@)
}

pub open spec fn same_rules(a: Seq<Rule>, b: Seq<Rule>) -> bool {
    a.len() == b.len()
        && forall|i: int| 0 <= i < a.len() ==> a[i] == b[i]
}

pub fn clone_rules(source: &[Rule]) -> (out: Vec<Rule>)
    ensures out@ == source@,
{
    let mut out = Vec::new();
    let mut i = 0;
    while i < source.len()
        invariant i <= source.len(), source.len() == source@.len(), out@.len() == i,
            forall|j: int| 0 <= j < i ==> out@[j] == source@[j],
        decreases source.len() - i,
    {
        out.push(source[i]);
        i += 1;
    }
    out
}

pub fn clone_recipe(source: &[Change]) -> (out: Vec<Change>)
    ensures out@ == source@,
{
    let mut out = Vec::new();
    let mut i = 0;
    while i < source.len()
        invariant i <= source.len(), source.len() == source@.len(), out@.len() == i,
            forall|j: int| 0 <= j < i ==> out@[j] == source@[j],
        decreases source.len() - i,
    {
        out.push(source[i]);
        i += 1;
    }
    out
}

pub fn clone_choices(source: &[Vec<Change>]) -> (out: Vec<Vec<Change>>)
    ensures same_choices(out@, source@),
{
    let mut out: Vec<Vec<Change>> = Vec::new();
    let mut i = 0;
    while i < source.len()
        invariant i <= source.len(), source.len() == source@.len(), out@.len() == i,
            forall|j: int| 0 <= j < i ==> same_recipe(out@[j]@, source@[j]@),
        decreases source.len() - i,
    {
        out.push(clone_recipe(&source[i]));
        i += 1;
    }
    out
}

pub fn clone_recipe_groups(source: &[Vec<Vec<Change>>]) -> (out: Vec<Vec<Vec<Change>>>)
    ensures same_recipe_groups(out@, source@),
{
    let mut out: Vec<Vec<Vec<Change>>> = Vec::new();
    let mut i = 0;
    while i < source.len()
        invariant i <= source.len(), source.len() == source@.len(), out@.len() == i,
            forall|j: int| 0 <= j < i ==> same_choices(out@[j]@, source@[j]@),
        decreases source.len() - i,
    {
        out.push(clone_choices(&source[i]));
        i += 1;
    }
    out
}

pub fn clone_form(source: &Form) -> (out: Form)
    ensures same_form(&out, source),
{
    Form {
        constraints: source.constraints.clone(),
        feet: source.feet,
        context: source.context,
        initial_only: source.initial_only,
        class: source.class,
        sadr: clone_recipe_groups(&source.sadr),
        ajuz: clone_recipe_groups(&source.ajuz),
    }
}

pub open spec fn prefix_forms(groups: Seq<Vec<Form>>, n: int) -> int
    decreases n,
{
    if n <= 0 {
        0
    } else {
        prefix_forms(groups, n - 1) + groups[n - 1]@.len()
    }
}

pub open spec fn total_forms(groups: Seq<Vec<Form>>) -> int {
    prefix_forms(groups, groups.len() as int)
}

pub open spec fn group_range(groups: Seq<Vec<Form>>, g: int) -> (usize, usize) {
    (prefix_forms(groups, g) as usize, groups[g]@.len() as usize)
}

pub open spec fn group_ranges(groups: Seq<Vec<Form>>) -> Seq<(usize, usize)> {
    Seq::new(groups.len(), |g: int| group_range(groups, g))
}

proof fn prefix_forms_step(groups: Seq<Vec<Form>>, n: int)
    requires 0 <= n < groups.len(),
    ensures prefix_forms(groups, n + 1)
        == prefix_forms(groups, n) + groups[n]@.len(),
{
}

pub fn flatten_groups(
    groups: &[Vec<Form>],
) -> (answer: Result<(Vec<Form>, BranchRanges), Error>)
    ensures match answer {
        Err(_) => false,
        Ok((out, ranges)) =>
            ranges@ == group_ranges(groups@)
            && out@.len() as int == total_forms(groups@)
            && (forall|g: int| 0 <= g < groups@.len() ==>
                ranges@[g].1 == groups@[g]@.len()
                && ranges@[g].0 + ranges@[g].1 <= out@.len()
                && forall|i: int| 0 <= i < groups@[g]@.len() ==>
                    same_form(&out[ranges@[g].0 + i], &groups@[g]@[i]))
            && contiguous(ranges@),
    },
{
    let mut out = Vec::new();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut g = 0;
    while g < groups.len()
        invariant g <= groups.len(),
            groups.len() == groups@.len(),
            out@.len() as int == prefix_forms(groups@, g as int),
            ranges@.len() == g,
            forall|q: int| 0 <= q < g ==>
                ranges@[q] == group_range(groups@, q)
                && ranges@[q].1 == groups@[q]@.len()
                && ranges@[q].0 + ranges@[q].1 <= out@.len()
                && forall|i: int| 0 <= i < groups@[q]@.len() ==>
                    same_form(&out[ranges@[q].0 + i], &groups@[q]@[i]),
            forall|q: int| 0 <= q < g - 1 ==>
                ranges@[q].0 + ranges@[q].1 == ranges@[q + 1].0,
            (g == 0 || ranges@[0].0 == 0),
        decreases groups.len() - g,
    {
        let group = &groups[g];
        let start = out.len();
        let len = group.len();
        let mut i = 0;
        while i < len
            invariant i <= len, len == group@.len(), group@ == groups@[g as int]@,
                out@.len() == start + i,
                start as int == prefix_forms(groups@, g as int),
                forall|q: int| 0 <= q < g ==>
                    ranges@[q] == group_range(groups@, q)
                    && ranges@[q].1 == groups@[q]@.len()
                    && ranges@[q].0 + ranges@[q].1 <= out@.len()
                    && forall|j: int| 0 <= j < groups@[q]@.len() ==>
                        same_form(&out[ranges@[q].0 + j], &groups@[q]@[j]),
                forall|q: int| 0 <= q < g - 1 ==>
                    ranges@[q].0 + ranges@[q].1 == ranges@[q + 1].0,
                forall|j: int| 0 <= j < i ==>
                    same_form(&out[start + j], &group@[j]),
            decreases len - i,
        {
            out.push(clone_form(&group[i]));
            i += 1;
        }
        proof {
            prefix_forms_step(groups@, g as int);
        }
        assert(out@.len() as int == prefix_forms(groups@, g as int) + len);
        assert((g as int) + 1 <= groups@.len());
        assert(prefix_forms(groups@, g as int + 1)
            == prefix_forms(groups@, g as int) + groups@[g as int]@.len());
        assert(groups@[g as int]@.len() == len);
        assert(start as int == prefix_forms(groups@, g as int));
        assert((start as int) as usize == prefix_forms(groups@, g as int) as usize);
        ranges.push((start, len));
        g += 1;
    }
    assert(contiguous(ranges@)) by {
        reveal(contiguous);
    }
    assert(ranges@.len() == groups@.len());
    assert forall|g: int| 0 <= g < groups@.len()
        implies ranges@[g] == group_ranges(groups@)[g] by {
        assert(ranges@[g] == group_range(groups@, g));
    }
    assert(ranges@ =~= group_ranges(groups@));
    Ok((out, ranges))
}


pub open spec fn member(xs: Seq<usize>, x: usize) -> bool {
    exists|i: int| 0 <= i < xs.len() && xs[i] == x
}

pub open spec fn strict_sorted(xs: Seq<usize>) -> bool {
    forall|i: int, j: int| 0 <= i < j < xs.len() ==> xs[i] < xs[j]
}

pub open spec fn range_covers(r: (usize, usize), p: usize) -> bool {
    r.0 <= p && p - r.0 < r.1
}

pub open spec fn covered(ranges: Seq<(usize, usize)>, index: usize, p: usize) -> bool {
    index < ranges.len() && range_covers(ranges[index as int], p)
}

/// Consecutive ranges meet exactly: later ranges start where earlier ones end.
pub closed spec fn contiguous(ranges: Seq<(usize, usize)>) -> bool {
    forall|g: int| 0 <= g < ranges.len() - 1
        ==> ranges[g].0 + ranges[g].1 == ranges[g + 1].0
}

pub open spec fn covered_group(ranges: Seq<(usize, usize)>, group: Seq<usize>, p: usize) -> bool {
    exists|j: int| 0 <= j < group.len() && covered(ranges, group[j], p)
}

pub open spec fn filter_covered(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    p: int,
) -> Seq<usize>
    decreases p,
{
    if p <= 0 {
        Seq::empty()
    } else {
        filter_covered(ranges, group, p - 1)
            + (if covered_group(ranges, group, (p - 1) as usize) {
                Seq::empty().push((p - 1) as usize)
            } else {
                Seq::empty()
            })
    }
}

pub open spec fn group_covers(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    total: int,
) -> bool {
    exists|p: usize| (p as int) < total && covered_group(ranges, group, p)
}

pub open spec fn group_valid(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    total: int,
) -> bool {
    group.len() > 0
        && (forall|j: int| 0 <= j < group.len()
            ==> (group[j] as int) < ranges.len())
        && group_covers(ranges, group, total)
}

pub open spec fn requirements_valid(
    requirements: Seq<Vec<usize>>,
    ranges: Seq<(usize, usize)>,
    total: int,
) -> bool {
    forall|g: int| 0 <= g < requirements.len() ==>
        #[trigger] group_valid(ranges, requirements[g]@, total)
}

pub open spec fn exact_union(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    total: int,
    out: Seq<usize>,
) -> bool {
    (forall|p: usize| (p as int) < total ==>
        (member(out, p) == covered_group(ranges, group, p)))
        && (forall|i: int| 0 <= i < out.len()
            ==> (out[i] as int) < total)
}

proof fn member_add(a: Seq<usize>, b: Seq<usize>, x: usize)
    ensures member(a + b, x) == (member(a, x) || member(b, x)),
{
    if member(a + b, x) {
        let i = choose|i: int| 0 <= i < (a + b).len() && (a + b)[i] == x;
        if i < a.len() {
            assert(a[i] == x);
            assert(member(a, x));
        } else {
            assert(b[i - a.len()] == x);
            assert(member(b, x));
        }
    }
    if member(a, x) {
        let i = choose|i: int| 0 <= i < a.len() && a[i] == x;
        assert((a + b)[i] == x);
        assert(member(a + b, x));
    }
    if member(b, x) {
        let i = choose|i: int| 0 <= i < b.len() && b[i] == x;
        assert((a + b)[a.len() + i] == x);
        assert(member(a + b, x));
    }
}

proof fn member_singleton(x: usize, y: usize)
    ensures member(Seq::empty().push(x), y) == (x == y),
{
    let s = Seq::empty().push(x);
    if x == y {
        assert(s[0] == x);
        assert(member(s, y));
    }
    if member(s, y) {
        let i = choose|i: int| 0 <= i < s.len() && s[i] == y;
        assert(i == 0);
        assert(x == y);
    }
}

proof fn filter_covered_sound(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    p: int,
    q: usize,
)
    requires 0 <= p, member(filter_covered(ranges, group, p), q),
    ensures (q as int) < p && covered_group(ranges, group, q),
    decreases p,
{
    if p > 0 {
        let prev = filter_covered(ranges, group, p - 1);
        let last = (p - 1) as usize;
        let tail = if covered_group(ranges, group, last) {
            Seq::empty().push(last)
        } else {
            Seq::empty()
        };
        assert(filter_covered(ranges, group, p) == prev + tail);
        member_add(prev, tail, q);
        if member(prev, q) {
            filter_covered_sound(ranges, group, p - 1, q);
        } else {
            if covered_group(ranges, group, last) {
                member_singleton(last, q);
                assert(q == last);
                assert(covered_group(ranges, group, q));
            } else {
                assert(tail == Seq::empty());
                assert(!member(tail, q));
            }
        }
    }
}

proof fn filter_covered_complete(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    p: int,
    q: usize,
)
    requires
        0 <= p,
        (q as int) < p,
        covered_group(ranges, group, q),
    ensures member(filter_covered(ranges, group, p), q),
    decreases p,
{
    if p > 0 {
        let last = (p - 1) as usize;
        if q == last {
            let prev = filter_covered(ranges, group, p - 1);
            let tail = if covered_group(ranges, group, last) {
                Seq::empty().push(last)
            } else {
                Seq::empty()
            };
            assert(filter_covered(ranges, group, p) == prev + tail);
            assert(tail == Seq::empty().push(last));
            member_singleton(last, q);
            assert(member(tail, q));
            member_add(prev, tail, q);
        } else {
            assert((q as int) < p - 1);
            filter_covered_complete(ranges, group, p - 1, q);
            let prev = filter_covered(ranges, group, p - 1);
            let tail = if covered_group(ranges, group, last) {
                Seq::empty().push(last)
            } else {
                Seq::empty()
            };
            assert(filter_covered(ranges, group, p) == prev + tail);
            assert(member(prev, q));
            member_add(prev, tail, q);
        }
    }
}

proof fn filter_covered_member(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    p: int,
    q: usize,
)
    requires 0 <= p,
    ensures member(filter_covered(ranges, group, p), q)
        == ((q as int) < p && covered_group(ranges, group, q)),
{
    if member(filter_covered(ranges, group, p), q) {
        filter_covered_sound(ranges, group, p, q);
    }
    if (q as int) < p && covered_group(ranges, group, q) {
        filter_covered_complete(ranges, group, p, q);
    }
}

proof fn filter_covered_bound(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    p: int,
)
    requires 0 <= p,
    ensures forall|i: int| 0 <= i < filter_covered(ranges, group, p).len()
        ==> 0 <= #[trigger] filter_covered(ranges, group, p)[i] < p,
{
    let filtered = filter_covered(ranges, group, p);
    assert forall|i: int| 0 <= i < filtered.len()
        implies 0 <= #[trigger] filtered[i] < p by {
        assert(member(filtered, filtered[i]));
        filter_covered_sound(ranges, group, p, filtered[i]);
    }
}

proof fn filter_covered_strict_sorted(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    p: nat,
)
    requires p <= usize::MAX as nat,
    ensures strict_sorted(filter_covered(ranges, group, p as int)),
    decreases p,
{
    if p > 0 {
        filter_covered_strict_sorted(ranges, group, (p - 1) as nat);
        filter_covered_bound(ranges, group, (p - 1) as int);
        let prev = filter_covered(ranges, group, (p - 1) as int);
        let last = (p - 1) as usize;
        let filtered = filter_covered(ranges, group, p as int);
        if covered_group(ranges, group, last) {
            assert(filtered == prev.push(last));
            assert(last as int == p - 1);
            assert(strict_sorted(prev));
            assert forall|i: int, j: int|
                0 <= i < j < filtered.len()
                implies filtered[i] < filtered[j] by {
                if j < prev.len() {
                    assert(filtered[i] == prev[i]);
                    assert(filtered[j] == prev[j]);
                    assert(prev[i] < prev[j]) by {
                        assert(strict_sorted(prev));
                    }
                } else {
                    assert(j == prev.len());
                    assert(filtered[j] == last);
                    assert(filtered[i] == prev[i]);
                    assert(prev[i] < p - 1);
                }
            }
        } else {
            assert(filtered == prev);
        }
    }
}

proof fn filter_covered_nonempty_iff(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    total: int,
)
    requires 0 <= total,
    ensures (filter_covered(ranges, group, total).len() > 0)
        == group_covers(ranges, group, total),
{
    let filtered = filter_covered(ranges, group, total);
    if filtered.len() > 0 {
        let q = filtered[0];
        assert(member(filtered, q));
        filter_covered_sound(ranges, group, total, q);
        assert(group_covers(ranges, group, total)) by {
            assert((q as int) < total);
            assert(covered_group(ranges, group, q));
        }
    }
    if group_covers(ranges, group, total) {
        let p = choose|p: usize|
            (p as int) < total && covered_group(ranges, group, p);
        filter_covered_complete(ranges, group, total, p);
        assert(member(filtered, p));
        assert(filtered.len() > 0);
    }
}

proof fn filter_covered_exact_union(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    total: int,
)
    requires 0 <= total,
    ensures exact_union(
        ranges,
        group,
        total,
        filter_covered(ranges, group, total),
    ),
{
    let filtered = filter_covered(ranges, group, total);
    assert forall|p: usize| (p as int) < total
        implies member(filtered, p) == covered_group(ranges, group, p) by {
        filter_covered_member(ranges, group, total, p);
    }
    assert forall|i: int| 0 <= i < filtered.len()
        implies (filtered[i] as int) < total by {
        assert(member(filtered, filtered[i]));
        filter_covered_sound(ranges, group, total, filtered[i]);
    }
}

fn covers(
    ranges: &[(usize, usize)],
    index: usize,
    p: usize,
) -> (yes: bool)
    requires index < ranges.len(), ranges.len() == ranges@.len(),
    ensures yes == covered(ranges@, index, p),
{
    let (start, len) = ranges[index];
    if start > p {
        false
    } else {
        p - start < len
    }
}

proof fn group_bound_at(
    ranges: Seq<(usize, usize)>,
    group: Seq<usize>,
    j: int,
)
    requires forall|i: int| 0 <= i < group.len() ==> group[i] < ranges.len(),
        0 <= j < group.len(),
    ensures group[j] < ranges.len(),
{
}

fn covers_group(
    ranges: &[(usize, usize)],
    group: &[usize],
    p: usize,
) -> (yes: bool)
    requires ranges.len() == ranges@.len(), group.len() == group@.len(),
        forall|j: int| 0 <= j < group@.len() ==> group@[j] < ranges@.len(),
    ensures yes == covered_group(ranges@, group@, p),
{
    let mut yes = false;
    let mut j = 0;
    while j < group.len()
        invariant j <= group.len(), group.len() == group@.len(),
            ranges.len() == ranges@.len(),
            forall|i: int| 0 <= i < group@.len() ==> group@[i] < ranges@.len(),
            yes == (exists|jj: int|
                0 <= jj < j && covered(ranges@, group@[jj], p)),
        decreases group.len() - j,
    {
        let index = group[j];
        proof {
            group_bound_at(ranges@, group@, j as int);
        }
        assert(index < ranges.len());
        let hit = covers(ranges, index, p);
        if hit {
            assert(exists|jj: int|
                0 <= jj < j + 1 && covered(ranges@, group@[jj], p)) by {
                assert(covered(ranges@, group@[j as int], p));
            }
        }
        yes = yes || hit;
        j += 1;
    }
    yes
}

/// Remap source-form requirement groups onto the contiguous expanded ranges.
/// The output group is exactly the increasing filter of every covered output
/// index, hence sorted, duplicate-free and nonempty when a range is selected.
pub fn remap_requirements(
    requirements: &[Vec<usize>],
    ranges: &[(usize, usize)],
    total: usize,
) -> (answer: Result<Vec<Vec<usize>>, Error>)
    requires
        ranges.len() == ranges@.len(),
        forall|i: int| 0 <= i < ranges@.len() ==>
            ranges@[i].0 <= total && total - ranges@[i].0 >= ranges@[i].1,
    ensures match answer {
        Ok(remapped) =>
            requirements_valid(requirements@, ranges@, total as int)
            && remapped@.len() == requirements@.len()
            && forall|g: int| 0 <= g < remapped@.len() ==>
                remapped@[g]@ == filter_covered(
                    ranges@,
                    requirements@[g]@,
                    total as int,
                )
                && remapped@[g]@.len() > 0
                && strict_sorted(remapped@[g]@)
                && exact_union(
                    ranges@,
                    requirements@[g]@,
                    total as int,
                    remapped@[g]@,
                ),
        Err(Error::Requirement) =>
            !requirements_valid(requirements@, ranges@, total as int),
        Err(_) => false,
    },
{
    let mut remapped: Vec<Vec<usize>> = Vec::new();
    let mut g = 0;
    while g < requirements.len()
        invariant g <= requirements.len(),
            requirements.len() == requirements@.len(),
            ranges.len() == ranges@.len(),
            remapped@.len() == g,
            forall|q: int| 0 <= q < g ==>
                group_valid(ranges@, requirements@[q]@, total as int),
            forall|q: int| 0 <= q < g ==>
                remapped@[q]@ == filter_covered(
                    ranges@,
                    requirements@[q]@,
                    total as int,
                ),
            forall|q: int| 0 <= q < g ==> remapped@[q]@.len() > 0,
            forall|q: int| 0 <= q < g ==> strict_sorted(remapped@[q]@),
            forall|q: int| 0 <= q < g ==>
                exact_union(
                    ranges@,
                    requirements@[q]@,
                    total as int,
                    remapped@[q]@,
                ),
        decreases requirements.len() - g,
    {
        let group = &requirements[g];
        assert(requirements@[g as int]@ == group@);
        let mut v = 0;
        while v < group.len()
            invariant v <= group.len(), group.len() == group@.len(),
                g < requirements.len(),
                requirements.len() == requirements@.len(),
                group@ == requirements@[g as int]@,
                ranges.len() == ranges@.len(),
                forall|i: int| 0 <= i < v ==> group@[i] < ranges@.len(),
            decreases group.len() - v,
        {
            if group[v] >= ranges.len() {
                assert(!group_valid(ranges@, group@, total as int)) by {
                    assert(group.len() == group@.len());
                    assert(0 <= v < group@.len());
                    assert(!(forall|i: int| 0 <= i < group@.len()
                        ==> group@[i] < ranges@.len()));
                }
                assert(!requirements_valid(requirements@, ranges@, total as int)) by {
                    if requirements_valid(requirements@, ranges@, total as int) {
                        assert(group_valid(
                            ranges@,
                            requirements@[g as int]@,
                            total as int,
                        ));
                        assert(requirements@[g as int]@ == group@);
                    }
                }
                return Err(Error::Requirement);
            }
            v += 1;
        }
        assert(v == group.len());
        assert forall|i: int| 0 <= i < group@.len()
            implies group@[i] < ranges@.len() by {
            assert(i < v);
        }
        let mut output: Vec<usize> = Vec::new();
        let mut p = 0;
        while p < total
            invariant p <= total,
                group.len() == group@.len(),
                g < requirements.len(),
                requirements.len() == requirements@.len(),
                group@ == requirements@[g as int]@,
                ranges.len() == ranges@.len(),
                forall|i: int| 0 <= i < group@.len() ==> group@[i] < ranges@.len(),
                output@ == filter_covered(ranges@, group@, p as int),
            decreases total - p,
        {
            let hit = covers_group(ranges, group, p);
            if hit {
                output.push(p);
            }
            assert(output@ == filter_covered(ranges@, group@, (p + 1) as int));
            p += 1;
        }
        proof {
            filter_covered_nonempty_iff(ranges@, group@, total as int);
        }
        if output.is_empty() {
            assert(!group_valid(ranges@, group@, total as int)) by {
                if group@.len() == 0 {
                } else {
                    assert(output@ == filter_covered(ranges@, group@, total as int));
                    assert(!group_covers(ranges@, group@, total as int));
                }
            }
            assert(!requirements_valid(requirements@, ranges@, total as int)) by {
                if requirements_valid(requirements@, ranges@, total as int) {
                    assert(group_valid(
                        ranges@,
                        requirements@[g as int]@,
                        total as int,
                    ));
                    assert(requirements@[g as int]@ == group@);
                }
            }
            return Err(Error::Requirement);
        }
        assert(group@ == requirements@[g as int]@);
        assert(output@ == filter_covered(ranges@, group@, total as int));
        proof {
            filter_covered_nonempty_iff(ranges@, group@, total as int);
            filter_covered_strict_sorted(ranges@, group@, total as nat);
            filter_covered_exact_union(ranges@, group@, total as int);
        }
        assert(group_covers(ranges@, group@, total as int));
        assert(group@.len() > 0) by {
            if group@.len() == 0 {
                assert(!group_covers(ranges@, group@, total as int));
            }
        }
        assert(group_valid(ranges@, group@, total as int));
        assert(strict_sorted(output@));
        assert(exact_union(ranges@, group@, total as int, output@));
        assert(group_valid(
            ranges@,
            requirements@[g as int]@,
            total as int,
        ));
        remapped.push(output);
        assert(remapped@[g as int]@ == output@);
        assert(remapped@[g as int]@ == filter_covered(
            ranges@,
            requirements@[g as int]@,
            total as int,
        ));
        assert(remapped@[g as int]@.len() > 0);
        assert(strict_sorted(remapped@[g as int]@));
        assert(exact_union(
            ranges@,
            requirements@[g as int]@,
            total as int,
            remapped@[g as int]@,
        ));
        g += 1;
    }
    assert(g == requirements.len());
    assert forall|q: int| 0 <= q < requirements@.len()
        implies #[trigger] group_valid(ranges@, requirements@[q]@, total as int) by {
    }
    assert(requirements_valid(requirements@, ranges@, total as int));
    Ok(remapped)
}

/// Compile ordered source-form branch groups into one output list and remap
/// requirement groups through the resulting ranges. This composes the two
/// verified steps so no caller has to recompute or trust range arithmetic.
pub fn compile_groups(
    groups: &[Vec<Form>],
    requirements: &[Vec<usize>],
) -> (answer: Result<CompiledGroups, Error>)
    ensures match answer {
        Ok((out, ranges, remapped)) =>
            ranges@ == group_ranges(groups@)
            && out@.len() as int == total_forms(groups@)
            && requirements_valid(requirements@, ranges@, out@.len() as int)
            && remapped@.len() == requirements@.len()
            && (forall|g: int| 0 <= g < groups@.len() ==>
                ranges@[g].1 == groups@[g]@.len()
                && ranges@[g].0 + ranges@[g].1 <= out@.len()
                && forall|i: int| 0 <= i < groups@[g]@.len() ==>
                    same_form(&out[ranges@[g].0 + i], &groups@[g]@[i]))
            && contiguous(ranges@)
            && (forall|g: int| 0 <= g < remapped@.len() ==>
                remapped@[g]@ == filter_covered(
                    ranges@,
                    requirements@[g]@,
                    out@.len() as int,
                )
                && remapped@[g]@.len() > 0
                && strict_sorted(remapped@[g]@)
                && exact_union(
                    ranges@,
                    requirements@[g]@,
                    out@.len() as int,
                    remapped@[g]@,
                )),
        Err(Error::Requirement) =>
            !requirements_valid(
                requirements@,
                group_ranges(groups@),
                total_forms(groups@),
            ),
        Err(_) => false,
    },
{
    let (out, ranges) = flatten_groups(groups)?;
    assert(ranges@ == group_ranges(groups@));
    assert(out@.len() as int == total_forms(groups@));
    assert(ranges.len() == ranges@.len());
    assert(ranges@.len() == groups@.len());
    assert forall|g: int| 0 <= g < groups@.len() implies
        ranges@[g].1 == groups@[g]@.len()
        && ranges@[g].0 + ranges@[g].1 <= out@.len()
        && forall|i: int| 0 <= i < groups@[g]@.len() ==>
            same_form(&out[ranges@[g].0 + i], &groups@[g]@[i]) by {
    }
    assert(contiguous(ranges@));
    assert forall|i: int| 0 <= i < ranges@.len()
        implies ranges@[i].0 <= out@.len()
            && out@.len() - ranges@[i].0 >= ranges@[i].1 by {
        assert(ranges@[i].0 + ranges@[i].1 <= out@.len());
    }
    let total = out.len();
    let remapped = match remap_requirements(requirements, &ranges, total) {
        Ok(remapped) => remapped,
        Err(error) => {
            assert(ranges@ == group_ranges(groups@));
            assert(out@.len() as int == total_forms(groups@));
            assert(!requirements_valid(requirements@, ranges@, total as int));
            assert(!requirements_valid(
                requirements@,
                group_ranges(groups@),
                total_forms(groups@),
            ));
            return Err(error);
        }
    };
    assert(remapped@.len() == requirements@.len());
    assert(requirements_valid(requirements@, ranges@, out@.len() as int));
    assert forall|g: int| 0 <= g < remapped@.len() implies
        remapped@[g]@ == filter_covered(
            ranges@,
            requirements@[g]@,
            out@.len() as int,
        )
        && remapped@[g]@.len() > 0
        && strict_sorted(remapped@[g]@)
        && exact_union(
            ranges@,
            requirements@[g]@,
            out@.len() as int,
            remapped@[g]@,
        ) by {
        assert(remapped@[g]@ == filter_covered(
            ranges@,
            requirements@[g]@,
            total as int,
        ));
    }
    Ok((out, ranges, remapped))
}

} // verus!
