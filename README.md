# Traitement d'images

Application de traitement d'images écrite en Rust dans le cadre du cours de
traitement d'images du Dr Tapamo. Toutes les opérations vues en cours sont
codées par nous : seuls la lecture et l'écriture des formats compressés
(PNG, JPEG...) passent par une bibliothèque externe.

## Membres du groupe

- Bala Andegue Francois Lionnel (22P448), chef de groupe
- Belekotan II Jeff Nichoss (24P765)
- Nankeng Tsamo Pierre Marcelle (22P292)
- Sountsa Djiele Pio Vianney (22P572)
- Nkamla Chedjou Johan (22P607)
- Nkolo Atangana Stacy (22P582)

La répartition du travail est dans [TACHES.md](TACHES.md), les règles pour
contribuer dans [CONTRIBUER.md](CONTRIBUER.md).

## Organisation du code

```
imagerie/            bibliothèque : un module par partie du cours
  src/image.rs         types d'images, gestion des bords, connexité
  src/es.rs            lecture/écriture PGM, PPM, PNG, JPEG...
  src/bases.rs         ch. 1  canaux, quantification, échantillonnage, profils
  src/ponctuel.rs      ch. 2  histogramme, contraste, égalisation
  src/arithmetique.rs  ch. 2  ET, OU, addition, soustraction, multiplication
  src/interpolation.rs ch. 2  plus proche voisin, bilinéaire, bicubique
  src/filtrage.rs      ch. 3  convolution, moyenneur, gaussien, médian
  src/fourier.rs       ch. 4  FFT, spectre, filtrage fréquentiel
  src/contours.rs      ch. 5  Roberts, Prewitt, Sobel, laplacien
  src/hough.rs         ch. 5  transformée de Hough
  src/segmentation.rs  ch. 6  seuillage, Otsu, k-moyennes, division-fusion, croissance
  src/binaire.rs       ch. 7  distances, Freeman, étiquetage, hystérésis
  src/morphologie.rs   ch. 7  érosion, dilatation, ouverture, fermeture
imagerie-cli/        application en ligne de commande (une commande par opération)
images/              images de test
rapport/             rapport : une section par module, figures
```

## Compiler et lancer

Il faut Rust 1.87 ou plus récent (`rustup update`).

```sh
cargo build --release
cargo test

# aide générale, puis aide d'un chapitre
cargo run --release -- --help
cargo run --release -- ponctuel --help

# exemples
cargo run --release -- info images/couleur.ppm
cargo run --release -- convertir images/couleur.ppm couleur.png
cargo run --release -- ponctuel egalisation images/sombre.pgm egalisee.pgm
cargo run --release -- contours sobel images/mire.pgm contours.pgm --seuil 60
cargo run --release -- binaire etiquetage images/formes.pgm etiquettes.ppm --huit
```

Le programme s'appelle `imagerie` : après `cargo install --path imagerie-cli`
on peut taper directement `imagerie ponctuel egalisation ...`.

## Images de test

| Fichier | Contenu |
|---|---|
| `mire.pgm` | dégradé, carré clair, disque sombre, disque texturé |
| `sombre.pgm` | la même mire avec une dynamique réduite à [90, 150] |
| `couleur.ppm` | trois formes rouge, verte et bleue sur fond clair |
| `formes.pgm` | objets blancs sur fond noir ; deux carrés ne se touchent que par un coin (2 objets en 4-connexité, 1 en 8-connexité) |

On peut ajouter d'autres images (photos, images du cours), de préférence en
PGM, PPM ou PNG. Éviter le JPEG pour les résultats : la compression modifie
les pixels.
