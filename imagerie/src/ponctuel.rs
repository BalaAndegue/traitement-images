//! Chapitre 2 (première partie) : histogramme et transformations ponctuelles.
//!
//! Responsable : Belekotan II Jeff Nichoss.
//!
//! Histogramme, dynamique, luminance, contraste, LUT, transformations linéaires
//! (simple, avec saturation, par morceaux), correction gamma, égalisation
//! d'histogramme globale et locale.

use crate::erreur::{Erreur, Resultat};
use crate::image::{ImageGris, ImageRvb};

pub type Histogramme = [u32; 256];

/// Table de correspondance : nouvelle valeur = lut[ancienne valeur].
pub type Lut = [u8; 256];

/// H(k) = nombre de pixels de valeur k.
pub fn histogramme(img: &ImageGris) -> Histogramme {
    let mut h = [0u32; 256];
    for &p in img.donnees() {
        h[p as usize] += 1;
    }
    h
}

/// Histogramme de chacun des canaux R, V, B.
pub fn histogramme_rvb(img: &ImageRvb) -> [Histogramme; 3] {
    let mut hr = [0u32; 256];
    let mut hv = [0u32; 256];
    let mut hb = [0u32; 256];
    for p in img.donnees() {
        hr[p.r as usize] += 1;
        hv[p.v as usize] += 1;
        hb[p.b as usize] += 1;
    }
    [hr, hv, hb]
}

/// hn(k) = H(k) / nombre de pixels.
pub fn histogramme_normalise(img: &ImageGris) -> [f64; 256] {
    let h = histogramme(img);
    let n = img.nb_pixels().max(1) as f64;
    std::array::from_fn(|k| h[k] as f64 / n)
}

/// C(k) = somme des hn(j) pour j <= k.
pub fn histogramme_cumule(img: &ImageGris) -> [f64; 256] {
    let hn = histogramme_normalise(img);
    let mut c = [0.0; 256];
    c[0] = hn[0];
    for k in 1..256 {
        c[k] = c[k - 1] + hn[k];
    }
    c
}

/// Dynamique de l'image : (valeur min, valeur max).
pub fn dynamique(img: &ImageGris) -> (u8, u8) {
    let mut min = u8::MAX;
    let mut max = u8::MIN;
    for &p in img.donnees() {
        min = min.min(p);
        max = max.max(p);
    }
    if img.nb_pixels() == 0 {
        (0, 0)
    } else {
        (min, max)
    }
}

/// Luminance (brillance) : moyenne de tous les pixels.
pub fn luminance(img: &ImageGris) -> f64 {
    let n = img.nb_pixels();
    if n == 0 {
        return 0.0;
    }
    let somme: u64 = img.donnees().iter().map(|&p| p as u64).sum();
    somme as f64 / n as f64
}

/// Contraste mesuré par l'écart-type des niveaux de gris.
pub fn contraste_ecart_type(img: &ImageGris) -> f64 {
    let n = img.nb_pixels();
    if n == 0 {
        return 0.0;
    }
    let m = luminance(img);
    let variance = img
        .donnees()
        .iter()
        .map(|&p| {
            let d = p as f64 - m;
            d * d
        })
        .sum::<f64>()
        / n as f64;
    variance.sqrt()
}

/// Contraste mesuré par la variation entre min et max : (max - min) / (max + min).
pub fn contraste_min_max(img: &ImageGris) -> f64 {
    let (min, max) = dynamique(img);
    let somme = max as f64 + min as f64;
    if somme == 0.0 {
        0.0
    } else {
        (max as f64 - min as f64) / somme
    }
}

/// Applique une LUT à tous les pixels (implémentation optimale vue en cours).
pub fn appliquer_lut(img: &ImageGris, lut: &Lut) -> ImageGris {
    img.map(|&p| lut[p as usize])
}

/// Inverse vidéo : 255 - I.
pub fn negatif(img: &ImageGris) -> ImageGris {
    let lut = std::array::from_fn(|i| (255 - i) as u8);
    appliquer_lut(img, &lut)
}

/// Transformation linéaire (étirement de la dynamique) :
/// I' = 255 (I - min) / (max - min).
pub fn transformation_lineaire(img: &ImageGris) -> ImageGris {
    let (min, max) = dynamique(img);
    if min == max {
        return img.clone();
    }
    let min_f = min as f64;
    let denom = (max - min) as f64;
    let lut = std::array::from_fn(|i| {
        if (i as u8) < min {
            0
        } else if (i as u8) > max {
            255
        } else {
            (255.0 * (i as f64 - min_f) / denom).round().clamp(0.0, 255.0) as u8
        }
    });
    appliquer_lut(img, &lut)
}

/// Transformation linéaire avec saturation entre `s_min` et `s_max` :
/// I' = 255 (I - Smin) / (Smax - Smin), ramené dans [0, 255].
pub fn transformation_lineaire_saturation(
    img: &ImageGris,
    s_min: u8,
    s_max: u8,
) -> Resultat<ImageGris> {
    if s_min >= s_max {
        return Err(Erreur::Parametre(format!(
            "s_min ({s_min}) doit être strictement inférieur à s_max ({s_max})"
        )));
    }
    let s_min_f = s_min as f64;
    let denom = (s_max - s_min) as f64;
    let lut = std::array::from_fn(|i| {
        let v = 255.0 * (i as f64 - s_min_f) / denom;
        v.round().clamp(0.0, 255.0) as u8
    });
    Ok(appliquer_lut(img, &lut))
}

/// Transformation linéaire par morceaux définie par des points (entrée, sortie)
/// triés par entrée croissante ; (0, 0) et (255, 255) sont ajoutés s'ils manquent.
pub fn transformation_lineaire_morceaux(
    img: &ImageGris,
    points: &[(u8, u8)],
) -> Resultat<ImageGris> {
    let mut pts: Vec<(u8, u8)> = points.to_vec();
    if !pts.iter().any(|&(x, _)| x == 0) {
        pts.push((0, 0));
    }
    if !pts.iter().any(|&(x, _)| x == 255) {
        pts.push((255, 255));
    }
    pts.sort_by_key(|&(x, _)| x);
    for w in pts.windows(2) {
        if w[0].0 == w[1].0 && w[0].1 != w[1].1 {
            return Err(Erreur::Parametre(format!(
                "deux sorties différentes pour l'entrée {}",
                w[0].0
            )));
        }
    }
    pts.dedup_by_key(|p| p.0);

    let mut lut = [0u8; 256];
    for w in pts.windows(2) {
        let (x0, y0) = (w[0].0 as f64, w[0].1 as f64);
        let (x1, y1) = (w[1].0 as f64, w[1].1 as f64);
        let dx = x1 - x0;
        for x in w[0].0..=w[1].0 {
            let t = if dx == 0.0 {
                0.0
            } else {
                (x as f64 - x0) / dx
            };
            lut[x as usize] = (y0 + t * (y1 - y0)).round().clamp(0.0, 255.0) as u8;
        }
    }
    Ok(appliquer_lut(img, &lut))
}

/// Correction gamma : n' = 255 (n / 255)^(1/gamma).
pub fn correction_gamma(img: &ImageGris, gamma: f64) -> Resultat<ImageGris> {
    if gamma <= 0.0 || !gamma.is_finite() {
        return Err(Erreur::Parametre(format!(
            "gamma doit être un réel strictement positif (reçu {gamma})"
        )));
    }
    let inv = 1.0 / gamma;
    let lut = std::array::from_fn(|i| {
        (255.0 * (i as f64 / 255.0).powf(inv))
            .round()
            .clamp(0.0, 255.0) as u8
    });
    Ok(appliquer_lut(img, &lut))
}

/// Autres transformations non linéaires du cours : logarithme et exponentielle.
pub fn transformation_log(img: &ImageGris) -> ImageGris {
    let denom = (1.0_f64 + 255.0).ln();
    let lut = std::array::from_fn(|i| {
        (255.0 * (1.0 + i as f64).ln() / denom)
            .round()
            .clamp(0.0, 255.0) as u8
    });
    appliquer_lut(img, &lut)
}

pub fn transformation_exp(img: &ImageGris) -> ImageGris {
    // Inverse approximatif du log du cours : 255 * ((1+255)^(I/255) - 1) / 255.
    let base = 1.0_f64 + 255.0;
    let lut = std::array::from_fn(|i| {
        let v = base.powf(i as f64 / 255.0) - 1.0;
        (v).round().clamp(0.0, 255.0) as u8
    });
    appliquer_lut(img, &lut)
}

/// Égalisation d'histogramme en 4 étapes : histogramme, normalisation,
/// densité cumulée C, puis f'(x, y) = C(f(x, y)) x 255.
pub fn egalisation_histogramme(img: &ImageGris) -> ImageGris {
    let c = histogramme_cumule(img);
    let lut = std::array::from_fn(|k| (c[k] * 255.0).round().clamp(0.0, 255.0) as u8);
    appliquer_lut(img, &lut)
}

/// Égalisation locale : chaque pixel est égalisé avec l'histogramme d'une
/// fenêtre `taille` x `taille` centrée sur lui (7 x 7 dans le cours).
pub fn egalisation_locale(img: &ImageGris, taille: usize) -> Resultat<ImageGris> {
    if taille == 0 || taille.is_multiple_of(2) {
        return Err(Erreur::Parametre(format!(
            "taille de fenêtre {taille} : doit être impaire et > 0"
        )));
    }
    let (largeur, hauteur) = img.dimensions();
    let rayon = (taille / 2) as isize;
    let mut sortie = ImageGris::nouvelle(largeur, hauteur, 0);

    for y in 0..hauteur {
        for x in 0..largeur {
            let mut h = [0u32; 256];
            let mut n = 0u32;
            for dy in -rayon..=rayon {
                for dx in -rayon..=rayon {
                    let xx = x as isize + dx;
                    let yy = y as isize + dy;
                    if xx >= 0 && yy >= 0 && (xx as usize) < largeur && (yy as usize) < hauteur {
                        let p = *img.get(xx as usize, yy as usize).unwrap();
                        h[p as usize] += 1;
                        n += 1;
                    }
                }
            }
            let centre = *img.get(x, y).unwrap();
            if n == 0 {
                *sortie.get_mut(x, y).unwrap() = centre;
                continue;
            }
            // C(centre) = somme hn(j) pour j <= centre
            let mut cum = 0u32;
            for v in 0..=centre as usize {
                cum += h[v];
            }
            let val = (255.0 * cum as f64 / n as f64).round().clamp(0.0, 255.0) as u8;
            *sortie.get_mut(x, y).unwrap() = val;
        }
    }
    Ok(sortie)
}

/// Égalisation d'une image couleur (canal par canal).
pub fn egalisation_rvb(img: &ImageRvb) -> ImageRvb {
    let [r, v, b] = img.separer_canaux();
    ImageRvb::fusionner_canaux(
        &egalisation_histogramme(&r),
        &egalisation_histogramme(&v),
        &egalisation_histogramme(&b),
    )
    .expect("canaux issus de la même image")
}

/// Dessine un histogramme (barres blanches sur fond noir) pour l'affichage.
pub fn image_histogramme(hist: &Histogramme, largeur: usize, hauteur: usize) -> ImageGris {
    let mut img = ImageGris::nouvelle(largeur.max(1), hauteur.max(1), 0);
    let max = hist.iter().copied().max().unwrap_or(0).max(1);
    let largeur = img.largeur();
    let hauteur = img.hauteur();

    for x in 0..largeur {
        let bin = x * 256 / largeur;
        let h = ((*hist.get(bin).unwrap_or(&0) as f64 / max as f64) * (hauteur as f64 - 1.0))
            .round() as usize;
        for y in (hauteur - h.max(0))..hauteur {
            if let Some(p) = img.get_mut(x, y) {
                *p = 255;
            }
        }
    }
    img
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::Rvb;

    fn img_test() -> ImageGris {
        // 2x2 : 10, 20, 30, 40
        ImageGris::depuis_vec(2, 2, vec![10, 20, 30, 40]).unwrap()
    }

    #[test]
    fn histogramme_compte_les_niveaux() {
        let h = histogramme(&img_test());
        assert_eq!(h[10], 1);
        assert_eq!(h[20], 1);
        assert_eq!(h[30], 1);
        assert_eq!(h[40], 1);
        assert_eq!(h.iter().sum::<u32>(), 4);
    }

    #[test]
    fn dynamique_min_max() {
        assert_eq!(dynamique(&img_test()), (10, 40));
        assert_eq!(dynamique(&ImageGris::nouvelle(1, 1, 128)), (128, 128));
    }

    #[test]
    fn luminance_moyenne() {
        assert!((luminance(&img_test()) - 25.0).abs() < 1e-9);
    }

    #[test]
    fn contraste_min_max_formule() {
        // (40-10)/(40+10) = 30/50 = 0.6
        assert!((contraste_min_max(&img_test()) - 0.6).abs() < 1e-9);
    }

    #[test]
    fn histogramme_normalise_et_cumule() {
        let hn = histogramme_normalise(&img_test());
        assert!((hn[10] - 0.25).abs() < 1e-9);
        let c = histogramme_cumule(&img_test());
        assert!((c[10] - 0.25).abs() < 1e-9);
        assert!((c[40] - 1.0).abs() < 1e-9);
        assert!((c[255] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn negatif_inverse() {
        let n = negatif(&img_test());
        assert_eq!(n.donnees(), &[245, 235, 225, 215]);
    }

    #[test]
    fn transformation_lineaire_etire() {
        let t = transformation_lineaire(&img_test());
        assert_eq!(*t.get(0, 0).unwrap(), 0); // 10 -> 0
        assert_eq!(*t.get(1, 1).unwrap(), 255); // 40 -> 255
    }

    #[test]
    fn egalisation_sur_degrade() {
        let img = ImageGris::depuis_vec(4, 1, vec![0, 50, 100, 200]).unwrap();
        let e = egalisation_histogramme(&img);
        // La CDF étend les valeurs ; min et max doivent s'écarter
        let (min, max) = dynamique(&e);
        assert!(min < max);
    }

    #[test]
    fn histogramme_rvb_separe_les_canaux() {
        let img = ImageRvb::depuis_vec(
            1,
            2,
            vec![Rvb::new(10, 20, 30), Rvb::new(10, 40, 30)],
        )
        .unwrap();
        let [hr, hv, hb] = histogramme_rvb(&img);
        assert_eq!(hr[10], 2);
        assert_eq!(hv[20], 1);
        assert_eq!(hv[40], 1);
        assert_eq!(hb[30], 2);
    }

    #[test]
    fn saturation_rejette_bornes_invalides() {
        assert!(transformation_lineaire_saturation(&img_test(), 100, 50).is_err());
    }

    #[test]
    fn gamma_identite_approximative() {
        let img = ImageGris::nouvelle(2, 2, 100);
        let out = correction_gamma(&img, 1.0).unwrap();
        assert_eq!(out.donnees(), img.donnees());
    }
}
