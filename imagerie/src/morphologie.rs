//! Chapitre 7 (deuxième partie) : opérateurs morphologiques.
//!
//! Responsable : Nkolo Atangana Stacy.

// À retirer quand toutes les fonctions sont écrites.
#![allow(unused_variables)]

use crate::erreur::Resultat;
use crate::image::{ImageBinaire, ImageGris};

/// Élément structurant : liste des décalages (dx, dy) par rapport à son origine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementStructurant {
    pub decalages: Vec<(isize, isize)>,
}

impl ElementStructurant {
    /// Croix 3x3 (connexité 4).
    pub fn croix() -> Self {
        todo!("ElementStructurant::croix")
    }

    /// Carré `taille` x `taille` (connexité 8 pour taille 3).
    pub fn carre(taille: usize) -> Resultat<Self> {
        todo!("ElementStructurant::carre")
    }

    /// Disque de rayon `rayon`.
    pub fn disque(rayon: usize) -> Self {
        todo!("ElementStructurant::disque")
    }

    /// Élément quelconque (pas forcément symétrique) décrit par un masque binaire
    /// dont l'origine est le pixel `origine`.
    pub fn depuis_masque(masque: &ImageBinaire, origine: (usize, usize)) -> Self {
        todo!("ElementStructurant::depuis_masque")
    }
}

/// Érosion : le pixel reste objet si tous les pixels sous l'élément sont objet.
pub fn erosion(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    todo!("erosion")
}

/// Dilatation : le pixel devient objet si au moins un pixel sous l'élément est objet.
pub fn dilatation(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    todo!("dilatation")
}

/// Ouverture : érosion puis dilatation.
pub fn ouverture(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    todo!("ouverture")
}

/// Fermeture : dilatation puis érosion.
pub fn fermeture(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    todo!("fermeture")
}

/// Gradient interne : image - érodée (contour intérieur).
pub fn gradient_interne(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    todo!("gradient_interne")
}

/// Gradient externe : dilatée - image (contour extérieur).
pub fn gradient_externe(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    todo!("gradient_externe")
}

/// Gradient morphologique : dilatée - érodée.
pub fn gradient_morphologique(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    todo!("gradient_morphologique")
}

/// Érosion en niveaux de gris : minimum sous l'élément.
pub fn erosion_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    todo!("erosion_gris")
}

/// Dilatation en niveaux de gris : maximum sous l'élément.
pub fn dilatation_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    todo!("dilatation_gris")
}

pub fn ouverture_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    todo!("ouverture_gris")
}

pub fn fermeture_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    todo!("fermeture_gris")
}

/// Gradient morphologique en niveaux de gris : dilatée - érodée.
pub fn gradient_morphologique_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    todo!("gradient_morphologique_gris")
}
