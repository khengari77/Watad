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
                 "|| (r.left_deleted[l] && r.right_deleted[a])", "|| false"),
                ("selected-choice-completeness", "choice.rs",
                 "        c += 1;\n    }\n    None\n}\n\npub fn recognize",
                 "        return None;\n    }\n    None\n}\n\npub fn recognize"),
                ("source-rule-flags", "structured.rs",
                 "out.push(programs[i].actions[letter] == m::Action::Delete);",
                 "out.push(false);"),
                ("verse-sadr-replay", "verse.rs",
                 "&& choice::check(&f.sadr, &f.sadr_rules, sadr, &c.sadr)",
                 "&& choice::check(&f.sadr, &[], sadr, &c.sadr)"),
                ("verse-ajuz-replay", "verse.rs",
                 "&& choice::check(&f.ajuz, &f.ajuz_rules, ajuz, &c.ajuz)",
                 "&& choice::check(&f.ajuz, &[], ajuz, &c.ajuz)"),
                ("certificate-profile", "verse.rs",
                 "if !validate(p) || c.profile != p.id || c.context != scope.context || c.form >= p.forms.len() {",
                 "if !validate(p) || c.profile != c.profile || c.context != scope.context || c.form >= p.forms.len() {"),
                ("alphabet", "adapter.rs", "Beat::Mutaharrik => 1,", "Beat::Mutaharrik => 0,"),
                ("input-bounds", "language.rs",
                 "if at > input.len() || word.len() > input.len() - at {\n        return false;",
                 "if at > input.len() || word.len() > input.len() - at {\n        return true;"),
                ("ownership", "morphology.rs",
                 "    scope != Scope::Zihaf || (letter.sabab && letter.offset == 1)\n}", "    true\n}"),
                ("poem-obligation", "obligations.rs",
                 "return if check_requirements(r, &selected)", "return if true"),
                ("adjacency", "adjacency.rs",
                 "&& !(p.actions[l] == Action::Delete && q.actions[r] == Action::Delete)", "&& true"),
                ("profile-completeness", "structured.rs",
                 "if !verse::validate(&target)", "if true"),
                ("circle-word", "layout.rs",
                 "if pattern[j] != word[index] { return false; }", "if false { return false; }"),
                ("circle-basis", "profile.rs",
                 "|| !check_basis(&source,&feet)", "|| false"),
                ("normalize-coordinate", "normalize.rs",
                 "Zihaf::Tayy => 3usize,", "Zihaf::Tayy => 4usize,"),
                ("normalize-precondition", "normalize.rs",
                 "if letters[at].beat != expected {\n        assert(edit_norm(before, letters@, at as int, expected, action) == Err(Error::Precondition));",
                 "if false {\n        assert(edit_norm(before, letters@, at as int, expected, action) == Err(Error::Precondition));"),
                ("normalize-suffix", "normalize.rs",
                 "p.suffix.extend_from_slice(&[1, 0]);", "p.suffix.extend_from_slice(&[1, 1]);"),
                ("expand-completeness", "expand.rs",
                 "if choice.len() != slots.len() { return false; }",
                 "if false { return false; }"),
                ("expand-adjacency", "expand.rs",
                 "if !adjacency::check_pair(\n            &slots[rule.left].programs[choice[rule.left]],\n            &slots[rule.right].programs[choice[rule.right]],\n            rule.left_letter,\n            rule.right_letter,\n        ) {",
                 "if false {"),
                ("expand-skip-first", "expand.rs",
                 "    let mut c = 0;\n    while c < slots[k].programs.len()",
                 "    let mut c = 1;\n    while c < slots[k].programs.len()"),
                ("expand-prefix-order", "expand.rs",
                 "        out.push(prefix[i]);", "        out.push(prefix[prefix.len() - 1 - i]);"),
                ("expand-order-reversed", "expand.rs",
                 "        out.append(&mut tail);\n        assert(out@ =~= before + tail_view);",
                 "        let mut reversed = tail;\n        reversed.append(&mut out);\n        out = reversed;"),
                ("catalog-sadr-alternative", "catalog.rs",
                 "    let mut s = 0;\n    while s < sadr_choices.len()",
                 "    let mut s = 1;\n    while s < sadr_choices.len()"),
                ("catalog-ajuz-alternative", "catalog.rs",
                 "        let mut a = 0;\n        while a < ajuz_choices.len()",
                 "        let mut a = 1;\n        while a < ajuz_choices.len()"),
                ("catalog-invented-recipe", "catalog.rs",
                 "        let one = vec![row[chosen].clone()];",
                 "        let one = vec![row[chosen].clone(), row[chosen].clone()];"),
                ("catalog-source-index", "catalog.rs",
                 "        let chosen = choice[i];", "        let chosen = choice[0];"),
                ("catalog-context", "catalog.rs",
                 "                context: source.context,\n                initial_only: source.initial_only,\n                class: source.class,\n                sadr: sadr_selected,",
                 "                context: source.context + 1,\n                initial_only: source.initial_only,\n                class: source.class,\n                sadr: sadr_selected,"),
                ("catalog-class", "catalog.rs",
                 "                context: source.context,\n                initial_only: source.initial_only,\n                class: source.class,\n                sadr: sadr_selected,",
                 "                context: source.context,\n                initial_only: source.initial_only,\n                class: DarbClass(source.class.0 + 1),\n                sadr: sadr_selected,"),
                ("catalog-requirement-index", "catalog.rs",
                 "                output.push(p);", "                output.push(p + 1);"),
                ("catalog-requirement-empty-output", "catalog.rs",
                 "        if output.is_empty() {",
                 "        if false {"),
                ("catalog-empty-invented-form", "catalog.rs",
                 "    assert(contiguous(ranges@)) by {\n        reveal(contiguous);\n    }\n    assert(ranges@.len() == groups@.len());",
                 "    if groups.len() == 0 {\n        out.push(Form {\n            constraints: vec![],\n            feet: 0,\n            context: 0,\n            initial_only: false,\n            class: DarbClass(0),\n            sadr: vec![],\n            ajuz: vec![],\n        });\n    }\n    assert(contiguous(ranges@)) by {\n        reveal(contiguous);\n    }\n    assert(ranges@.len() == groups@.len());"),
            ]
            for label, filename, old, new in controls:
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
                if not any(reason in output for reason in ["postcondition not satisfied", "invariant not satisfied", "assertion failed", "type invariant"]):
                    raise RuntimeError("control failed for a reason other than the proof obligation")


if __name__ == "__main__":
    main()
