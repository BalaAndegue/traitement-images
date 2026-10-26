# Règles pour contribuer

## Mise en place

```sh
git clone <adresse du dépôt>
cd traitement-images
cargo build
cargo test
```

Si ça compile et que les tests passent, tout est prêt.

## Une branche par personne

On ne travaille jamais directement sur `main`.

```sh
git checkout main
git pull
git checkout -b <prenom>/<module>      # ex. jeff/ponctuel, stacy/morphologie
```

Pendant le travail :

```sh
git add imagerie/src/ponctuel.rs
git commit -m "ponctuel : égalisation d'histogramme"
git push -u origin jeff/ponctuel
```

Récupérer régulièrement ce que les autres ont fusionné :

```sh
git checkout main && git pull
git checkout jeff/ponctuel && git merge main
```

## Demandes de fusion (pull requests)

- Une demande par groupe de fonctions cohérent (par exemple « transformations
  linéaires »), pas une seule énorme à la fin.
- Avant d'ouvrir la demande :
  ```sh
  cargo fmt --all
  cargo clippy --all-targets
  cargo test
  ```
- Bala relit et fusionne. Si quelqu'un d'autre a besoin de ta fonction, dis-le
  dans le groupe dès qu'elle est fusionnée.

## Messages de commit

En français, au présent, préfixés par le module :

```
filtrage : convolution avec gestion du bord miroir
segmentation : seuil d'Otsu
cli : commandes du chapitre 7
```

## Règles de code

- On ne modifie que ses propres fichiers. Pour changer un fichier commun
  (`image.rs`, `es.rs`, `lib.rs`, `main.rs`), on prévient Bala d'abord.
- On ne change pas la signature d'une fonction utilisée par quelqu'un d'autre
  sans le prévenir.
- Les opérations du cours sont codées à la main : pas de bibliothèque de
  traitement d'images (pas d'`imageproc`, pas d'`opencv`, pas de crate FFT).
- Les fonctions renvoient une `Resultat` au lieu de paniquer quand un paramètre
  est invalide (taille paire, seuil impossible, images de tailles différentes...).
- Noms en français, comme le reste du code.
- Chaque fonction a au moins un test sur une petite image dont on connaît le
  résultat à la main.

## Rapport

Chacun écrit sa section dans `rapport/<module>.md` et met ses figures dans
`rapport/figures/<module>/`. Pour chaque opération : le principe, la formule
du cours, nos choix d'implémentation, et une image avant/après commentée.
