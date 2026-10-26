//! Chapitre 2 (deuxième partie) : opérations logiques et arithmétiques sur les images.
//!
//! Responsable : Belekotan II Jeff Nichoss.

// À retirer quand toutes les fonctions sont écrites.
#![allow(unused_variables)]

use crate::erreur::Resultat;
use crate::image::{ImageBinaire, ImageGris};

/// ET logique bit à bit entre deux images.
pub fn et(a: &ImageGris, b: &ImageGris) -> Resultat<ImageGris> {
    todo!("et")
}

/// OU logique bit à bit entre deux images.
pub fn ou(a: &ImageGris, b: &ImageGris) -> Resultat<ImageGris> {
    todo!("ou")
}

/// ET entre deux masques binaires.
pub fn et_binaire(a: &ImageBinaire, b: &ImageBinaire) -> Resultat<ImageBinaire> {
    todo!("et_binaire")
}

/// OU entre deux masques binaires.
pub fn ou_binaire(a: &ImageBinaire, b: &ImageBinaire) -> Resultat<ImageBinaire> {
    todo!("ou_binaire")
}

/// NON d'un masque binaire.
pub fn non_binaire(a: &ImageBinaire) -> ImageBinaire {
    todo!("non_binaire")
}

/// Applique un masque : garde les pixels de `img` où le masque vaut 1, met 0 ailleurs.
pub fn appliquer_masque(img: &ImageGris, masque: &ImageBinaire) -> Resultat<ImageGris> {
    todo!("appliquer_masque")
}

/// R(x, y) = min(f(x, y) + g(x, y), 255).
pub fn addition(f: &ImageGris, g: &ImageGris) -> Resultat<ImageGris> {
    todo!("addition")
}

/// S(x, y) = max(f(x, y) - g(x, y), 0).
pub fn soustraction(f: &ImageGris, g: &ImageGris) -> Resultat<ImageGris> {
    todo!("soustraction")
}

/// |f - g| : détection de mouvement / de défauts.
pub fn difference_absolue(f: &ImageGris, g: &ImageGris) -> Resultat<ImageGris> {
    todo!("difference_absolue")
}

/// S(x, y) = min(f(x, y) x ratio, 255).
pub fn multiplication(f: &ImageGris, ratio: f64) -> Resultat<ImageGris> {
    todo!("multiplication")
}

/// alpha F + (1 - alpha) G, avec alpha dans [0, 1] (0.5 dans l'exemple du cours).
pub fn combinaison_lineaire(f: &ImageGris, g: &ImageGris, alpha: f64) -> Resultat<ImageGris> {
    todo!("combinaison_lineaire")
}

/// Moyenne d'une série d'images de même taille (réduction du bruit).
pub fn moyenne_images(images: &[ImageGris]) -> Resultat<ImageGris> {
    todo!("moyenne_images")
}
