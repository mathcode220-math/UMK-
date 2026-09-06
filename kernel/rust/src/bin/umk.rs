use umk_kernel::{Expr, Law, RewriteEngine, Type};

fn main() {
    let a = Type::abstract_carrier("A");
    let star = Expr::symbol(
        "star",
        Some(Type::arrow(a.clone(), Type::arrow(a.clone(), a.clone()))),
    );
    let e = Expr::symbol("e", Some(a.clone()));
    let sa = Expr::symbol("a", Some(a.clone()));
    let sb = Expr::symbol("b", Some(a.clone()));
    let x = Expr::var("x", Some(a.clone()));
    let y = Expr::var("y", Some(a.clone()));
    let z = Expr::var("z", Some(a));

    let laws = vec![
        Law::new(
            "left_identity",
            Expr::apply(star.clone(), vec![e.clone(), x.clone()]),
            x.clone(),
        ),
        Law::new(
            "right_identity",
            Expr::apply(star.clone(), vec![x.clone(), e.clone()]),
            x.clone(),
        ),
        Law::new(
            "associativity",
            Expr::apply(
                star.clone(),
                vec![Expr::apply(star.clone(), vec![x.clone(), y.clone()]), z.clone()],
            ),
            Expr::apply(star.clone(), vec![x, Expr::apply(star.clone(), vec![y, z])]),
        ),
    ];

    let expr = Expr::apply(
        star.clone(),
        vec![
            Expr::apply(star.clone(), vec![sa.clone(), e.clone()]),
            Expr::apply(star.clone(), vec![e, sb.clone()]),
        ],
    );
    let (result, proof) = RewriteEngine::bottom_up(&expr, &laws, 10);
    println!("input  {expr}");
    println!("output {result}");
    println!("proof  {}", proof.join(" -> "));
}
