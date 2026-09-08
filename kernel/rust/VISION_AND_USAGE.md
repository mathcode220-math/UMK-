# Universal Mathematical Kernel — Vision and Usage

This file is the instruction document for the Rust kernel after the vision-alignment work.

The kernel was tested after those changes:

```text
cargo test --manifest-path kernel/rust/Cargo.toml
16 passed; 0 failed
```

---

## 1. Vision

The kernel is not a computer algebra system.

```text
Kernel != Mathematics
Kernel + Laws = Mathematical System
```

The kernel knows nothing about groups, rings, plus, times, sine, or numbers as arithmetic. It only knows:

1. How to represent expressions
2. How to match patterns
3. How to apply first-class rewrite laws
4. How to check types
5. How to record proof objects
6. How to intern identical nodes
7. How to use algebraic properties only when a Space declares them

Mathematics enters the system as data: symbols, laws, and spaces loaded at runtime.

If the kernel is later taught calculus, matrices, or probability by adding kernel code, the vision has failed. Those domains must arrive as libraries of laws.

---

## 2. Architectural invariants

Keep these as hard rules.

| Invariant | Meaning |
| --- | --- |
| Universal expression | Every mathematical object is `Head[Arguments]` |
| No hidden algebra | The kernel never assumes associativity, commutativity, or identity |
| Law is data | Rewrite rules are objects, not `if/else` inside an evaluator |
| Space determines meaning | The same expression can mean different things in different spaces |
| Atoms are opaque | `Int(5)` is a literal, not a number the kernel can add |
| Cycle detection is not termination | Seeing a repeated node is not a proof that the system terminates |
| AC is declared, never global | Flattening and sorting happen only if a Space says so |

Forbidden inside the kernel:

- `Plus`, `Times`, `Sin`
- numeric evaluation such as `5 + 5 = 10`
- default commutativity
- Mathics, calculus, matrices, probability, monads

---

## 3. Expression model

There are exactly four constructors:

```text
Expr ::= Symbol(name, type?)
       | Atom(value, type?)
       | Variable(name, type?)
       | Apply(head, args, type?)
```

`Apply` is the only composite form. The intended shape is:

```text
Head[Arguments]
```

Examples:

```text
star(a, e)
Plus(x, Plus(y, z))
inv(a)
```

`Variable` is for patterns. `Symbol` is for names that exist in a space. Do not mix them.

`AtomValue` may be `Int`, `Float`, or `Str`. These are opaque literals. The kernel must not implement arithmetic on them.

---

## 4. Types and phases

Types are:

```text
Type ::= AtomType(name)
       | AbstractType(name)
       | Arrow(domain, codomain)
```

An untyped expression is allowed as syntax. A mathematically valid expression is not the same thing.

Use three phases:

```text
Parsed -> Typed -> Validated
```

| Phase | Meaning |
| --- | --- |
| Parsed | The expression exists as syntax |
| Typed | A type was inferred or checked |
| Validated | The expression has a type and is accepted in that space |

API:

```rust
use umk_kernel::{Expr, Judgement, Phase, Type};

let r = Type::atom("R");
let f = Expr::symbol("f", Some(Type::arrow(r.clone(), r.clone())));
let x = Expr::symbol("x", Some(r));
let expr = Expr::apply(f, vec![x]);

let parsed = Judgement::parsed(expr);
assert_eq!(parsed.phase, Phase::Parsed);
let validated = parsed.validated().unwrap();
assert_eq!(validated.phase, Phase::Validated);
```

---

## 5. Laws and spaces

A law is a first-class rewrite rule:

```text
Law(name, pattern, replacement)
```

A space is more than a bag of laws. Treat it as:

```text
Space = (Signature, Typing, Laws, Semantics)
```

| Part | In the kernel | Meaning |
| --- | --- | --- |
| Signature | `operations` | Which symbols exist |
| Typing | types on those symbols | Domains and codomains |
| Laws | `laws` | Rewrite rules |
| Semantics | `properties` plus interpret | Declared algebraic meaning |

Properties are not kernel assumptions. A space may declare:

```text
Associative(Plus)
Commutative(Plus)
```

Another space may declare neither. Then:

```text
Plus(x, Plus(y, z))
Plus(Plus(z, x), y)
```

are equal only in the AC space.

---

## 6. Rewrite strategies

The names are not aliases.

| Strategy | Semantics |
| --- | --- |
| Innermost | Fully rewrite a redex that has no inner redex first |
| BottomUp | Normalize children as far as the step budget allows, then rewrite the parent |
| Outermost | Try the parent first; only then rewrite one child |

Cycle detection uses interned node IDs. If the same node is seen again, the engine stops and sets `cycled = true`. That reports a cycle. It does not prove termination.

---

## 7. Proof objects

A compact name list is not enough. Each step now records:

```text
ProofObject = (
  LawID,
  Instantiation,
  PremiseProofs,
  Before,
  After
)
```

Example:

```text
(a star e) -> a
law: right_identity
instantiation: x |-> a
```

That is stronger than saying only `"right_identity"`.

Verification:

- `verify(start, expected, laws, names)` replays law names
- `verify_objects(...)` checks law id, instantiation, before, and after

Tampering with a proof must fail.

---

## 8. Semantics

Interpretation is space-relative:

```text
[[Expr]]_{Space, rho}
```

The kernel substitutes the environment, rewrites with the space laws, then canonicalizes only with properties declared by that space.

The result is still an expression. The kernel does not evaluate mathematics; it rewrites structure.

---

## 9. Crate layout

```text
kernel/rust/
  src/
    expr.rs         Universal expression
    types.rs        Atom, Abstract, Arrow
    pattern.rs      Match and substitute
    space.rs        Law, Space, declared properties
    rewrite.rs      Strategies
    proof.rs        Proof objects and verification
    intern.rs       Hash-consing / node IDs
    ac.rs           Space-declared AC canonicalization
    semantics.rs    Space-relative interpretation
    typecheck.rs    Typing and validation phases
    lib.rs
    bin/umk.rs      Small demo
  tests/core.rs
  VISION_AND_USAGE.md
  README.md
```

---

## 10. Build and test

From the repository root:

```bash
export PATH="$HOME/.cargo/bin:$PATH"

cargo test --manifest-path kernel/rust/Cargo.toml
```

Run the demo:

```bash
cargo run --manifest-path kernel/rust/Cargo.toml --bin umk
```

Expected demo shape:

```text
input  star(star(a, e), star(e, b))
output star(a, b)
proof  right_identity -> left_identity
```

---

## 11. Usage: load mathematics the kernel has never seen

This is the critical experiment. The kernel does not know what `star` means.

```rust
use umk_kernel::{Expr, Law, RewriteEngine, Type};

let a_ty = Type::abstract_carrier("A");
let star = Expr::symbol(
    "star",
    Some(Type::arrow(a_ty.clone(), Type::arrow(a_ty.clone(), a_ty.clone()))),
);
let e = Expr::symbol("e", Some(a_ty.clone()));
let a = Expr::symbol("a", Some(a_ty.clone()));
let b = Expr::symbol("b", Some(a_ty.clone()));
let x = Expr::var("x", Some(a_ty.clone()));
let y = Expr::var("y", Some(a_ty.clone()));
let z = Expr::var("z", Some(a_ty));

let laws = vec![
    Law::new("left_identity", Expr::apply(star.clone(), vec![e.clone(), x.clone()]), x.clone()),
    Law::new("right_identity", Expr::apply(star.clone(), vec![x.clone(), e.clone()]), x.clone()),
    Law::new(
        "associativity",
        Expr::apply(star.clone(), vec![Expr::apply(star.clone(), vec![x.clone(), y.clone()]), z.clone()]),
        Expr::apply(star.clone(), vec![x, Expr::apply(star.clone(), vec![y, z])]),
    ),
];

let expr = Expr::apply(
    star.clone(),
    vec![
        Expr::apply(star.clone(), vec![a.clone(), e.clone()]),
        Expr::apply(star.clone(), vec![e, b.clone()]),
    ],
);

let (result, proof) = RewriteEngine::bottom_up(&expr, &laws, 10);
assert_eq!(result, Expr::apply(star, vec![a, b]));
assert_eq!(proof, vec!["right_identity".to_string(), "left_identity".to_string()]);
```

The kernel never saw a monoid. It only applied the laws it was given.

---

## 12. Usage: no hidden algebra

Give identity, but not associativity:

```text
(a star b) star c
```

must stay unchanged.

Then add associativity and reduce again. The result may become:

```text
a star (b star c)
```

If the first run rewrites without the associativity law, the kernel has hidden algebra and the vision is broken.

---

## 13. Usage: AC only when a space declares it

```rust
use umk_kernel::{ac_eq, canonicalize, Expr, OpProperty, Space, Type};

let a_ty = Type::abstract_carrier("A");
let plus = Expr::symbol(
    "Plus",
    Some(Type::arrow(a_ty.clone(), Type::arrow(a_ty.clone(), a_ty.clone()))),
);
let x = Expr::symbol("x", Some(a_ty.clone()));
let y = Expr::symbol("y", Some(a_ty.clone()));
let z = Expr::symbol("z", Some(a_ty.clone()));

let left = Expr::apply(plus.clone(), vec![x.clone(), Expr::apply(plus.clone(), vec![y.clone(), z.clone()])]);
let right = Expr::apply(plus.clone(), vec![Expr::apply(plus.clone(), vec![z, x]), y]);

let ac = Space::new("AC", vec![a_ty.clone()], vec![plus.clone()], vec![]).with_properties(vec![
    ("Plus".into(), OpProperty::Associative),
    ("Plus".into(), OpProperty::Commutative),
]);
let non_ac = Space::new("NonAC", vec![a_ty], vec![plus], vec![]);

assert!(ac_eq(&left, &right, &ac));
assert!(!ac_eq(&left, &right, &non_ac));
```

Do not put commutativity into the kernel. Put it on the space.

---

## 14. Usage: proof objects

```rust
use umk_kernel::{RewriteEngine, Strategy};

let reduction = RewriteEngine::reduce(&expr, &laws, 10, Strategy::Innermost);
let step = &reduction.steps[0];
println!("{}", step.law_id);
println!("{}", step.instantiation.get("x").unwrap());
println!("{} -> {}", step.before, step.after);
```

If a proof is edited by hand, verification must return false.

---

## 15. Usage: interpretation

```rust
use umk_kernel::{interpret, Environment, Space};

let space = Space::new("Monoid", vec![], vec![], laws);
let empty = Space::new("Empty", vec![], vec![], vec![]);
let rho = Environment::new();

let in_monoid = interpret(&expr, &space, &rho, 10);
let in_empty = interpret(&expr, &empty, &rho, 10);
```

Same expression. Different spaces. Different meaning.

---

## 16. Tests that protect the vision

| Test | Claim it protects |
| --- | --- |
| `monoid_reduction` | Mathematics the kernel has never seen |
| `non_commutative` | No hidden commutativity |
| `no_hidden_associativity` | No hidden algebra |
| `different_spaces` | Structure determines meaning |
| `space_declared_ac_is_not_global` | AC is not a kernel axiom |
| `type_safety` | Invalid applications are rejected |
| `proof_object_records_instantiation` | Proofs are objects, not name lists |
| `cycle_is_detected_not_claimed_terminating` | Cycle detection is not a termination proof |
| `semantics_are_space_relative` | Meaning lives in the space |
| `innermost_and_outermost_are_not_aliases` | Strategies have distinct semantics |
| `kernel_has_no_builtin_math` | Static ban on Plus/Times/Sin/Monoid/Group in kernel sources |

---

## 17. What this kernel does not claim

Do not claim these yet:

- A full mathematical type theory
- A proof of confluence
- A proof of termination
- A complete AC matcher for every pattern shape
- A Mathics replacement
- Built-in calculus, linear algebra, or probability

Those belong outside the kernel, or later, after the invariants above remain true.

---

## 18. Working rule for future changes

Before adding a feature, ask:

1. Does the kernel still know no mathematics?
2. Can the feature be expressed as a law or a space property?
3. Does the same expression still behave differently in two spaces?
4. Is the proof still an object with instantiation?
5. Did `cargo test --manifest-path kernel/rust/Cargo.toml` pass?

If the answer to (1) or (2) is no, do not put it in the kernel.
