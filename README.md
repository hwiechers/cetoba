# Chess Engine Test Opening Book Analyser

This is a utility for analyzing opening books used in Chess Engine Testing.

The utility takes a PGN file containing the results of a chess
engine playing itself using the opening book of interest. The file should
include many repeated games of the each opening.

The utility will then output some useful stats to stdout including the
parameters of a fitted Dirichlet distribution. It will also generate the
following files:

## opening_stats.csv

A CSV file with the stats for each opening:

```
FEN,total,white_win,draw,black_win
rnbqkbnr/pppp1pp1/8/4p2p/7P/7N/PPPPPPP1/RNBQKB1R,10,0.1,0.9,0
rnbqkbnr/pp2pppp/2p5/3p4/2P5/7N/PP1PPPPP/RNBQKB1R,10,0.3,0.6,0.1
rnbqkbnr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/RNBQKBNR,10,0,0.2,0.8
rnbqkbnr/ppp1ppp1/3p4/7p/8/3BP3/PPPP1PPP/RNBQK1NR,10,0,1,0
rnbqkbnr/ppp1p1pp/3p4/5p2/P7/6P1/1PPPPP1P/RNBQKBNR,10,0.1,0.8,0.1
r1bqkbnr/1ppppppp/n7/p7/8/4P2N/PPPP1PPP/RNBQKB1R,10,0.2,0.8,0
...
```

## wbd_count.csv

A CSV file with the count of each "Win Win/Draw/Black Win" outcome.

```
WDB,Count
0.2-0.5-0.3,4
0.1-0.7-0.2,6
0-0.6-0.4,10
0-0.7-0.3,5
...
```

## scatter_plot.svg

A ternary scatter plot of the outcomes.

## dirichlet_contour_plot.svg

A contour plot of the fitted Dirichlet distribution.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
