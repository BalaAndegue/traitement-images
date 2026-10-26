//! Types d'images communs à tous les modules.
//!
//! Une image est une matrice `largeur x hauteur` stockée ligne par ligne.
//! L'origine (0, 0) est le coin haut-gauche, `x` est la colonne et `y` la ligne.

use std::ops::{Index, IndexMut};

use crate::erreur::{Erreur, Resultat};

#[derive(Clone, Debug, PartialEq)]
pub struct Image<T> {
    largeur: usize,
    hauteur: usize,
    donnees: Vec<T>,
}

/// Pixel couleur dans l'espace RVB, 8 bits par canal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rvb {
    pub r: u8,
    pub v: u8,
    pub b: u8,
}

impl Rvb {
    pub const fn new(r: u8, v: u8, b: u8) -> Self {
        Rvb { r, v, b }
    }
}

/// Niveaux de gris 8 bits : 0 = noir, 255 = blanc.
pub type ImageGris = Image<u8>;
/// Image couleur 24 bits.
pub type ImageRvb = Image<Rvb>;
/// Image à valeurs réelles (résultats de convolution, gradient, Fourier...).
pub type ImageReelle = Image<f64>;
/// Image binaire : `true` = objet (1), `false` = fond (0).
pub type ImageBinaire = Image<bool>;
/// Image d'étiquettes : 0 = fond, 1..n = numéro de région.
pub type ImageEtiquettes = Image<u32>;

/// Gestion des pixels situés hors de l'image (convolution, médian, morphologie...).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Bord {
    /// Les pixels hors image valent zéro.
    #[default]
    Zero,
    /// Miroir de l'image : f(-x, y) = f(x, y).
    Miroir,
    /// On recopie le pixel du bord le plus proche.
    Replique,
    /// Convolution partielle : on ignore la partie du noyau qui sort de l'image.
    Partiel,
}

/// Voisinage utilisé pour la connexité des pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Connexite {
    /// N4 : haut, bas, gauche, droite.
    #[default]
    Quatre,
    /// N8 : N4 + les 4 diagonales.
    Huit,
}

impl Connexite {
    /// Décalages (dx, dy) des voisins.
    pub fn voisins(self) -> &'static [(isize, isize)] {
        const N4: [(isize, isize); 4] = [(1, 0), (0, -1), (-1, 0), (0, 1)];
        const N8: [(isize, isize); 8] = [
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        match self {
            Connexite::Quatre => &N4,
            Connexite::Huit => &N8,
        }
    }
}

impl<T: Clone> Image<T> {
    /// Image remplie avec une valeur constante.
    pub fn nouvelle(largeur: usize, hauteur: usize, valeur: T) -> Self {
        Image {
            largeur,
            hauteur,
            donnees: vec![valeur; largeur * hauteur],
        }
    }
}

impl<T> Image<T> {
    /// Construit une image à partir de ses pixels rangés ligne par ligne.
    pub fn depuis_vec(largeur: usize, hauteur: usize, donnees: Vec<T>) -> Resultat<Self> {
        if donnees.len() != largeur * hauteur {
            return Err(Erreur::Dimensions(format!(
                "{} pixels fournis pour une image {largeur}x{hauteur}",
                donnees.len()
            )));
        }
        Ok(Image {
            largeur,
            hauteur,
            donnees,
        })
    }

    /// Construit une image en calculant chaque pixel avec `f(x, y)`.
    pub fn depuis_fonction(
        largeur: usize,
        hauteur: usize,
        mut f: impl FnMut(usize, usize) -> T,
    ) -> Self {
        let mut donnees = Vec::with_capacity(largeur * hauteur);
        for y in 0..hauteur {
            for x in 0..largeur {
                donnees.push(f(x, y));
            }
        }
        Image {
            largeur,
            hauteur,
            donnees,
        }
    }

    pub fn largeur(&self) -> usize {
        self.largeur
    }

    pub fn hauteur(&self) -> usize {
        self.hauteur
    }

    /// (largeur, hauteur)
    pub fn dimensions(&self) -> (usize, usize) {
        (self.largeur, self.hauteur)
    }

    pub fn nb_pixels(&self) -> usize {
        self.donnees.len()
    }

    pub fn meme_dimensions<U>(&self, autre: &Image<U>) -> bool {
        self.dimensions() == autre.dimensions()
    }

    pub fn donnees(&self) -> &[T] {
        &self.donnees
    }

    pub fn donnees_mut(&mut self) -> &mut [T] {
        &mut self.donnees
    }

    pub fn into_donnees(self) -> Vec<T> {
        self.donnees
    }

    /// Vrai si (x, y) est à l'intérieur de l'image (coordonnées signées).
    pub fn contient(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.largeur && (y as usize) < self.hauteur
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x < self.largeur && y < self.hauteur {
            Some(&self.donnees[y * self.largeur + x])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x < self.largeur && y < self.hauteur {
            Some(&mut self.donnees[y * self.largeur + x])
        } else {
            None
        }
    }

    /// Ligne `y` de l'image.
    pub fn ligne(&self, y: usize) -> &[T] {
        &self.donnees[y * self.largeur..(y + 1) * self.largeur]
    }

    /// Parcourt les pixels avec leurs coordonnées : (x, y, &pixel).
    pub fn pixels(&self) -> impl Iterator<Item = (usize, usize, &T)> {
        let l = self.largeur.max(1);
        self.donnees
            .iter()
            .enumerate()
            .map(move |(i, p)| (i % l, i / l, p))
    }

    /// Applique `f` à chaque pixel (transformation ponctuelle).
    pub fn map<U>(&self, f: impl FnMut(&T) -> U) -> Image<U> {
        Image {
            largeur: self.largeur,
            hauteur: self.hauteur,
            donnees: self.donnees.iter().map(f).collect(),
        }
    }

    /// Combine deux images de même taille pixel à pixel.
    pub fn zip_map<U, V>(
        &self,
        autre: &Image<U>,
        mut f: impl FnMut(&T, &U) -> V,
    ) -> Resultat<Image<V>> {
        if !self.meme_dimensions(autre) {
            return Err(Erreur::Dimensions(format!(
                "images de tailles différentes : {}x{} et {}x{}",
                self.largeur, self.hauteur, autre.largeur, autre.hauteur
            )));
        }
        Ok(Image {
            largeur: self.largeur,
            hauteur: self.hauteur,
            donnees: self
                .donnees
                .iter()
                .zip(&autre.donnees)
                .map(|(a, b)| f(a, b))
                .collect(),
        })
    }
}

impl<T: Copy + Default> Image<T> {
    /// Valeur du pixel (x, y), éventuellement hors de l'image, selon la gestion du bord.
    ///
    /// Renvoie `None` uniquement avec `Bord::Partiel` quand le pixel est hors image :
    /// l'appelant doit alors ignorer ce terme.
    pub fn pixel_etendu(&self, x: isize, y: isize, bord: Bord) -> Option<T> {
        if self.contient(x, y) {
            return Some(self[(x as usize, y as usize)]);
        }
        if self.largeur == 0 || self.hauteur == 0 {
            return None;
        }
        match bord {
            Bord::Zero => Some(T::default()),
            Bord::Partiel => None,
            Bord::Replique => {
                let xc = x.clamp(0, self.largeur as isize - 1) as usize;
                let yc = y.clamp(0, self.hauteur as isize - 1) as usize;
                Some(self[(xc, yc)])
            }
            Bord::Miroir => {
                let xm = miroir(x, self.largeur);
                let ym = miroir(y, self.hauteur);
                Some(self[(xm, ym)])
            }
        }
    }
}

/// Indice réfléchi dans [0, n) : -1 -> 1, n -> n-2 (sans répéter le bord).
fn miroir(i: isize, n: usize) -> usize {
    if n == 1 {
        return 0;
    }
    let periode = 2 * (n as isize - 1);
    let mut j = i.rem_euclid(periode);
    if j >= n as isize {
        j = periode - j;
    }
    j as usize
}

impl<T> Index<(usize, usize)> for Image<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &T {
        assert!(
            x < self.largeur && y < self.hauteur,
            "pixel ({x}, {y}) hors d'une image {}x{}",
            self.largeur,
            self.hauteur
        );
        &self.donnees[y * self.largeur + x]
    }
}

impl<T> IndexMut<(usize, usize)> for Image<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut T {
        assert!(
            x < self.largeur && y < self.hauteur,
            "pixel ({x}, {y}) hors d'une image {}x{}",
            self.largeur,
            self.hauteur
        );
        &mut self.donnees[y * self.largeur + x]
    }
}

// ---------------------------------------------------------------------------
// Conversions entre types d'images
// ---------------------------------------------------------------------------

impl ImageGris {
    pub fn vers_reelle(&self) -> ImageReelle {
        self.map(|&p| p as f64)
    }
}

impl ImageReelle {
    /// Arrondit et sature chaque valeur dans [0, 255].
    pub fn vers_gris_sature(&self) -> ImageGris {
        self.map(|&v| v.round().clamp(0.0, 255.0) as u8)
    }

    /// Ramène linéairement [min, max] sur [0, 255] (pour afficher un gradient, un spectre...).
    pub fn vers_gris_normalise(&self) -> ImageGris {
        let (min, max) = self
            .donnees
            .iter()
            .filter(|v| v.is_finite())
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), &v| {
                (mn.min(v), mx.max(v))
            });
        if !min.is_finite() || max <= min {
            return self.map(|_| 0);
        }
        self.map(|&v| (255.0 * (v - min) / (max - min)).round().clamp(0.0, 255.0) as u8)
    }

    /// Valeur absolue de chaque pixel (utile pour afficher une dérivée).
    pub fn abs(&self) -> ImageReelle {
        self.map(|v| v.abs())
    }
}

impl ImageBinaire {
    /// 1 -> 255 (blanc), 0 -> 0 (noir), pour l'affichage et l'enregistrement.
    pub fn vers_gris(&self) -> ImageGris {
        self.map(|&b| if b { 255 } else { 0 })
    }

    pub fn nb_pixels_objet(&self) -> usize {
        self.donnees.iter().filter(|&&b| b).count()
    }
}

impl ImageRvb {
    /// Sépare les trois canaux R, V, B.
    pub fn separer_canaux(&self) -> [ImageGris; 3] {
        [self.map(|p| p.r), self.map(|p| p.v), self.map(|p| p.b)]
    }

    /// Reconstruit une image couleur à partir de trois canaux de même taille.
    pub fn fusionner_canaux(r: &ImageGris, v: &ImageGris, b: &ImageGris) -> Resultat<ImageRvb> {
        let rv = r.zip_map(v, |&r, &v| (r, v))?;
        rv.zip_map(b, |&(r, v), &b| Rvb::new(r, v, b))
    }

    /// Applique un traitement prévu pour les niveaux de gris sur chaque canal.
    pub fn par_canal(&self, mut f: impl FnMut(&ImageGris) -> ImageGris) -> Resultat<ImageRvb> {
        let [r, v, b] = self.separer_canaux();
        ImageRvb::fusionner_canaux(&f(&r), &f(&v), &f(&b))
    }
}

impl ImageEtiquettes {
    /// Plus grande étiquette présente (= nombre de régions si numérotées 1..n).
    pub fn etiquette_max(&self) -> u32 {
        self.donnees.iter().copied().max().unwrap_or(0)
    }

    /// Une couleur distincte par étiquette, fond (0) en noir.
    pub fn vers_couleurs(&self) -> ImageRvb {
        self.map(|&e| couleur_etiquette(e))
    }

    /// Étiquettes réparties sur [0, 255] (affichage en niveaux de gris).
    pub fn vers_gris(&self) -> ImageGris {
        let max = self.etiquette_max().max(1) as f64;
        self.map(|&e| (255.0 * e as f64 / max).round() as u8)
    }
}

/// Couleur vive et stable pour une étiquette donnée (teinte par nombre d'or).
pub fn couleur_etiquette(e: u32) -> Rvb {
    if e == 0 {
        return Rvb::default();
    }
    let teinte = (e as f64 * 0.618_033_988_749_895).fract() * 6.0;
    let secteur = teinte.floor() as u32;
    let f = teinte - secteur as f64;
    let (s, v) = (0.75, 1.0);
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    let (r, g, b) = match secteur {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    Rvb::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexation_ligne_par_ligne() {
        let img = ImageGris::depuis_fonction(3, 2, |x, y| (10 * y + x) as u8);
        assert_eq!(img.donnees(), &[0, 1, 2, 10, 11, 12]);
        assert_eq!(img[(2, 1)], 12);
        assert_eq!(img.ligne(1), &[10, 11, 12]);
    }

    #[test]
    fn mauvaise_taille_refusee() {
        assert!(ImageGris::depuis_vec(2, 2, vec![0; 3]).is_err());
    }

    #[test]
    fn gestion_des_bords() {
        let img = ImageGris::depuis_vec(3, 1, vec![1, 2, 3]).unwrap();
        assert_eq!(img.pixel_etendu(-1, 0, Bord::Zero), Some(0));
        assert_eq!(img.pixel_etendu(-1, 0, Bord::Replique), Some(1));
        assert_eq!(img.pixel_etendu(-1, 0, Bord::Miroir), Some(2));
        assert_eq!(img.pixel_etendu(3, 0, Bord::Miroir), Some(2));
        assert_eq!(img.pixel_etendu(4, 0, Bord::Miroir), Some(1));
        assert_eq!(img.pixel_etendu(5, 0, Bord::Partiel), None);
    }

    #[test]
    fn normalisation_reelle() {
        let img = ImageReelle::depuis_vec(3, 1, vec![-10.0, 0.0, 10.0]).unwrap();
        assert_eq!(img.vers_gris_normalise().donnees(), &[0, 128, 255]);
        assert_eq!(img.vers_gris_sature().donnees(), &[0, 0, 10]);
    }

    #[test]
    fn canaux_aller_retour() {
        let img = ImageRvb::depuis_fonction(4, 3, |x, y| Rvb::new(x as u8, y as u8, 7));
        let [r, v, b] = img.separer_canaux();
        assert_eq!(ImageRvb::fusionner_canaux(&r, &v, &b).unwrap(), img);
    }
}
