//! Chapitre 7 (deuxième partie) : opérateurs morphologiques.
//!
//! Responsable : Nkolo Atangana Stacy Julie.

// À retirer quand toutes les fonctions sont écrites.
#![allow(unused_variables)]

use crate::erreur::{Erreur, Resultat};
use crate::image::{Bord, ImageBinaire, ImageGris};

/// Élément structurant : liste des décalages (dx, dy) par rapport à son origine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementStructurant {
    pub decalages: Vec<(isize, isize)>,
}

impl ElementStructurant {
    /// Croix 3x3 (connexité 4), origine comprise.
    pub fn croix() -> Self {
        let decalages = vec![(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)];
        Self { decalages }
    }

    /// Carré `taille` x `taille` centré sur l'origine.
    ///
    /// `taille` doit être impaire et au moins 1. Pour 3, c'est la 8-connexité.
    pub fn carre(taille: usize) -> Resultat<Self> {
        if taille == 0 {
            return Err(Erreur::Parametre(
                "La taille doit être supérieure à 0".to_string(),
            ));
        }
        let mut decalages = Vec::new();
        let centre = (taille as isize - 1) / 2;
        for y in 0..taille {
            for x in 0..taille {
                decalages.push((x as isize - centre, y as isize - centre));
            }
        }
        Ok(Self { decalages })
    }

    /// Disque de rayon `rayon` : les décalages dont dx² + dy² ≤ rayon².
    pub fn disque(rayon: usize) -> Self {
        let mut decalages = Vec::new();
        let rayon_f = rayon as f64;
        for y in -(rayon as isize)..=(rayon as isize) {
            for x in -(rayon as isize)..=(rayon as isize) {
                if (x as f64).powi(2) + (y as f64).powi(2) <= rayon_f.powi(2) {
                    decalages.push((x, y));
                }
            }
        }
        Self { decalages }
    }

    /// Élément quelconque (pas forcément symétrique) décrit par un masque binaire
    /// dont l'origine est le pixel `origine`.
    pub fn depuis_masque(masque: &ImageBinaire, origine: (usize, usize)) -> Self {
        let mut decalages = Vec::new();
        let (orig_x, orig_y) = origine;
        for y in 0..masque.hauteur() {
            for x in 0..masque.largeur() {
                if masque.contient(x as isize, y as isize) {
                    decalages.push((x as isize - orig_x as isize, y as isize - orig_y as isize));
                }
            }
        }
        Self { decalages }
    }
}

/// Érosion : le pixel reste objet si tous les pixels sous l'élément sont objet.
pub fn erosion(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    ImageBinaire::depuis_fonction(img.largeur(), img.hauteur(), |x, y| {
        es.decalages.iter().all(|&(dx, dy)| {
            img.pixel_etendu(x as isize + dx, y as isize + dy, Bord::Zero) == Some(true)
        })
    })
}

/// Dilatation : le pixel devient objet si au moins un pixel sous l'élément est objet.
pub fn dilatation(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    ImageBinaire::depuis_fonction(img.largeur(), img.hauteur(), |x, y| {
        es.decalages.iter().any(|&(dx, dy)| {
            img.pixel_etendu(x as isize + dx, y as isize + dy, Bord::Zero) == Some(true)
        })
    })
}

/// Ouverture : érosion puis dilatation.
pub fn ouverture(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    dilatation(&erosion(img, es), es)
}

/// Fermeture : dilatation puis érosion.
pub fn fermeture(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    erosion(&dilatation(img, es), es)
}

/// Gradient interne : image - érodée (contour intérieur).
pub fn gradient_interne(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    let erodee = erosion(img, es);
    img.zip_map(&erodee, |&a, &b| a && !b)
        .expect("l'érosion garde les dimensions de l'image")
}

/// Gradient externe : dilatée - image (contour extérieur).
pub fn gradient_externe(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    let dilatee = dilatation(img, es);
    img.zip_map(&dilatee, |&a, &b| !a && b)
        .expect("la dilatation garde les dimensions de l'image")
}

/// Gradient morphologique : dilatée - érodée.
pub fn gradient_morphologique(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    let dilatee = dilatation(img, es);
    let erodee = erosion(img, es);
    dilatee
        .zip_map(&erodee, |&a, &b| a && !b)
        .expect("la dilatation et l'érosion gardent les dimensions de l'image")
}

/// Érosion en niveaux de gris : minimum sous l'élément.
pub fn erosion_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    ImageGris::depuis_fonction(img.largeur(), img.hauteur(), |x, y| {
        es.decalages
            .iter()
            .map(|&(dx, dy)| {
                img.pixel_etendu(x as isize + dx, y as isize + dy, Bord::Zero)
                    .unwrap_or(0)
            })
            .min()
            .unwrap_or(0)
    })
}

/// Dilatation en niveaux de gris : maximum sous l'élément.
pub fn dilatation_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    ImageGris::depuis_fonction(img.largeur(), img.hauteur(), |x, y| {
        es.decalages
            .iter()
            .map(|&(dx, dy)| {
                img.pixel_etendu(x as isize + dx, y as isize + dy, Bord::Zero)
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0)
    })
}

pub fn ouverture_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    dilatation_gris(&erosion_gris(img, es), es)
}

pub fn fermeture_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    erosion_gris(&dilatation_gris(img, es), es)
}

/// Gradient morphologique en niveaux de gris : dilatée - érodée.
pub fn gradient_morphologique_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    let dilatee = dilatation_gris(img, es);
    let erodee = erosion_gris(img, es);
    ImageGris::depuis_fonction(img.largeur(), img.hauteur(), |x, y| {
        let d = dilatee.get(x, y).unwrap_or(&0);
        let e = erodee.get(x, y).unwrap_or(&0);
        d.saturating_sub(*e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binaire(pixels: Vec<bool>, largeur: usize) -> ImageBinaire {
        let hauteur = pixels.len() / largeur;
        ImageBinaire::depuis_vec(largeur, hauteur, pixels).unwrap()
    }

    fn gris(pixels: Vec<u8>, largeur: usize) -> ImageGris {
        let hauteur = pixels.len() / largeur;
        ImageGris::depuis_vec(largeur, hauteur, pixels).unwrap()
    }

    #[test]
    fn croix_4_connexite_et_carre_8_connexite() {
        let croix = ElementStructurant::croix();
        assert!(croix.decalages.contains(&(0, 0)));
        assert!(croix.decalages.contains(&(1, 0)));
        assert!(!croix.decalages.contains(&(1, 1)));

        let carre = ElementStructurant::carre(3).unwrap();
        assert_eq!(carre.decalages.len(), 9);
        assert!(carre.decalages.contains(&(1, 1)));
        assert!(carre.decalages.contains(&(-1, -1)));
    }

    #[test]
    fn carre_impair_seulement() {
        assert!(ElementStructurant::carre(0).is_err());
        assert!(ElementStructurant::carre(2).is_err());
        assert_eq!(
            ElementStructurant::carre(1).unwrap().decalages,
            vec![(0, 0)]
        );
    }

    #[test]
    fn disque_et_masque() {
        let disque = ElementStructurant::disque(1);
        assert!(disque.decalages.contains(&(0, 0)));
        assert!(disque.decalages.contains(&(1, 0)));
        assert!(!disque.decalages.contains(&(1, 1)));

        let masque = binaire(
            vec![false, true, false, true, true, true, false, true, false],
            3,
        );
        let depuis = ElementStructurant::depuis_masque(&masque, (1, 1));
        let mut attendu = ElementStructurant::croix().decalages;
        let mut obtenu = depuis.decalages.clone();
        attendu.sort();
        obtenu.sort();
        assert_eq!(obtenu, attendu);
    }

    #[test]
    fn dilatation_croix_ne_joint_pas_la_diagonale() {
        let img = binaire(vec![true, false, false, false], 2);
        let croix = dilatation(&img, &ElementStructurant::croix());
        let carre = dilatation(&img, &ElementStructurant::carre(3).unwrap());
        assert!(!croix[(1, 1)]);
        assert!(carre[(1, 1)]);
        assert!(croix[(1, 0)]);
        assert!(croix[(0, 1)]);
    }

    #[test]
    fn erosion_ouverture_et_fermeture() {
        let croix = ElementStructurant::croix();
        let point = binaire(
            vec![false, false, false, false, true, false, false, false, false],
            3,
        );
        assert!(ouverture(&point, &croix).donnees().iter().all(|p| !p));

        let mut trou = vec![true; 16];
        trou[5] = false;
        let img = binaire(trou, 4);
        assert!(!img[(1, 1)]);
        assert!(fermeture(&img, &croix)[(1, 1)]);
    }

    #[test]
    fn gradients_binaires() {
        let img = binaire(
            vec![false, false, false, false, true, false, false, false, false],
            3,
        );
        let es = ElementStructurant::croix();
        let interne = gradient_interne(&img, &es);
        let externe = gradient_externe(&img, &es);
        assert!(interne[(1, 1)]);
        assert!(!interne[(0, 1)]);
        assert!(externe[(0, 1)]);
        assert!(!externe[(1, 1)]);
        let morpho = gradient_morphologique(&img, &es);
        assert!(morpho[(1, 1)]);
        assert!(morpho[(1, 0)]);
    }

    #[test]
    fn niveaux_de_gris_min_max() {
        let img = gris(vec![1, 8, 3, 4, 9, 2, 0, 7, 5], 3);
        let es = ElementStructurant::croix();
        assert_eq!(erosion_gris(&img, &es)[(1, 1)], 2);
        assert_eq!(dilatation_gris(&img, &es)[(1, 1)], 9);
        let gradient = gradient_morphologique_gris(&img, &es);
        assert_eq!(gradient[(1, 1)], 7);
        assert!(ouverture_gris(&img, &es)[(1, 1)] <= img[(1, 1)]);
        assert!(fermeture_gris(&img, &es)[(1, 1)] >= img[(1, 1)]);
    }
}
