# Répartition des tâches

Devoir : application de traitement d'images en Rust qui reprend toutes les
opérations des 7 chapitres du cours du Dr Tapamo.

Chef de groupe : **Bala Andegue Francois Lionnel**

| Membre | Matricule | Partie |
|---|---|---|
| Bala Andegue Francois Lionnel | 22P448 | Noyau, chapitre 1, ligne de commande, interface graphique, intégration, rapport |
| Belekotan II Jeff Nichoss | 24P765 | Chapitre 2 : histogramme, contraste, opérations sur les images |
| Nankeng Tsamo Pierre Marcelle | 22P292 | Chapitre 2 : interpolation |
| Sountsa Djiele Pio Vianney | 22P572 | Chapitre 4 : Fourier ; chapitre 5 : gradient et laplacien |
| Nkamla Chedjou Johan | 22P607 | Chapitre 5 : Hough ; chapitre 6 : segmentation |
| Nkolo Atangana Stacy | 22P582 | Chapitre 7 : images binaires et morphologie |
| Tagatsing Fotsing Samuel Sean | 22P215 | Chapitre 3 : convolution et filtres |

Chaque fonction à écrire existe déjà dans la bibliothèque avec sa signature,
sa documentation et un `todo!()` à la place du corps. Le travail consiste à
remplacer les `todo!()`, écrire les tests, ajouter les commandes dans
l'application, et rédiger sa partie du rapport.

Pour chaque membre, la même chose est attendue en plus du code :

- des tests unitaires dans le module (`#[cfg(test)] mod tests`) sur de petites
  images construites à la main, dont on connaît le résultat ;
- une sous-commande par fonction dans `imagerie-cli/src/commandes/<module>.rs`
  (un exemple y est déjà) ;
- les images de résultat dans `rapport/figures/<module>/` ;
- sa section du rapport dans `rapport/<module>.md` : principe, formule du cours,
  choix d'implémentation, résultats commentés.

Quand toutes les fonctions d'un module sont écrites, on retire la ligne
`#![allow(unused_variables)]` en haut du fichier.

---

## Bala Andegue Francois Lionnel (chef de groupe)

Fichiers : `imagerie/src/image.rs`, `es.rs`, `bases.rs`, `imagerie-cli/`

Déjà fait :
- [x] types d'images (`ImageGris`, `ImageRvb`, `ImageReelle`, `ImageBinaire`, `ImageEtiquettes`), gestion des bords, connexité
- [x] lecture/écriture PGM et PPM (P2, P3, P5, P6) codée à la main, autres formats via la crate `image`
- [x] structure de la ligne de commande, images de test dans `images/`

Chapitre 1 (`bases.rs`) :
- [x] conversion couleur -> gris (luminance et moyenne)
- [x] extraction et isolement des canaux R, V, B
- [x] quantification (résolution tonale, L = 2^K)
- [x] échantillonnage et pixelisation (résolution spatiale)
- [x] taille de stockage b = M x N x K
- [x] profils d'intensité (ligne, colonne, segment) et tracé du profil

Application :
- [x] interface graphique (egui), réalisée par Jeff (#10) : ouvrir une image, choisir une opération dans un menu par chapitre, régler les paramètres, voir l'avant/après, afficher l'histogramme, enregistrer
- [x] intégration des branches, relecture des demandes de fusion
- [ ] assemblage du rapport final et préparation de la démonstration

## Belekotan II Jeff Nichoss

Fichiers : `imagerie/src/ponctuel.rs`, `imagerie/src/arithmetique.rs`

À livrer en premier (Johan en a besoin pour Otsu) : `histogramme`, `dynamique`.

Histogramme et statistiques :
- [x] histogramme (gris et RVB), histogramme normalisé, histogramme cumulé
- [x] dynamique [min, max], luminance (moyenne)
- [x] contraste : écart-type et variation min/max
- [x] dessin de l'histogramme dans une image

Amélioration du contraste :
- [x] application d'une LUT (version optimale du cours), négatif
- [x] transformation linéaire (étirement de la dynamique)
- [x] transformation linéaire avec saturation (Smin, Smax)
- [x] transformation linéaire par morceaux
- [x] correction gamma, transformations log et exp
- [x] égalisation d'histogramme (les 4 étapes du cours), version couleur
- [x] égalisation locale (fenêtre 7 x 7)

Opérations sur les images :
- [x] ET, OU (gris et binaire), NON, application d'un masque
- [x] addition et soustraction saturées, différence absolue
- [x] multiplication par un ratio, combinaison linéaire a F + (1 - a) G
- [x] moyenne d'une série d'images (réduction du bruit)

## Nankeng Tsamo Pierre Marcelle

Fichiers : `imagerie/src/interpolation.rs`

Interpolation (chapitre 2) :
- [x] plus proche voisin, bilinéaire, bicubique : redimensionnement et zoom
- [x] agrandissement par copie des lignes et colonnes
- [x] formules 1D du cours (x2 linéaire et cubique, x3 cubique) et zoom x2 séparable
- [x] version couleur, rotation

## Sountsa Djiele Pio Vianney

Fichiers : `imagerie/src/fourier.rs`, `imagerie/src/contours.rs`

Transformée de Fourier (chapitre 4), FFT codée par nous :
- [x] opérations sur `Complexe`
- [x] FFT 1D radix-2 et inverse, complétion à une puissance de 2
- [x] FFT 2D et FFT 2D inverse
- [x] inversion des quadrants, module, phase, parties réelle et imaginaire
- [x] spectre affichable log(1 + |F|)
- [x] énergie contenue dans un rayon (90 %, 95 %, 98 %, 99 %, 99,5 %, 99,9 %)
- [x] filtres passe-bas, passe-haut, passe-bande, coupe-pics (bruit périodique)
- [x] rehaussement image + passe-haut
- [x] images de test : sinus, rectangle, signal carré ; vérifier qu'une rotation de l'image fait tourner la FFT

Contours par dérivées (chapitre 5), après la convolution de Samuel :
- [x] noyaux dérivée simple, centrée, Roberts, Prewitt, Sobel, laplaciens 4 et 8
- [x] gradient Gx, Gy ; norme exacte et approchée ; direction
- [x] contours par seuillage du gradient (S = 25 et S = 60 comme dans le cours)
- [x] laplacien et passages par zéro
- [x] suppression des non-maxima (contours d'un pixel d'épaisseur)

## Nkamla Chedjou Johan

Fichiers : `imagerie/src/segmentation.rs`, `imagerie/src/hough.rs`

Segmentation (chapitre 6) :
- [x] seuillage simple et multiple
- [x] choix du seuil : moyenne, milieu de la dynamique, médiane
- [x] Otsu (variance intra-classes), Kittler en bonus
- [x] seuillage local adaptatif (blocs, variance < 100 non traitée)
- [x] k-moyennes sur l'intensité et sur la couleur
- [x] division-fusion (quadtree, critère variance ou max-min)
- [x] croissance de régions avec un ou plusieurs germes, germes automatiques

Transformée de Hough (chapitre 5) :
- [x] accumulateur en coordonnées polaires (rho, theta)
- [x] recherche des pics, image de l'accumulateur
- [x] tracé des droites détectées sur l'image
- [x] exemple du cours : points alignés contre points aléatoires

## Nkolo Atangana Stacy

Fichiers : `imagerie/src/binaire.rs`, `imagerie/src/morphologie.rs`

Images binaires (chapitre 7) :
- [x] distances D4, D8, euclidienne ; carte des distances
- [x] codage de Freeman (4 et 8 connexités) et décodage
- [x] étiquetage des composantes connexes par parcours successifs (méthode du cours)
- [x] étiquetage en une passe avec table d'équivalences ; comparer les deux
- [x] comptage des objets, surfaces, suppression des petits objets
- [x] seuillage par hystérésis (fermeture des contours)

Morphologie :
- [x] éléments structurants : croix (4-connexité), carré (8-connexité), disque, masque quelconque
- [x] érosion, dilatation, ouverture, fermeture
- [x] gradients interne, externe et morphologique
- [x] versions en niveaux de gris (min / max)
- [x] montrer sur `images/formes.pgm` l'effet de la connexité (4 contre 8) et du choix de l'élément structurant

## Tagatsing Fotsing Samuel Sean

Fichiers : `imagerie/src/filtrage.rs`

À livrer en premier (Pio en a besoin pour les contours) : `convoluer`,
`Noyau::moyenneur`, `Noyau::normalise`.

Convolution et filtres (chapitre 3) :
- [x] convolution avec les 4 gestions de bord (zéro, miroir, réplique, partielle)
- [x] noyaux moyenneur n x n, gaussien du cours (5 x 5, somme 98), gaussien de sigma quelconque
- [x] filtres moyenneur, gaussien, rehausseur (passe-haut)
- [x] filtres non linéaires : médian, min, max
- [x] bruit poivre et sel, bruit gaussien (utiliser `crate::alea::Alea`)
- [ ] comparaison moyenne / médian sur une image bruitée pour le rapport

---

## Avancement au 28 novembre

Toutes les fonctions de la bibliothèque sont écrites et testées (151 tests),
l'application en ligne de commande et l'interface graphique fonctionnent, `main`
passe toutes les vérifications. Version `v1.0.0` publiée.

Reste à faire :

| Qui | Quoi |
|---|---|
| Jeff | Section du rapport `ponctuel.md` et `arithmetique.md` avec figures ; commandes de la ligne de commande pour ponctuel et arithmétique (seuls les exemples de départ existent) |
| Samuel | Section du rapport `filtrage.md` avec figures, dont la comparaison moyenne / médian sur une image bruitée |
| Pio | Sections du rapport `fourier.md` et `contours.md` avec figures ; commandes de la ligne de commande pour Fourier (seul l'exemple de départ existe) |
| Stacy | Section du rapport `binaire.md` avec figures |
| Bala | Introduction, conclusion, assemblage du rapport final, répétition de la démonstration |

Sections déjà rédigées : `bases`, `interpolation`, `hough`, `segmentation`,
`morphologie`.

## Dépendances entre les parties

- Contours (Pio) utilise `filtrage::convoluer` (Samuel). En attendant, Pio peut
  commencer par Fourier.
- Otsu et le seuillage adaptatif (Johan) utilisent `ponctuel::histogramme` (Jeff).
- Hough (Johan) prend en entrée des contours binaires : en attendant ceux de Pio,
  tester sur des images de points dessinées à la main.
- L'hystérésis (Stacy) prend une norme de gradient (`ImageReelle`) : tester avec
  des images réelles construites à la main, pas besoin d'attendre Pio.
