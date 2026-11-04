# Chapitre 7 — Morphologie

Les opérateurs sont dans `imagerie::morphologie`. Les signatures prévues au
départ sont inchangées : les autres modules peuvent les appeler directement.

## Usage

```rust
use imagerie::morphologie::{self, ElementStructurant};

let croix = ElementStructurant::croix();          // 4-connexité
let carre = ElementStructurant::carre(3)?;        // 8-connexité
let disque = ElementStructurant::disque(2);
let erodee = morphologie::erosion(&binaire, &croix);
let min = morphologie::erosion_gris(&gris, &carre);
```

En ligne de commande, les objets sont les pixels ≥ 128 :

```sh
cargo run -p imagerie-cli -- morphologie erosion images/formes.pgm sortie.pgm --forme croix
cargo run -p imagerie-cli -- morphologie dilatation images/formes.pgm sortie.pgm --forme carre --taille 3
```

`--forme` vaut `croix` (défaut), `carre` ou `disque`. `--taille` est le côté
du carré (impair) ou le rayon du disque.

## Convention utile aux autres modules

- L'origine fait partie de la croix, du carré et du disque.
- Un pixel hors image compte comme du fond (`false`). En niveaux de gris, il
  est ignoré : le min ou le max ne porte que sur les pixels réellement dans
  l'image.
- La croix ne voit pas les diagonales. Le carré 3×3 les voit. Une dilatation
  par la croix ne joint donc pas deux pixels qui se touchent seulement par un
  coin ; le carré, si.
- `carre` refuse une taille paire ou nulle et renvoie une `Erreur::Parametre`.
- L'ouverture enlève les détails plus petits que l'élément, la fermeture
  bouche les trous de cette taille.

Les figures `rapport/figures/morphologie/` montrent l'érosion, la dilatation
et le gradient de `images/formes.pgm` avec la croix et avec le carré 3×3.
