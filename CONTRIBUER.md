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

## Ce qui est imposé par GitHub

La branche `main` est protégée : personne ne peut pousser dessus directement,
pas même le chef de groupe. Tout passe par une demande de fusion, qui ne peut
être fusionnée que si :

- les vérifications automatiques passent : formatage, clippy, tests
  (`verification`) et conventions de nommage (`conventions`) ;
- la branche s'appelle `prenom/module` ;
- le titre de la demande et chaque message de commit sont de la forme
  `module : description` ;
- au moins un autre membre a approuvé la demande (Bala est sollicité
  automatiquement) ;
- la branche est à jour avec `main` et toutes les discussions sont résolues.

La fusion se fait en « Squash and merge » : la demande devient un seul commit
sur `main`, avec le titre de la demande comme message. La branche est
supprimée automatiquement après la fusion.

## Demandes de fusion (pull requests)

- Une demande par groupe de fonctions cohérent (par exemple « transformations
  linéaires »), pas une seule énorme à la fin.
- Avant d'ouvrir la demande :
  ```sh
  cargo fmt --all
  cargo clippy --all-targets
  cargo test
  ```
- Remplir le modèle proposé à l'ouverture de la demande (cases à cocher).
- Bala relit et fusionne ; pour les demandes de Bala, un autre membre relit. Si quelqu'un d'autre a besoin de ta fonction, dis-le
  dans le groupe dès qu'elle est fusionnée.

## Messages de commit

En français, au présent, préfixés par le module. Préfixes acceptés :
`bases`, `ponctuel`, `arithmetique`, `interpolation`, `filtrage`, `fourier`,
`contours`, `hough`, `segmentation`, `binaire`, `morphologie`, `noyau`, `cli`,
`gui`, `rapport`, `doc`, `ci`.

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
