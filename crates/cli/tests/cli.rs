use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_watad"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn scans_explicit_beats_and_reports_named_span_witnesses() {
    let out = run(&[
        "scan",
        "/0/0//0 /0/0//0 /0//0",
        "/0/0//0 /0/0//0 /0//00",
        "--meter",
        "11",
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("السريع"));
    assert!(text.contains("الكَسْف"));
    assert!(text.contains("0..7"));
    assert_eq!(
        run(&["scan", "0", "0", "--meter", "11"]).status.code(),
        Some(1)
    );
    assert_eq!(run(&["scan", "عربي", "0"]).status.code(), Some(2));
    assert_eq!(
        run(&["scan", "0", "0", "--meter", "99"]).status.code(),
        Some(2)
    );
}

#[test]
fn lists_all_rotations_and_keeps_unused_circle_profiles_explicit() {
    let out = run(&["meters"]);
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap().lines().count(), 22);
    assert_eq!(run(&["example", "2"]).status.code(), Some(2));
    assert!(run(&["example", "2", "--circle"]).status.success());
    let out = run(&["example", "9", "3"]); // Explicit single-line rajaz.
    assert!(out.status.success());
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .trim_end()
            .ends_with('|')
    );
}

#[test]
fn poem_command_enforces_global_witnesses_and_distinguishes_bad_input() {
    let path = std::env::temp_dir().join(format!("watad-cli-poem-{}.txt", std::process::id()));
    let name = path.to_str().unwrap();
    let altered = "//0///0 //0/0/0 | //0///0 //0///0\n";
    std::fs::write(&path, altered).unwrap();
    assert_eq!(run(&["poem", name, "--meter", "5"]).status.code(), Some(1));
    std::fs::write(
        &path,
        format!("{altered}//0///0 //0///0 | //0///0 //0///0\n"),
    )
    .unwrap();
    let out = run(&["poem", name, "--meter", "5"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::write(&path, "//0/0").unwrap();
    assert_eq!(run(&["poem", name]).status.code(), Some(2));
    let first = "1: //0/0 //0/0/0 //0/0 //0/0/0 | //0/0 //0/0/0 //0/0 //0/0/0\n";
    let ordinary = "//0/0 //0/0/0 //0/0 //0//0 | //0/0 //0/0/0 //0/0 //0/0/0\n";
    std::fs::write(&path, format!("{first}{ordinary}")).unwrap();
    assert!(run(&["poem", name, "--meter", "0"]).status.success());
    std::fs::write(&path, format!("{ordinary}{first}")).unwrap();
    assert_eq!(run(&["poem", name, "--meter", "0"]).status.code(), Some(1));
    std::fs::remove_file(&path).unwrap();
}
