//! Chapitre 2 (troisième partie) : interpolation et changement d'échelle.
//!
//! Responsable : Nankeng Tsamo Pierre Marcelle.

// À retirer quand toutes les fonctions sont écrites.
#![allow(unused_variables)]

use crate::erreur::Resultat;
use crate::image::{ImageGris, ImageRvb};

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

/// Redimensionne l'image à une taille quelconque.
pub fn redimensionner(
    img: &ImageGris,
    nouvelle_largeur: usize,
    nouvelle_hauteur: usize,
    methode: Methode,
) -> Resultat<ImageGris> {
    todo!("redimensionner")
}

/// Zoom d'un facteur réel (> 1 agrandit, < 1 réduit).
pub fn zoom(img: &ImageGris, facteur: f64, methode: Methode) -> Resultat<ImageGris> {
    todo!("zoom")
}

/// Version couleur de `redimensionner` (canal par canal).
pub fn redimensionner_rvb(
    img: &ImageRvb,
    nouvelle_largeur: usize,
    nouvelle_hauteur: usize,
    methode: Methode,
) -> Resultat<ImageRvb> {
    todo!("redimensionner_rvb")
}

/// Plus proche voisin par copie de chaque ligne et de chaque colonne `facteur` fois.
pub fn agrandir_par_copie(img: &ImageGris, facteur: usize) -> Resultat<ImageGris> {
    todo!("agrandir_par_copie")
}

/// Interpolation 1D d'un facteur 2 avec les formules du cours :
/// - linéaire : y(i+1/2) = (x(i) + x(i+1)) / 2, extrapolation 2 x(N) - x(N-1) au bout ;
/// - cubique : y(i+1/2) = (-x(i-1) + 9 x(i) + 9 x(i+1) - x(i+2)) / 16, avec les
///   extrapolations parabolique et linéaire aux extrémités.
pub fn interpoler_ligne_x2(ligne: &[f64], methode: Methode) -> Vec<f64> {
    todo!("interpoler_ligne_x2")
}

/// Interpolation cubique 1D d'un facteur 3 (formules en /81 du cours).
pub fn interpoler_ligne_x3_cubique(ligne: &[f64]) -> Vec<f64> {
    todo!("interpoler_ligne_x3_cubique")
}

/// Zoom x2 séparable : `interpoler_ligne_x2` sur les lignes puis sur les colonnes.
pub fn zoom_x2_separable(img: &ImageGris, methode: Methode) -> ImageGris {
    todo!("zoom_x2_separable")
}

/// Rotation de l'image d'un angle en degrés autour de son centre
/// (utilisée pour montrer que la FFT tourne du même angle).
pub fn rotation(img: &ImageGris, angle_degres: f64, methode: Methode) -> ImageGris {
    todo!("rotation")
}
