use umk_kernel::{type_check, AtomValue, Expr, Interner, Law, RewriteEngine, Space, Type, TypeCheckError};

fn carrier_a() -> Type {
    Type::abstract_carrier("A")
}

fn star() -> Expr {
    let a = carrier_a();
    Expr::symbol("star", Some(Type::arrow(a.clone(), Type::arrow(a.clone(), a))))
}

fn typed_symbol(name: &str) -> Expr {
    Expr::symbol(name, Some(carrier_a()))
}

fn var(name: &str) -> Expr {
    Expr::var(name, Some(carrier_a()))
}

fn monoid_laws() -> Vec<Law> {
    let star = star();
    let e = typed_symbol("e");
    let x = var("x");
    let y = var("y");
    let z = var("z");
    vec![
        Law::new(
            "left_identity",
            Expr::apply(star.clone(), vec![e.clone(), x.clone()]),
            x.clone(),
        ),
        Law::new(
            "right_identity",
            Expr::apply(star.clone(), vec![x.clone(), e]),
            x.clone(),
        ),
        Law::new(
            "associativity",
            Expr::apply(
                star.clone(),
                vec![Expr::apply(star.clone(), vec![x.clone(), y.clone()]), z.clone()],
            ),
            Expr::apply(
                star.clone(),
                vec![x, Expr::apply(star, vec![y, z])],
            ),
        ),
    ]
}

#[test]
fn monoid_reduction() {
    let star = star();
    let e = typed_symbol("e");
    let a = typed_symbol("a");
    let b = typed_symbol("b");
    let laws = monoid_laws();
    let expr = Expr::apply(
        star.clone(),
        vec![
            Expr::apply(star.clone(), vec![a.clone(), e.clone()]),
            Expr::apply(star.clone(), vec![e, b.clone()]),
        ],
    );
    let (result, proof) = RewriteEngine::bottom_up(&expr, &laws, 10);
    let expected = Expr::apply(star, vec![a, b]);
    assert_eq!(result, expected);
    assert_eq!(proof, vec!["right_identity", "left_identity"]);
}

#[test]
fn non_commutative() {
    let star = star();
    let a = typed_symbol("a");
    let b = typed_symbol("b");
    let x = var("x");
    let laws = vec![Law::new(
        "left_identity",
        Expr::apply(star.clone(), vec![typed_symbol("e"), x.clone()]),
        x,
    )];
    let expr = Expr::apply(star, vec![a, b]);
    let (result, proof) = RewriteEngine::bottom_up(&expr, &laws, 10);
    assert_eq!(result, expr);
    assert!(proof.is_empty());
}

#[test]
fn different_spaces() {
    let a = carrier_a();
    let plus = Expr::symbol(
        "Plus",
        Some(Type::arrow(a.clone(), Type::arrow(a.clone(), a.clone()))),
    );
    let zero = Expr::symbol("Zero", Some(a.clone()));
    let x = Expr::symbol("x", Some(a.clone()));
    let vx = Expr::var("x", Some(a.clone()));
    let space1 = Space::new(
        "SpaceWithIdentity",
        vec![a.clone()],
        vec![plus.clone(), zero.clone()],
        vec![Law::new(
            "right_identity",
            Expr::apply(plus.clone(), vec![vx, zero.clone()]),
            Expr::symbol("x", Some(a)),
        )],
    );
    let space2 = Space::new(
        "SpaceWithoutIdentity",
        vec![carrier_a()],
        vec![plus.clone(), zero.clone()],
        vec![],
    );
    let expr = Expr::apply(plus, vec![x.clone(), zero]);
    let (result1, proof1) = RewriteEngine::bottom_up(&expr, &space1.laws, 10);
    let (result2, proof2) = RewriteEngine::bottom_up(&expr, &space2.laws, 10);
    assert_eq!(result1, x);
    assert_eq!(proof1, vec!["right_identity"]);
    assert_eq!(result2, expr);
    assert!(proof2.is_empty());
}

#[test]
fn type_safety() {
    let r = Type::atom("R");
    let group = Type::atom("Group");
    let f = Expr::symbol("f", Some(Type::arrow(r.clone(), r.clone())));
    let x = Expr::symbol("x", Some(r.clone()));
    let g = Expr::symbol("G", Some(group));
    let valid = Expr::apply(f.clone(), vec![x]);
    let inferred = type_check(&valid).expect("valid application");
    assert_eq!(inferred, Some(r.clone()));
    assert_eq!(valid.ty(), Some(&r));
    let invalid = Expr::apply(f.clone(), vec![g]);
    let err = type_check(&invalid).expect_err("invalid application");
    assert!(matches!(err, TypeCheckError(_)));
    let extra = Expr::apply(
        f,
        vec![
            Expr::symbol("x", Some(r.clone())),
            Expr::symbol("y", Some(r)),
        ],
    );
    let extra_err = type_check(&extra).expect_err("too many arguments");
    assert!(extra_err.0.contains("Too many arguments"));
}

#[test]
fn proof_trace_replayable() {
    let star = star();
    let e = typed_symbol("e");
    let a = typed_symbol("a");
    let b = typed_symbol("b");
    let laws = monoid_laws();
    let expr = Expr::apply(
        star.clone(),
        vec![
            Expr::apply(star.clone(), vec![a.clone(), e.clone()]),
            Expr::apply(star, vec![e, b.clone()]),
        ],
    );
    let (result, proof) = RewriteEngine::bottom_up(&expr, &laws, 10);
    assert_eq!(proof.len(), 2);
    let (replay_result, replay_proof) = RewriteEngine::bottom_up(&expr, &laws, 10);
    assert_eq!(replay_result, result);
    assert_eq!(replay_proof, proof);
    assert!(umk_kernel::verify(&expr, &result, &laws, &proof));
    let mut tampered = proof.clone();
    tampered[0] = "associativity".to_string();
    assert!(!umk_kernel::verify(&expr, &result, &laws, &tampered));
}

#[test]
fn apply_stores_inferred_type() {
    let star = star();
    let a = typed_symbol("a");
    let b = typed_symbol("b");
    let expr = Expr::apply(star, vec![a, b]);
    assert_eq!(expr.ty(), Some(&carrier_a()));
}

#[test]
fn float_nan_is_not_equal() {
    let left = Expr::atom(AtomValue::Float(f64::NAN), None);
    let right = Expr::atom(AtomValue::Float(f64::NAN), None);
    assert_ne!(left, right);
    let zero = Expr::atom(AtomValue::Float(0.0), None);
    let neg_zero = Expr::atom(AtomValue::Float(-0.0), None);
    assert_eq!(zero, neg_zero);
}

#[test]
fn interner_reuses_identical_nodes() {
    let mut intern = Interner::new();
    let a = typed_symbol("a");
    let first = intern.intern(&a);
    let second = intern.intern(&typed_symbol("a"));
    assert_eq!(first, second);
    assert_eq!(intern.len(), 1);
}

#[test]
fn kernel_has_no_builtin_math() {
    let src = include_str!("../src/lib.rs");
    let rewrite = include_str!("../src/rewrite.rs");
    let expr = include_str!("../src/expr.rs");
    for forbidden in ["Plus", "Times", "Sin", "Monoid", "Group"] {
        assert!(
            !src.contains(forbidden) && !rewrite.contains(forbidden) && !expr.contains(forbidden),
            "kernel must not mention {forbidden}"
        );
    }
}
