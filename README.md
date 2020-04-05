# A solver for a game 'A good snowman is hard to build'

`snowman-rs` is a solver for a game '[A Good Snowman Is Hard To Build](https://agoodsnowman.com/)'.

## Usage

Put the state in the following order
1. player x pos (0-based)
2. player y pos (0-based)
3. Board states
  * `#` is a block
  * `_` is a snow
  * `.` is a grass
  * `1`, `2`, `4` are snowballs.
  
```shell script
cargo run --release <<EOF
0
1
##4##
__1__
__1__
_____
EOF
```

Then `snowman-rs` will show you a final state, and how you should move snowballs.
Balls are indexed in the first state, and their indexes don't change.
Balls are automatically indexed in top-down, left-right order (0-based).

```
    Finished release [optimized] target(s) in 0.00s
     Running `target/release/snowman-rs`
 # #7. # #
 _ _A. . _
 _ _ . _ _
 _ _ _ _ _
1	Right
1	Left
2	Up
2	Right
1	Up
2	Left
2	Up
```
