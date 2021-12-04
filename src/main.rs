use egg::{
    define_language, rewrite, Applier, CostFunction, EGraph, Extractor, Id, Language, PatternAst,
    RecExpr, Rewrite, Runner, Subst, Symbol, Var,
};

use itertools::Itertools;

use num::Integer;

use std::{env::args, time::Instant, usize::MAX};

const SHARED: bool = false;

define_language! {
    enum SimpleLang {
        "l" = Lst(Box<[Id]>),
        "*" = Mul([Id; 2]),
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "<<" = Shl([Id; 2]),
        Int(usize),
        Sym(Symbol),
    }
}

use crate::SimpleLang::*;

struct AstSize;

impl CostFunction<SimpleLang> for AstSize {
    type Cost = usize;

    fn cost<C>(&mut self, enode: &SimpleLang, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        enode.fold(
            match enode {
                Mul(_) => MAX,
                Add(_) | Sub(_) => 2,
                _ => 1,
            },
            |sum, id| sum.saturating_add(costs(id)),
        )
    }
}

struct MultipleOfTwo {
    x: Var,
    y: Var,
}

impl MultipleOfTwo {
    fn new(x: &'static str, y: &'static str) -> Self {
        MultipleOfTwo {
            x: x.parse().unwrap(),
            y: y.parse().unwrap(),
        }
    }
}

impl Applier<SimpleLang, ()> for MultipleOfTwo {
    fn apply_one(
        &self,
        egraph: &mut EGraph<SimpleLang, ()>,
        eclass: Id,
        subst: &Subst,
        _: Option<&PatternAst<SimpleLang>>,
        _: Symbol,
    ) -> Vec<Id> {
        let rhs = egraph[subst[self.x]].leaves().next();
        if let Some(&Int(mut vx)) = rhs {
            if vx.is_multiple_of(&2) {
                vx /= 2;
                if vx >= 1 {
                    let mut xy = subst[self.y];
                    if vx >= 2 {
                        let x = egraph.add(Int(vx));
                        xy = egraph.add(Mul([x, xy]));
                    }
                    let o = egraph.add(Int(1));
                    let xyo = egraph.add(Shl([xy, o]));
                    if egraph.union(eclass, xyo) {
                        return vec![xyo];
                    }
                }
            }
        }
        vec![]
    }
}

struct AddMinusOne {
    x: Var,
    y: Var,
}

impl AddMinusOne {
    fn new(x: &'static str, y: &'static str) -> Self {
        AddMinusOne {
            x: x.parse().unwrap(),
            y: y.parse().unwrap(),
        }
    }
}

impl Applier<SimpleLang, ()> for AddMinusOne {
    fn apply_one(
        &self,
        egraph: &mut EGraph<SimpleLang, ()>,
        eclass: Id,
        subst: &Subst,
        _: Option<&PatternAst<SimpleLang>>,
        _: Symbol,
    ) -> Vec<Id> {
        let rhs = egraph[subst[self.x]].leaves().next();
        if let Some(&Int(vx)) = rhs {
            if SHARED || !vx.is_multiple_of(&2) {
                if let Some(vx) = vx.checked_sub(1) {
                    let x = egraph.add(Int(vx));
                    let y = subst[self.y];
                    let xy = egraph.add(Mul([x, y]));
                    let xyy = egraph.add(Add([xy, y]));
                    if egraph.union(eclass, xyy) {
                        return vec![xyy];
                    }
                }
            }
        }
        vec![]
    }
}

struct SubPlusOne {
    x: Var,
    y: Var,
}

impl SubPlusOne {
    fn new(x: &'static str, y: &'static str) -> Self {
        SubPlusOne {
            x: x.parse().unwrap(),
            y: y.parse().unwrap(),
        }
    }
}

impl Applier<SimpleLang, ()> for SubPlusOne {
    fn apply_one(
        &self,
        egraph: &mut EGraph<SimpleLang, ()>,
        eclass: Id,
        subst: &Subst,
        _: Option<&PatternAst<SimpleLang>>,
        _: Symbol,
    ) -> Vec<Id> {
        let rhs = egraph[subst[self.x]].leaves().next();
        if let Some(&Int(vx)) = rhs {
            if SHARED || !vx.is_multiple_of(&2) {
                if let Some(vx) = vx.checked_add(1) {
                    let x = egraph.add(Int(vx));
                    let y = subst[self.y];
                    let xy = egraph.add(Mul([x, y]));
                    let xyy = egraph.add(Sub([xy, y]));
                    if egraph.union(eclass, xyy) {
                        return vec![xyy];
                    }
                }
            }
        }
        vec![]
    }
}

struct FactorShift {
    x: Var,
    y: Var,
    z: Var,
}

impl FactorShift {
    fn new(x: &'static str, y: &'static str, z: &'static str) -> Self {
        FactorShift {
            x: x.parse().unwrap(),
            y: y.parse().unwrap(),
            z: z.parse().unwrap(),
        }
    }
}

impl Applier<SimpleLang, ()> for FactorShift {
    fn apply_one(
        &self,
        egraph: &mut EGraph<SimpleLang, ()>,
        eclass: Id,
        subst: &Subst,
        _: Option<&PatternAst<SimpleLang>>,
        _: Symbol,
    ) -> Vec<Id> {
        let rhs = egraph[subst[self.z]].leaves().next();
        if let Some(&Int(vz)) = rhs {
            let rhs = egraph[subst[self.y]].leaves().next();
            if let Some(&Int(vy)) = rhs {
                let zy = egraph.add(Int(vz + vy));
                let x = subst[self.x];
                let xzy = egraph.add(Shl([x, zy]));
                if egraph.union(eclass, xzy) {
                    return vec![xzy];
                }
            }
        }
        vec![]
    }
}

fn main() {
    let now = Instant::now();

    let rules: &[Rewrite<SimpleLang, ()>] = &[
        rewrite!("multiple-of-two"; "(* ?x ?y)" => { MultipleOfTwo::new("?x", "?y") }),
        rewrite!("add-minus-one"; "(* ?x ?y)" => { AddMinusOne::new("?x", "?y") }),
        rewrite!("sub-plus-one"; "(* ?x ?y)" => { SubPlusOne::new("?x", "?y") }),
        rewrite!("factor-shift"; "(<< (<< ?x ?y) ?z)" => { FactorShift::new("?x", "?y", "?z") }),
    ];

    let input = format!(
        "(l {})",
        args().skip(1).map(|arg| format!("(* {} x)", arg)).join(" ")
    );

    let start: RecExpr<SimpleLang> = input.parse().unwrap();

    let runner = Runner::default().with_expr(&start).run(rules);

    let extractor = Extractor::new(&runner.egraph, AstSize);

    let (_, best_expr) = extractor.find_best(runner.roots[0]);

    println!(
        "{} = {} in {}s",
        input,
        best_expr,
        now.elapsed().as_secs_f32()
    );

    let size = runner.egraph.total_size();

    println!("size -> {}", size);

    if size <= 250 {
        runner.egraph.dot().to_svg("egraph.svg").unwrap();
    }
}
