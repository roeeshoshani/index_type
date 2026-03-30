# Repository Guidelines

## Project Structure & Module Organization
`src/` contains the main `index_type` library. Core modules include `typed_vec.rs`, `typed_array.rs`, `typed_array_vec.rs`, `typed_range_iter.rs`, and `typed_slice/` for slice-specific logic. Shared traits and scalar helpers live in `base_index_types.rs`, `index_scalar_types.rs`, `error.rs`, and `utils.rs`.

`index_type_macros/` is a workspace member that provides the derive macros used by the main crate. Integration and regression tests live in `tests/`; keep new coverage there unless a private helper needs a unit test in the defining module. `README.md` doubles as public usage documentation, and its examples are exercised by doctests.

## Build, Test, and Development Commands
Use Cargo from the repository root:

- `cargo test`: runs integration tests and doctests for the workspace.
- `cargo miri test`: runs tests under Miri to detect undefined behavior.
- `cargo fmt --check`: verifies standard Rust formatting.
- `cargo clippy --all-targets --all-features`: runs lint checks across library code, tests, and examples.
- `cargo test --test typed_vec`: runs a focused integration test file while iterating on one area.

## README Generation
The `README.md` file is generated from the crate documentation in `src/lib.rs` using `cargo readme`. To regenerate it after updating docs, run:

```
cargo readme > README.md
```

Do not edit `README.md` directly; any changes will be overwritten. Update the documentation in `src/lib.rs` instead.

## Coding Style & Naming Conventions
Follow default `rustfmt` style with 4-space indentation and standard import ordering. Use `snake_case` for modules, files, functions, and test names. Public types and traits use `UpperCamelCase`; index newtypes follow the existing pattern like `NodeId`, `MyIndex`, or `BufferIndex`.

Prefer small, explicit APIs and keep `no_std` compatibility in mind. If a change touches user-facing behavior, update `README.md` examples or docs in `src/lib.rs`.

## Testing Guidelines
This project aims for high test coverage and high quality tests to ensure correctness. All tests must pass under both standard execution and Miri (Rust's interpreter for detecting undefined behavior).

Add regression coverage in `tests/` next to the affected feature area, for example `tests/typed_slice.rs` or `tests/typed_range_iter.rs`. Name tests with a `test_` prefix and describe the behavior under test, such as `test_try_push_overflow`.

Before opening a PR, run `cargo test` and `cargo miri test`. Run `cargo fmt --check` to verify formatting. Always run `cargo clippy --all-targets --all-features` before opening a PR; the project must always have zero warnings.

## Commit & Pull Request Guidelines
Recent commits use short, imperative summaries such as `minor fixes` and `add tests for typed range iter`. Keep commit subjects concise and specific.

PRs should explain the behavior change, note any `no_std` or macro impact, and mention added tests. Include code snippets or example usage when public APIs or docs change.
