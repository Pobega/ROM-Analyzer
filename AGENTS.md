# ROM-Analyzer AGENTS.md

## What this repository is

This repository contains code that analyzes video game ROM file headers to figure out their region and other metainfo stored in the header. The metainfo stored is different for each console.
  * The codebase is written in Rust, and optimized to run on devices such as handheld Linux game consoles.
  * Consider if changes will affect compiling against an aarch64 target before adding new project dependencies.
  * Opt for readability over smart implementations; This code ultimately has to be maintained by humans.

## How to speak

When suggesting changes interactively please try to explain your logic, and why it would be good for the code base. When possible, please reference upstream Rust/Crate documentation referencing your claims.

## How to code

When writing code:
  * Be succinct over verbose. Explain the change in a short manner, but without losing context of the implementation.
  * Do not explain that we added tests, unless that's the only change in the code.

For comments:
  * Any comment that is on its own line should be a full sentence, and end with punctuation (preferably a period.)

## How to test

Whenever code changes are made, please run the following commands to ensure that everything works as expected:
  * cargo check
    * This is the first test to check that there are no build errors.
    * Only run this as a quick-check between changes, this is not necessary if cargo clippy or cargo test are going to be run.
  * cargo fmt
    * Formats the code.
  * cargo clippy --all-targets --all-features -- -D warnings
    * This is for linting, to see if Cargo has better suggestions for our new implementation.
  * cargo test --quiet
    * This runs all the tests in the code. We want to ensure that we don't break any tests when making code changes.
  * cargo mutants -f changed_file
    * After a file is changed, we should run mutation tests on it to ensure we didn't introduce any new failures.
    * You can run it sequentially on each changed file (running it for the whole codebase is time consuming)

## How to commit code

You should never commit code without asking the user first.

If the user explicitly tells you to do so, then follow these instructions:
  * Provide a succinct but useful error message.
  * Do not explain every individual change, but rather the overview of the change(s).
  * Prefer a one-line commit message (92 characters wide), but if a change is larger in scope:
    * Write a short summary for the first line that clearly explains the changes.
    * Write bullet points that go into a bit detail on each important change.
    * Link to upstream docs when applicable.

## Cross-compilation testing

When adding new features that may break cross-compilation support, you can test really quickly by using 'cross'
  * cross build --target=aarch64-unknown-linux-gnu
  * cross test --target=aarch64-unknown-linux-gnu
