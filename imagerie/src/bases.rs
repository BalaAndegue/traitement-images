//! Chapitre 1 : Introduction.
//!
//! Responsable : Bala Andegue Francois Lionnel.
//!
//! Conversion couleur -> gris, extraction des canaux R/V/B, échantillonnage
//! (résolution spatiale), quantification (résolution tonale), profils d'intensité.

use crate::erreur::{Erreur, Resultat};
use crate::image::{ImageGris, ImageRvb, Rvb};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Canal {
    Rouge,
    Vert,
    Bleu,
}

/// Conversion en niveaux de gris par luminance : 0.299 R + 0.587 V + 0.114 B.
pub fn vers_gris(img: &ImageRvb) -> ImageGris {
    img.map(|p| {
        let y = 0.299 * f64::from(p.r) + 0.587 * f64::from(p.v) + 0.114 * f64::from(p.b);
        y.round().clamp(0.0, 255.0) as u8
    })
}

/// Conversion en niveaux de gris par moyenne simple : (R + V + B) / 3.
pub fn vers_gris_moyenne(img: &ImageRvb) -> ImageGris {
    img.map(|p| {
        let somme = u16::from(p.r) + u16::from(p.v) + u16::from(p.b);
        (somme / 3) as u8
    })
}

/// Extrait un canal sous forme d'image en niveaux de gris.
pub fn extraire_canal(img: &ImageRvb, canal: Canal) -> ImageGris {
    img.map(|p| valeur_canal(*p, canal))
}

/// Garde seulement le canal demandé, les deux autres à 0 (pour l'affichage en couleur).
pub fn isoler_canal(img: &ImageRvb, canal: Canal) -> ImageRvb {
    img.map(|p| match canal {
        Canal::Rouge => Rvb::new(p.r, 0, 0),
        Canal::Vert => Rvb::new(0, p.v, 0),
        Canal::Bleu => Rvb::new(0, 0, p.b),
    })
}

/// Quantification : réduit l'image à `nb_niveaux` niveaux de gris (L = 2^K),
/// chaque niveau étant ramené au centre de son intervalle sur [0, 255].
pub fn quantifier(img: &ImageGris, nb_niveaux: u16) -> Resultat<ImageGris> {
    if !(2..=256).contains(&nb_niveaux) {
        return Err(Erreur::Parametre(format!(
            "le nombre de niveaux doit être entre 2 et 256, reçu {nb_niveaux}"
        )));
    }
    let largeur_bin = 256.0 / f64::from(nb_niveaux);
    let dernier = f64::from(nb_niveaux - 1);
    Ok(img.map(|&p| {
        let k = (f64::from(p) / largeur_bin).floor().min(dernier);
        ((k + 0.5) * largeur_bin).round().clamp(0.0, 255.0) as u8
    }))
}

/// Échantillonnage : ne garde qu'un pixel sur `pas` dans chaque direction
/// (l'image résultat est plus petite).
pub fn echantillonner(img: &ImageGris, pas: usize) -> Resultat<ImageGris> {
    verifier_pas(pas)?;
    let largeur = img.largeur().div_ceil(pas);
    let hauteur = img.hauteur().div_ceil(pas);
    Ok(ImageGris::depuis_fonction(largeur, hauteur, |x, y| {
        img[(x * pas, y * pas)]
    }))
}

/// Même effet visuel que `echantillonner` mais en gardant la taille d'origine
/// (blocs de `pas` x `pas` pixels identiques), comme sur les figures du cours.
pub fn pixeliser(img: &ImageGris, pas: usize) -> Resultat<ImageGris> {
    verifier_pas(pas)?;
    Ok(ImageGris::depuis_fonction(
        img.largeur(),
        img.hauteur(),
        |x, y| img[(x - x % pas, y - y % pas)],
    ))
}

/// Nombre de bits pour stocker l'image : b = M x N x K.
pub fn taille_en_bits(largeur: usize, hauteur: usize, bits_par_pixel: u32) -> u64 {
    (largeur as u64)
        .saturating_mul(hauteur as u64)
        .saturating_mul(u64::from(bits_par_pixel))
}

/// Profil d'intensité de la ligne `y`.
pub fn profil_ligne(img: &ImageGris, y: usize) -> Vec<u8> {
    if y >= img.hauteur() {
        return Vec::new();
    }
    img.ligne(y).to_vec()
}

/// Profil d'intensité de la colonne `x`.
pub fn profil_colonne(img: &ImageGris, x: usize) -> Vec<u8> {
    if x >= img.largeur() {
        return Vec::new();
    }
    (0..img.hauteur()).map(|y| img[(x, y)]).collect()
}

/// Profil d'intensité le long d'un segment quelconque (tracé de Bresenham).
pub fn profil_segment(img: &ImageGris, depart: (usize, usize), arrivee: (usize, usize)) -> Vec<u8> {
    bresenham(depart, arrivee)
        .into_iter()
        .filter_map(|(x, y)| img.get(x, y).copied())
        .collect()
}

/// Dessine un profil sous forme de courbe (abscisse = position, ordonnée = niveau).
///
/// Le fond est blanc, la courbe est noire. Le niveau 0 est en bas, le niveau 255 en haut.
pub fn image_profil(profil: &[u8], hauteur: usize) -> ImageGris {
    let hauteur = hauteur.max(1);
    let largeur = profil.len().max(1);
    let mut img = ImageGris::nouvelle(largeur, hauteur, 255);
    if profil.is_empty() {
        return img;
    }
    let mut precedent = ordonnee_profil(profil[0], hauteur);
    for (x, &niveau) in profil.iter().enumerate() {
        let y = ordonnee_profil(niveau, hauteur);
        let (bas, haut) = if precedent < y {
            (precedent, y)
        } else {
            (y, precedent)
        };
        for ligne in bas..=haut {
            img[(x, ligne)] = 0;
        }
        precedent = y;
    }
    img
}

fn valeur_canal(pixel: Rvb, canal: Canal) -> u8 {
    match canal {
        Canal::Rouge => pixel.r,
        Canal::Vert => pixel.v,
        Canal::Bleu => pixel.b,
    }
}

fn verifier_pas(pas: usize) -> Resultat<()> {
    if pas == 0 {
        Err(Erreur::Parametre(
            "le pas d'échantillonnage doit être au moins 1".into(),
        ))
    } else {
        Ok(())
    }
}

/// Ordonnée image d'un niveau : 255 en haut, 0 en bas.
fn ordonnee_profil(niveau: u8, hauteur: usize) -> usize {
    let max = (hauteur - 1) as u32;
    let depuis_le_bas = u32::from(niveau) * max / 255;
    max as usize - depuis_le_bas as usize
}

/// Pixels du segment, extrémités comprises (algorithme de Bresenham).
fn bresenham(depart: (usize, usize), arrivee: (usize, usize)) -> Vec<(usize, usize)> {
    let mut x = depart.0 as isize;
    let mut y = depart.1 as isize;
    let x1 = arrivee.0 as isize;
    let y1 = arrivee.1 as isize;
    let dx = (x1 - x).abs();
    let dy = -(y1 - y).abs();
    let sx = if x < x1 { 1 } else { -1 };
    let sy = if y < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut points = Vec::new();
    loop {
        if x >= 0 && y >= 0 {
            points.push((x as usize, y as usize));
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    fn couleur(pixels: Vec<Rvb>, largeur: usize) -> ImageRvb {
        let hauteur = pixels.len() / largeur;
        ImageRvb::depuis_vec(largeur, hauteur, pixels).unwrap()
    }

    fn gris(pixels: Vec<u8>, largeur: usize) -> ImageGris {
        let hauteur = pixels.len() / largeur;
        ImageGris::depuis_vec(largeur, hauteur, pixels).unwrap()
    }

    #[test]
    fn luminance_et_moyenne_sur_un_pixel_rouge() {
        let img = couleur(vec![Rvb::new(255, 0, 0)], 1);
        assert_eq!(vers_gris(&img).donnees(), &[76]);
        assert_eq!(vers_gris_moyenne(&img).donnees(), &[85]);
    }

    #[test]
    fn blanc_reste_blanc_dans_les_deux_conversions() {
        let img = couleur(vec![Rvb::new(255, 255, 255), Rvb::new(0, 0, 0)], 2);
        assert_eq!(vers_gris(&img).donnees(), &[255, 0]);
        assert_eq!(vers_gris_moyenne(&img).donnees(), &[255, 0]);
    }

    #[test]
    fn extraction_et_isolement_des_canaux() {
        let img = couleur(vec![Rvb::new(10, 20, 30)], 1);
        assert_eq!(extraire_canal(&img, Canal::Rouge).donnees(), &[10]);
        assert_eq!(extraire_canal(&img, Canal::Vert).donnees(), &[20]);
        assert_eq!(extraire_canal(&img, Canal::Bleu).donnees(), &[30]);
        assert_eq!(
            isoler_canal(&img, Canal::Vert).donnees(),
            &[Rvb::new(0, 20, 0)]
        );
    }

    #[test]
    fn quantification_a_deux_niveaux() {
        let img = gris(vec![0, 127, 128, 255], 4);
        assert_eq!(quantifier(&img, 2).unwrap().donnees(), &[64, 64, 192, 192]);
    }

    #[test]
    fn quantification_refuse_un_nombre_de_niveaux_invalide() {
        let img = gris(vec![0], 1);
        assert!(quantifier(&img, 1).is_err());
        assert!(quantifier(&img, 257).is_err());
    }

    #[test]
    fn echantillonnage_garde_un_pixel_sur_deux() {
        let img = gris(vec![1, 2, 3, 4, 5, 6, 7, 8], 4);
        let petit = echantillonner(&img, 2).unwrap();
        assert_eq!(petit.dimensions(), (2, 1));
        assert_eq!(petit.donnees(), &[1, 3]);
    }

    #[test]
    fn pixelisation_recopie_le_coin_de_chaque_bloc() {
        let img = gris(vec![1, 2, 3, 4, 5, 6, 7, 8], 4);
        let blocs = pixeliser(&img, 2).unwrap();
        assert_eq!(blocs.dimensions(), (4, 2));
        assert_eq!(blocs.donnees(), &[1, 1, 3, 3, 1, 1, 3, 3]);
    }

    #[test]
    fn pas_nul_refuse() {
        let img = gris(vec![1], 1);
        assert!(echantillonner(&img, 0).is_err());
        assert!(pixeliser(&img, 0).is_err());
    }

    #[test]
    fn taille_de_stockage() {
        assert_eq!(taille_en_bits(256, 256, 8), 256 * 256 * 8);
        assert_eq!(taille_en_bits(10, 20, 24), 4800);
    }

    #[test]
    fn profils_ligne_colonne_et_diagonale() {
        let img = gris(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], 3);
        assert_eq!(profil_ligne(&img, 1), vec![4, 5, 6]);
        assert_eq!(profil_colonne(&img, 2), vec![3, 6, 9]);
        assert_eq!(profil_segment(&img, (0, 0), (2, 2)), vec![1, 5, 9]);
        assert!(profil_ligne(&img, 9).is_empty());
    }

    #[test]
    fn trace_du_profil_place_le_blanc_en_haut() {
        let courbe = image_profil(&[0, 255], 256);
        assert_eq!(courbe.dimensions(), (2, 256));
        assert_eq!(courbe[(0, 255)], 0);
        assert_eq!(courbe[(1, 0)], 0);
        assert_eq!(courbe[(0, 0)], 255);
    }
}
