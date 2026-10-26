//! Chapitre 1 : Introduction.
//!
//! Responsable : Bala Andegue Francois Lionnel.
//!
//! Conversion couleur -> gris, extraction des canaux R/V/B, échantillonnage
//! (résolution spatiale), quantification (résolution tonale), profils d'intensité.

// À retirer quand toutes les fonctions sont écrites.
#![allow(unused_variables)]

use crate::erreur::Resultat;
use crate::image::{ImageGris, ImageRvb};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Canal {
    Rouge,
    Vert,
    Bleu,
}

/// Conversion en niveaux de gris par luminance : 0.299 R + 0.587 V + 0.114 B.
pub fn vers_gris(img: &ImageRvb) -> ImageGris {
    todo!("vers_gris")
}

/// Conversion en niveaux de gris par moyenne simple : (R + V + B) / 3.
pub fn vers_gris_moyenne(img: &ImageRvb) -> ImageGris {
    todo!("vers_gris_moyenne")
}

/// Extrait un canal sous forme d'image en niveaux de gris.
pub fn extraire_canal(img: &ImageRvb, canal: Canal) -> ImageGris {
    todo!("extraire_canal")
}

/// Garde seulement le canal demandé, les deux autres à 0 (pour l'affichage en couleur).
pub fn isoler_canal(img: &ImageRvb, canal: Canal) -> ImageRvb {
    todo!("isoler_canal")
}

/// Quantification : réduit l'image à `nb_niveaux` niveaux de gris (L = 2^K),
/// chaque niveau étant ramené au centre de son intervalle sur [0, 255].
pub fn quantifier(img: &ImageGris, nb_niveaux: u16) -> Resultat<ImageGris> {
    todo!("quantifier")
}

/// Échantillonnage : ne garde qu'un pixel sur `pas` dans chaque direction
/// (l'image résultat est plus petite).
pub fn echantillonner(img: &ImageGris, pas: usize) -> Resultat<ImageGris> {
    todo!("echantillonner")
}

/// Même effet visuel que `echantillonner` mais en gardant la taille d'origine
/// (blocs de `pas` x `pas` pixels identiques), comme sur les figures du cours.
pub fn pixeliser(img: &ImageGris, pas: usize) -> Resultat<ImageGris> {
    todo!("pixeliser")
}

/// Nombre de bits pour stocker l'image : b = M x N x K.
pub fn taille_en_bits(largeur: usize, hauteur: usize, bits_par_pixel: u32) -> u64 {
    todo!("taille_en_bits")
}

/// Profil d'intensité de la ligne `y`.
pub fn profil_ligne(img: &ImageGris, y: usize) -> Vec<u8> {
    todo!("profil_ligne")
}

/// Profil d'intensité de la colonne `x`.
pub fn profil_colonne(img: &ImageGris, x: usize) -> Vec<u8> {
    todo!("profil_colonne")
}

/// Profil d'intensité le long d'un segment quelconque (tracé de Bresenham).
pub fn profil_segment(img: &ImageGris, depart: (usize, usize), arrivee: (usize, usize)) -> Vec<u8> {
    todo!("profil_segment")
}

/// Dessine un profil sous forme de courbe (abscisse = position, ordonnée = niveau).
pub fn image_profil(profil: &[u8], hauteur: usize) -> ImageGris {
    todo!("image_profil")
}
