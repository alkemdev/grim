//! Integration tests for the apply engine against a real (temporary) filesystem.

use std::fs;
use std::path::Path;

use grim_apply::{Change, RenderContext, execute, plan};
use grim_core::{Arch, Facts, Os};

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn applies_templates_and_plain_files_with_conventions() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("files");
    let target = tmp.path().join("home");

    write(&src.join("dot_zshrc"), "export A=1\n");
    write(
        &src.join("dot_config/app/conf.toml.tmpl"),
        "host = \"{{ facts.hostname }}\"\n",
    );
    write(&src.join("private_dot_secret"), "shh\n");

    let facts = Facts::new(Os::Linux, Arch::X86_64).with_hostname("box");
    let ctx = RenderContext::new(facts, vec!["linux".into()]);

    let p = plan(&src, &target, &ctx).unwrap();
    assert!(p.changed().count() >= 3, "everything is new");
    execute(&p, false).unwrap();

    assert_eq!(
        fs::read_to_string(target.join(".zshrc")).unwrap(),
        "export A=1\n"
    );
    assert_eq!(
        fs::read_to_string(target.join(".config/app/conf.toml")).unwrap(),
        "host = \"box\"\n"
    );
    assert_eq!(fs::read_to_string(target.join(".secret")).unwrap(), "shh\n");

    // A second plan against the now-applied target sees no changes.
    let again = plan(&src, &target, &ctx).unwrap();
    assert_eq!(again.changed().count(), 0, "re-apply is a no-op");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(target.join(".secret"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "private_ sets 0600");
    }
}

#[test]
fn dry_run_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("files");
    let target = tmp.path().join("home");
    write(&src.join("dot_thing"), "x\n");

    let ctx = RenderContext::new(Facts::new(Os::Macos, Arch::Aarch64), vec![]);
    let p = plan(&src, &target, &ctx).unwrap();
    execute(&p, true).unwrap();

    assert!(!target.join(".thing").exists(), "dry run must not write");
}

#[test]
fn update_produces_a_diff() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("files");
    let target = tmp.path().join("home");
    write(&src.join("dot_conf"), "v2\n");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join(".conf"), "v1\n").unwrap();

    let ctx = RenderContext::new(Facts::new(Os::Linux, Arch::X86_64), vec![]);
    let p = plan(&src, &target, &ctx).unwrap();
    let action = p
        .actions
        .iter()
        .find(|a| a.target.ends_with(".conf"))
        .unwrap();
    assert_eq!(action.change, Change::Update);
    let diff = action.diff.as_ref().unwrap();
    assert!(
        diff.contains("v1") && diff.contains("v2"),
        "diff shows both versions"
    );
}
