# axum-hello-world
Small project to familiarize myself with Rust and Axum

## watch mode
First, start the server in "watch mode"
```
cargo watch -q -c -w src/ -x run
```
// cargo watch -q -c -w src/ -x run
// q quiet
// c clear
// x execute

## running a quick test against local server
```
cargo watch -q -c -w tests/ -x "test -q quick_dev -- --nocapture"
```
