# UMK Kernel (Rust)

Production rewrite kernel for the Universal Mathematical Kernel.

The kernel knows nothing about mathematics. It only:

- represents expressions (`Symbol | Atom | Variable | Apply`)
- matches patterns
- applies first-class rewrite laws
- checks types
- records compact proof traces

```
Kernel != Mathematics
Kernel + Laws = Mathematical System
```

## Layout

```
src/
  expr.rs
  types.rs
  pattern.rs
  rewrite.rs
  space.rs
  proof.rs
  intern.rs
  typecheck.rs
tests/
  core.rs
```

## Tests

```bash
cargo test --manifest-path kernel/rust/Cargo.toml
```
