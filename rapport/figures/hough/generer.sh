#!/bin/sh
# Régénère les figures de rapport/hough.md (à lancer depuis la racine du dépôt).
set -e
cargo build --release -q
B=target/release/imagerie
H=rapport/figures/hough

# Exemple du cours : points aléatoires, points alignés, et les deux mélangés.
$B hough points $H/points-aleatoires.pgm --alignes 0 --aleatoires 20
$B hough points $H/points-alignes.pgm --alignes 20
$B hough points $H/points-mixte.pgm --alignes 20 --aleatoires 40
for n in aleatoires alignes mixte; do
    $B hough accumulateur $H/points-$n.pgm $H/accumulateur-$n.pgm --pas-theta 0.5
    $B hough droites $H/points-$n.pgm $H/droites-$n.ppm -n 1 --votes-min 8
done

# Image de formes : contour intérieur après Otsu, puis 8 droites.
$B hough accumulateur images/formes.pgm $H/accumulateur-formes.pgm --contours --pas-theta 0.5
$B hough droites images/formes.pgm $H/droites-formes.ppm --contours -n 8 --votes-min 30

for f in $H/*.pgm $H/*.ppm; do $B convertir "$f" "${f%.*}.png"; done
