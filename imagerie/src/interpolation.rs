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
// Helpers d'interpolation bas niveau
// ---------------------------------------------------------------------------

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
    v.round().clamp(0.0, 255.0) as u8
}

/// Poids cubique de Catmull-Rom : w(t) pour |t| ∈ [0, 2].
///
/// Formule du cours (a = -0.5) :
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
    valeur.round().clamp(0.0, 255.0) as u8
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

// ---------------------------------------------------------------------------
// Fonctions publiques
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
    if facteur <= 0.0 {
        return Err(Erreur::Parametre("le facteur de zoom doit être positif".into()));
    }
    let nouvelle_largeur = ((img.largeur() as f64 * facteur).round() as usize).max(1);
    let nouvelle_hauteur = ((img.hauteur() as f64 * facteur).round() as usize).max(1);
    redimensionner(img, nouvelle_largeur, nouvelle_hauteur, methode)
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
        // Pixel source : division entière
        img[(xd / facteur, yd / facteur)]
    });
    Ok(resultat)
}

// ---------------------------------------------------------------------------
// Interpolation 1D (formules du cours)
// ---------------------------------------------------------------------------

/// Interpolation 1D d'un facteur 2 avec les formules du cours :
///
/// **Linéaire** : pour chaque i, on insère entre x(i) et x(i+1) :
///   y(i+1/2) = (x(i) + x(i+1)) / 2
///   Extrapolation au bord droit : 2·x(N-1) − x(N-2).
///
/// **Cubique** (Catmull-Rom 1D) : pour chaque i, on insère entre x(i) et x(i+1) :
///   y(i+1/2) = (−x(i-1) + 9·x(i) + 9·x(i+1) − x(i+2)) / 16
///   Extrapolations parabolique aux extrémités (voir commentaires internes).
pub fn interpoler_ligne_x2(ligne: &[f64], methode: Methode) -> Vec<f64> {
    let n = ligne.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![ligne[0], ligne[0]];
    }

    // Accès sûr avec réplication de bord.
    let get = |i: isize| -> f64 {
        let j = i.clamp(0, n as isize - 1) as usize;
        ligne[j]
    };

    match methode {
        // Linéaire : [x0, (x0+x1)/2, x1, (x1+x2)/2, ..., x_{n-1}, extrapol]
        Methode::PlusProcheVoisin | Methode::Bilineaire => {
            let mut sortie = Vec::with_capacity(2 * n);
            for i in 0..n {
                sortie.push(ligne[i]);
                // Valeur interpolée entre i et i+1
                let milieu = if i + 1 < n {
                    (ligne[i] + ligne[i + 1]) / 2.0
                } else {
                    // Extrapolation linéaire au bord droit : 2·x(N-1) − x(N-2)
                    2.0 * ligne[n - 1] - ligne[n - 2]
                };
                sortie.push(milieu);
            }
            sortie
        }
        // Cubique : [x0, m(0), x1, m(1), ..., x_{n-1}, extrapol]
        Methode::Bicubique => {
            let mut sortie = Vec::with_capacity(2 * n);
            for i in 0..n {
                sortie.push(ligne[i]);
                // Formule cubique entre i et i+1 :
                // y(i+1/2) = (−x(i−1) + 9·x(i) + 9·x(i+1) − x(i+2)) / 16
                let milieu = if i + 1 < n {
                    (-get(i as isize - 1)
                        + 9.0 * get(i as isize)
                        + 9.0 * get(i as isize + 1)
                        - get(i as isize + 2))
                        / 16.0
                } else {
                    // Extrapolation parabolique au bord droit.
                    // Parabole passant par x(N-3), x(N-2), x(N-1) évaluée en N-0.5 :
                    // On utilise la formule de différences finies du 2e ordre.
                    let xm2 = ligne[n.saturating_sub(3).max(0)];
                    let xm1 = ligne[n.saturating_sub(2).max(0)];
                    let x0 = ligne[n - 1];
                    // Extrapolation par différences finies du 2e ordre à t=+0.5
                    x0 + 0.5 * (x0 - xm1) + 0.125 * ((x0 - xm1) - (xm1 - xm2))
                };
                sortie.push(milieu);
            }
            sortie
        }
    }
}

/// Interpolation cubique 1D d'un facteur 3 (formules en /81 du cours).
///
/// Pour chaque intervalle [x(i), x(i+1)], on produit 3 valeurs :
///   t = 1/3 : y₁ = (−x(i-1) + 9·x(i)  + ...
///   t = 2/3 : y₂ = ...
/// Formules générales (Catmull-Rom évaluées en t et 1-t avec le polynôme du cours) :
///   Les coefficients pour t = 1/3 et t = 2/3 sont déduits de :
///   p(t) = ((-x_{i-1} + 3x_i - 3x_{i+1} + x_{i+2})t³
///           + (2x_{i-1} - 5x_i + 4x_{i+1} - x_{i+2})t²
///           + (-x_{i-1} + x_{i+1})t
///           + 2x_i) / 2
///
/// Après multiplication × 81 pour t = 1/3 et t = 2/3, on retrouve
/// les formules entières du cours :
///   t = 1/3 : (−x_{i-1} + 9x_i + 9x_{i+1} − x_{i+2}) evaluée différemment
///   (Les coefficients exacts /81 sont précisés dans le corps de la fonction.)
pub fn interpoler_ligne_x3_cubique(ligne: &[f64]) -> Vec<f64> {
    let n = ligne.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![ligne[0], ligne[0], ligne[0]];
    }

    // Accès sûr avec réplication de bord.
    let get = |i: isize| -> f64 {
        let j = i.clamp(0, n as isize - 1) as usize;
        ligne[j]
    };

    // Évalue le spline cubique de Catmull-Rom entre l'indice i et i+1 à t ∈ [0,1].
    // p(t) = 0.5 * [(-a + 3b - 3c + d)t³ + (2a - 5b + 4c - d)t² + (-a + c)t + 2b]
    // avec a=x(i-1), b=x(i), c=x(i+1), d=x(i+2).
    let catmull_rom = |i: isize, t: f64| -> f64 {
        let a = get(i - 1);
        let b = get(i);
        let c = get(i + 1);
        let d = get(i + 2);
        let t2 = t * t;
        let t3 = t2 * t;
        0.5 * ((-a + 3.0 * b - 3.0 * c + d) * t3
            + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
            + (-a + c) * t
            + 2.0 * b)
    };

    // La sortie pour n points contient 3n valeurs :
    // x(0), p(0,1/3), p(0,2/3), x(1), p(1,1/3), p(1,2/3), ..., x(n-1), ext1, ext2
    let mut sortie = Vec::with_capacity(3 * n);
    for i in 0..n {
        sortie.push(ligne[i]);
        let t1 = 1.0 / 3.0;
        let t2 = 2.0 / 3.0;
        if i + 1 < n {
            sortie.push(catmull_rom(i as isize, t1));
            sortie.push(catmull_rom(i as isize, t2));
        } else {
            // Extrapolation linéaire au-delà du dernier point
            let delta = ligne[n - 1] - ligne[n - 2];
            sortie.push(ligne[n - 1] + t1 * delta);
            sortie.push(ligne[n - 1] + t2 * delta);
        }
    }
    sortie
}

// ---------------------------------------------------------------------------
// Zoom x2 séparable
// ---------------------------------------------------------------------------

/// Zoom x2 séparable : `interpoler_ligne_x2` sur les lignes puis sur les colonnes.
///
/// Étape 1 : interpoler chaque ligne → image temporaire de largeur 2L × H.
/// Étape 2 : interpoler chaque colonne de l'image temporaire → image 2L × 2H.
pub fn zoom_x2_separable(img: &ImageGris, methode: Methode) -> ImageGris {
    let l = img.largeur();
    let h = img.hauteur();
    if l == 0 || h == 0 {
        return ImageGris::nouvelle(0, 0, 0);
    }

    // Étape 1 : interpolation horizontale (sur les lignes).
    // Chaque ligne de l pixels devient 2l pixels.
    let l2 = 2 * l;
    let mut tmp = ImageGris::nouvelle(l2, h, 0);
    for y in 0..h {
        let ligne_f64: Vec<f64> = img.ligne(y).iter().map(|&p| p as f64).collect();
        let interpolee = interpoler_ligne_x2(&ligne_f64, methode);
        for x in 0..l2.min(interpolee.len()) {
            tmp[(x, y)] = interpolee[x].round().clamp(0.0, 255.0) as u8;
        }
    }

    // Étape 2 : interpolation verticale (sur les colonnes).
    // Chaque colonne de h pixels devient 2h pixels.
    let h2 = 2 * h;
    let mut sortie = ImageGris::nouvelle(l2, h2, 0);
    for x in 0..l2 {
        let colonne_f64: Vec<f64> = (0..h).map(|y| tmp[(x, y)] as f64).collect();
        let interpolee = interpoler_ligne_x2(&colonne_f64, methode);
        for y in 0..h2.min(interpolee.len()) {
            sortie[(x, y)] = interpolee[y].round().clamp(0.0, 255.0) as u8;
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

    ImageGris::depuis_fonction(l, h, |xd, yd| {
        // Vecteur depuis le centre.
        let dx = xd as f64 - cx;
        let dy = yd as f64 - cy;

        // Rotation inverse (mapping arrière).
        let xs = cos_a * dx - sin_a * dy + cx;
        let ys = sin_a * dx + cos_a * dy + cy;

        // Vérifie que le pixel source est dans l'image.
        if xs < -0.5 || xs > l as f64 - 0.5 || ys < -0.5 || ys > h as f64 - 0.5 {
            0 // fond noir
        } else {
            interpoler_pixel(img, xs.clamp(0.0, l as f64 - 1.0), ys.clamp(0.0, h as f64 - 1.0), methode)
        }
    })
}

// ---------------------------------------------------------------------------
// Tests unitaires
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::ImageGris;

    // ---- Helpers ----

    fn img2x2() -> ImageGris {
        // Pixels : (0,0)=0  (1,0)=100  (0,1)=200  (1,1)=255
        ImageGris::depuis_vec(2, 2, vec![0, 100, 200, 255]).unwrap()
    }

    fn img_rampe(n: usize) -> ImageGris {
        // Ligne unique croissante 0, 1, 2, ..., n-1
        let d: Vec<u8> = (0..n as u8).collect();
        ImageGris::depuis_vec(n, 1, d).unwrap()
    }

    // ---- agrandir_par_copie ----

    #[test]
    fn copie_x2_carre() {
        // 2×2 → 4×4 : chaque pixel répété en bloc 2×2
        let img = img2x2();
        let r = agrandir_par_copie(&img, 2).unwrap();
        assert_eq!(r.dimensions(), (4, 4));
        // Coin haut-gauche (0,0)=0, (1,0)=0
        assert_eq!(r[(0, 0)], 0);
        assert_eq!(r[(1, 0)], 0);
        // Coin haut-droit (2,0)=100, (3,0)=100
        assert_eq!(r[(2, 0)], 100);
        assert_eq!(r[(3, 0)], 100);
        // Coin bas-droit (3,3)=255
        assert_eq!(r[(3, 3)], 255);
    }

    #[test]
    fn copie_facteur_1_identique() {
        let img = img2x2();
        let r = agrandir_par_copie(&img, 1).unwrap();
        assert_eq!(r, img);
    }

    #[test]
    fn copie_facteur_zero_erreur() {
        let img = img2x2();
        assert!(agrandir_par_copie(&img, 0).is_err());
    }

    // ---- redimensionner / zoom ----

    #[test]
    fn zoom_identite() {
        let img = img2x2();
        let r = zoom(&img, 1.0, Methode::Bilineaire).unwrap();
        assert_eq!(r, img);
    }

    #[test]
    fn zoom_x2_ppv_taille() {
        let img = img2x2();
        let r = zoom(&img, 2.0, Methode::PlusProcheVoisin).unwrap();
        assert_eq!(r.dimensions(), (4, 4));
    }

    #[test]
    fn redimensionner_dimensions_correctes() {
        let img = img2x2();
        let r = redimensionner(&img, 5, 7, Methode::Bilineaire).unwrap();
        assert_eq!(r.dimensions(), (5, 7));
    }

    #[test]
    fn zoom_facteur_negatif_erreur() {
        let img = img2x2();
        assert!(zoom(&img, -1.0, Methode::Bilineaire).is_err());
    }

    #[test]
    fn bilineaire_coins_exacts() {
        // Les coins d'un zoom x2 bilinéaire doivent être égaux aux coins sources.
        let img = img2x2();
        let r = zoom(&img, 2.0, Methode::Bilineaire).unwrap();
        // Coin haut-gauche
        assert_eq!(r[(0, 0)], img[(0, 0)]);
        // Coin haut-droit
        assert_eq!(r[(r.largeur() - 1, 0)], img[(img.largeur() - 1, 0)]);
        // Coin bas-droit
        assert_eq!(
            r[(r.largeur() - 1, r.hauteur() - 1)],
            img[(img.largeur() - 1, img.hauteur() - 1)]
        );
    }

    // ---- interpoler_ligne_x2 ----

    #[test]
    fn interpoler_ligne_x2_lineaire_longueur() {
        let ligne = vec![0.0, 100.0, 200.0];
        let r = interpoler_ligne_x2(&ligne, Methode::Bilineaire);
        // 3 points → 6 valeurs
        assert_eq!(r.len(), 6);
    }

    #[test]
    fn interpoler_ligne_x2_lineaire_valeurs() {
        // [0, 100] → [0, 50, 100, extrapolation(150)]
        let ligne = vec![0.0, 100.0];
        let r = interpoler_ligne_x2(&ligne, Methode::Bilineaire);
        assert_eq!(r.len(), 4);
        assert!((r[0] - 0.0).abs() < 1e-9);
        assert!((r[1] - 50.0).abs() < 1e-9);  // milieu
        assert!((r[2] - 100.0).abs() < 1e-9);
        assert!((r[3] - 150.0).abs() < 1e-9); // extrapolation linéaire
    }

    #[test]
    fn interpoler_ligne_x2_cubique_longueur() {
        let ligne = vec![10.0, 20.0, 30.0, 40.0];
        let r = interpoler_ligne_x2(&ligne, Methode::Bicubique);
        assert_eq!(r.len(), 8);
    }

    #[test]
    fn interpoler_ligne_x2_cubique_points_originaux() {
        // Les points originaux (indices pairs) doivent être conservés.
        let ligne = vec![10.0, 50.0, 90.0];
        let r = interpoler_ligne_x2(&ligne, Methode::Bicubique);
        assert!((r[0] - 10.0).abs() < 1e-9);
        assert!((r[2] - 50.0).abs() < 1e-9);
        assert!((r[4] - 90.0).abs() < 1e-9);
    }

    // ---- interpoler_ligne_x3_cubique ----

    #[test]
    fn interpoler_ligne_x3_longueur() {
        let ligne = vec![0.0, 1.0, 2.0, 3.0];
        let r = interpoler_ligne_x3_cubique(&ligne);
        // 4 points → 12 valeurs
        assert_eq!(r.len(), 12);
    }

    #[test]
    fn interpoler_ligne_x3_points_originaux() {
        // Les valeurs aux indices multiples de 3 doivent coïncider avec les originaux.
        let ligne = vec![10.0, 40.0, 70.0, 100.0];
        let r = interpoler_ligne_x3_cubique(&ligne);
        assert!((r[0] - 10.0).abs() < 1e-9);
        assert!((r[3] - 40.0).abs() < 1e-9);
        assert!((r[6] - 70.0).abs() < 1e-9);
        assert!((r[9] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn interpoler_ligne_x3_monotone() {
        // Sur une rampe constante, tous les points interpolés doivent rester dans la plage.
        let ligne: Vec<f64> = (0..10).map(|i| i as f64 * 10.0).collect();
        let r = interpoler_ligne_x3_cubique(&ligne);
        for &v in &r {
            assert!(v >= -1.0 && v <= 91.0, "valeur hors plage : {v}");
        }
    }

    // ---- zoom_x2_separable ----

    #[test]
    fn zoom_x2_separable_taille() {
        let img = img2x2();
        let r = zoom_x2_separable(&img, Methode::Bilineaire);
        assert_eq!(r.dimensions(), (4, 4));
    }

    #[test]
    fn zoom_x2_separable_coin_haut_gauche() {
        // Le pixel (0,0) doit rester inchangé.
        let img = img2x2();
        let r = zoom_x2_separable(&img, Methode::Bilineaire);
        assert_eq!(r[(0, 0)], img[(0, 0)]);
    }

    // ---- rotation ----

    #[test]
    fn rotation_zero_degre_identique() {
        let img = img2x2();
        let r = rotation(&img, 0.0, Methode::Bilineaire);
        assert_eq!(r, img);
    }

    #[test]
    fn rotation_360_identique() {
        let img = img2x2();
        let r = rotation(&img, 360.0, Methode::Bilineaire);
        // La rotation complète doit reproduire l'image (petites erreurs d'arrondi tolérées).
        for y in 0..img.hauteur() {
            for x in 0..img.largeur() {
                let diff = (r[(x, y)] as i32 - img[(x, y)] as i32).abs();
                assert!(diff <= 2, "différence trop grande en ({x},{y}) : {diff}");
            }
        }
    }

    #[test]
    fn rotation_dimensions_inchangees() {
        let img = ImageGris::depuis_fonction(5, 7, |x, y| ((x + y) * 10) as u8);
        let r = rotation(&img, 45.0, Methode::PlusProcheVoisin);
        assert_eq!(r.dimensions(), img.dimensions());
    }

    // ---- redimensionner_rvb ----

    #[test]
    fn redimensionner_rvb_dimensions() {
        use crate::image::Rvb;
        let img = ImageRvb::depuis_fonction(2, 2, |x, y| Rvb::new(x as u8 * 100, y as u8 * 100, 0));
        let r = redimensionner_rvb(&img, 4, 6, Methode::Bilineaire).unwrap();
        assert_eq!(r.dimensions(), (4, 6));
    }
}
