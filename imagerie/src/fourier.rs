//! Chapitre 4 : transformée de Fourier et filtrage fréquentiel.
//!
//! Responsable : Sountsa Djiele Pio Vianney.
//!
//! La FFT doit être codée par nos soins (algorithme radix-2 des Numerical Recipes,
//! chapitre 12), sans bibliothèque externe.

use std::f64::consts::PI;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use crate::erreur::{Erreur, Resultat};
use crate::image::{Image, ImageGris, ImageReelle};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complexe {
    pub re: f64,
    pub im: f64,
}

impl Complexe {
    pub const fn new(re: f64, im: f64) -> Self {
        Complexe { re, im }
    }

    /// Norme |z| = sqrt(re² + im²).
    pub fn norme(self) -> f64 {
        self.re.hypot(self.im)
    }

    /// Argument (phase) de z.
    pub fn phase(self) -> f64 {
        self.im.atan2(self.re)
    }

    /// e^(i theta).
    pub fn polaire(theta: f64) -> Self {
        let (sin, cos) = theta.sin_cos();
        Complexe::new(cos, sin)
    }

    pub fn conjugue(self) -> Self {
        Complexe::new(self.re, -self.im)
    }
}

impl Add for Complexe {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Complexe::new(self.re + rhs.re, self.im + rhs.im)
    }
}

impl AddAssign for Complexe {
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl Sub for Complexe {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Complexe::new(self.re - rhs.re, self.im - rhs.im)
    }
}

impl SubAssign for Complexe {
    fn sub_assign(&mut self, rhs: Self) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}

impl Mul for Complexe {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Complexe::new(
            self.re * rhs.re - self.im * rhs.im,
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}

impl Mul<f64> for Complexe {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Complexe::new(self.re * rhs, self.im * rhs)
    }
}

impl Mul<Complexe> for f64 {
    type Output = Complexe;
    fn mul(self, rhs: Complexe) -> Complexe {
        Complexe::new(self * rhs.re, self * rhs.im)
    }
}

impl Div<f64> for Complexe {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Complexe::new(self.re / rhs, self.im / rhs)
    }
}

impl Neg for Complexe {
    type Output = Self;
    fn neg(self) -> Self {
        Complexe::new(-self.re, -self.im)
    }
}

/// Spectre : image de nombres complexes, de côtés puissances de 2.
pub type Spectre = Image<Complexe>;

/// FFT 1D en place (taille puissance de 2). `inverse` : FFT⁻¹ avec division par n.
///
/// Algorithme radix-2 de Cooley-Tukey (Numerical Recipes in C, chapitre 12).
pub fn fft_1d(donnees: &mut [Complexe], inverse: bool) -> Resultat<()> {
    let n = donnees.len();
    if n <= 1 {
        return Ok(());
    }
    if !n.is_power_of_two() {
        return Err(Erreur::Parametre(format!(
            "la taille ({n}) n'est pas une puissance de 2"
        )));
    }

    // 1. Inversion de bits (bit-reversal permutation)
    let mut j = 0;
    for i in 0..n {
        if i < j {
            donnees.swap(i, j);
        }
        let mut m = n >> 1;
        while m >= 1 && j >= m {
            j -= m;
            m >>= 1;
        }
        j += m;
    }

    // 2. Papillons de Cooley-Tukey (Danielson-Lanczos)
    let mut len = 2;
    while len <= n {
        let demi = len / 2;
        let sign = if inverse { 1.0 } else { -1.0 };
        let theta = sign * 2.0 * PI / (len as f64);
        let w_pas = Complexe::polaire(theta);

        let mut i = 0;
        while i < n {
            let mut w = Complexe::new(1.0, 0.0);
            for k in 0..demi {
                let u = donnees[i + k];
                let v = donnees[i + k + demi] * w;
                donnees[i + k] = u + v;
                donnees[i + k + demi] = u - v;
                w = w * w_pas;
            }
            i += len;
        }
        len <<= 1;
    }

    // 3. Normalisation si transformée inverse
    if inverse {
        let inv_n = 1.0 / (n as f64);
        for x in donnees.iter_mut() {
            *x = *x * inv_n;
        }
    }

    Ok(())
}

/// Pré-traitement : complète avec des zéros jusqu'à des côtés puissances de 2.
pub fn completer_puissance_de_2(img: &ImageReelle) -> ImageReelle {
    let w = img.largeur().max(1).next_power_of_two();
    let h = img.hauteur().max(1).next_power_of_two();

    if w == img.largeur() && h == img.hauteur() {
        return img.clone();
    }

    let mut res = ImageReelle::nouvelle(w, h, 0.0);
    for y in 0..img.hauteur() {
        for x in 0..img.largeur() {
            res[(x, y)] = img[(x, y)];
        }
    }
    res
}

/// FFT 2D (lignes puis colonnes) d'une image en niveaux de gris.
pub fn fft_2d(img: &ImageGris) -> Spectre {
    let reelle = img.vers_reelle();
    let reelle_pad = completer_puissance_de_2(&reelle);
    let (w, h) = reelle_pad.dimensions();

    let mut spectre = Spectre::depuis_fonction(w, h, |x, y| {
        Complexe::new(reelle_pad[(x, y)], 0.0)
    });

    // 1. FFT 1D sur chaque ligne
    for y in 0..h {
        let debut = y * w;
        let fin = debut + w;
        let ligne = &mut spectre.donnees_mut()[debut..fin];
        let _ = fft_1d(ligne, false);
    }

    // 2. FFT 1D sur chaque colonne
    let mut col = vec![Complexe::default(); h];
    for x in 0..w {
        for y in 0..h {
            col[y] = spectre[(x, y)];
        }
        let _ = fft_1d(&mut col, false);
        for y in 0..h {
            spectre[(x, y)] = col[y];
        }
    }

    spectre
}

/// FFT 2D inverse ; on ne garde que la partie réelle, recadrée à la taille voulue.
pub fn fft_2d_inverse(spectre: &Spectre, largeur: usize, hauteur: usize) -> ImageReelle {
    let mut buffer = spectre.clone();
    let (w, h) = buffer.dimensions();

    // 1. IFFT 1D sur chaque colonne
    let mut col = vec![Complexe::default(); h];
    for x in 0..w {
        for y in 0..h {
            col[y] = buffer[(x, y)];
        }
        let _ = fft_1d(&mut col, true);
        for y in 0..h {
            buffer[(x, y)] = col[y];
        }
    }

    // 2. IFFT 1D sur chaque ligne
    for y in 0..h {
        let debut = y * w;
        let fin = debut + w;
        let ligne = &mut buffer.donnees_mut()[debut..fin];
        let _ = fft_1d(ligne, true);
    }

    // 3. Recadrage à la taille d'origine (largeur, hauteur) et partie réelle
    ImageReelle::depuis_fonction(largeur, hauteur, |x, y| {
        if x < w && y < h {
            buffer[(x, y)].re
        } else {
            0.0
        }
    })
}

/// Inversion des quadrants : met la fréquence nulle (composante continue) au centre.
/// Appliquée deux fois, elle rend le spectre d'origine.
pub fn permuter_quadrants<T: Clone>(img: &Image<T>) -> Image<T> {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return img.clone();
    }
    let cx = w / 2;
    let cy = h / 2;
    Image::depuis_fonction(w, h, |x, y| {
        let src_x = (x + cx) % w;
        let src_y = (y + cy) % h;
        img[(src_x, src_y)].clone()
    })
}

/// |F(u, v)|.
pub fn module(spectre: &Spectre) -> ImageReelle {
    spectre.map(|c| c.norme())
}

/// Phase de F(u, v).
pub fn phase(spectre: &Spectre) -> ImageReelle {
    spectre.map(|c| c.phase())
}

pub fn partie_reelle(spectre: &Spectre) -> ImageReelle {
    spectre.map(|c| c.re)
}

pub fn partie_imaginaire(spectre: &Spectre) -> ImageReelle {
    spectre.map(|c| c.im)
}

/// Spectre rehaussé pour l'affichage : log(1 + |F(u, v)|), centré et ramené sur [0, 255].
pub fn spectre_affichable(spectre: &Spectre) -> ImageGris {
    let centre = permuter_quadrants(spectre);
    let log_spectre = centre.map(|c| (1.0 + c.norme()).ln());
    log_spectre.vers_gris_normalise()
}

/// Détermine si un spectre est déjà centré ou non, et renvoie une version centrée.
fn assurer_spectre_centre(spectre: &Spectre) -> Spectre {
    let (w, h) = spectre.dimensions();
    if w == 0 || h == 0 {
        return spectre.clone();
    }
    if spectre[(0, 0)].norme() > spectre[(w / 2, h / 2)].norme() {
        permuter_quadrants(spectre)
    } else {
        spectre.clone()
    }
}

/// Pourcentage de l'énergie contenue dans un disque de rayon `rayon` autour du centre.
pub fn pourcentage_energie(spectre: &Spectre, rayon: f64) -> f64 {
    let (w, h) = spectre.dimensions();
    if w == 0 || h == 0 {
        return 0.0;
    }
    let centre = assurer_spectre_centre(spectre);
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;

    let mut energie_disque = 0.0;
    let mut energie_totale = 0.0;

    for y in 0..h {
        for x in 0..w {
            let n = centre[(x, y)].norme();
            let e = n * n;
            energie_totale += e;
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            if dx.hypot(dy) <= rayon {
                energie_disque += e;
            }
        }
    }

    if energie_totale <= 0.0 {
        0.0
    } else {
        (energie_disque / energie_totale) * 100.0
    }
}

/// Plus petit rayon contenant `pourcentage` de l'énergie (90 %, 95 %, 98 %... du cours).
pub fn rayon_pour_energie(spectre: &Spectre, pourcentage: f64) -> f64 {
    let (w, h) = spectre.dimensions();
    if w == 0 || h == 0 {
        return 0.0;
    }
    let centre = assurer_spectre_centre(spectre);
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;

    let fraction_cible = if pourcentage > 1.0 {
        (pourcentage / 100.0).clamp(0.0, 1.0)
    } else {
        pourcentage.clamp(0.0, 1.0)
    };

    let mut pixels: Vec<(f64, f64)> = Vec::with_capacity(w * h);
    let mut energie_totale = 0.0;

    for y in 0..h {
        for x in 0..w {
            let n = centre[(x, y)].norme();
            let e = n * n;
            energie_totale += e;
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            pixels.push((dx.hypot(dy), e));
        }
    }

    if energie_totale <= 0.0 {
        return 0.0;
    }

    pixels.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let seuil_energie = fraction_cible * energie_totale;
    let mut cumul = 0.0;
    for (dist, e) in pixels {
        cumul += e;
        if cumul >= seuil_energie {
            return dist;
        }
    }

    cx.hypot(cy)
}

/// Applique une fonction de transfert fréquentielle $H(\text{dist}, x, y)$ à une image.
fn appliquer_filtre<F>(img: &ImageGris, mut masque: F) -> ImageReelle
where
    F: FnMut(f64, usize, usize) -> f64,
{
    let spectre = fft_2d(img);
    let mut centre = permuter_quadrants(&spectre);
    let (w, h) = centre.dimensions();
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;

    for y in 0..h {
        for x in 0..w {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let dist = dx.hypot(dy);
            let gain = masque(dist, x, y);
            centre[(x, y)] = centre[(x, y)] * gain;
        }
    }

    let decentre = permuter_quadrants(&centre);
    fft_2d_inverse(&decentre, img.largeur(), img.hauteur())
}

/// Passe-bas idéal : on met à zéro les fréquences au-delà de `rayon`.
pub fn passe_bas(img: &ImageGris, rayon: f64) -> ImageGris {
    let reelle = appliquer_filtre(img, |dist, _, _| {
        if dist <= rayon {
            1.0
        } else {
            0.0
        }
    });
    reelle.vers_gris_sature()
}

/// Passe-haut idéal : on met à zéro les fréquences en deçà de `rayon`.
pub fn passe_haut(img: &ImageGris, rayon: f64) -> ImageGris {
    let reelle = appliquer_filtre(img, |dist, _, _| {
        if dist >= rayon {
            1.0
        } else {
            0.0
        }
    });
    reelle.vers_gris_sature()
}

/// Passe-bande : on garde les fréquences entre `rayon_min` et `rayon_max`.
pub fn passe_bande(img: &ImageGris, rayon_min: f64, rayon_max: f64) -> Resultat<ImageGris> {
    if rayon_min < 0.0 || rayon_min > rayon_max {
        return Err(Erreur::Parametre(format!(
            "rayons invalides : rayon_min ({rayon_min}) > rayon_max ({rayon_max})"
        )));
    }
    let reelle = appliquer_filtre(img, |dist, _, _| {
        if dist >= rayon_min && dist <= rayon_max {
            1.0
        } else {
            0.0
        }
    });
    Ok(reelle.vers_gris_sature())
}

/// Filtre « notch » contre le bruit périodique : on annule de petits disques de
/// rayon `rayon` autour des pics donnés (et de leurs symétriques).
pub fn coupe_pics(img: &ImageGris, pics: &[(usize, usize)], rayon: f64) -> ImageGris {
    let spectre = fft_2d(img);
    let mut centre = permuter_quadrants(&spectre);
    let (w, h) = centre.dimensions();
    let cx = w / 2;
    let cy = h / 2;

    for y in 0..h {
        for x in 0..w {
            let mut annuler = false;
            for &(px, py) in pics {
                let d1 = (x as f64 - px as f64).hypot(y as f64 - py as f64);
                let sym_x = (2 * cx).saturating_sub(px);
                let sym_y = (2 * cy).saturating_sub(py);
                let d2 = (x as f64 - sym_x as f64).hypot(y as f64 - sym_y as f64);

                if d1 <= rayon || d2 <= rayon {
                    annuler = true;
                    break;
                }
            }
            if annuler {
                centre[(x, y)] = Complexe::new(0.0, 0.0);
            }
        }
    }

    let decentre = permuter_quadrants(&centre);
    let reelle = fft_2d_inverse(&decentre, img.largeur(), img.hauteur());
    reelle.vers_gris_sature()
}

/// Rehaussement de contraste : image + k x (image filtrée passe-haut).
pub fn rehaussement(img: &ImageGris, rayon: f64, k: f64) -> ImageGris {
    let hp = appliquer_filtre(img, |dist, _, _| {
        if dist >= rayon {
            1.0
        } else {
            0.0
        }
    });
    ImageGris::depuis_fonction(img.largeur(), img.hauteur(), |x, y| {
        let v = (img[(x, y)] as f64) + k * hp[(x, y)];
        v.round().clamp(0.0, 255.0) as u8
    })
}

/// Images de test du cours : sinusoïde de période `periode` pixels orientée de `angle` degrés.
pub fn creer_sinus(largeur: usize, hauteur: usize, periode: f64, angle_degres: f64) -> ImageGris {
    let rad = angle_degres.to_radians();
    let (sin_a, cos_a) = rad.sin_cos();
    let omega = if periode > 0.0 {
        2.0 * PI / periode
    } else {
        0.0
    };

    Image::depuis_fonction(largeur, hauteur, |x, y| {
        let proj = (x as f64) * cos_a + (y as f64) * sin_a;
        let v = 127.5 + 127.5 * (omega * proj).sin();
        v.round().clamp(0.0, 255.0) as u8
    })
}

/// Rectangle blanc centré sur fond noir.
pub fn creer_rectangle(
    largeur: usize,
    hauteur: usize,
    larg_rect: usize,
    haut_rect: usize,
) -> ImageGris {
    let x0 = largeur.saturating_sub(larg_rect) / 2;
    let y0 = hauteur.saturating_sub(haut_rect) / 2;
    let x1 = x0 + larg_rect;
    let y1 = y0 + haut_rect;

    Image::depuis_fonction(largeur, hauteur, |x, y| {
        if x >= x0 && x < x1 && y >= y0 && y < y1 {
            255
        } else {
            0
        }
    })
}

/// Signal carré (bandes noires et blanches) de période `periode`.
pub fn creer_signal_carre(largeur: usize, hauteur: usize, periode: usize) -> ImageGris {
    if periode == 0 {
        return Image::nouvelle(largeur, hauteur, 0);
    }
    let demi = periode / 2;
    Image::depuis_fonction(largeur, hauteur, |x, _| {
        if (x % periode) < demi {
            255
        } else {
            0
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};

    #[test]
    fn complexe_operations_de_base() {
        let z = Complexe::new(3.0, 4.0);
        assert!((z.norme() - 5.0).abs() < 1e-12);
        assert_eq!(z.conjugue(), Complexe::new(3.0, -4.0));

        let z_reel = Complexe::new(2.0, 0.0);
        assert!((z_reel.phase() - 0.0).abs() < 1e-12);

        let z_imag = Complexe::new(0.0, 2.0);
        assert!((z_imag.phase() - FRAC_PI_2).abs() < 1e-12);

        let p0 = Complexe::polaire(0.0);
        assert!((p0.re - 1.0).abs() < 1e-12 && p0.im.abs() < 1e-12);

        let p_pi = Complexe::polaire(PI);
        assert!((p_pi.re - (-1.0)).abs() < 1e-12 && p_pi.im.abs() < 1e-12);

        // Opérateurs arithmétiques
        let a = Complexe::new(1.0, 2.0);
        let b = Complexe::new(3.0, 4.0);
        assert_eq!(a + b, Complexe::new(4.0, 6.0));
        assert_eq!(a - b, Complexe::new(-2.0, -2.0));
        assert_eq!(a * b, Complexe::new(-5.0, 10.0));
        assert_eq!(a * 2.0, Complexe::new(2.0, 4.0));
        assert_eq!(2.0 * a, Complexe::new(2.0, 4.0));
        assert_eq!(a / 2.0, Complexe::new(0.5, 1.0));
        assert_eq!(-a, Complexe::new(-1.0, -2.0));
    }

    #[test]
    fn fft_1d_aller_retour() {
        let mut signal = vec![
            Complexe::new(1.0, 0.0),
            Complexe::new(2.0, 0.0),
            Complexe::new(3.0, 0.0),
            Complexe::new(4.0, 0.0),
            Complexe::new(5.0, 0.0),
            Complexe::new(6.0, 0.0),
            Complexe::new(7.0, 0.0),
            Complexe::new(8.0, 0.0),
        ];
        let original = signal.clone();

        fft_1d(&mut signal, false).unwrap();
        fft_1d(&mut signal, true).unwrap();

        for (orig, rec) in original.iter().zip(&signal) {
            assert!((orig.re - rec.re).abs() < 1e-10);
            assert!(rec.im.abs() < 1e-10);
        }
    }

    #[test]
    fn fft_2d_aller_retour() {
        let img = ImageGris::depuis_fonction(8, 8, |x, y| ((x * 10 + y * 20) % 256) as u8);
        let spectre = fft_2d(&img);
        let rec = fft_2d_inverse(&spectre, 8, 8);

        for y in 0..8 {
            for x in 0..8 {
                let diff = (img[(x, y)] as f64 - rec[(x, y)]).abs();
                assert!(diff < 1e-9, "diff = {diff} at ({x}, {y})");
            }
        }
    }

    #[test]
    fn permuter_quadrants_involutif() {
        let img = ImageGris::depuis_fonction(8, 8, |x, y| (x + y * 8) as u8);
        let perm = permuter_quadrants(&img);
        let perm_perm = permuter_quadrants(&perm);
        assert_eq!(img, perm_perm);
    }

    #[test]
    fn energie_disque_et_rayon() {
        let img = ImageGris::nouvelle(8, 8, 100);
        let spectre = fft_2d(&img);

        let r95 = rayon_pour_energie(&spectre, 95.0);
        let pct = pourcentage_energie(&spectre, r95);
        assert!(pct >= 95.0);
    }

    #[test]
    fn filtrage_passe_bas_uniforme() {
        let img = ImageGris::nouvelle(8, 8, 128);
        let pb = passe_bas(&img, 10.0);
        assert_eq!(pb[(4, 4)], 128);
    }
}
