use umk_kernel::{
    ac_eq, canonicalize, interpret, type_check, AtomValue, Environment, Expr, Interner, Judgement,
    Law, OpProperty, Phase, RewriteEngine, Space, Strategy, Type, TypeCheckError,
};

fn carrier_a() -> Type {
    Type::abstract_carrier("A")
}

fn star() -> Expr {
    let a = carrier_a();
    Expr::symbol(
        "star",
        Some(Type::arrow(a.clone(), Type::arrow(a.clone(), a))),
    )
}

fn typed_symbol(name: &str) -> Expr {
    Expr::symbol(name, Some(carrier_a()))
}

fn var(name: &str) -> Expr {
    Expr::var(name, Some(carrier_a()))
}

fn plus() -> Expr {
    let a = carrier_a();
    Expr::symbol(
        "Plus",
        Some(Type::arrow(a.clone(), Type::arrow(a.clone(), a))),
    )
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
                vec![
                    Expr::apply(star.clone(), vec![x.clone(), y.clone()]),
                    z.clone(),
                ],
            ),
            Expr::apply(star.clone(), vec![x, Expr::apply(star, vec![y, z])]),
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
    let plus = plus();
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
    let reduction = RewriteEngine::reduce(&expr, &laws, 10, Strategy::Innermost);
    assert_eq!(reduction.law_names().len(), 2);
    assert!(reduction.steps.iter().all(|step| !step.instantiation.bindings.is_empty()
        || step.law_id.contains("identity")));
    assert!(umk_kernel::verify(
        &expr,
        &reduction.result,
        &laws,
        &reduction.law_names()
    ));
    let mut tampered = reduction.law_names();
    tampered[0] = "associativity".to_string();
    assert!(!umk_kernel::verify(
        &expr,
        &reduction.result,
        &laws,
        &tampered
    ));
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

#[test]
fn no_hidden_associativity() {
    let star = star();
    let a = typed_symbol("a");
    let b = typed_symbol("b");
    let c = typed_symbol("c");
    let x = var("x");
    let identity_only = vec![Law::new(
        "right_identity",
        Expr::apply(star.clone(), vec![x.clone(), typed_symbol("e")]),
        x,
    )];
    let expr = Expr::apply(
        star.clone(),
        vec![Expr::apply(star.clone(), vec![a.clone(), b.clone()]), c.clone()],
    );
    let (result, proof) = RewriteEngine::bottom_up(&expr, &identity_only, 10);
    assert_eq!(result, expr);
    assert!(proof.is_empty());

    let with_assoc = monoid_laws();
    let (assoc_result, assoc_proof) = RewriteEngine::bottom_up(&expr, &with_assoc, 10);
    let expected = Expr::apply(star.clone(), vec![a, Expr::apply(star, vec![b, c])]);
    assert_eq!(assoc_result, expected);
    assert_eq!(assoc_proof, vec!["associativity"]);
}

#[test]
fn space_declared_ac_is_not_global() {
    let plus = plus();
    let x = typed_symbol("x");
    let y = typed_symbol("y");
    let z = typed_symbol("z");
    let nested = Expr::apply(
        plus.clone(),
        vec![x.clone(), Expr::apply(plus.clone(), vec![y.clone(), z.clone()])],
    );
    let reordered = Expr::apply(
        plus.clone(),
        vec![Expr::apply(plus.clone(), vec![z.clone(), x.clone()]), y.clone()],
    );
    let ac_space = Space::new("AC", vec![carrier_a()], vec![plus.clone()], vec![]).with_properties(
        vec![
            ("Plus".into(), OpProperty::Associative),
            ("Plus".into(), OpProperty::Commutative),
        ],
    );
    let non_ac = Space::new("NonAC", vec![carrier_a()], vec![plus], vec![]);
    assert!(ac_eq(&nested, &reordered, &ac_space));
    assert!(!ac_eq(&nested, &reordered, &non_ac));
    let canonical = canonicalize(&nested, &ac_space);
    match canonical {
        Expr::Apply { args, .. } => assert_eq!(args.len(), 3),
        _ => panic!("expected flattened Plus"),
    }
}

#[test]
fn proof_object_records_instantiation() {
    let star = star();
    let e = typed_symbol("e");
    let a = typed_symbol("a");
    let laws = monoid_laws();
    let expr = Expr::apply(star, vec![a.clone(), e]);
    let reduction = RewriteEngine::reduce(&expr, &laws, 4, Strategy::Innermost);
    assert_eq!(reduction.result, a);
    assert_eq!(reduction.steps.len(), 1);
    let step = &reduction.steps[0];
    assert_eq!(step.law_id, "right_identity");
    assert_eq!(step.instantiation.get("x"), Some(&a));
}

#[test]
fn cycle_is_detected_not_claimed_terminating() {
    let a = typed_symbol("a");
    let b = typed_symbol("b");
    let laws = vec![
        Law::new("a_to_b", a.clone(), b.clone()),
        Law::new("b_to_a", b, a.clone()),
    ];
    let reduction = RewriteEngine::reduce(&a, &laws, 8, Strategy::Innermost);
    assert!(reduction.cycled);
}

#[test]
fn semantics_are_space_relative() {
    let star = star();
    let e = typed_symbol("e");
    let a = typed_symbol("a");
    let expr = Expr::apply(star, vec![a.clone(), e]);
    let space = Space::new("Monoid", vec![carrier_a()], vec![], monoid_laws());
    let empty = Space::new("Empty", vec![carrier_a()], vec![], vec![]);
    let rho = Environment::new();
    match interpret(&expr, &space, &rho, 8) {
        umk_kernel::SemanticValue::Opaque(value) => assert_eq!(value, a),
    }
    match interpret(&expr, &empty, &rho, 8) {
        umk_kernel::SemanticValue::Opaque(value) => assert_eq!(value, expr),
    }
}

#[test]
fn parsed_typed_validated_phases() {
    let r = Type::atom("R");
    let f = Expr::symbol("f", Some(Type::arrow(r.clone(), r.clone())));
    let x = Expr::symbol("x", Some(r));
    let expr = Expr::apply(f, vec![x]);
    let parsed = Judgement::parsed(expr);
    assert_eq!(parsed.phase, Phase::Parsed);
    let validated = parsed.validated().expect("typed");
    assert_eq!(validated.phase, Phase::Validated);
}

#[test]
fn innermost_and_outermost_are_not_aliases() {
    let f = Expr::symbol("f", None);
    let g = Expr::symbol("g", None);
    let x = Expr::symbol("x", None);
    let inner = Expr::apply(g.clone(), vec![x.clone()]);
    let outer = Expr::apply(f.clone(), vec![inner.clone()]);
    let laws = vec![
        Law::new("g_to_x", inner.clone(), x.clone()),
        Law::new("f_outer", outer.clone(), Expr::symbol("done", None)),
    ];
    let inn = RewriteEngine::reduce(&outer, &laws, 4, Strategy::Innermost);
    let out = RewriteEngine::reduce(&outer, &laws, 4, Strategy::Outermost);
    assert_eq!(inn.steps.first().map(|s| s.law_id.as_str()), Some("g_to_x"));
    assert_eq!(out.steps.first().map(|s| s.law_id.as_str()), Some("f_outer"));
}
