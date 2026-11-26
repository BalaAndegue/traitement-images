# Chapitre 2 — Interpolation et changement d'échelle

**Responsable : Nankeng Tsamo Pierre Marcelle (22P292)**

---

## 1. Principe général

Lorsqu'on redimensionne une image, on doit calculer la valeur des pixels de la nouvelle grille à des positions qui ne coïncident pas forcément avec ceux de l'image d'origine.  
La technique universelle est le **mapping inverse** : pour chaque pixel de sortie $(x_d, y_d)$, on calcule les coordonnées sources correspondantes

$$x_s = x_d \cdot \frac{L_{\text{src}} - 1}{L_{\text{dst}} - 1}, \quad y_s = y_d \cdot \frac{H_{\text{src}} - 1}{H_{\text{dst}} - 1}$$

puis on **interpole** la valeur de l'image source au point $(x_s, y_s)$.

---

## 2. Plus proche voisin

La méthode la plus simple : on arrondit $(x_s, y_s)$ à l'entier le plus proche.

$$I'(x_d, y_d) = I\!\left(\text{round}(x_s),\, \text{round}(y_s)\right)$$

**Avantage** : très rapide, préserve les valeurs exactes.  
**Inconvénient** : effet de blocs (crénelage) visible à fort facteur.

---

## 3. Interpolation bilinéaire

On utilise les 4 voisins immédiats du point source $(x_s, y_s)$ :

Soit $x_0 = \lfloor x_s \rfloor$, $y_0 = \lfloor y_s \rfloor$, $dx = x_s - x_0$, $dy = y_s - y_0$.

$$I'(x_d, y_d) = (1-dx)(1-dy)\,I_{00} + dx(1-dy)\,I_{10} + (1-dx)dy\,I_{01} + dx\,dy\,I_{11}$$

où $I_{ij} = I(x_0+j,\, y_0+i)$.

**Avantage** : lisse, rapide (4 accès mémoire).  
**Inconvénient** : légèrement flou sur les contours.

---

## 4. Interpolation bicubique (Catmull-Rom)

On utilise les 16 voisins (grille 4×4). Le poids cubique de Catmull-Rom (paramètre $a = -0{,}5$) est :

$$w(t) = \begin{cases} (a+2)|t|^3 - (a+3)|t|^2 + 1 & |t| < 1 \\ a|t|^3 - 5a|t|^2 + 8a|t| - 4a & 1 \le |t| < 2 \\ 0 & |t| \ge 2 \end{cases}$$

La valeur interpolée est :

$$I'(x_d, y_d) = \sum_{m=-1}^{2} \sum_{n=-1}^{2} w(dy - m)\, w(dx - n)\, I(x_0+n,\, y_0+m)$$

**Avantage** : meilleure netteté que le bilinéaire, bons résultats sur les contours.  
**Inconvénient** : peut créer de légères oscillations (ringing) près des forts contrastes.

---

## 5. Agrandissement par copie (blocs)

Pour un facteur entier $k$, chaque pixel $(i, j)$ est répété $k \times k$ fois :

$$I'(k\,x + p,\; k\,y + q) = I(x, y), \quad 0 \le p, q < k$$

Équivalent au plus proche voisin mais avec un facteur entier et une implémentation directe (sans division flottante).

---

## 6. Interpolation 1D d'un facteur 2

### 6.1 Linéaire

Entre chaque paire de points consécutifs $x(i)$ et $x(i+1)$, on insère :

$$y\!\left(i + \tfrac{1}{2}\right) = \frac{x(i) + x(i+1)}{2}$$

Extrapolation au bord droit : $2\,x(N-1) - x(N-2)$.

### 6.2 Cubique (formule du cours)

$$y\!\left(i + \tfrac{1}{2}\right) = \frac{-x(i-1) + 9\,x(i) + 9\,x(i+1) - x(i+2)}{16}$$

Extrapolations aux extrémités : parabolique au bord droit, bord répliqué au bord gauche.

---

## 7. Interpolation cubique 1D d'un facteur 3 (formule en /81)

Le spline de Catmull-Rom évalué en $t = 1/3$ et $t = 2/3$ dans l'intervalle $[x_i, x_{i+1}]$ :

$$p(t) = \frac{1}{2}\bigl[(-a+3b-3c+d)\,t^3 + (2a-5b+4c-d)\,t^2 + (-a+c)\,t + 2b\bigr]$$

avec $a = x(i-1)$, $b = x(i)$, $c = x(i+1)$, $d = x(i+2)$.

La sortie contient $3n$ valeurs : $x(0),\, p(0,\tfrac{1}{3}),\, p(0,\tfrac{2}{3}),\, x(1), \ldots$

---

## 8. Zoom x2 séparable

Application de l'interpolation 1D de manière séparable :

1. Interpoler chaque **ligne** : image $L \times H \to 2L \times H$.
2. Interpoler chaque **colonne** du résultat : $2L \times H \to 2L \times 2H$.

Complexité : $O(L \cdot H)$, identique à la version directe mais plus facile à optimiser.

---

## 9. Rotation

Mapping inverse autour du centre $(c_x, c_y)$ avec l'angle $-\theta$ :

$$\begin{pmatrix} x_s \\ y_s \end{pmatrix} = \begin{pmatrix} \cos\theta & -\sin\theta \\ \sin\theta & \cos\theta \end{pmatrix} \begin{pmatrix} x_d - c_x \\ y_d - c_y \end{pmatrix} + \begin{pmatrix} c_x \\ c_y \end{pmatrix}$$

Les pixels sources hors image sont mis à 0 (fond noir). L'interpolation choisie (PPV, bilinéaire, bicubique) est appliquée au point $(x_s, y_s)$.

---

## 10. Choix d'implémentation

| Point | Choix |
|---|---|
| Gestion des bords (redimensionnement, rotation) | Réplication du pixel bord le plus proche |
| Méthode par défaut | Bilinéaire (bon compromis qualité/vitesse) |
| Bicubique | Catmull-Rom ($a = -0{,}5$), noyau séparable |
| Zoom x2 séparable | Lignes puis colonnes, arrondi intermédiaire en `u8` |
| Rotation — fond | Noir (0) pour les pixels hors image source |

---

## 11. Résultats

> Les figures sont dans `rapport/figures/interpolation/`.

Les commandes pour les générer :

```bash
# Zoom x2 bilinéaire
imagerie interpolation zoom images/mire.pgm rapport/figures/interpolation/zoom2-bilineaire.pgm --facteur 2 --methode bilineaire

# Zoom x2 plus proche voisin
imagerie interpolation zoom images/mire.pgm rapport/figures/interpolation/zoom2-ppv.pgm --facteur 2 --methode ppv

# Zoom x2 bicubique
imagerie interpolation zoom images/mire.pgm rapport/figures/interpolation/zoom2-bicubique.pgm --facteur 2 --methode bicubique

# Zoom x2 séparable bilinéaire
imagerie interpolation zoom-separable images/mire.pgm rapport/figures/interpolation/zoom2-separable.pgm --methode bilineaire

# Agrandissement par copie x4
imagerie interpolation copie images/mire.pgm rapport/figures/interpolation/copie-x4.pgm --facteur 4

# Réduction de moitié
imagerie interpolation zoom images/mire.pgm rapport/figures/interpolation/zoom05-bilineaire.pgm --facteur 0.5

# Rotation 30°
imagerie interpolation rotation images/mire.pgm rapport/figures/interpolation/rotation-30.pgm --angle 30

# Zoom couleur x2
imagerie interpolation zoom-couleur images/couleur.ppm rapport/figures/interpolation/zoom2-couleur.ppm --facteur 2
```

**Observations :**
- Le **plus proche voisin** produit des artefacts de crénelage bien visibles sur la mire.
- Le **bilinéaire** lisse ces artefacts au prix d'un léger flou.
- Le **bicubique** préserve mieux les contours tout en supprimant le crénelage.
- Le **zoom séparable** donne des résultats identiques au zoom direct bilinéaire.
- La **rotation** introduit un fond noir aux coins ; avec PPV les bords sont crénelés, avec bilinéaire ils sont doux.
