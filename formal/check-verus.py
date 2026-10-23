#!/usr/bin/env python3
"""Verify and compile the workspace checker, then exercise its Cargo build."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parent
PIN = json.loads((ROOT / "verus/toolchain.json").read_text())
CHECKER = ROOT.parent / "crates/checker/src"


def control_expect(kind, file, *fragments):
    return {"kind": kind, "file": file, "fragments": list(fragments)}


def proof_error_blocks(output):
    """Yield (kind, location, block) for actual Verus `error:` blocks.

    A block ends at the next top-level diagnostic, so a following warning or
    note cannot supply an obligation fragment for an earlier unrelated error.
    """
    starts = list(re.finditer(r"(?m)^(error|warning|note): ([^\n]+)$", output))
    for index, match in enumerate(starts):
        if match.group(1) != "error":
            continue
        kind = match.group(2).strip()
        if kind.startswith("aborting due to"):
            continue
        end = starts[index + 1].start() if index + 1 < len(starts) else len(output)
        block = output[match.start():end]
        location = re.search(r"(?m)^\s*--> ([^\n]+)$", block)
        yield kind, location.group(1).strip() if location else "", block


def control_failure_matches(output, label, expected):
    """True only for an error block at the mutated control's intended contract."""
    marker = f"broken-{label}/"
    for kind, location, block in proof_error_blocks(output):
        if kind != expected["kind"]:
            continue
        if expected["file"] not in location or marker not in location:
            continue
        fragments = expected["fragments"]
        if not fragments or all(fragment in block for fragment in fragments):
            return True
    return False


def run(label, command, *, env=None, reject=False):
    start = time.perf_counter()
    result = subprocess.run(command, cwd=ROOT.parent, env=env, text=True,
                            capture_output=True, timeout=180)
    output = result.stdout + result.stderr
    if (result.returncode == 0) == reject:
        raise RuntimeError(f"{label}: unexpected exit {result.returncode}\n{output}")
    print(f"{label}: {'rejected' if reject else 'passed'} ({time.perf_counter() - start:.2f}s)", flush=True)
    if output.strip():
        print(output.strip(), flush=True)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verus", default="verus")
    parser.add_argument("--rustup-home", help="optional isolated toolchain used only by Verus")
    parser.add_argument("--negative-control", action="store_true")
    args = parser.parse_args()
    verus = shutil.which(args.verus)
    if verus is None:
        raise RuntimeError("Verus is missing; see formal/README.md for the pinned setup")
    proof_env = dict(os.environ)
    if args.rustup_home:
        proof_env["RUSTUP_HOME"] = str(Path(args.rustup_home).resolve())
    version = run("Verus version", [verus, "--version", "--output-json"], env=proof_env)
    info = json.loads(version)["verus"]
    if info["version"] != PIN["version"] or info["commit"] != PIN["commit"]:
        raise RuntimeError("Verus version/commit differs from formal/verus/toolchain.json")
    if info["toolchain"].split("-", 1)[0] != PIN["rust"]:
        raise RuntimeError("Verus Rust toolchain differs from the pin")
    with tempfile.TemporaryDirectory(prefix="watad-verus-check-") as directory:
        work = Path(directory)
        artifacts = {}
        def imports(names):
            flags = ["-L", f"dependency={work}"]
            for name in names:
                library, metadata = artifacts[name]
                flags += ["--extern", f"{name}={library}", "--import", f"{name}={metadata}"]
            return flags

        # Export actual shared enum definitions, rather than trusted lookalikes.
        for crate, dependencies in [("types", []), ("daira", ["watad_types"]),
                                     ("checker", ["watad_types"])]:
            name = f"watad_{crate}"
            library, metadata = work / f"lib{name}.rlib", work / f"{name}.vir"
            source = ROOT.parent / "crates" / crate / "src/lib.rs"
            output = run(f"Verus {crate} and native compilation", [verus, str(source),
                         "--edition=2024", "--crate-type", "lib", "--crate-name", name,
                         "--no-cheating", "--triggers-mode", "silent", "--compile", "--export", str(metadata),
                         "-o", str(library), *imports(dependencies)], env=proof_env)
            if not re.search(r"verification results:: [1-9][0-9]* verified, 0 errors", output):
                raise RuntimeError("missing successful verification summary")
            artifacts[name] = library, metadata
        print(f"Verus-compiled checker library: {library.stat().st_size} bytes", flush=True)
        # Cargo keeps the workspace's Rust 1.92 pin and normal toolchain home.
        test_env = dict(os.environ)
        test_env.setdefault("PROPTEST_CASES", "512")
        run("Cargo runtime integration (all lugha suites and independent oracles)",
            ["cargo", "test", "-p", "watad-lugha", "--tests",
             "--locked", "--offline"], env=test_env)
        if args.negative_control:
            controls = [
                ("selected-adjacency", "choice.rs",
                 "|| (r.left_deleted[l] && r.right_deleted[a])", "|| false",
                 control_expect("invariant not satisfied at end of loop body", "choice.rs",
                                "permits_rule(&rules@[j],steps@)")),
                ("selected-choice-completeness", "choice.rs",
                 "        c += 1;\n    }\n    None\n}\n\npub fn recognize",
                 "        return None;\n    }\n    None\n}\n\npub fn recognize",
                 control_expect("postcondition not satisfied", "choice.rs",
                                "ensures answer.is_some() == derives_from")),
                ("source-rule-flags", "structured.rs",
                 "out.push(programs[i].actions[letter] == m::Action::Delete);",
                 "out.push(false);",
                 control_expect("invariant not satisfied at end of loop body", "structured.rs",
                                "out@[j] == (letter < programs@[j].actions@.len()",
                                "programs@[j].actions@[letter as int] == m::Action::Delete")),
                ("verse-sadr-replay", "verse.rs",
                 "&& choice::check(&f.sadr, &f.sadr_rules, sadr, &c.sadr)",
                 "&& choice::check(&f.sadr, &[], sadr, &c.sadr)",
                 control_expect("postcondition not satisfied", "verse.rs",
                                "ensures yes == legal(p, scope, sadr@, ajuz@, c)")),
                ("verse-ajuz-replay", "verse.rs",
                 "&& choice::check(&f.ajuz, &f.ajuz_rules, ajuz, &c.ajuz)",
                 "&& choice::check(&f.ajuz, &[], ajuz, &c.ajuz)",
                 control_expect("postcondition not satisfied", "verse.rs",
                                "ensures yes == legal(p, scope, sadr@, ajuz@, c)")),
                ("certificate-profile", "verse.rs",
                 "if !validate(p) || c.profile != p.id || c.context != scope.context || c.form >= p.forms.len() {",
                 "if !validate(p) || c.profile != c.profile || c.context != scope.context || c.form >= p.forms.len() {",
                 control_expect("postcondition not satisfied", "verse.rs",
                                "ensures yes == legal(p, scope, sadr@, ajuz@, c)")),
                ("alphabet", "adapter.rs", "Beat::Mutaharrik => 1,", "Beat::Mutaharrik => 0,",
                 control_expect("postcondition not satisfied", "adapter.rs",
                                "ensures code == beat_code(beat)")),
                ("input-bounds", "language.rs",
                 "if at > input.len() || word.len() > input.len() - at {\n        return false;",
                 "if at > input.len() || word.len() > input.len() - at {\n        return true;",
                 control_expect("postcondition not satisfied", "language.rs",
                                "ensures yes == prefix(input@, at as int, word@)")),
                ("ownership", "morphology.rs",
                 "    scope != Scope::Zihaf || (letter.sabab && letter.offset == 1)\n}", "    true\n}",
                 control_expect("postcondition not satisfied", "morphology.rs",
                                "ensures yes == change_valid(letter, action, expected, scope)")),
                ("poem-obligation", "obligations.rs",
                 "return if check_requirements(r, &selected)", "return if true",
                 control_expect("postcondition not satisfied", "obligations.rs",
                                "ensures answer.is_some() == derives_from")),
                ("adjacency", "adjacency.rs",
                 "&& !(p.actions[l] == Action::Delete && q.actions[r] == Action::Delete)", "&& true",
                 control_expect("postcondition not satisfied", "adjacency.rs",
                                "ensures yes == compatible(p,q,l,r)")),
                ("profile-completeness", "structured.rs",
                 "if !verse::validate(&target)", "if true",
                 control_expect("postcondition not satisfied", "structured.rs",
                                "Err(_) => !valid(source)")),
                ("circle-word", "layout.rs",
                 "if pattern[j] != word[index] { return false; }", "if false { return false; }",
                 control_expect("invariant not satisfied at end of loop body", "layout.rs",
                                "pattern@[k] == expected(word@,rot,_reps)")),
                ("circle-basis", "profile.rs",
                 "|| !check_basis(&source,&feet)", "|| false",
                 control_expect("constructed value may fail to meet its declared type invariant", "profile.rs",
                                "origin:Some(Origin")),
                ("normalize-coordinate", "normalize.rs",
                 "Zihaf::Tayy => 3usize,", "Zihaf::Tayy => 4usize,",
                 control_expect("postcondition not satisfied", "normalize.rs",
                                "apply_change(units@, letters@, program_view(&p), change)")),
                ("normalize-precondition", "normalize.rs",
                 "if letters[at].beat != expected {\n        assert(edit_norm(before, letters@, at as int, expected, action) == Err(Error::Precondition));",
                 "if false {\n        assert(edit_norm(before, letters@, at as int, expected, action) == Err(Error::Precondition));",
                 control_expect("postcondition not satisfied", "normalize.rs",
                                "edit_norm(program_view(&p), letters@, at as int, expected, action)")),
                ("normalize-suffix", "normalize.rs",
                 "p.suffix.extend_from_slice(&[1, 0]);", "p.suffix.extend_from_slice(&[1, 1]);",
                 control_expect("assertion failed", "normalize.rs",
                                "assert(p.suffix@ == before.suffix + suffix_tarfil())")),
                ("expand-completeness", "expand.rs",
                 "if choice.len() != slots.len() { return false; }",
                 "if false { return false; }",
                 control_expect("invariant not satisfied before loop", "expand.rs",
                                "choice@.len() == slots@.len()")),
                ("expand-adjacency", "expand.rs",
                 "if !adjacency::check_pair(\n            &slots[rule.left].programs[choice[rule.left]],\n            &slots[rule.right].programs[choice[rule.right]],\n            rule.left_letter,\n            rule.right_letter,\n        ) {",
                 "if false {",
                 control_expect("assertion failed", "expand.rs",
                                "assert(rule_satisfied(slots@, rule, choice@))")),
                ("expand-skip-first", "expand.rs",
                 "    let mut c = 0;\n    while c < slots[k].programs.len()",
                 "    let mut c = 1;\n    while c < slots[k].programs.len()",
                 control_expect("invariant not satisfied before loop", "expand.rs",
                                "complete(slots@, rules@, choice)")),
                ("expand-prefix-order", "expand.rs",
                 "        out.push(prefix[i]);", "        out.push(prefix[prefix.len() - 1 - i]);",
                 control_expect("invariant not satisfied at end of loop body", "expand.rs",
                                "out@ == prefix@.subrange(0, i as int)")),
                ("expand-order-reversed", "expand.rs",
                 "        out.append(&mut tail);\n        assert(out@ =~= before + tail_view);",
                 "        let mut reversed = tail;\n        reversed.append(&mut out);\n        out = reversed;",
                 control_expect("assertion failed", "expand.rs",
                                "assert(ordered_vecs(out@))")),
                ("catalog-sadr-alternative", "catalog.rs",
                 "    let mut s = 0;\n    while s < sadr_choices.len()",
                 "    let mut s = 1;\n    while s < sadr_choices.len()",
                 control_expect("invariant not satisfied before loop", "catalog.rs",
                                "s <= sadr_choices.len()")),
                ("catalog-ajuz-alternative", "catalog.rs",
                 "        let mut a = 0;\n        while a < ajuz_choices.len()",
                 "        let mut a = 1;\n        while a < ajuz_choices.len()",
                 control_expect("invariant not satisfied before loop", "catalog.rs",
                                "a <= ajuz_choices.len()")),
                ("catalog-invented-recipe", "catalog.rs",
                 "        let one = vec![row[chosen].clone()];",
                 "        let one = vec![row[chosen].clone(), row[chosen].clone()];",
                 control_expect("invariant not satisfied at end of loop body", "catalog.rs",
                                "selected[a]@.len() == 1")),
                ("catalog-source-index", "catalog.rs",
                 "        let chosen = choice[i];", "        let chosen = choice[0];",
                 control_expect("invariant not satisfied at end of loop body", "catalog.rs",
                                "selected[a]@[0]@ == source@[a]@[choice[a] as int]@")),
                ("catalog-context", "catalog.rs",
                 "                context: source.context,\n                initial_only: source.initial_only,\n                class: source.class,\n                sadr: sadr_selected,",
                 "                context: source.context ^ 1,\n                initial_only: source.initial_only,\n                class: source.class,\n                sadr: sadr_selected,",
                 control_expect("assertion failed", "catalog.rs",
                                "assert(same_meta(source, &candidate));")),
                ("catalog-class", "catalog.rs",
                 "                context: source.context,\n                initial_only: source.initial_only,\n                class: source.class,\n                sadr: sadr_selected,",
                 "                context: source.context,\n                initial_only: source.initial_only,\n                class: DarbClass(source.class.0 ^ 1),\n                sadr: sadr_selected,",
                 control_expect("assertion failed", "catalog.rs",
                                "assert(same_meta(source, &candidate));")),
                ("catalog-requirement-index", "catalog.rs",
                 "                output.push(p);", "                output.push(p + 1);",
                 control_expect("assertion failed", "catalog.rs",
                                "assert(output@ == filter_covered(ranges@, group@, (p + 1) as int))")),
                ("catalog-requirement-empty-output", "catalog.rs",
                 "        if output.is_empty() {",
                 "        if false {",
                 control_expect("assertion failed", "catalog.rs",
                                "assert(group_covers(ranges@, group@, total as int))")),
                ("catalog-empty-invented-form", "catalog.rs",
                 "    assert(contiguous(ranges@)) by {\n        reveal(contiguous);\n    }\n    assert(ranges@.len() == groups@.len());",
                 "    if groups.len() == 0 {\n        out.push(Form {\n            constraints: vec![],\n            feet: 0,\n            context: 0,\n            initial_only: false,\n            class: DarbClass(0),\n            sadr: vec![],\n            ajuz: vec![],\n        });\n    }\n    assert(contiguous(ranges@)) by {\n        reveal(contiguous);\n    }\n    assert(ranges@.len() == groups@.len());",
                 control_expect("postcondition not satisfied", "catalog.rs",
                                "ranges@ == group_ranges(groups@)")),
            ]
            for label, filename, old, new, expected in controls:
                source = (CHECKER / filename).read_text()
                if source.count(old) != 1:
                    raise RuntimeError(f"{label} mutation target is no longer unique")
                broken = work / f"broken-{label}"
                shutil.copytree(CHECKER, broken)
                (broken / filename).write_text(source.replace(old, new))
                output = run(f"Changed-{label} negative control", [verus,
                             str(broken / "lib.rs"), "--edition=2024", "--crate-type", "lib",
                             "--no-cheating", "--triggers-mode", "silent", *imports(["watad_types"])],
                             env=proof_env, reject=True)
                if not re.search(r"verification results:: [0-9]+ verified, [1-9][0-9]* errors", output):
                    raise RuntimeError("control failed before proof checking")
                if not control_failure_matches(output, label, expected):
                    raise RuntimeError(
                        f"control {label} did not fail at its intended {expected['kind']} "
                        f"in {expected['file']} with one of {expected['fragments']}")


if __name__ == "__main__":
    main()
