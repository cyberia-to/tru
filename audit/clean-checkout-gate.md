# tru builds and tests clean against its siblings' origin default branches

date: 2026-09-23 · property #39 · repo: tru

## claim

property 39 asks whether every phase-1 component builds and tests from a
clean checkout of its default branch against the default branches of its
siblings, with no dead path, no version pin behind a sibling, and no
crate present only in an owner's working tree. the sweep of 2026-09-22/23
found bbg, cybergraph, foculus, mudra, neuron, nox, prysm, zheng,
true-cyber, glia, honeycrisp, rune, vault and radio failing this gate.
tru was not in that list. this measures it directly.

## method

`origin/master` of [[tru]] checked out in an isolated worktree
(`89416291dd4cc993a79d63cbd3e2445294b173d3`). tru's three path
dependencies are [[strata]]'s `nebu` and `core`/`compute`/`ext` crates,
[[hemera]], and [[honeycrisp]]'s `acpu`/`unimem`. strata's owner working
tree already matched its origin default branch
(`56aedb2d12b3126c601eb333419136d403614dbb`); hemera and honeycrisp did
not (hemera's tree carried unpushed local commits, honeycrisp's tree was
one commit behind origin and several ahead), so each was checked out
fresh from `origin/main` into a detached worktree and substituted for the
owner's tree for this run: hemera at
`23f3bbcff910ea6d504ceb505680a539260869da`, honeycrisp at
`9593c7218e14e2b5b816d9008b45ab60b9580cf3`.

## result

```
$ cargo check --tests --workspace
    Checking cyber-tru v0.1.1 (rs)
    Checking tru-cli v0.1.0 (cli)
warning: unused import: `super::*` (rs/pass/embed.rs:173, pre-existing, unrelated)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.05s

$ cargo test --workspace
test result: ok. 85 passed; 0 failed; 0 ignored (lib)
test result: ok. 1 passed; 0 failed; 0 ignored (tests/honesty_loop.rs)
test result: ok. 1 passed; 0 failed; 0 ignored (tests/smoke.rs)
test result: ok. 0 passed; 0 failed; 0 ignored (cli)
test result: ok. 0 passed; 0 failed; 1 ignored (doc-tests)
```

87 tests pass, 0 fail, on `rustc 1.98.0 (88d9e12ae 2026-08-18)`. no dead
path dependency, no pin behind a sibling's published version, no crate
resolved only from an owner's working tree. tru closes its row of
property 39; the row stays open until every listed repo is covered.

## caveat

this is one repo's slice of a multi-repo row. it says nothing about
tru's own dependents (plumb/tok, foculus) resolving tru correctly, and
nothing about the fourteen repos the sweep already found broken.

## re-run 2026-10-17

the 2026-09-23 numbers above are kept as measured. re-run during the
2026-10-17 branch triage, on `origin/master` of [[tru]] at
`74ecfb13517fad364048add5308e7c7feaa40d19` plus this page, in a
detached worktree, against detached worktrees of the siblings'
`origin/main`: [[strata]] `494df7a`, [[hemera]] `1c385e5` (cyber-hemera
0.3.1), and [[honeycrisp]] `3ffc0d5` (its main checkout was clean and
equal to origin, so it was linked rather than copied). rustc/cargo
1.95.0 (Homebrew).

```
$ cargo fmt --all --check                                         # clean
$ cargo clippy --locked --workspace --all-targets -- -D warnings  # clean
$ cargo test --locked --workspace
test result: ok. 164 passed; 0 failed; 0 ignored (lib)
test result: ok. 1 passed; 0 failed; 0 ignored (tests/honesty_loop.rs)
test result: ok. 1 passed; 0 failed; 0 ignored (tests/smoke.rs)
test result: ok. 0 passed; 0 failed; 0 ignored (cli)
test result: ok. 0 passed; 0 failed; 1 ignored (doc-tests)
$ cargo build --locked --workspace --examples                    # 0 warnings
```

166 tests pass, 0 fail, 0 warnings. the same day the GitHub CI went
green for the first time: it had failed on every run because the
workflow did not check out honeycrisp, whose `acpu` manifest cargo reads
even on linux (tru#38). tru still holds its slice of property 39.
