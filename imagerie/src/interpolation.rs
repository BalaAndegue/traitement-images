//! Chapitre 2 (troisième partie) : interpolation et changement d'échelle.
//!
//! Responsable : Nankeng Tsamo Pierre Marcelle.

use crate::erreur::{Erreur, Resultat};
use crate::image::{ImageGris, ImageRvb};

// ---------------------------------------------------------------------------
// Méthode d'interpolation
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Methode {
    /// Copie du pixel le plus proche.
    PlusProcheVoisin,
    /// Fonction bilinéaire des 4 voisins.
    #[default]
    Bilineaire,
    /// Fonction bicubique des 16 voisins.
    Bicubique,
}

// ---------------------------------------------------------------------------
// Fonctions auxiliaires
// ---------------------------------------------------------------------------

/// Arrondit et sature une valeur réelle dans [0, 255].
#[inline]
fn vers_u8(v: f64) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// Accès sûr à un pixel en niveaux de gris, bord répliqué.
#[inline]
fn pixel_gris(img: &ImageGris, x: isize, y: isize) -> f64 {
    let xc = x.clamp(0, img.largeur() as isize - 1) as usize;
    let yc = y.clamp(0, img.hauteur() as isize - 1) as usize;
    img[(xc, yc)] as f64
}

/// Interpolation bilinéaire pour un point (px, py) en coordonnées fractionnaires.
///
/// Formule : f(x,y) = (1-dx)(1-dy) f00 + dx(1-dy) f10 + (1-dx)dy f01 + dx dy f11
fn bilineaire(img: &ImageGris, px: f64, py: f64) -> u8 {
    let x0 = px.floor() as isize;
    let y0 = py.floor() as isize;
    let dx = px - x0 as f64;
    let dy = py - y0 as f64;

    let f00 = pixel_gris(img, x0, y0);
    let f10 = pixel_gris(img, x0 + 1, y0);
    let f01 = pixel_gris(img, x0, y0 + 1);
    let f11 = pixel_gris(img, x0 + 1, y0 + 1);

    let v = (1.0 - dx) * (1.0 - dy) * f00
        + dx * (1.0 - dy) * f10
        + (1.0 - dx) * dy * f01
        + dx * dy * f11;
    vers_u8(v)
}

/// Poids cubique de Catmull-Rom : w(t) pour |t| ∈ [0, 2].
///
/// Formule (a = -0.5) :
/// |t| < 1 : (a+2)|t|³ − (a+3)|t|² + 1
/// 1 ≤ |t| < 2 : a|t|³ − 5a|t|² + 8a|t| − 4a
#[inline]
fn poids_cubique(t: f64) -> f64 {
    let a = -0.5_f64;
    let t = t.abs();
    if t < 1.0 {
        (a + 2.0) * t * t * t - (a + 3.0) * t * t + 1.0
    } else if t < 2.0 {
        a * t * t * t - 5.0 * a * t * t + 8.0 * a * t - 4.0 * a
    } else {
        0.0
    }
}

/// Interpolation bicubique pour un point (px, py) en coordonnées fractionnaires.
///
/// Utilise les 16 voisins (grille 4×4) avec les poids de Catmull-Rom.
fn bicubique(img: &ImageGris, px: f64, py: f64) -> u8 {
    let x0 = px.floor() as isize;
    let y0 = py.floor() as isize;
    let dx = px - x0 as f64;
    let dy = py - y0 as f64;

    let mut valeur = 0.0_f64;
    for m in -1_isize..=2 {
        let wy = poids_cubique(dy - m as f64);
        for n in -1_isize..=2 {
            let wx = poids_cubique(dx - n as f64);
            valeur += wx * wy * pixel_gris(img, x0 + n, y0 + m);
        }
    }
    vers_u8(valeur)
}

/// Applique la méthode d'interpolation choisie en un point (px, py) flottant.
fn interpoler_pixel(img: &ImageGris, px: f64, py: f64, methode: Methode) -> u8 {
    match methode {
        Methode::PlusProcheVoisin => {
            let x = px.round() as isize;
            let y = py.round() as isize;
            pixel_gris(img, x, y) as u8
        }
        Methode::Bilineaire => bilineaire(img, px, py),
        Methode::Bicubique => bicubique(img, px, py),
    }
}

/// Dimensions obtenues en multipliant (largeur, hauteur) par `facteur`.
fn dimensions_zoom(largeur: usize, hauteur: usize, facteur: f64) -> Resultat<(usize, usize)> {
    if facteur <= 0.0 || !facteur.is_finite() {
        return Err(Erreur::Parametre(
            "le facteur de zoom doit être un réel strictement positif".into(),
        ));
    }
    let l = ((largeur as f64 * facteur).round() as usize).max(1);
    let h = ((hauteur as f64 * facteur).round() as usize).max(1);
    Ok((l, h))
}

// ---------------------------------------------------------------------------
// Redimensionnement et zoom
// ---------------------------------------------------------------------------

/// Redimensionne l'image à une taille quelconque.
///
/// On utilise le **mapping inverse** : pour chaque pixel (xd, yd) de la
/// destination, on calcule le pixel source correspondant
/// xs = xd * (largeur_src − 1) / (largeur_dst − 1) et on interpole.
pub fn redimensionner(
    img: &ImageGris,
    nouvelle_largeur: usize,
    nouvelle_hauteur: usize,
    methode: Methode,
) -> Resultat<ImageGris> {
    if nouvelle_largeur == 0 || nouvelle_hauteur == 0 {
        return Err(Erreur::Parametre(
            "les dimensions cibles doivent être strictement positives".into(),
        ));
    }
    let lsrc = img.largeur();
    let hsrc = img.hauteur();
    if lsrc == 0 || hsrc == 0 {
        return Ok(ImageGris::nouvelle(nouvelle_largeur, nouvelle_hauteur, 0));
    }

    // Facteurs d'échelle : combien de pixels sources par pixel destination.
    let sx = (lsrc as f64 - 1.0) / (nouvelle_largeur as f64 - 1.0).max(1.0);
    let sy = (hsrc as f64 - 1.0) / (nouvelle_hauteur as f64 - 1.0).max(1.0);

    let resultat = ImageGris::depuis_fonction(nouvelle_largeur, nouvelle_hauteur, |xd, yd| {
        let xs = xd as f64 * sx;
        let ys = yd as f64 * sy;
        interpoler_pixel(img, xs, ys, methode)
    });
    Ok(resultat)
}

/// Zoom d'un facteur réel (> 1 agrandit, < 1 réduit).
pub fn zoom(img: &ImageGris, facteur: f64, methode: Methode) -> Resultat<ImageGris> {
    let (l, h) = dimensions_zoom(img.largeur(), img.hauteur(), facteur)?;
    redimensionner(img, l, h, methode)
}

/// Version couleur de `redimensionner` (canal par canal).
pub fn redimensionner_rvb(
    img: &ImageRvb,
    nouvelle_largeur: usize,
    nouvelle_hauteur: usize,
    methode: Methode,
) -> Resultat<ImageRvb> {
    // On sépare les 3 canaux, on redimensionne chacun, puis on les fusionne.
    let [r, v, b] = img.separer_canaux();
    let r2 = redimensionner(&r, nouvelle_largeur, nouvelle_hauteur, methode)?;
    let v2 = redimensionner(&v, nouvelle_largeur, nouvelle_hauteur, methode)?;
    let b2 = redimensionner(&b, nouvelle_largeur, nouvelle_hauteur, methode)?;
    ImageRvb::fusionner_canaux(&r2, &v2, &b2)
}

/// Version couleur de `zoom` (canal par canal).
pub fn zoom_rvb(img: &ImageRvb, facteur: f64, methode: Methode) -> Resultat<ImageRvb> {
    let (l, h) = dimensions_zoom(img.largeur(), img.hauteur(), facteur)?;
    redimensionner_rvb(img, l, h, methode)
}

/// Plus proche voisin par copie de chaque ligne et de chaque colonne `facteur` fois.
///
/// Chaque pixel source (x, y) est répété `facteur` fois en colonne et `facteur`
/// fois en ligne, ce qui donne une image de taille `(L*facteur) × (H*facteur)`.
pub fn agrandir_par_copie(img: &ImageGris, facteur: usize) -> Resultat<ImageGris> {
    if facteur == 0 {
        return Err(Erreur::Parametre("le facteur doit être au moins 1".into()));
    }
    let nouvelle_largeur = img.largeur() * facteur;
    let nouvelle_hauteur = img.hauteur() * facteur;
    if nouvelle_largeur == 0 || nouvelle_hauteur == 0 {
        return Ok(ImageGris::nouvelle(0, 0, 0));
    }
    let resultat = ImageGris::depuis_fonction(nouvelle_largeur, nouvelle_hauteur, |xd, yd| {
        // Pixel source : division entière des coordonnées.
        img[(xd / facteur, yd / facteur)]
    });
    Ok(resultat)
}

// ---------------------------------------------------------------------------
// Interpolation 1D (formules du cours)
// ---------------------------------------------------------------------------

/// Valeur cubique au milieu de l'intervalle [x(i), x(i+1)].
///
/// - intérieur : (−x(i−1) + 9 x(i) + 9 x(i+1) − x(i+2)) / 16 ;
/// - premier intervalle (pas de x(i−1)) : parabole par x(0), x(1), x(2),
///   soit (3 x(0) + 6 x(1) − x(2)) / 8 ;
/// - dernier intervalle (pas de x(i+2)) : parabole par x(i−1), x(i), x(i+1),
///   soit (−x(i−1) + 6 x(i) + 3 x(i+1)) / 8 ;
///
/// Les deux formules de bord reviennent à extrapoler l'échantillon manquant par
/// une parabole (x(−1) = 3 x(0) − 3 x(1) + x(2)) puis à appliquer la formule /16.
/// - deux points seulement : milieu linéaire.
fn milieu_cubique(x: &[f64], i: usize) -> f64 {
    let n = x.len();
    if n < 3 {
        (x[i] + x[i + 1]) / 2.0
    } else if i == 0 {
        (3.0 * x[0] + 6.0 * x[1] - x[2]) / 8.0
    } else if i + 2 >= n {
        (-x[i - 1] + 6.0 * x[i] + 3.0 * x[i + 1]) / 8.0
    } else {
        (-x[i - 1] + 9.0 * x[i] + 9.0 * x[i + 1] - x[i + 2]) / 16.0
    }
}

/// Interpolation 1D d'un facteur 2 avec les formules du cours :
/// - linéaire : y(i+1/2) = (x(i) + x(i+1)) / 2, extrapolation 2 x(N) - x(N-1) au bout ;
/// - cubique : y(i+1/2) = (-x(i-1) + 9 x(i) + 9 x(i+1) - x(i+2)) / 16, avec les
///   extrapolations parabolique et linéaire aux extrémités.
///
/// La sortie contient 2N valeurs : x(0), y(1/2), x(1), y(3/2), ..., x(N−1), y(N−1/2)
/// (indices à partir de 0). Pour la dernière, l'échantillon manquant est extrapolé
/// linéairement, x(N) = 2 x(N−1) − x(N−2), puis on prend le milieu avec x(N−1).
/// `PlusProcheVoisin` est traité comme la version linéaire.
pub fn interpoler_ligne_x2(ligne: &[f64], methode: Methode) -> Vec<f64> {
    let n = ligne.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![ligne[0], ligne[0]];
    }

    let mut sortie = Vec::with_capacity(2 * n);
    for (i, paire) in ligne.windows(2).enumerate() {
        let milieu = match methode {
            Methode::PlusProcheVoisin | Methode::Bilineaire => (paire[0] + paire[1]) / 2.0,
            Methode::Bicubique => milieu_cubique(ligne, i),
        };
        sortie.push(paire[0]);
        sortie.push(milieu);
    }
    // Dernier point, puis milieu avec l'échantillon extrapolé 2 x(N−1) − x(N−2).
    let extrapole = 2.0 * ligne[n - 1] - ligne[n - 2];
    sortie.push(ligne[n - 1]);
    sortie.push((ligne[n - 1] + extrapole) / 2.0);
    sortie
}

/// Valeurs cubiques en t = 1/3 et t = 2/3 de l'intervalle [x(i), x(i+1)].
///
/// Avec a = x(i−1), b = x(i), c = x(i+1), d = x(i+2) (spline de Catmull-Rom) :
/// - t = 1/3 : (−6a + 63b + 27c − 3d) / 81 ;
/// - t = 2/3 : (−3a + 27b + 63c − 6d) / 81.
///
/// Aux extrémités, parabole par les trois points disponibles :
/// - premier intervalle : (45 x(0) + 45 x(1) − 9 x(2)) / 81 et (18 x(0) + 72 x(1) − 9 x(2)) / 81 ;
/// - dernier intervalle : (−9 a + 72 b + 18 c) / 81 et (−9 a + 45 b + 45 c) / 81.
fn tiers_cubiques(x: &[f64], i: usize) -> (f64, f64) {
    let n = x.len();
    if n < 3 {
        let (b, c) = (x[i], x[i + 1]);
        ((2.0 * b + c) / 3.0, (b + 2.0 * c) / 3.0)
    } else if i == 0 {
        let (b, c, d) = (x[0], x[1], x[2]);
        (
            (45.0 * b + 45.0 * c - 9.0 * d) / 81.0,
            (18.0 * b + 72.0 * c - 9.0 * d) / 81.0,
        )
    } else if i + 2 >= n {
        let (a, b, c) = (x[i - 1], x[i], x[i + 1]);
        (
            (-9.0 * a + 72.0 * b + 18.0 * c) / 81.0,
            (-9.0 * a + 45.0 * b + 45.0 * c) / 81.0,
        )
    } else {
        let (a, b, c, d) = (x[i - 1], x[i], x[i + 1], x[i + 2]);
        (
            (-6.0 * a + 63.0 * b + 27.0 * c - 3.0 * d) / 81.0,
            (-3.0 * a + 27.0 * b + 63.0 * c - 6.0 * d) / 81.0,
        )
    }
}

/// Interpolation cubique 1D d'un facteur 3 (formules en /81 du cours).
///
/// La sortie contient 3N valeurs : x(0), y(1/3), y(2/3), x(1), ..., x(N−1), puis
/// deux valeurs extrapolées linéairement x(N−1) + Δ/3 et x(N−1) + 2Δ/3, avec
/// Δ = x(N−1) − x(N−2). Les coefficients sont donnés dans `tiers_cubiques`.
pub fn interpoler_ligne_x3_cubique(ligne: &[f64]) -> Vec<f64> {
    let n = ligne.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![ligne[0], ligne[0], ligne[0]];
    }

    let mut sortie = Vec::with_capacity(3 * n);
    for (i, &xi) in ligne[..n - 1].iter().enumerate() {
        let (t1, t2) = tiers_cubiques(ligne, i);
        sortie.push(xi);
        sortie.push(t1);
        sortie.push(t2);
    }
    // Dernier point, puis extrapolation linéaire.
    let delta = ligne[n - 1] - ligne[n - 2];
    sortie.push(ligne[n - 1]);
    sortie.push(ligne[n - 1] + delta / 3.0);
    sortie.push(ligne[n - 1] + 2.0 * delta / 3.0);
    sortie
}

// ---------------------------------------------------------------------------
// Zoom x2 séparable
// ---------------------------------------------------------------------------

/// Zoom x2 séparable : `interpoler_ligne_x2` sur les lignes puis sur les colonnes.
///
/// Étape 1 : interpoler chaque ligne → image temporaire de largeur 2L × H.
/// Étape 2 : interpoler chaque colonne de l'image temporaire → image 2L × 2H.
/// Les valeurs intermédiaires restent en réels ; l'arrondi se fait à la fin.
pub fn zoom_x2_separable(img: &ImageGris, methode: Methode) -> ImageGris {
    let l = img.largeur();
    let h = img.hauteur();
    if l == 0 || h == 0 {
        return ImageGris::nouvelle(0, 0, 0);
    }

    // Étape 1 : interpolation horizontale, lignes de l → 2l valeurs.
    let lignes: Vec<Vec<f64>> = (0..h)
        .map(|y| {
            let ligne: Vec<f64> = img.ligne(y).iter().map(|&p| p as f64).collect();
            interpoler_ligne_x2(&ligne, methode)
        })
        .collect();

    // Étape 2 : interpolation verticale, colonnes de h → 2h valeurs.
    let l2 = 2 * l;
    let h2 = 2 * h;
    let mut sortie = ImageGris::nouvelle(l2, h2, 0);
    for x in 0..l2 {
        let colonne: Vec<f64> = lignes.iter().map(|ligne| ligne[x]).collect();
        for (y, &v) in interpoler_ligne_x2(&colonne, methode).iter().enumerate() {
            sortie[(x, y)] = vers_u8(v);
        }
    }
    sortie
}

// ---------------------------------------------------------------------------
// Rotation
// ---------------------------------------------------------------------------

/// Rotation de l'image d'un angle en degrés autour de son centre
/// (utilisée pour montrer que la FFT tourne du même angle).
///
/// Utilise le **mapping inverse** : pour chaque pixel de sortie, on calcule
/// le pixel source correspondant par rotation de −angle, puis on interpole.
/// Les pixels en dehors de l'image source sont mis à 0 (fond noir).
pub fn rotation(img: &ImageGris, angle_degres: f64, methode: Methode) -> ImageGris {
    let l = img.largeur();
    let h = img.hauteur();
    if l == 0 || h == 0 {
        return ImageGris::nouvelle(l, h, 0);
    }

    let angle_rad = -angle_degres.to_radians(); // mapping inverse → angle opposé
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    // Centre de l'image (en coordonnées réelles).
    let cx = (l as f64 - 1.0) / 2.0;
    let cy = (h as f64 - 1.0) / 2.0;
    let xmax = l as f64 - 1.0;
    let ymax = h as f64 - 1.0;

    ImageGris::depuis_fonction(l, h, |xd, yd| {
        // Vecteur depuis le centre.
        let dx = xd as f64 - cx;
        let dy = yd as f64 - cy;

        // Rotation inverse (mapping arrière).
        let xs = cos_a * dx - sin_a * dy + cx;
        let ys = sin_a * dx + cos_a * dy + cy;

        // Pixel source hors de l'image : fond noir.
        let dedans = (-0.5..=xmax + 0.5).contains(&xs) && (-0.5..=ymax + 0.5).contains(&ys);
        if dedans {
            interpoler_pixel(img, xs.clamp(0.0, xmax), ys.clamp(0.0, ymax), methode)
        } else {
            0
        }
    })
}

// ---------------------------------------------------------------------------
// Tests unitaires
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::Rvb;

    fn img2x2() -> ImageGris {
        // Pixels : (0,0)=0  (1,0)=100  (0,1)=200  (1,1)=255
        ImageGris::depuis_vec(2, 2, vec![0, 100, 200, 255]).unwrap()
    }

    fn proches(a: &[f64], b: &[f64]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-9)
    }

    // ---- agrandir_par_copie ----

    #[test]
    fn copie_x2_carre() {
        // 2×2 → 4×4 : chaque pixel répété en bloc 2×2.
        let r = agrandir_par_copie(&img2x2(), 2).unwrap();
        assert_eq!(r.dimensions(), (4, 4));
        assert_eq!(r.ligne(0), &[0, 0, 100, 100]);
        assert_eq!(r.ligne(1), &[0, 0, 100, 100]);
        assert_eq!(r.ligne(2), &[200, 200, 255, 255]);
        assert_eq!(r.ligne(3), &[200, 200, 255, 255]);
    }

    #[test]
    fn copie_facteur_1_identique() {
        let img = img2x2();
        assert_eq!(agrandir_par_copie(&img, 1).unwrap(), img);
    }

    #[test]
    fn copie_facteur_zero_erreur() {
        assert!(agrandir_par_copie(&img2x2(), 0).is_err());
    }

    // ---- redimensionner / zoom ----

    #[test]
    fn zoom_identite() {
        let img = img2x2();
        assert_eq!(zoom(&img, 1.0, Methode::Bilineaire).unwrap(), img);
        assert_eq!(zoom(&img, 1.0, Methode::Bicubique).unwrap(), img);
    }

    #[test]
    fn redimensionner_bilineaire_valeurs() {
        // Ligne [0, 100] → 3 pixels : le milieu vaut 50.
        let img = ImageGris::depuis_vec(2, 1, vec![0, 100]).unwrap();
        let r = redimensionner(&img, 3, 1, Methode::Bilineaire).unwrap();
        assert_eq!(r, ImageGris::depuis_vec(3, 1, vec![0, 50, 100]).unwrap());
    }

    #[test]
    fn redimensionner_ppv_valeurs() {
        // Ligne [10, 20, 30] → 5 pixels : xs = 0, 0.5, 1, 1.5, 2 arrondis.
        let img = ImageGris::depuis_vec(3, 1, vec![10, 20, 30]).unwrap();
        let r = redimensionner(&img, 5, 1, Methode::PlusProcheVoisin).unwrap();
        assert_eq!(
            r,
            ImageGris::depuis_vec(5, 1, vec![10, 20, 20, 30, 30]).unwrap()
        );
    }

    #[test]
    fn redimensionner_dimensions_correctes() {
        let r = redimensionner(&img2x2(), 5, 7, Methode::Bilineaire).unwrap();
        assert_eq!(r.dimensions(), (5, 7));
    }

    #[test]
    fn redimensionner_dimension_nulle_erreur() {
        let img = img2x2();
        assert!(redimensionner(&img, 0, 3, Methode::Bilineaire).is_err());
    }

    #[test]
    fn zoom_facteur_invalide_erreur() {
        let img = img2x2();
        assert!(zoom(&img, -1.0, Methode::Bilineaire).is_err());
        assert!(zoom(&img, 0.0, Methode::Bilineaire).is_err());
        assert!(zoom(&img, f64::NAN, Methode::Bilineaire).is_err());
        assert!(zoom(&img, f64::INFINITY, Methode::Bilineaire).is_err());
    }

    #[test]
    fn zoom_x2_coins_conserves() {
        // Les coins d'un zoom (mapping aux coins) sont ceux de l'image source.
        let img = img2x2();
        for methode in [
            Methode::PlusProcheVoisin,
            Methode::Bilineaire,
            Methode::Bicubique,
        ] {
            let r = zoom(&img, 2.0, methode).unwrap();
            assert_eq!(r.dimensions(), (4, 4));
            assert_eq!(r[(0, 0)], 0);
            assert_eq!(r[(3, 0)], 100);
            assert_eq!(r[(0, 3)], 200);
            assert_eq!(r[(3, 3)], 255);
        }
    }

    #[test]
    fn bicubique_image_constante() {
        // Les poids de Catmull-Rom somment à 1 : une image constante reste constante.
        let img = ImageGris::nouvelle(3, 3, 77);
        let r = zoom(&img, 2.5, Methode::Bicubique).unwrap();
        assert!(r.ligne(0).iter().all(|&p| p == 77));
        assert!(r.ligne(r.hauteur() - 1).iter().all(|&p| p == 77));
    }

    // ---- versions couleur ----

    #[test]
    fn redimensionner_rvb_image_constante() {
        let couleur = Rvb::new(10, 20, 30);
        let img = ImageRvb::nouvelle(2, 2, couleur);
        let r = redimensionner_rvb(&img, 4, 6, Methode::Bilineaire).unwrap();
        assert_eq!(r, ImageRvb::nouvelle(4, 6, couleur));
    }

    #[test]
    fn zoom_rvb_taille_et_erreur() {
        let img = ImageRvb::nouvelle(2, 3, Rvb::new(1, 2, 3));
        let r = zoom_rvb(&img, 2.0, Methode::PlusProcheVoisin).unwrap();
        assert_eq!(r.dimensions(), (4, 6));
        assert!(zoom_rvb(&img, -2.0, Methode::Bilineaire).is_err());
    }

    // ---- interpoler_ligne_x2 ----

    #[test]
    fn interpoler_ligne_x2_lineaire_valeurs() {
        // [0, 100] → [0, 50, 100, (100 + 200) / 2] avec x(2) extrapolé = 2·100 − 0.
        let r = interpoler_ligne_x2(&[0.0, 100.0], Methode::Bilineaire);
        assert!(proches(&r, &[0.0, 50.0, 100.0, 150.0]));
    }

    #[test]
    fn interpoler_ligne_x2_lineaire_longueur() {
        let r = interpoler_ligne_x2(&[0.0, 100.0, 200.0], Methode::Bilineaire);
        assert_eq!(r.len(), 6);
    }

    #[test]
    fn interpoler_ligne_x2_cubique_valeurs() {
        // x = [0, 0, 16, 0, 0], calculé à la main :
        // i=0 (parabole)  : (3·0 + 6·0 − 16) / 8         = −2
        // i=1 (intérieur) : (−0 + 9·0 + 9·16 − 0) / 16   = 9
        // i=2 (intérieur) : (−0 + 9·16 + 9·0 − 0) / 16   = 9
        // i=3 (parabole)  : (−16 + 6·0 + 3·0) / 8        = −2
        // fin (linéaire)  : (0 + (2·0 − 0)) / 2           = 0
        let r = interpoler_ligne_x2(&[0.0, 0.0, 16.0, 0.0, 0.0], Methode::Bicubique);
        let attendu = [0.0, -2.0, 0.0, 9.0, 16.0, 9.0, 0.0, -2.0, 0.0, 0.0];
        assert!(proches(&r, &attendu), "{r:?}");
    }

    #[test]
    fn interpoler_ligne_x2_cubique_rampe() {
        // Toutes les formules (cubique, paraboles, extrapolation) sont exactes sur une droite.
        let r = interpoler_ligne_x2(&[0.0, 10.0, 20.0, 30.0], Methode::Bicubique);
        assert!(proches(&r, &[0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0]));
    }

    #[test]
    fn interpoler_ligne_x2_cas_limites() {
        assert!(interpoler_ligne_x2(&[], Methode::Bicubique).is_empty());
        assert!(proches(
            &interpoler_ligne_x2(&[7.0], Methode::Bicubique),
            &[7.0, 7.0]
        ));
    }

    // ---- interpoler_ligne_x3_cubique ----

    #[test]
    fn interpoler_ligne_x3_valeurs() {
        // x = [0, 0, 81, 0, 0], calculé à la main avec les formules en /81 :
        // i=0 (parabole)  : −9·81/81 = −9 et −9
        // i=1 (intérieur) : 27·81/81 = 27 et 63
        // i=2 (intérieur) : 63 et 27
        // i=3 (parabole)  : −9 et −9
        // fin (linéaire)  : Δ = 0 → 0 et 0
        let r = interpoler_ligne_x3_cubique(&[0.0, 0.0, 81.0, 0.0, 0.0]);
        let attendu = [
            0.0, -9.0, -9.0, 0.0, 27.0, 63.0, 81.0, 63.0, 27.0, 0.0, -9.0, -9.0, 0.0, 0.0, 0.0,
        ];
        assert!(proches(&r, &attendu), "{r:?}");
    }

    #[test]
    fn interpoler_ligne_x3_rampe_exacte() {
        // Sur une droite de pente 30, la sortie est la droite de pente 10.
        let ligne: Vec<f64> = (0..6).map(|i| i as f64 * 30.0).collect();
        let r = interpoler_ligne_x3_cubique(&ligne);
        let attendu: Vec<f64> = (0..18).map(|k| k as f64 * 10.0).collect();
        assert!(proches(&r, &attendu), "{r:?}");
    }

    #[test]
    fn interpoler_ligne_x3_cas_limites() {
        assert!(interpoler_ligne_x3_cubique(&[]).is_empty());
        assert!(proches(&interpoler_ligne_x3_cubique(&[5.0]), &[5.0; 3]));
        // Deux points : interpolation linéaire.
        let r = interpoler_ligne_x3_cubique(&[0.0, 30.0]);
        assert!(proches(&r, &[0.0, 10.0, 20.0, 30.0, 40.0, 50.0]));
    }

    // ---- zoom_x2_separable ----

    #[test]
    fn zoom_x2_separable_bilineaire_valeurs() {
        // [0, 100] / [200, 255] lignes puis colonnes, saturé à 255.
        // Lignes : [0, 50, 100, 150] et [200, 227.5, 255, 282.5].
        // Colonnes : x(0), milieu, x(1), x(1) + (x(1) − x(0)) / 2.
        let r = zoom_x2_separable(&img2x2(), Methode::Bilineaire);
        assert_eq!(r.dimensions(), (4, 4));
        assert_eq!(r.ligne(0), &[0, 50, 100, 150]);
        assert_eq!(r.ligne(1), &[100, 139, 178, 216]);
        assert_eq!(r.ligne(2), &[200, 228, 255, 255]);
        assert_eq!(r.ligne(3), &[255, 255, 255, 255]);
    }

    #[test]
    fn zoom_x2_separable_image_vide() {
        let vide = ImageGris::nouvelle(0, 0, 0);
        let r = zoom_x2_separable(&vide, Methode::Bicubique);
        assert_eq!(r.dimensions(), (0, 0));
    }

    // ---- rotation ----

    #[test]
    fn rotation_zero_degre_identique() {
        let img = img2x2();
        assert_eq!(rotation(&img, 0.0, Methode::Bilineaire), img);
    }

    #[test]
    fn rotation_90_degres() {
        // Rotation de 90° autour du centre d'une image 3×3 (PPV, sans perte).
        // Le sens suit le repère image (y vers le bas).
        let img = ImageGris::depuis_vec(3, 3, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]).unwrap();
        let r = rotation(&img, 90.0, Methode::PlusProcheVoisin);
        let attendu = ImageGris::depuis_vec(3, 3, vec![7, 4, 1, 8, 5, 2, 9, 6, 3]).unwrap();
        assert_eq!(r, attendu);
    }

    #[test]
    fn rotation_360_identique() {
        let img = img2x2();
        let r = rotation(&img, 360.0, Methode::Bilineaire);
        for y in 0..img.hauteur() {
            for x in 0..img.largeur() {
                let diff = (r[(x, y)] as i32 - img[(x, y)] as i32).abs();
                assert!(diff <= 1, "différence trop grande en ({x},{y}) : {diff}");
            }
        }
    }

    #[test]
    fn rotation_coins_noirs() {
        // À 45°, les coins de sortie viennent de l'extérieur de l'image : fond noir.
        let img = ImageGris::nouvelle(9, 9, 200);
        let r = rotation(&img, 45.0, Methode::Bilineaire);
        assert_eq!(r.dimensions(), (9, 9));
        assert_eq!(r[(0, 0)], 0);
        assert_eq!(r[(8, 8)], 0);
        assert_eq!(r[(4, 4)], 200);
    }
}
