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
    /// Croix 3x3 (connexité 4).
    pub fn croix() -> Self {
        let decalages = vec![(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)];
        Self { decalages }
    }

    /// Carré `taille` x `taille` (connexité 8 pour taille 3).
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

    /// Disque de rayon `rayon`.
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
