#!/bin/sh
# Régénère les figures de rapport/segmentation.md (à lancer depuis la racine du dépôt).
set -e
cargo build --release -q
B=target/release/imagerie
S=rapport/figures/segmentation

$B segmentation seuil images/mire.pgm $S/seuil-128-mire.pgm 128
$B segmentation multiple images/mire.pgm $S/multiple-mire.pgm 60 180
$B segmentation moyenne images/mire.pgm $S/moyenne-mire.pgm
$B segmentation milieu images/mire.pgm $S/milieu-mire.pgm
$B segmentation mediane images/mire.pgm $S/mediane-mire.pgm
$B segmentation otsu images/mire.pgm $S/otsu-mire.pgm
$B segmentation otsu images/sombre.pgm $S/otsu-sombre.pgm
$B segmentation kittler images/mire.pgm $S/kittler-mire.pgm
$B segmentation adaptatif images/mire.pgm $S/adaptatif-mire.pgm --bloc 32 --variance-min 100
$B segmentation adaptatif images/mire.pgm $S/adaptatif-mire-sans-variance.pgm --bloc 32 --variance-min 0
$B segmentation kmoyennes images/mire.pgm $S/kmoyennes-mire.pgm -k 3
$B segmentation kmoyennes images/couleur.ppm $S/kmoyennes-intensite-couleur.pgm -k 4
$B segmentation kmoyennes-couleur images/couleur.ppm $S/kmoyennes-couleur.ppm -k 4
$B segmentation division images/mire.pgm $S/division-mire.ppm --critere max-min --valeur 20 --taille-min 4
$B segmentation division-fusion images/mire.pgm $S/division-fusion-mire.ppm --critere max-min --valeur 20 --taille-min 4
$B segmentation division-fusion images/couleur.ppm $S/division-fusion-couleur.ppm --critere variance --valeur 50 --taille-min 2
$B segmentation germes images/mire.pgm $S/germes-mire.ppm --espacement 40
$B segmentation croissance images/mire.pgm $S/croissance-un-germe.ppm --germe 70,70 -t 20
$B segmentation croissance images/mire.pgm $S/croissance-germes.ppm --germe 70,70 --germe 175,80 --germe 5,5 --germe 250,250 -t 20
$B segmentation croissance images/mire.pgm $S/croissance-auto.ppm --espacement 40 -t 15

# Copies PNG pour l'affichage dans le rapport.
for f in $S/*.pgm $S/*.ppm; do $B convertir "$f" "${f%.*}.png"; done
