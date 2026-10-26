//! Lecture et écriture des fichiers d'images.
//!
//! PGM (niveaux de gris) et PPM (couleur) sont codés à la main, en ASCII (P2/P3)
//! comme en binaire (P5/P6). Les autres formats (PNG, JPEG, BMP, TIFF, GIF)
//! passent par la crate `image` quand la fonctionnalité `formats` est active.
//!
//! Rappel du cours : éviter d'enregistrer les résultats en JPEG (compression avec perte).

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::erreur::{Erreur, Resultat};
use crate::image::{ImageGris, ImageRvb, Rvb};

/// Image telle qu'elle a été lue sur le disque.
#[derive(Clone, Debug, PartialEq)]
pub enum ImageChargee {
    Gris(ImageGris),
    Couleur(ImageRvb),
}

impl ImageChargee {
    pub fn dimensions(&self) -> (usize, usize) {
        match self {
            ImageChargee::Gris(i) => i.dimensions(),
            ImageChargee::Couleur(i) => i.dimensions(),
        }
    }

    /// Version en niveaux de gris (luminance 0.299 R + 0.587 V + 0.114 B si couleur).
    pub fn en_gris(&self) -> ImageGris {
        match self {
            ImageChargee::Gris(i) => i.clone(),
            ImageChargee::Couleur(i) => i.map(|p| {
                (0.299 * p.r as f64 + 0.587 * p.v as f64 + 0.114 * p.b as f64).round() as u8
            }),
        }
    }

    /// Version couleur (les trois canaux sont égaux si l'image était en gris).
    pub fn en_couleur(&self) -> ImageRvb {
        match self {
            ImageChargee::Gris(i) => i.map(|&g| Rvb::new(g, g, g)),
            ImageChargee::Couleur(i) => i.clone(),
        }
    }
}

fn extension(chemin: &Path) -> String {
    chemin
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// Charge une image en choisissant le format d'après le contenu (PNM) ou l'extension.
pub fn charger(chemin: impl AsRef<Path>) -> Resultat<ImageChargee> {
    let chemin = chemin.as_ref();
    let octets = fs::read(chemin)?;
    if octets.first() == Some(&b'P') && matches!(octets.get(1), Some(b'2' | b'3' | b'5' | b'6')) {
        return decoder_pnm(&octets);
    }
    charger_autre_format(chemin, &octets)
}

/// Charge une image et la convertit en niveaux de gris si besoin.
pub fn charger_gris(chemin: impl AsRef<Path>) -> Resultat<ImageGris> {
    Ok(charger(chemin)?.en_gris())
}

/// Charge une image en couleur.
pub fn charger_couleur(chemin: impl AsRef<Path>) -> Resultat<ImageRvb> {
    Ok(charger(chemin)?.en_couleur())
}

/// Enregistre une image en niveaux de gris (format déduit de l'extension, PGM binaire par défaut).
pub fn enregistrer_gris(img: &ImageGris, chemin: impl AsRef<Path>) -> Resultat<()> {
    let chemin = chemin.as_ref();
    match extension(chemin).as_str() {
        "pgm" | "pnm" | "" => fs::write(chemin, encoder_pgm(img, false))?,
        _ => enregistrer_autre_format(&ImageChargee::Gris(img.clone()), chemin)?,
    }
    Ok(())
}

/// Enregistre une image couleur (format déduit de l'extension, PPM binaire par défaut).
pub fn enregistrer_couleur(img: &ImageRvb, chemin: impl AsRef<Path>) -> Resultat<()> {
    let chemin = chemin.as_ref();
    match extension(chemin).as_str() {
        "ppm" | "pnm" | "" => fs::write(chemin, encoder_ppm(img, false))?,
        _ => enregistrer_autre_format(&ImageChargee::Couleur(img.clone()), chemin)?,
    }
    Ok(())
}

pub fn enregistrer(img: &ImageChargee, chemin: impl AsRef<Path>) -> Resultat<()> {
    match img {
        ImageChargee::Gris(i) => enregistrer_gris(i, chemin),
        ImageChargee::Couleur(i) => enregistrer_couleur(i, chemin),
    }
}

// ---------------------------------------------------------------------------
// PGM / PPM
// ---------------------------------------------------------------------------

/// Encode en PGM : P2 (texte) si `ascii`, sinon P5 (binaire).
pub fn encoder_pgm(img: &ImageGris, ascii: bool) -> Vec<u8> {
    let (l, h) = img.dimensions();
    let mut sortie = Vec::with_capacity(img.nb_pixels() * if ascii { 4 } else { 1 } + 20);
    if ascii {
        let _ = writeln!(sortie, "P2\n{l} {h}\n255");
        for y in 0..h {
            let ligne: Vec<String> = img.ligne(y).iter().map(|p| p.to_string()).collect();
            let _ = writeln!(sortie, "{}", ligne.join(" "));
        }
    } else {
        let _ = write!(sortie, "P5\n{l} {h}\n255\n");
        sortie.extend_from_slice(img.donnees());
    }
    sortie
}

/// Encode en PPM : P3 (texte) si `ascii`, sinon P6 (binaire).
pub fn encoder_ppm(img: &ImageRvb, ascii: bool) -> Vec<u8> {
    let (l, h) = img.dimensions();
    let mut sortie = Vec::with_capacity(img.nb_pixels() * if ascii { 12 } else { 3 } + 20);
    if ascii {
        let _ = writeln!(sortie, "P3\n{l} {h}\n255");
        for y in 0..h {
            let ligne: Vec<String> = img
                .ligne(y)
                .iter()
                .map(|p| format!("{} {} {}", p.r, p.v, p.b))
                .collect();
            let _ = writeln!(sortie, "{}", ligne.join("  "));
        }
    } else {
        let _ = write!(sortie, "P6\n{l} {h}\n255\n");
        for p in img.donnees() {
            sortie.extend_from_slice(&[p.r, p.v, p.b]);
        }
    }
    sortie
}

/// Lecteur d'en-tête PNM : entiers séparés par des blancs, commentaires `#`.
struct Lecteur<'a> {
    octets: &'a [u8],
    pos: usize,
}

impl Lecteur<'_> {
    fn sauter_blancs(&mut self) {
        while let Some(&c) = self.octets.get(self.pos) {
            if c == b'#' {
                while self.octets.get(self.pos).is_some_and(|&c| c != b'\n') {
                    self.pos += 1;
                }
            } else if c.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn entier(&mut self) -> Resultat<usize> {
        self.sauter_blancs();
        let debut = self.pos;
        while self.octets.get(self.pos).is_some_and(u8::is_ascii_digit) {
            self.pos += 1;
        }
        std::str::from_utf8(&self.octets[debut..self.pos])
            .ok()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| Erreur::Format(format!("entier attendu à l'octet {debut}")))
    }
}

/// Décode un fichier PGM/PPM (P2, P3, P5, P6), valeur maximale jusqu'à 65535.
pub fn decoder_pnm(octets: &[u8]) -> Resultat<ImageChargee> {
    if octets.len() < 2 || octets[0] != b'P' {
        return Err(Erreur::Format("signature PNM absente".into()));
    }
    let type_pnm = octets[1];
    let mut lect = Lecteur { octets, pos: 2 };
    let largeur = lect.entier()?;
    let hauteur = lect.entier()?;
    let max = lect.entier()?;
    if max == 0 || max > 65535 {
        return Err(Erreur::Format(format!("valeur maximale {max} invalide")));
    }
    let canaux = match type_pnm {
        b'2' | b'5' => 1,
        b'3' | b'6' => 3,
        _ => {
            return Err(Erreur::Format(format!(
                "type P{} non supporté",
                type_pnm as char
            )))
        }
    };
    let nb = largeur * hauteur * canaux;

    let brutes: Vec<usize> = if matches!(type_pnm, b'2' | b'3') {
        (0..nb).map(|_| lect.entier()).collect::<Resultat<_>>()?
    } else {
        // Un seul blanc sépare la valeur max des données binaires.
        let debut = lect.pos + 1;
        let octets_par_val = if max < 256 { 1 } else { 2 };
        let fin = debut + nb * octets_par_val;
        if octets.len() < fin {
            return Err(Erreur::Format("fichier tronqué".into()));
        }
        let donnees = &octets[debut..fin];
        if octets_par_val == 1 {
            donnees.iter().map(|&o| o as usize).collect()
        } else {
            donnees
                .chunks_exact(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]) as usize)
                .collect()
        }
    };

    let vers_8bits = |v: usize| -> u8 {
        if max == 255 {
            v.min(255) as u8
        } else {
            ((v.min(max) * 255 + max / 2) / max) as u8
        }
    };

    if canaux == 1 {
        let pixels = brutes.into_iter().map(vers_8bits).collect();
        Ok(ImageChargee::Gris(ImageGris::depuis_vec(
            largeur, hauteur, pixels,
        )?))
    } else {
        let pixels = brutes
            .chunks_exact(3)
            .map(|c| Rvb::new(vers_8bits(c[0]), vers_8bits(c[1]), vers_8bits(c[2])))
            .collect();
        Ok(ImageChargee::Couleur(ImageRvb::depuis_vec(
            largeur, hauteur, pixels,
        )?))
    }
}

// ---------------------------------------------------------------------------
// Autres formats (crate `image`)
// ---------------------------------------------------------------------------

#[cfg(feature = "formats")]
fn charger_autre_format(_chemin: &Path, octets: &[u8]) -> Resultat<ImageChargee> {
    let dyn_img = image::load_from_memory(octets).map_err(|e| Erreur::Format(e.to_string()))?;
    let (l, h) = (dyn_img.width() as usize, dyn_img.height() as usize);
    if dyn_img.color().has_color() {
        let rgb = dyn_img.into_rgb8();
        let pixels = rgb.pixels().map(|p| Rvb::new(p[0], p[1], p[2])).collect();
        Ok(ImageChargee::Couleur(ImageRvb::depuis_vec(l, h, pixels)?))
    } else {
        let gris = dyn_img.into_luma8();
        Ok(ImageChargee::Gris(ImageGris::depuis_vec(
            l,
            h,
            gris.into_raw(),
        )?))
    }
}

#[cfg(not(feature = "formats"))]
fn charger_autre_format(chemin: &Path, _octets: &[u8]) -> Resultat<ImageChargee> {
    Err(Erreur::Format(format!(
        "{} : seuls PGM/PPM sont lus sans la fonctionnalité `formats`",
        chemin.display()
    )))
}

#[cfg(feature = "formats")]
fn enregistrer_autre_format(img: &ImageChargee, chemin: &Path) -> Resultat<()> {
    let resultat = match img {
        ImageChargee::Gris(i) => {
            image::GrayImage::from_raw(i.largeur() as u32, i.hauteur() as u32, i.donnees().to_vec())
                .expect("taille cohérente")
                .save(chemin)
        }
        ImageChargee::Couleur(i) => {
            let brut = i.donnees().iter().flat_map(|p| [p.r, p.v, p.b]).collect();
            image::RgbImage::from_raw(i.largeur() as u32, i.hauteur() as u32, brut)
                .expect("taille cohérente")
                .save(chemin)
        }
    };
    resultat.map_err(|e| Erreur::Format(e.to_string()))
}

#[cfg(not(feature = "formats"))]
fn enregistrer_autre_format(_img: &ImageChargee, chemin: &Path) -> Resultat<()> {
    Err(Erreur::Format(format!(
        "{} : seuls PGM/PPM sont écrits sans la fonctionnalité `formats`",
        chemin.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mire() -> ImageGris {
        ImageGris::depuis_fonction(5, 3, |x, y| (x * 50 + y) as u8)
    }

    #[test]
    fn pgm_binaire_aller_retour() {
        let img = mire();
        assert_eq!(
            decoder_pnm(&encoder_pgm(&img, false)).unwrap(),
            ImageChargee::Gris(img)
        );
    }

    #[test]
    fn pgm_ascii_aller_retour() {
        let img = mire();
        assert_eq!(
            decoder_pnm(&encoder_pgm(&img, true)).unwrap(),
            ImageChargee::Gris(img)
        );
    }

    #[test]
    fn ppm_aller_retour() {
        let img = ImageRvb::depuis_fonction(4, 2, |x, y| Rvb::new(x as u8, y as u8, 200));
        for ascii in [false, true] {
            let lu = decoder_pnm(&encoder_ppm(&img, ascii)).unwrap();
            assert_eq!(lu, ImageChargee::Couleur(img.clone()));
        }
    }

    #[test]
    fn commentaires_et_max_different() {
        let texte = b"P2\n# un commentaire\n2 1\n# encore\n15\n0 15\n";
        let lu = decoder_pnm(texte).unwrap();
        assert_eq!(
            lu,
            ImageChargee::Gris(ImageGris::depuis_vec(2, 1, vec![0, 255]).unwrap())
        );
    }

    #[test]
    fn fichier_tronque() {
        assert!(decoder_pnm(b"P5\n4 4\n255\n\x00\x01").is_err());
    }
}
