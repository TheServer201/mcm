# Multiple constant multiplication problem

# How to use

1. Setup a nightly installation of Rust using [Rustup](https://rustup.rs/).
2. Type "git clone https://github.com/TheServer201/mcm.git && cd mcm && cargo run --release 7" to clone, compile and execute with 7 as argument
3. You should see  
(l (* 7 x)) = (l (- (<< x 3) x)) in 0.0003527s  
size -> 26
4. If the size <= 250 then a representation of the egraph will be in the file egraph.svg

Note. Set the constant SHARED to false when solving the single constant multiplication problem and to true otherwise. This help to prune the egraph because in the SCM when we find a multiple of two we always want to replace it with a shift.

# How it works
It uses rewrite rules and assign costs to operations as follow :
- (* 2n x) => (<< (* n x) 1)
- (* n x) => (+ (* n-1 x) x)
- (* n x) => (- (* n+1 x) x)
- (<< (<< x m) n) => (<< x m+n)

where m, n are integers and x a symbol

- cost(*) = MAX
- cost(+) = cost(-) = 2
- cost(<<) = 1
