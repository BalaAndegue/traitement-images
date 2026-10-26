# Répartition des tâches

Devoir : application de traitement d'images en Rust qui reprend toutes les
opérations des 7 chapitres du cours du Dr Tapamo.

Chef de groupe : **Bala Andegue Francois Lionnel**

| Membre | Matricule | Partie |
|---|---|---|
| Bala Andegue Francois Lionnel | 22P448 | Noyau, chapitre 1, ligne de commande, interface graphique, intégration, rapport |
| Belekotan II Jeff Nichoss | 24P765 | Chapitre 2 : histogramme, contraste, opérations sur les images |
| Nankeng Tsamo Pierre Marcelle | 22P292 | Chapitre 2 : interpolation ; chapitre 3 : convolution et filtres |
| Sountsa Djiele Pio Vianney | 22P572 | Chapitre 4 : Fourier ; chapitre 5 : gradient et laplacien |
| Nkamla Chedjou Johan | 22P607 | Chapitre 5 : Hough ; chapitre 6 : segmentation |
| Nkolo Atangana Stacy | 22P582 | Chapitre 7 : images binaires et morphologie |

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
- [ ] conversion couleur -> gris (luminance et moyenne)
- [ ] extraction et isolement des canaux R, V, B
- [ ] quantification (résolution tonale, L = 2^K)
- [ ] échantillonnage et pixelisation (résolution spatiale)
- [ ] taille de stockage b = M x N x K
- [ ] profils d'intensité (ligne, colonne, segment) et tracé du profil

Application :
- [ ] interface graphique (egui) : ouvrir une image, choisir une opération dans un menu par chapitre, régler les paramètres, voir l'avant/après, afficher l'histogramme, enregistrer
- [ ] intégration des branches, relecture des demandes de fusion
- [ ] assemblage du rapport final et préparation de la démonstration

## Belekotan II Jeff Nichoss

Fichiers : `imagerie/src/ponctuel.rs`, `imagerie/src/arithmetique.rs`

À livrer en premier (Johan en a besoin pour Otsu) : `histogramme`, `dynamique`.

Histogramme et statistiques :
- [ ] histogramme (gris et RVB), histogramme normalisé, histogramme cumulé
- [ ] dynamique [min, max], luminance (moyenne)
- [ ] contraste : écart-type et variation min/max
- [ ] dessin de l'histogramme dans une image

Amélioration du contraste :
- [ ] application d'une LUT (version optimale du cours), négatif
- [ ] transformation linéaire (étirement de la dynamique)
- [ ] transformation linéaire avec saturation (Smin, Smax)
- [ ] transformation linéaire par morceaux
- [ ] correction gamma, transformations log et exp
- [ ] égalisation d'histogramme (les 4 étapes du cours), version couleur
- [ ] égalisation locale (fenêtre 7 x 7)

Opérations sur les images :
- [ ] ET, OU (gris et binaire), NON, application d'un masque
- [ ] addition et soustraction saturées, différence absolue
- [ ] multiplication par un ratio, combinaison linéaire a F + (1 - a) G
- [ ] moyenne d'une série d'images (réduction du bruit)

## Nankeng Tsamo Pierre Marcelle

Fichiers : `imagerie/src/interpolation.rs`, `imagerie/src/filtrage.rs`

À livrer en premier (Pio en a besoin pour les contours) : `convoluer`,
`Noyau::moyenneur`, `Noyau::normalise`.

Convolution et filtres (chapitre 3) :
- [ ] convolution avec les 4 gestions de bord (zéro, miroir, réplique, partielle)
- [ ] noyaux moyenneur n x n, gaussien du cours (5 x 5, somme 98), gaussien de sigma quelconque
- [ ] filtres moyenneur, gaussien, rehausseur (passe-haut)
- [ ] filtres non linéaires : médian, min, max
- [ ] bruit poivre et sel, bruit gaussien (utiliser `crate::alea::Alea`)
- [ ] comparaison moyenne / médian sur une image bruitée pour le rapport

Interpolation (chapitre 2) :
- [ ] plus proche voisin, bilinéaire, bicubique : redimensionnement et zoom
- [ ] agrandissement par copie des lignes et colonnes
- [ ] formules 1D du cours (x2 linéaire et cubique, x3 cubique) et zoom x2 séparable
- [ ] version couleur, rotation

## Sountsa Djiele Pio Vianney

Fichiers : `imagerie/src/fourier.rs`, `imagerie/src/contours.rs`

Transformée de Fourier (chapitre 4), FFT codée par nous :
- [ ] opérations sur `Complexe`
- [ ] FFT 1D radix-2 et inverse, complétion à une puissance de 2
- [ ] FFT 2D et FFT 2D inverse
- [ ] inversion des quadrants, module, phase, parties réelle et imaginaire
- [ ] spectre affichable log(1 + |F|)
- [ ] énergie contenue dans un rayon (90 %, 95 %, 98 %, 99 %, 99,5 %, 99,9 %)
- [ ] filtres passe-bas, passe-haut, passe-bande, coupe-pics (bruit périodique)
- [ ] rehaussement image + passe-haut
- [ ] images de test : sinus, rectangle, signal carré ; vérifier qu'une rotation de l'image fait tourner la FFT

Contours par dérivées (chapitre 5), après la convolution de Pierre :
- [ ] noyaux dérivée simple, centrée, Roberts, Prewitt, Sobel, laplaciens 4 et 8
- [ ] gradient Gx, Gy ; norme exacte et approchée ; direction
- [ ] contours par seuillage du gradient (S = 25 et S = 60 comme dans le cours)
- [ ] laplacien et passages par zéro
- [ ] suppression des non-maxima (contours d'un pixel d'épaisseur)

## Nkamla Chedjou Johan

Fichiers : `imagerie/src/segmentation.rs`, `imagerie/src/hough.rs`

Segmentation (chapitre 6) :
- [ ] seuillage simple et multiple
- [ ] choix du seuil : moyenne, milieu de la dynamique, médiane
- [ ] Otsu (variance intra-classes), Kittler en bonus
- [ ] seuillage local adaptatif (blocs, variance < 100 non traitée)
- [ ] k-moyennes sur l'intensité et sur la couleur
- [ ] division-fusion (quadtree, critère variance ou max-min)
- [ ] croissance de régions avec un ou plusieurs germes, germes automatiques

Transformée de Hough (chapitre 5) :
- [ ] accumulateur en coordonnées polaires (rho, theta)
- [ ] recherche des pics, image de l'accumulateur
- [ ] tracé des droites détectées sur l'image
- [ ] exemple du cours : points alignés contre points aléatoires

## Nkolo Atangana Stacy

Fichiers : `imagerie/src/binaire.rs`, `imagerie/src/morphologie.rs`

Images binaires (chapitre 7) :
- [ ] distances D4, D8, euclidienne ; carte des distances
- [ ] codage de Freeman (4 et 8 connexités) et décodage
- [ ] étiquetage des composantes connexes par parcours successifs (méthode du cours)
- [ ] étiquetage en une passe avec table d'équivalences ; comparer les deux
- [ ] comptage des objets, surfaces, suppression des petits objets
- [ ] seuillage par hystérésis (fermeture des contours)

Morphologie :
- [ ] éléments structurants : croix (4-connexité), carré (8-connexité), disque, masque quelconque
- [ ] érosion, dilatation, ouverture, fermeture
- [ ] gradients interne, externe et morphologique
- [ ] versions en niveaux de gris (min / max)
- [ ] montrer sur `images/formes.pgm` l'effet de la connexité (4 contre 8) et du choix de l'élément structurant

---

## Calendrier

Dates provisoires, à recaler sur la date de remise donnée par le Dr Tapamo.

| Date | Étape |
|---|---|
| mardi 27 octobre | Tout le monde a cloné le dépôt, compile, et a créé sa branche |
| vendredi 30 octobre | Fonctions bloquantes fusionnées : `histogramme`, `dynamique` (Jeff), `convoluer` et noyaux de base (Pierre) |
| dimanche 8 novembre | Tous les modules écrits et testés |
| dimanche 15 novembre | Commandes et interface graphique branchées, figures du rapport générées |
| dimanche 22 novembre | Rapport final relu, démonstration répétée |

Point d'avancement chaque dimanche soir dans le groupe : ce qui est fait, ce
qui bloque.

## Dépendances entre les parties

- Contours (Pio) utilise `filtrage::convoluer` (Pierre). En attendant, Pio peut
  commencer par Fourier.
- Otsu et le seuillage adaptatif (Johan) utilisent `ponctuel::histogramme` (Jeff).
- Hough (Johan) prend en entrée des contours binaires : en attendant ceux de Pio,
  tester sur des images de points dessinées à la main.
- L'hystérésis (Stacy) prend une norme de gradient (`ImageReelle`) : tester avec
  des images réelles construites à la main, pas besoin d'attendre Pio.
