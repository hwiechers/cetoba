#!/usr/bin/env bash

./cutechess-cli -rounds 250 \
-games 10 \
-repeat 10 \
-noswap \
-pgnout "results_$(date +%Y%m%d_%H%M%S).pgn" \
-resign movecount=3 score=400 \
-draw movenumber=34 movecount=8 score=20 \
-concurrency 3 \
-openings file=2moves_v1.pgn format=pgn order=random plies=16 \
-engine name=stockfish cmd=stockfish option.Hash=4 \
-engine name=stockfish2 cmd=stockfish option.Hash=4 \
-each proto=uci tc=6.84+0.07 option.Threads=1

