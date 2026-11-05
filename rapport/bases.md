# Chapitre 1 — Introduction

Bala Andegue Francois Lionnel.

## Principe

Une image numérique est une matrice de M lignes et N colonnes. Chaque élément
est un niveau de gris, ou trois niveaux R, V et B. Passer du signal continu
du capteur à cette matrice demande deux opérations : l'échantillonnage, qui
fixe la résolution spatiale, et la quantification, qui fixe la résolution
tonale. Le chapitre couvre aussi le passage couleur vers gris, la séparation
des canaux, le coût de stockage et les profils d'intensité.

## Formules du cours

Luminance (coefficients usuels) :

```
Y = 0,299 R + 0,587 V + 0,114 B
```

Moyenne simple :

```
Y = (R + V + B) / 3
```

Relation entre le nombre de niveaux L et le nombre de bits K :

```
L = 2^K
```

Taille de stockage d'une image grise :

```
b = M × N × K
```

Quantification uniforme sur [0, 255] en L niveaux. Le pixel p tombe dans
l'intervalle k, de largeur 256 / L, et on le remplace par le centre de cet
intervalle :

```
k = min( floor(p / (256 / L)) , L - 1 )
q = (k + 0,5) × (256 / L)
```

Pour L = 2, les valeurs reconstruites sont 64 et 192. L'échantillonnage de
pas s ne conserve que les pixels dont les coordonnées sont des multiples de
s. La pixelisation recolle ces valeurs en blocs de s × s pour garder la
taille d'affichage du cours.

## Implémentation

Tout est dans `imagerie/src/bases.rs`. Les conversions et les canaux sont des
parcours pixel à pixel, en temps proportionnel au nombre de pixels. La
quantification refuse un nombre de niveaux hors de 2…256. Le pas nul est
refusé pour l'échantillonnage et la pixelisation.

Le profil d'une ligne ou d'une colonne recopie les pixels correspondants. Le
profil d'un segment suit l'algorithme de Bresenham et ignore les pixels qui
sortent de l'image. `image_profil` dessine la courbe sur fond blanc, le
niveau 0 en bas et le niveau 255 en haut.

Les commandes sont dans `imagerie-cli`, groupe `bases`.

## Résultats

Images produites dans `rapport/figures/bases/` à partir de `images/couleur.ppm`
et `images/mire.pgm`.

| Figure | Lecture |
|---|---|
| `luminance.pgm`, `moyenne.pgm` | Les deux conversions séparent les trois formes colorées du fond. La luminance assombrit le bleu (coefficient 0,114) par rapport à la moyenne. |
| `canal-rouge.pgm`, `canal-vert.pgm`, `canal-bleu.pgm` | Chaque forme n'apparaît que sur son canal. |
| `isoler-rouge.ppm` | Le rouge est conservé, le vert et le bleu sont annulés. |
| `quantification-8.pgm`, `quantification-2.pgm` | À 8 niveaux les aplats se voient. À 2 niveaux l'image ne contient plus que 64 et 192. |
| `echantillon-4.pgm`, `pixelise-8.pgm` | Le pas 4 réduit la mire. La pixelisation de pas 8 montre les mêmes pertes en gardant 256 × 256. |
| `profil-ligne.pgm` | Courbe de la ligne centrale : le dégradé, puis les objets de la mire. |

La mire 256 × 256 en 8 bits occupe `256 × 256 × 8 = 524 288` bits.

## Limites

La quantification au centre d'intervalle ne redonne pas l'image d'origine
quand L = 256 : chaque niveau est décalé d'un demi-pas. Pour une image déjà
codée sur 8 bits, L = 256 n'a donc pas d'intérêt. La moyenne (R+V+B)/3 est
une division entière, sans arrondi. Le tracé de Bresenham choisit une
4-connexité : une diagonale douce est en escalier, ce qui est le comportement
attendu pour un profil.
