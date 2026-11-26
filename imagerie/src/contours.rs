//! Chapitre 5 (première partie) : détection de contours par dérivées.
//!
//! Responsable : Sountsa Djiele Pio Vianney.
//!
//! Dépend de `filtrage::convoluer` (Pierre Marcelle).

use crate::filtrage::{convoluer, Noyau};
use crate::image::{Bord, ImageBinaire, ImageGris, ImageReelle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Operateur {
    /// Différence simple [-1 1].
    DeriveeSimple,
    /// Différence centrée [-1 0 1].
    DeriveeCentree,
    Roberts,
    Prewitt,
    #[default]
    Sobel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Laplacien {
    /// [0 1 0; 1 -4 1; 0 1 0]
    #[default]
    Quatre,
    /// [1 1 1; 1 -8 1; 1 1 1]
    Huit,
}

/// Dérivées en x et en y de l'image.
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub gx: ImageReelle,
    pub gy: ImageReelle,
}

impl Gradient {
    /// Norme exacte : sqrt(Gx² + Gy²).
    pub fn norme(&self) -> ImageReelle {
        self.gx
            .zip_map(&self.gy, |&gx, &gy| gx.hypot(gy))
            .expect("gx et gy doivent avoir les mêmes dimensions")
    }

    /// Norme approchée : |Gx| + |Gy|.
    pub fn norme_approchee(&self) -> ImageReelle {
        self.gx
            .zip_map(&self.gy, |&gx, &gy| gx.abs() + gy.abs())
            .expect("gx et gy doivent avoir les mêmes dimensions")
    }

    /// Direction : theta = arctan(Gy / Gx), en radians (utiliser atan2).
    pub fn direction(&self) -> ImageReelle {
        self.gx
            .zip_map(&self.gy, |&gx, &gy| gy.atan2(gx))
            .expect("gx et gy doivent avoir les mêmes dimensions")
    }
}

/// Paire de noyaux (Gx, Gy) de l'opérateur.
pub fn noyaux(op: Operateur) -> (Noyau, Noyau) {
    match op {
        Operateur::DeriveeSimple => (
            Noyau::depuis_3x3([[0.0, 0.0, 0.0], [0.0, -1.0, 1.0], [0.0, 0.0, 0.0]]),
            Noyau::depuis_3x3([[0.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 1.0, 0.0]]),
        ),
        Operateur::DeriveeCentree => (
            Noyau::depuis_3x3([[0.0, 0.0, 0.0], [-1.0, 0.0, 1.0], [0.0, 0.0, 0.0]]),
            Noyau::depuis_3x3([[0.0, -1.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
        ),
        Operateur::Roberts => (
            Noyau::depuis_3x3([[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]]),
            Noyau::depuis_3x3([[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, -1.0, 0.0]]),
        ),
        Operateur::Prewitt => (
            Noyau::depuis_3x3([[-1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], [-1.0, 0.0, 1.0]]),
            Noyau::depuis_3x3([[-1.0, -1.0, -1.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]),
        ),
        Operateur::Sobel => (
            Noyau::depuis_3x3([[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]]),
            Noyau::depuis_3x3([[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]]),
        ),
    }
}

pub fn noyau_laplacien(variante: Laplacien) -> Noyau {
    match variante {
        Laplacien::Quatre => {
            Noyau::depuis_3x3([[0.0, 1.0, 0.0], [1.0, -4.0, 1.0], [0.0, 1.0, 0.0]])
        }
        Laplacien::Huit => Noyau::depuis_3x3([[1.0, 1.0, 1.0], [1.0, -8.0, 1.0], [1.0, 1.0, 1.0]]),
    }
}

/// Calcule Gx et Gy par convolution avec les noyaux de l'opérateur.
pub fn gradient(img: &ImageGris, op: Operateur, bord: Bord) -> Gradient {
    let reelle = img.vers_reelle();
    let (kx, ky) = noyaux(op);
    Gradient {
        gx: convoluer(&reelle, &kx, bord),
        gy: convoluer(&reelle, &ky, bord),
    }
}

/// Contours = pixels dont la norme du gradient dépasse `seuil` (S = 25, 60 dans le cours).
pub fn contours_gradient(img: &ImageGris, op: Operateur, seuil: f64) -> ImageBinaire {
    let g = gradient(img, op, Bord::Replique);
    g.norme().map(|&v| v > seuil)
}

/// Laplacien (dérivée seconde) par convolution.
pub fn laplacien(img: &ImageGris, variante: Laplacien, bord: Bord) -> ImageReelle {
    let noyau = noyau_laplacien(variante);
    convoluer(&img.vers_reelle(), &noyau, bord)
}

/// Passages par zéro du laplacien dont l'amplitude dépasse `seuil`.
pub fn passages_par_zero(laplacien: &ImageReelle, seuil: f64) -> ImageBinaire {
    let (l, h) = laplacien.dimensions();
    let lire = |x: isize, y: isize| -> Option<f64> {
        if laplacien.contient(x, y) {
            Some(laplacien[(x as usize, y as usize)])
        } else {
            None
        }
    };

    ImageBinaire::depuis_fonction(l, h, |x, y| {
        let xi = x as isize;
        let yi = y as isize;
        let v = laplacien[(x, y)];

        if v == 0.0 {
            let paires = [
                (lire(xi - 1, yi), lire(xi + 1, yi)),
                (lire(xi, yi - 1), lire(xi, yi + 1)),
            ];
            return paires.into_iter().any(|(a, b)| match (a, b) {
                (Some(va), Some(vb)) => va * vb < 0.0 && (va - vb).abs() > seuil,
                _ => false,
            });
        }

        if v > 0.0 {
            let voisins = [(1, 0), (-1, 0), (0, 1), (0, -1)];
            return voisins.into_iter().any(|(dx, dy)| {
                lire(xi + dx, yi + dy).is_some_and(|vn| vn < 0.0 && (v - vn).abs() > seuil)
            });
        }

        false
    })
}

/// Amincissement : on ne garde que les maxima locaux de la norme dans la direction
/// du gradient, pour des contours d'un pixel d'épaisseur.
pub fn suppression_non_maxima(gradient: &Gradient) -> ImageReelle {
    let norme = gradient.norme();
    let direction = gradient.direction();
    let (l, h) = norme.dimensions();

    let lire = |x: isize, y: isize| -> f64 {
        if norme.contient(x, y) {
            norme[(x as usize, y as usize)]
        } else {
            0.0
        }
    };

    ImageReelle::depuis_fonction(l, h, |x, y| {
        let m = norme[(x, y)];
        if m == 0.0 {
            return 0.0;
        }
        let angle = direction[(x, y)].to_degrees().rem_euclid(180.0);
        let xi = x as isize;
        let yi = y as isize;

        // Axe y orienté vers le bas : 45° correspond à (+1, +1) / (-1, -1).
        let (n1, n2) = if !(22.5..157.5).contains(&angle) {
            (lire(xi - 1, yi), lire(xi + 1, yi))
        } else if angle < 67.5 {
            (lire(xi - 1, yi - 1), lire(xi + 1, yi + 1))
        } else if angle < 112.5 {
            (lire(xi, yi - 1), lire(xi, yi + 1))
        } else {
            (lire(xi - 1, yi + 1), lire(xi + 1, yi - 1))
        };

        if m >= n1 && m >= n2 {
            m
        } else {
            0.0
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn gradient_normes_et_direction() {
        let gx = ImageReelle::depuis_vec(2, 1, vec![3.0, 0.0]).unwrap();
        let gy = ImageReelle::depuis_vec(2, 1, vec![4.0, -5.0]).unwrap();
        let g = Gradient { gx, gy };

        let n = g.norme();
        assert!(approx_eq(n[(0, 0)], 5.0));
        assert!(approx_eq(n[(1, 0)], 5.0));

        let na = g.norme_approchee();
        assert!(approx_eq(na[(0, 0)], 7.0));
        assert!(approx_eq(na[(1, 0)], 5.0));

        let dir = g.direction();
        assert!(approx_eq(dir[(0, 0)], 4.0_f64.atan2(3.0)));
        assert!(approx_eq(dir[(1, 0)], (-5.0_f64).atan2(0.0)));
    }

    #[test]
    fn noyaux_derivee_et_laplacien() {
        for op in [
            Operateur::DeriveeSimple,
            Operateur::DeriveeCentree,
            Operateur::Roberts,
            Operateur::Prewitt,
            Operateur::Sobel,
        ] {
            let (kx, ky) = noyaux(op);
            assert!(approx_eq(kx.somme(), 0.0));
            assert!(approx_eq(ky.somme(), 0.0));
        }

        let l4 = noyau_laplacien(Laplacien::Quatre);
        assert!(approx_eq(l4.somme(), 0.0));
        assert!(approx_eq(l4.coef(1, 1), -4.0));

        let l8 = noyau_laplacien(Laplacien::Huit);
        assert!(approx_eq(l8.somme(), 0.0));
        assert!(approx_eq(l8.coef(1, 1), -8.0));
    }

    #[test]
    fn gradient_bord_vertical_sobel_et_prewitt() {
        // Image 5x3 : colonnes 0..2 = 0, colonnes 2..5 = 100
        let img = ImageGris::depuis_fonction(5, 3, |x, _| if x < 2 { 0 } else { 100 });

        let g_sobel = gradient(&img, Operateur::Sobel, Bord::Replique);
        // En x=1, y=1 : colonne gauche = 0, colonne droite = 100 => Gx = 400, Gy = 0
        assert!(approx_eq(g_sobel.gx[(1, 1)], 400.0));
        assert!(approx_eq(g_sobel.gy[(1, 1)], 0.0));

        let g_prewitt = gradient(&img, Operateur::Prewitt, Bord::Replique);
        assert!(approx_eq(g_prewitt.gx[(1, 1)], 300.0));
        assert!(approx_eq(g_prewitt.gy[(1, 1)], 0.0));

        let g_simple = gradient(&img, Operateur::DeriveeSimple, Bord::Replique);
        assert!(approx_eq(g_simple.gx[(1, 1)], 100.0));
        assert!(approx_eq(g_simple.gx[(2, 1)], 0.0));

        let g_centree = gradient(&img, Operateur::DeriveeCentree, Bord::Replique);
        assert!(approx_eq(g_centree.gx[(1, 1)], 100.0));
        assert!(approx_eq(g_centree.gx[(2, 1)], 100.0));

        let g_roberts = gradient(&img, Operateur::Roberts, Bord::Replique);
        assert!(approx_eq(g_roberts.gx[(1, 1)], -100.0));
        assert!(approx_eq(g_roberts.gy[(1, 1)], 100.0));
    }

    #[test]
    fn contours_gradient_seuils_cours() {
        // Saut de 40 sur fond 0 : dérivée simple donne norme 40 à la frontière
        let img = ImageGris::depuis_fonction(4, 3, |x, _| if x < 2 { 0 } else { 40 });
        let c25 = contours_gradient(&img, Operateur::DeriveeSimple, 25.0);
        let c60 = contours_gradient(&img, Operateur::DeriveeSimple, 60.0);

        assert!(c25[(1, 1)]);
        assert!(!c25[(0, 1)]);
        assert!(!c60[(1, 1)]);
    }

    #[test]
    fn laplacien_et_passages_par_zero() {
        // Point isolé blanc au centre d'une image 3x3 noire
        let mut img = ImageGris::nouvelle(3, 3, 0);
        img[(1, 1)] = 10;

        let lap4 = laplacien(&img, Laplacien::Quatre, Bord::Zero);
        assert!(approx_eq(lap4[(1, 1)], -40.0));
        assert!(approx_eq(lap4[(1, 0)], 10.0));

        let lap8 = laplacien(&img, Laplacien::Huit, Bord::Zero);
        assert!(approx_eq(lap8[(1, 1)], -80.0));
        assert!(approx_eq(lap8[(0, 0)], 10.0));

        // Rampe produisant un passage par zéro exact en x=2 (+50, 0, -50)
        let champ = ImageReelle::depuis_vec(5, 1, vec![0.0, 50.0, 0.0, -50.0, 0.0]).unwrap();
        let pz = passages_par_zero(&champ, 20.0);
        assert_eq!(pz.donnees(), &[false, false, true, false, false]);

        // Saut franc (+100, -100) sans zéro exact
        let saut = ImageReelle::depuis_vec(4, 1, vec![0.0, 100.0, -100.0, 0.0]).unwrap();
        let pz_saut = passages_par_zero(&saut, 50.0);
        assert_eq!(pz_saut.donnees(), &[false, true, false, false]);
    }

    #[test]
    fn suppression_non_maxima_amincit_contour_vertical() {
        // Profil horizontal de norme : [10, 50, 100, 50, 10] avec gradient horizontal (gy = 0)
        let gx = ImageReelle::depuis_vec(
            5,
            3,
            vec![
                10.0, 50.0, 100.0, 50.0, 10.0, //
                10.0, 50.0, 100.0, 50.0, 10.0, //
                10.0, 50.0, 100.0, 50.0, 10.0, //
            ],
        )
        .unwrap();
        let gy = ImageReelle::nouvelle(5, 3, 0.0);
        let grad = Gradient { gx, gy };

        let minces = suppression_non_maxima(&grad);
        for y in 0..3 {
            assert!(approx_eq(minces[(0, y)], 0.0));
            assert!(approx_eq(minces[(1, y)], 0.0));
            assert!(approx_eq(minces[(2, y)], 100.0));
            assert!(approx_eq(minces[(3, y)], 0.0));
            assert!(approx_eq(minces[(4, y)], 0.0));
        }
    }
}
