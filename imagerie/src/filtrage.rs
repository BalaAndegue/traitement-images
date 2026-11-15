//! Chapitre 3 : convolution et filtrage spatial.
//!
//! Responsable : Nankeng Tsamo Pierre Marcelle.
//!
//! Convolution = corrélation pour tout noyau symétrique (moyenneur, gaussien).
//! Pour les noyaux de dérivée (Sobel, Prewitt, chapitre 5), on suit la
//! convention des transparents (pages 7 et 9) : corrélation.

use crate::alea::Alea;
use crate::erreur::{Erreur, Resultat};
use crate::image::{Bord, ImageGris, ImageReelle};

// ---------------------------------------------------------------------------
// Accès pixels — l'API de Image<T> renvoie Option<&T>.
// On garde les helpers petits pour ne pas polluer la logique des filtres.
// ---------------------------------------------------------------------------

#[inline]
fn lire_gris(img: &ImageGris, x: usize, y: usize) -> u8 {
    img.get(x, y).copied().unwrap_or(0)
}

#[inline]
fn ecrire_gris(img: &mut ImageGris, x: usize, y: usize, v: u8) {
    if let Some(p) = img.get_mut(x, y) {
        *p = v;
    }
}

#[inline]
fn lire_reel(img: &ImageReelle, x: usize, y: usize) -> f64 {
    img.get(x, y).copied().unwrap_or(0.0)
}

#[inline]
fn ecrire_reel(img: &mut ImageReelle, x: usize, y: usize, v: f64) {
    if let Some(p) = img.get_mut(x, y) {
        *p = v;
    }
}

// ---------------------------------------------------------------------------
// Noyau
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Noyau {
    largeur: usize,
    hauteur: usize,
    coefs: Vec<f64>,
}

impl Noyau {
    pub fn nouveau(largeur: usize, hauteur: usize, coefs: Vec<f64>) -> Resultat<Self> {
        if largeur.is_multiple_of(2) || hauteur.is_multiple_of(2) {
            return Err(Erreur::Parametre(format!(
                "noyau {largeur}x{hauteur} : la taille doit être impaire"
            )));
        }
        if coefs.len() != largeur * hauteur {
            return Err(Erreur::Dimensions(format!(
                "{} coefficients pour un noyau {largeur}x{hauteur}",
                coefs.len()
            )));
        }
        Ok(Noyau {
            largeur,
            hauteur,
            coefs,
        })
    }

    pub fn depuis_3x3(m: [[f64; 3]; 3]) -> Self {
        Noyau {
            largeur: 3,
            hauteur: 3,
            coefs: m.iter().flatten().copied().collect(),
        }
    }

    pub fn largeur(&self) -> usize {
        self.largeur
    }
    pub fn hauteur(&self) -> usize {
        self.hauteur
    }
    pub fn coef(&self, u: usize, v: usize) -> f64 {
        self.coefs[v * self.largeur + u]
    }
    pub fn coefs(&self) -> &[f64] {
        &self.coefs
    }
    pub fn somme(&self) -> f64 {
        self.coefs.iter().sum()
    }

    pub fn moyenneur(taille: usize) -> Resultat<Self> {
        if taille == 0 || taille.is_multiple_of(2) {
            return Err(Erreur::Parametre(format!(
                "Noyau::moyenneur : taille {taille} doit être impaire et > 0"
            )));
        }
        let n = (taille * taille) as f64;
        let coefs = vec![1.0 / n; taille * taille];
        Self::nouveau(taille, taille, coefs)
    }

    pub fn gaussien_cours() -> Self {
        let m = [
            [1.0, 2.0, 3.0, 2.0, 1.0],
            [2.0, 6.0, 8.0, 6.0, 2.0],
            [3.0, 8.0, 10.0, 8.0, 3.0],
            [2.0, 6.0, 8.0, 6.0, 2.0],
            [1.0, 2.0, 3.0, 2.0, 1.0],
        ];
        let coefs: Vec<f64> = m.iter().flatten().copied().collect();
        Noyau::nouveau(5, 5, coefs)
            .expect("gaussien_cours : 5x5 valide")
            .normalise()
    }

    pub fn gaussien(taille: usize, sigma: f64) -> Resultat<Self> {
        if taille == 0 || taille.is_multiple_of(2) {
            return Err(Erreur::Parametre(format!(
                "Noyau::gaussien : taille {taille} doit être impaire et > 0"
            )));
        }
        if sigma <= 0.0 {
            return Err(Erreur::Parametre(format!(
                "Noyau::gaussien : sigma {sigma} doit être > 0"
            )));
        }
        let c = (taille - 1) as f64 / 2.0;
        let mut coefs = Vec::with_capacity(taille * taille);
        let mut somme = 0.0;
        for v in 0..taille {
            for u in 0..taille {
                let x = u as f64 - c;
                let y = v as f64 - c;
                let w = (-(x * x + y * y) / (2.0 * sigma * sigma)).exp();
                coefs.push(w);
                somme += w;
            }
        }
        if somme <= 0.0 {
            return Err(Erreur::Parametre("Noyau::gaussien : somme nulle".into()));
        }
        for c in &mut coefs {
            *c /= somme;
        }
        Self::nouveau(taille, taille, coefs)
    }

    pub fn normalise(&self) -> Self {
        let s = self.somme();
        if s.abs() < 1e-12 {
            return self.clone();
        }
        let coefs = self.coefs.iter().map(|c| c / s).collect();
        Noyau {
            largeur: self.largeur,
            hauteur: self.hauteur,
            coefs,
        }
    }
}

// ---------------------------------------------------------------------------
// Gestion des bords
// ---------------------------------------------------------------------------

fn mirroir(pos: isize, n: isize) -> isize {
    if n <= 1 {
        return 0;
    }
    let period = 2 * (n - 1);
    let p = pos.rem_euclid(period);
    if p >= n {
        period - p
    } else {
        p
    }
}

fn lire_bord(sx: isize, sy: isize, l: isize, h: isize, bord: Bord) -> Option<(usize, usize)> {
    match bord {
        Bord::Zero | Bord::Partiel => {
            if sx >= 0 && sx < l && sy >= 0 && sy < h {
                Some((sx as usize, sy as usize))
            } else {
                None
            }
        }
        Bord::Miroir => Some((mirroir(sx, l) as usize, mirroir(sy, h) as usize)),
        Bord::Replique => Some((sx.clamp(0, l - 1) as usize, sy.clamp(0, h - 1) as usize)),
    }
}

// ---------------------------------------------------------------------------
// Convolution
// ---------------------------------------------------------------------------

pub fn convoluer(img: &ImageReelle, noyau: &Noyau, bord: Bord) -> ImageReelle {
    let (l, h) = (img.largeur(), img.hauteur());
    let mut res = ImageReelle::nouvelle(l, h, 0.0);
    let kw = noyau.largeur() as isize;
    let kh = noyau.hauteur() as isize;
    let dw = kw / 2;
    let dh = kh / 2;
    let partiel = matches!(bord, Bord::Partiel);

    for y in 0..h {
        for x in 0..l {
            let mut somme = 0.0f64;
            let mut somme_coefs = 0.0f64;
            for v in 0..kh {
                for u in 0..kw {
                    let sx = x as isize + u - dw;
                    let sy = y as isize + v - dh;
                    if let Some((px, py)) = lire_bord(sx, sy, l as isize, h as isize, bord) {
                        let c = noyau.coef(u as usize, v as usize);
                        somme += lire_reel(img, px, py) * c;
                        somme_coefs += c;
                    }
                }
            }
            let val = if partiel && somme_coefs.abs() > 1e-12 {
                somme / somme_coefs
            } else {
                somme
            };
            ecrire_reel(&mut res, x, y, val);
        }
    }
    res
}

pub fn convoluer_gris(img: &ImageGris, noyau: &Noyau, bord: Bord) -> ImageGris {
    let (l, h) = (img.largeur(), img.hauteur());
    let mut res = ImageGris::nouvelle(l, h, 0);
    let kw = noyau.largeur() as isize;
    let kh = noyau.hauteur() as isize;
    let dw = kw / 2;
    let dh = kh / 2;
    let partiel = matches!(bord, Bord::Partiel);

    for y in 0..h {
        for x in 0..l {
            let mut somme = 0.0f64;
            let mut somme_coefs = 0.0f64;
            for v in 0..kh {
                for u in 0..kw {
                    let sx = x as isize + u - dw;
                    let sy = y as isize + v - dh;
                    if let Some((px, py)) = lire_bord(sx, sy, l as isize, h as isize, bord) {
                        let c = noyau.coef(u as usize, v as usize);
                        somme += lire_gris(img, px, py) as f64 * c;
                        somme_coefs += c;
                    }
                }
            }
            let val = if partiel && somme_coefs.abs() > 1e-12 {
                somme / somme_coefs
            } else {
                somme
            };
            ecrire_gris(&mut res, x, y, val.round().clamp(0.0, 255.0) as u8);
        }
    }
    res
}

// ---------------------------------------------------------------------------
// Filtres linéaires
// ---------------------------------------------------------------------------

pub fn filtre_moyenneur(img: &ImageGris, taille: usize, bord: Bord) -> Resultat<ImageGris> {
    let noyau = Noyau::moyenneur(taille)?;
    Ok(convoluer_gris(img, &noyau, bord))
}

pub fn filtre_gaussien(
    img: &ImageGris,
    taille: usize,
    sigma: f64,
    bord: Bord,
) -> Resultat<ImageGris> {
    let noyau = Noyau::gaussien(taille, sigma)?;
    Ok(convoluer_gris(img, &noyau, bord))
}

pub fn filtre_rehausseur(img: &ImageGris, bord: Bord) -> ImageGris {
    let noyau = Noyau::depuis_3x3([[0.0, -1.0, 0.0], [-1.0, 5.0, -1.0], [0.0, -1.0, 0.0]]);
    convoluer_gris(img, &noyau, bord)
}

// ---------------------------------------------------------------------------
// Filtres non linéaires
// ---------------------------------------------------------------------------

pub fn filtre_median(img: &ImageGris, taille: usize, bord: Bord) -> Resultat<ImageGris> {
    if taille == 0 || taille.is_multiple_of(2) {
        return Err(Erreur::Parametre(format!(
            "filtre_median : taille {taille} doit être impaire et > 0"
        )));
    }
    let (l, h) = (img.largeur(), img.hauteur());
    let mut res = ImageGris::nouvelle(l, h, 0);
    let d = (taille / 2) as isize;
    for y in 0..h {
        for x in 0..l {
            let mut voisins = Vec::with_capacity(taille * taille);
            for v in 0..taille as isize {
                for u in 0..taille as isize {
                    let sx = x as isize - d + u;
                    let sy = y as isize - d + v;
                    match lire_bord(sx, sy, l as isize, h as isize, bord) {
                        Some((px, py)) => voisins.push(lire_gris(img, px, py)),
                        None if matches!(bord, Bord::Zero) => voisins.push(0),
                        None => {}
                    }
                }
            }
            voisins.sort_unstable();
            ecrire_gris(&mut res, x, y, voisins[voisins.len() / 2]);
        }
    }
    Ok(res)
}

pub fn filtre_min(img: &ImageGris, taille: usize, bord: Bord) -> Resultat<ImageGris> {
    if taille == 0 || taille.is_multiple_of(2) {
        return Err(Erreur::Parametre(format!(
            "filtre_min : taille {taille} doit être impaire et > 0"
        )));
    }
    let (l, h) = (img.largeur(), img.hauteur());
    let mut res = ImageGris::nouvelle(l, h, 0);
    let d = (taille / 2) as isize;
    for y in 0..h {
        for x in 0..l {
            let mut m = u8::MAX;
            for v in 0..taille as isize {
                for u in 0..taille as isize {
                    let sx = x as isize - d + u;
                    let sy = y as isize - d + v;
                    match lire_bord(sx, sy, l as isize, h as isize, bord) {
                        Some((px, py)) => m = m.min(lire_gris(img, px, py)),
                        None if matches!(bord, Bord::Zero) => m = 0,
                        None => {}
                    }
                }
            }
            ecrire_gris(&mut res, x, y, m);
        }
    }
    Ok(res)
}

pub fn filtre_max(img: &ImageGris, taille: usize, bord: Bord) -> Resultat<ImageGris> {
    if taille == 0 || taille.is_multiple_of(2) {
        return Err(Erreur::Parametre(format!(
            "filtre_max : taille {taille} doit être impaire et > 0"
        )));
    }
    let (l, h) = (img.largeur(), img.hauteur());
    let mut res = ImageGris::nouvelle(l, h, 0);
    let d = (taille / 2) as isize;
    for y in 0..h {
        for x in 0..l {
            let mut m = 0u8;
            for v in 0..taille as isize {
                for u in 0..taille as isize {
                    let sx = x as isize - d + u;
                    let sy = y as isize - d + v;
                    if let Some((px, py)) = lire_bord(sx, sy, l as isize, h as isize, bord) {
                        m = m.max(lire_gris(img, px, py));
                    }
                }
            }
            ecrire_gris(&mut res, x, y, m);
        }
    }
    Ok(res)
}

// ---------------------------------------------------------------------------
// Bruit
// ---------------------------------------------------------------------------

pub fn bruit_poivre_sel(img: &ImageGris, proportion: f64, graine: u64) -> Resultat<ImageGris> {
    if !(0.0..=1.0).contains(&proportion) {
        return Err(Erreur::Parametre(format!(
            "bruit_poivre_sel : proportion {proportion} doit être dans [0, 1]"
        )));
    }
    let (l, h) = (img.largeur(), img.hauteur());
    let mut res = img.clone();
    let mut alea = Alea::new(graine);
    let nb = (proportion * (l * h) as f64).round() as usize;
    for _ in 0..nb {
        let x = alea.entier(l);
        let y = alea.entier(h);
        let v = if alea.uniforme() < 0.5 { 0 } else { 255 };
        ecrire_gris(&mut res, x, y, v);
    }
    Ok(res)
}

pub fn bruit_gaussien(img: &ImageGris, sigma: f64, graine: u64) -> ImageGris {
    let (l, h) = (img.largeur(), img.hauteur());
    let mut res = ImageGris::nouvelle(l, h, 0);
    let mut alea = Alea::new(graine);
    for y in 0..h {
        for x in 0..l {
            let bruit = alea.gaussien() * sigma;
            let v = (lire_gris(img, x, y) as f64 + bruit)
                .round()
                .clamp(0.0, 255.0) as u8;
            ecrire_gris(&mut res, x, y, v);
        }
    }
    res
}

// ---------------------------------------------------------------------------
// Tests unitaires
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn img_5x5() -> ImageGris {
        let mut img = ImageGris::nouvelle(5, 5, 0);
        let vals = [
            10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180, 190,
            200, 210, 220, 230, 240, 250,
        ];
        for (i, &v) in vals.iter().enumerate() {
            ecrire_gris(&mut img, i % 5, i / 5, v);
        }
        img
    }

    #[test]
    fn moyenneur_somme_1() {
        let n = Noyau::moyenneur(3).unwrap();
        assert_eq!(n.largeur(), 3);
        assert!((n.somme() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn moyenneur_refuse_pair() {
        assert!(Noyau::moyenneur(4).is_err());
        assert!(Noyau::moyenneur(0).is_err());
    }

    #[test]
    fn gaussien_cours_normalise() {
        let n = Noyau::gaussien_cours();
        assert_eq!(n.largeur(), 5);
        assert!((n.somme() - 1.0).abs() < 1e-9);
        assert!((n.coef(2, 2) - 10.0 / 98.0).abs() < 1e-9);
    }

    #[test]
    fn gaussien_symetrique_et_normalise() {
        let n = Noyau::gaussien(5, 1.0).unwrap();
        assert!((n.coef(0, 0) - n.coef(4, 4)).abs() < 1e-12);
        assert!((n.somme() - 1.0).abs() < 1e-9);
        assert!(n.coef(2, 2) > n.coef(0, 0));
    }

    #[test]
    fn convolution_identite() {
        let img = img_5x5();
        let id = Noyau::depuis_3x3([[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]]);
        let res = convoluer_gris(&img, &id, Bord::Zero);
        for y in 1..4 {
            for x in 1..4 {
                assert_eq!(lire_gris(&res, x, y), lire_gris(&img, x, y));
            }
        }
    }

    #[test]
    fn moyenneur_reduit_variance() {
        let mut img = ImageGris::nouvelle(6, 6, 0);
        for y in 0..6 {
            for x in 0..6 {
                ecrire_gris(&mut img, x, y, if (x + y) % 2 == 0 { 0 } else { 200 });
            }
        }
        let res = filtre_moyenneur(&img, 3, Bord::Replique).unwrap();
        let c = lire_gris(&res, 3, 3) as i32;
        assert!((c - 100).abs() < 25, "centre {c} trop loin de 100");
    }

    #[test]
    fn median_supprime_pixel_isole() {
        let mut img = ImageGris::nouvelle(3, 3, 0);
        for y in 0..3 {
            for x in 0..3 {
                ecrire_gris(&mut img, x, y, 10);
            }
        }
        ecrire_gris(&mut img, 1, 1, 250);
        let res = filtre_median(&img, 3, Bord::Replique).unwrap();
        assert_eq!(lire_gris(&res, 1, 1), 10);
    }

    #[test]
    fn min_et_max() {
        let mut img = ImageGris::nouvelle(3, 3, 0);
        for y in 0..3 {
            for x in 0..3 {
                ecrire_gris(&mut img, x, y, 50);
            }
        }
        ecrire_gris(&mut img, 1, 1, 0);
        assert_eq!(
            lire_gris(&filtre_min(&img, 3, Bord::Replique).unwrap(), 1, 1),
            0
        );
        assert_eq!(
            lire_gris(&filtre_max(&img, 3, Bord::Replique).unwrap(), 0, 0),
            50
        );
    }

    #[test]
    fn median_refuse_taille_paire() {
        let img = img_5x5();
        assert!(filtre_median(&img, 4, Bord::Replique).is_err());
    }

    #[test]
    fn poivre_sel_change_des_pixels() {
        let img = ImageGris::nouvelle(20, 20, 0);
        let b = bruit_poivre_sel(&img, 0.5, 42).unwrap();
        let mut n255 = 0;
        for y in 0..20 {
            for x in 0..20 {
                if lire_gris(&b, x, y) == 255 {
                    n255 += 1;
                }
            }
        }
        assert!(n255 > 0);
    }

    #[test]
    fn poivre_sel_proportion_invalide() {
        let img = img_5x5();
        assert!(bruit_poivre_sel(&img, 1.5, 0).is_err());
        assert!(bruit_poivre_sel(&img, -0.1, 0).is_err());
    }

    #[test]
    fn gaussien_moyenne_proche() {
        let mut img = ImageGris::nouvelle(50, 50, 0);
        for y in 0..50 {
            for x in 0..50 {
                ecrire_gris(&mut img, x, y, 128);
            }
        }
        let b = bruit_gaussien(&img, 5.0, 7);
        let mut somme: u64 = 0;
        for y in 0..50 {
            for x in 0..50 {
                somme += lire_gris(&b, x, y) as u64;
            }
        }
        let moyenne = somme as f64 / 2500.0;
        assert!((moyenne - 128.0).abs() < 3.0);
    }
}
