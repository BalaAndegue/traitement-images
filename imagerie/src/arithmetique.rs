//! Chapitre 2 (deuxième partie) : opérations logiques et arithmétiques sur les images.
//!
//! Responsable : Belekotan II Jeff Nichoss.

use crate::erreur::{Erreur, Resultat};
use crate::image::{ImageBinaire, ImageGris};

/// ET logique bit à bit entre deux images.
pub fn et(a: &ImageGris, b: &ImageGris) -> Resultat<ImageGris> {
    a.zip_map(b, |&x, &y| x & y)
}

/// OU logique bit à bit entre deux images.
pub fn ou(a: &ImageGris, b: &ImageGris) -> Resultat<ImageGris> {
    a.zip_map(b, |&x, &y| x | y)
}

/// ET entre deux masques binaires.
pub fn et_binaire(a: &ImageBinaire, b: &ImageBinaire) -> Resultat<ImageBinaire> {
    a.zip_map(b, |&x, &y| x && y)
}

/// OU entre deux masques binaires.
pub fn ou_binaire(a: &ImageBinaire, b: &ImageBinaire) -> Resultat<ImageBinaire> {
    a.zip_map(b, |&x, &y| x || y)
}

/// NON d'un masque binaire.
pub fn non_binaire(a: &ImageBinaire) -> ImageBinaire {
    a.map(|&x| !x)
}

/// Applique un masque : garde les pixels de `img` où le masque vaut 1, met 0 ailleurs.
pub fn appliquer_masque(img: &ImageGris, masque: &ImageBinaire) -> Resultat<ImageGris> {
    img.zip_map(masque, |&p, &m| if m { p } else { 0 })
}

/// R(x, y) = min(f(x, y) + g(x, y), 255).
pub fn addition(f: &ImageGris, g: &ImageGris) -> Resultat<ImageGris> {
    f.zip_map(g, |&a, &b| a.saturating_add(b))
}

/// S(x, y) = max(f(x, y) - g(x, y), 0).
pub fn soustraction(f: &ImageGris, g: &ImageGris) -> Resultat<ImageGris> {
    f.zip_map(g, |&a, &b| a.saturating_sub(b))
}

/// |f - g| : détection de mouvement / de défauts.
pub fn difference_absolue(f: &ImageGris, g: &ImageGris) -> Resultat<ImageGris> {
    f.zip_map(g, |&a, &b| a.abs_diff(b))
}

/// S(x, y) = min(f(x, y) x ratio, 255).
pub fn multiplication(f: &ImageGris, ratio: f64) -> Resultat<ImageGris> {
    if !ratio.is_finite() || ratio < 0.0 {
        return Err(Erreur::Parametre(format!(
            "ratio doit être un réel >= 0 (reçu {ratio})"
        )));
    }
    Ok(f.map(|&p| ((p as f64) * ratio).round().clamp(0.0, 255.0) as u8))
}

/// alpha F + (1 - alpha) G, avec alpha dans [0, 1] (0.5 dans l'exemple du cours).
pub fn combinaison_lineaire(f: &ImageGris, g: &ImageGris, alpha: f64) -> Resultat<ImageGris> {
    if !(0.0..=1.0).contains(&alpha) || !alpha.is_finite() {
        return Err(Erreur::Parametre(format!(
            "alpha doit être dans [0, 1] (reçu {alpha})"
        )));
    }
    f.zip_map(g, |&a, &b| {
        (alpha * a as f64 + (1.0 - alpha) * b as f64)
            .round()
            .clamp(0.0, 255.0) as u8
    })
}

/// Moyenne d'une série d'images de même taille (réduction du bruit).
pub fn moyenne_images(images: &[ImageGris]) -> Resultat<ImageGris> {
    let Some(premiere) = images.first() else {
        return Err(Erreur::Parametre(
            "moyenne_images : la liste d'images est vide".into(),
        ));
    };
    for (i, img) in images.iter().enumerate().skip(1) {
        if !premiere.meme_dimensions(img) {
            return Err(Erreur::Dimensions(format!(
                "image {i} : taille {}x{} incompatible avec {}x{}",
                img.largeur(),
                img.hauteur(),
                premiere.largeur(),
                premiere.hauteur()
            )));
        }
    }

    let n = images.len() as f64;
    let (largeur, hauteur) = premiere.dimensions();
    let mut sommes = vec![0u32; premiere.nb_pixels()];
    for img in images {
        for (s, &p) in sommes.iter_mut().zip(img.donnees()) {
            *s += p as u32;
        }
    }
    let donnees = sommes
        .into_iter()
        .map(|s| ((s as f64) / n).round().clamp(0.0, 255.0) as u8)
        .collect();
    ImageGris::depuis_vec(largeur, hauteur, donnees)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gris(vals: &[u8]) -> ImageGris {
        ImageGris::depuis_vec(vals.len(), 1, vals.to_vec()).unwrap()
    }

    fn bin(vals: &[bool]) -> ImageBinaire {
        ImageBinaire::depuis_vec(vals.len(), 1, vals.to_vec()).unwrap()
    }

    #[test]
    fn et_ou_bit_a_bit() {
        let a = gris(&[0b1100, 0b1010]);
        let b = gris(&[0b1010, 0b0101]);
        assert_eq!(et(&a, &b).unwrap().donnees(), &[0b1000, 0b0000]);
        assert_eq!(ou(&a, &b).unwrap().donnees(), &[0b1110, 0b1111]);
    }

    #[test]
    fn operations_binaires() {
        let a = bin(&[true, true, false]);
        let b = bin(&[true, false, false]);
        assert_eq!(et_binaire(&a, &b).unwrap().donnees(), &[true, false, false]);
        assert_eq!(ou_binaire(&a, &b).unwrap().donnees(), &[true, true, false]);
        assert_eq!(non_binaire(&a).donnees(), &[false, false, true]);
    }

    #[test]
    fn masque_et_arithmetique() {
        let img = gris(&[10, 20, 30]);
        let m = bin(&[true, false, true]);
        assert_eq!(appliquer_masque(&img, &m).unwrap().donnees(), &[10, 0, 30]);

        let a = gris(&[200, 50]);
        let b = gris(&[100, 80]);
        assert_eq!(addition(&a, &b).unwrap().donnees(), &[255, 130]);
        assert_eq!(soustraction(&a, &b).unwrap().donnees(), &[100, 0]);
        assert_eq!(difference_absolue(&a, &b).unwrap().donnees(), &[100, 30]);
    }

    #[test]
    fn multiplication_et_combinaison() {
        let f = gris(&[100, 200]);
        assert_eq!(multiplication(&f, 1.5).unwrap().donnees(), &[150, 255]);
        let g = gris(&[0, 100]);
        assert_eq!(
            combinaison_lineaire(&f, &g, 0.5).unwrap().donnees(),
            &[50, 150]
        );
        assert!(combinaison_lineaire(&f, &g, 1.5).is_err());
        assert!(multiplication(&f, -1.0).is_err());
    }

    #[test]
    fn moyenne_de_serie() {
        let imgs = [gris(&[10, 20]), gris(&[30, 40]), gris(&[50, 60])];
        assert_eq!(moyenne_images(&imgs).unwrap().donnees(), &[30, 40]);
        assert!(moyenne_images(&[]).is_err());
    }

    #[test]
    fn dimensions_incompatibles() {
        let a = gris(&[1, 2]);
        let b = ImageGris::nouvelle(1, 1, 0);
        assert!(addition(&a, &b).is_err());
    }
}
