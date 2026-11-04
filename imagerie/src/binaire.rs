//! Chapitre 7 (première partie) : images binaires.
//!
//! Responsable : Nkolo Atangana Stacy Julie.
//!
//! Distances discrètes, codage de Freeman, étiquetage des composantes connexes,
//! seuillage par hystérésis.

use crate::erreur::Resultat;
use crate::image::{Connexite, Image, ImageBinaire, ImageEtiquettes, ImageReelle};

pub type Point = (usize, usize);

// ---------------------------------------------------------------------------
// Distances
// ---------------------------------------------------------------------------

/// D4 (Manhattan) : |x - s| + |y - t|.
pub fn distance_d4(p: Point, q: Point) -> usize {
    p.0.abs_diff(q.0) + p.1.abs_diff(q.1)
}

/// D8 (échiquier) : max(|x - s|, |y - t|).
pub fn distance_d8(p: Point, q: Point) -> usize {
    p.0.abs_diff(q.0).max(p.1.abs_diff(q.1))
}

/// Distance euclidienne.
pub fn distance_euclidienne(p: Point, q: Point) -> f64 {
    let dx = p.0.abs_diff(q.0) as f64;
    let dy = p.1.abs_diff(q.1) as f64;
    (dx * dx + dy * dy).sqrt()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Distance {
    D4,
    D8,
}

/// Carte des distances : pour chaque pixel objet, distance au fond le plus proche
/// (algorithme en deux passes).
///
/// Les pixels du fond valent 0. Les pixels hors image ne comptent pas comme du fond.
/// S'il n'y a aucun pixel de fond, les pixels objet valent `u32::MAX`.
pub fn carte_distances(img: &ImageBinaire, distance: Distance) -> Image<u32> {
    let (l, h) = img.dimensions();
    let mut d = img.map(|&b| if b { u32::MAX } else { 0 });

    // Voisins déjà visités dans un parcours haut-gauche -> bas-droite.
    let avant: &[(isize, isize)] = match distance {
        Distance::D4 => &[(-1, 0), (0, -1)],
        Distance::D8 => &[(-1, 0), (-1, -1), (0, -1), (1, -1)],
    };
    // Voisins symétriques pour le parcours inverse.
    let arriere: &[(isize, isize)] = match distance {
        Distance::D4 => &[(1, 0), (0, 1)],
        Distance::D8 => &[(1, 0), (1, 1), (0, 1), (-1, 1)],
    };

    // Relaxe le pixel (x, y) : d(x, y) = min(d(x, y), d(voisin) + 1).
    fn relaxer(d: &mut Image<u32>, x: usize, y: usize, voisins: &[(isize, isize)]) {
        let mut meilleur = d[(x, y)];
        for &(dx, dy) in voisins {
            let (nx, ny) = (x as isize + dx, y as isize + dy);
            if d.contient(nx, ny) {
                let candidat = d[(nx as usize, ny as usize)].saturating_add(1);
                meilleur = meilleur.min(candidat);
            }
        }
        d[(x, y)] = meilleur;
    }

    // Passe aller : haut-gauche -> bas-droite.
    for y in 0..h {
        for x in 0..l {
            if img[(x, y)] {
                relaxer(&mut d, x, y, avant);
            }
        }
    }
    // Passe retour : bas-droite -> haut-gauche.
    for y in (0..h).rev() {
        for x in (0..l).rev() {
            if img[(x, y)] {
                relaxer(&mut d, x, y, arriere);
            }
        }
    }
    d
}

// ---------------------------------------------------------------------------
// Code de Freeman
// ---------------------------------------------------------------------------

/// Contour codé par la chaîne de Freeman.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeFreeman {
    pub depart: Point,
    pub connexite: Connexite,
    /// Directions 0..4 (connexité 4) ou 0..8 (connexité 8), 0 = vers la droite,
    /// numérotées dans le sens trigonométrique comme dans le cours.
    pub codes: Vec<u8>,
}

/// Le voisin de `p` dans la direction (dx, dy), s'il est dans l'image et objet.
fn voisin_objet(img: &ImageBinaire, p: Point, (dx, dy): (isize, isize)) -> Option<Point> {
    let (nx, ny) = (p.0 as isize + dx, p.1 as isize + dy);
    if img.contient(nx, ny) && img[(nx as usize, ny as usize)] {
        Some((nx as usize, ny as usize))
    } else {
        None
    }
}

/// Connexité 4 : on longe le fond qui est à gauche de la marche (sens horaire).
/// À chaque pas on essaie : tourner à gauche, tout droit, tourner à droite, demi-tour.
/// `din` est la direction du dernier déplacement.
fn suivant_n4(img: &ImageBinaire, pos: Point, din: usize) -> Option<usize> {
    let table = Connexite::Quatre.voisins();
    [1, 0, 3, 2]
        .into_iter()
        .map(|t| (din + t) % 4)
        .find(|&d| voisin_objet(img, pos, table[d]).is_some())
}

/// Connexité 8 : suivi de contour de Moore. On tourne dans le sens horaire (indices
/// décroissants) en partant de `debut`, et on prend le premier voisin objet.
fn suivant_n8(img: &ImageBinaire, pos: Point, debut: usize) -> Option<usize> {
    let table = Connexite::Huit.voisins();
    (0..8)
        .map(|k| (debut + 8 - k) % 8)
        .find(|&d| voisin_objet(img, pos, table[d]).is_some())
}

/// Code le contour de l'objet en partant du pixel le plus en haut à gauche et en
/// tournant dans le sens des aiguilles d'une montre. `None` si l'image est vide.
///
/// Seul le contour de la première composante rencontrée (ordre de lecture) est codé.
/// Un pixel isolé donne un code vide.
pub fn freeman(img: &ImageBinaire, connexite: Connexite) -> Option<CodeFreeman> {
    let depart = img.pixels().find(|&(_, _, &p)| p).map(|(x, y, _)| (x, y))?;
    let table = connexite.voisins();
    let n4 = connexite == Connexite::Quatre;

    let mut codes = Vec::new();
    let mut pos = depart;
    let mut premier: Option<usize> = None;
    // N4 : direction du dernier déplacement (on fait comme si on arrivait par la gauche).
    // N8 : première direction testée (NO : haut, gauche et haut-gauche sont du fond).
    let mut etat = if n4 { 0 } else { 3 };
    let limite = 8 * img.nb_pixels() + 8; // garde-fou

    for _ in 0..limite {
        let suivant = if n4 {
            suivant_n4(img, pos, etat)
        } else {
            suivant_n8(img, pos, etat)
        };
        let Some(d) = suivant else {
            break; // pixel isolé
        };

        // Arrêt : on repart du départ dans la même direction que la première fois.
        match premier {
            None => premier = Some(d),
            Some(p) if pos == depart && d == p => break,
            Some(_) => {}
        }

        codes.push(d as u8);
        let (dx, dy) = table[d];
        pos = (
            (pos.0 as isize + dx) as usize,
            (pos.1 as isize + dy) as usize,
        );
        etat = if n4 {
            d
        } else if d % 2 == 0 {
            (d + 1) % 8
        } else {
            (d + 2) % 8
        };
    }

    Some(CodeFreeman {
        depart,
        connexite,
        codes,
    })
}

/// Points du contour retrouvés à partir du code.
///
/// Le premier point est `depart` ; pour un contour fermé le dernier point est de
/// nouveau `depart`. Un code invalide ou qui sort du quadrant positif arrête le décodage.
pub fn decoder_freeman(code: &CodeFreeman) -> Vec<Point> {
    let table = code.connexite.voisins();
    let mut p = code.depart;
    let mut points = vec![p];
    for &u in &code.codes {
        let Some(&(dx, dy)) = table.get(u as usize) else {
            break;
        };
        let (nx, ny) = (p.0 as isize + dx, p.1 as isize + dy);
        if nx < 0 || ny < 0 {
            break;
        }
        p = (nx as usize, ny as usize);
        points.push(p);
    }
    points
}

// ---------------------------------------------------------------------------
// Étiquetage en composantes connexes
// ---------------------------------------------------------------------------

/// Voisins qui précèdent le pixel dans l'ordre de lecture (haut et gauche).
fn voisins_avant(c: Connexite) -> Vec<(isize, isize)> {
    c.voisins()
        .iter()
        .copied()
        .filter(|&(dx, dy)| dy < 0 || (dy == 0 && dx < 0))
        .collect()
}

/// Voisins qui suivent le pixel dans l'ordre de lecture (bas et droite).
fn voisins_apres(c: Connexite) -> Vec<(isize, isize)> {
    c.voisins()
        .iter()
        .copied()
        .filter(|&(dx, dy)| dy > 0 || (dy == 0 && dx > 0))
        .collect()
}

/// Plus petite étiquette non nulle parmi les voisins indiqués.
fn min_voisins(
    et: &ImageEtiquettes,
    x: usize,
    y: usize,
    voisins: &[(isize, isize)],
) -> Option<u32> {
    voisins
        .iter()
        .filter_map(|&(dx, dy)| {
            let (nx, ny) = (x as isize + dx, y as isize + dy);
            if et.contient(nx, ny) {
                let e = et[(nx as usize, ny as usize)];
                (e != 0).then_some(e)
            } else {
                None
            }
        })
        .min()
}

/// Renumérote les étiquettes en 1..=n dans l'ordre de lecture. Renvoie n.
/// Les deux méthodes d'étiquetage donnent ainsi exactement la même image.
fn renumeroter(et: &mut ImageEtiquettes) -> u32 {
    let mut table = vec![0u32; et.etiquette_max() as usize + 1];
    let mut n = 0;
    for e in et.donnees_mut() {
        if *e != 0 {
            let nouvelle = &mut table[*e as usize];
            if *nouvelle == 0 {
                n += 1;
                *nouvelle = n;
            }
            *e = *nouvelle;
        }
    }
    n
}

/// Étiquetage par parcours successifs (méthode du cours) : premier parcours avec
/// les voisins haut/gauche, puis parcours alternés jusqu'à stabilité.
/// Renvoie les étiquettes renumérotées 1..=n et le nombre n de composantes.
pub fn etiqueter(img: &ImageBinaire, connexite: Connexite) -> (ImageEtiquettes, u32) {
    let (l, h) = img.dimensions();
    let mut et = ImageEtiquettes::nouvelle(l, h, 0);
    let avant = voisins_avant(connexite);
    let apres = voisins_apres(connexite);

    // Premier parcours : nouvelle étiquette si aucun voisin (haut/gauche) n'est objet,
    // sinon le minimum de leurs étiquettes.
    let mut derniere = 0u32;
    for y in 0..h {
        for x in 0..l {
            if img[(x, y)] {
                let e = match min_voisins(&et, x, y, &avant) {
                    Some(m) => m,
                    None => {
                        derniere += 1;
                        derniere
                    }
                };
                et[(x, y)] = e;
            }
        }
    }

    // Parcours alternés (retour puis aller) : un pixel prend le minimum de son
    // étiquette et de celles des voisins déjà visités dans ce sens.
    loop {
        let mut change = false;
        for y in (0..h).rev() {
            for x in (0..l).rev() {
                if img[(x, y)] {
                    if let Some(m) = min_voisins(&et, x, y, &apres) {
                        if m < et[(x, y)] {
                            et[(x, y)] = m;
                            change = true;
                        }
                    }
                }
            }
        }
        for y in 0..h {
            for x in 0..l {
                if img[(x, y)] {
                    if let Some(m) = min_voisins(&et, x, y, &avant) {
                        if m < et[(x, y)] {
                            et[(x, y)] = m;
                            change = true;
                        }
                    }
                }
            }
        }
        if !change {
            break;
        }
    }

    let n = renumeroter(&mut et);
    (et, n)
}

/// Racine de `i` dans la table d'équivalences (avec compression de chemin).
fn trouver(parent: &mut [u32], mut i: u32) -> u32 {
    while parent[i as usize] != i {
        let pere = parent[i as usize];
        parent[i as usize] = parent[pere as usize];
        i = parent[i as usize];
    }
    i
}

/// Déclare que les étiquettes `a` et `b` sont la même région (la plus petite racine gagne).
fn unir(parent: &mut [u32], a: u32, b: u32) {
    let (ra, rb) = (trouver(parent, a), trouver(parent, b));
    if ra < rb {
        parent[rb as usize] = ra;
    } else if rb < ra {
        parent[ra as usize] = rb;
    }
}

/// Étiquetage en un seul parcours avec table d'équivalences (union-find).
/// Doit donner le même découpage que `etiqueter`.
pub fn etiqueter_une_passe(img: &ImageBinaire, connexite: Connexite) -> (ImageEtiquettes, u32) {
    let (l, h) = img.dimensions();
    let mut et = ImageEtiquettes::nouvelle(l, h, 0);
    let avant = voisins_avant(connexite);
    // parent[e] = représentant de l'étiquette e ; l'indice 0 (fond) est inutilisé.
    let mut parent: Vec<u32> = vec![0];

    for y in 0..h {
        for x in 0..l {
            if !img[(x, y)] {
                continue;
            }
            let e = match min_voisins(&et, x, y, &avant) {
                None => {
                    // Aucun voisin déjà étiqueté : nouvelle région provisoire.
                    let nouvelle = parent.len() as u32;
                    parent.push(nouvelle);
                    nouvelle
                }
                Some(m) => {
                    // Tous les voisins étiquetés appartiennent à la même région
                    // que ce pixel : on note les équivalences.
                    for &(dx, dy) in &avant {
                        let (nx, ny) = (x as isize + dx, y as isize + dy);
                        if et.contient(nx, ny) {
                            let v = et[(nx as usize, ny as usize)];
                            if v != 0 {
                                unir(&mut parent, m, v);
                            }
                        }
                    }
                    m
                }
            };
            et[(x, y)] = e;
        }
    }

    // Remplace chaque étiquette provisoire par son représentant, puis renumérote.
    for e in et.donnees_mut() {
        if *e != 0 {
            *e = trouver(&mut parent, *e);
        }
    }
    let n = renumeroter(&mut et);
    (et, n)
}

/// Nombre d'objets (composantes connexes).
pub fn compter_objets(img: &ImageBinaire, connexite: Connexite) -> u32 {
    etiqueter(img, connexite).1
}

/// Surface (en pixels) de chaque étiquette ; indice 0 = fond.
pub fn surfaces(etiquettes: &ImageEtiquettes) -> Vec<usize> {
    let mut s = vec![0usize; etiquettes.etiquette_max() as usize + 1];
    for &e in etiquettes.donnees() {
        s[e as usize] += 1;
    }
    s
}

/// Nettoyage post-segmentation : supprime les objets de moins de `surface_min` pixels.
pub fn supprimer_petits_objets(
    img: &ImageBinaire,
    surface_min: usize,
    connexite: Connexite,
) -> ImageBinaire {
    let (et, _) = etiqueter(img, connexite);
    let s = surfaces(&et);
    et.map(|&e| e != 0 && s[e as usize] >= surface_min)
}

// ---------------------------------------------------------------------------
// Hystérésis
// ---------------------------------------------------------------------------

/// Seuillage par hystérésis : on garde les pixels > `haut`, puis on ajoute ceux > `bas`
/// qui leur sont connectés (8-connexité).
pub fn seuillage_hysteresis(norme: &ImageReelle, bas: f64, haut: f64) -> Resultat<ImageBinaire> {
    let (l, h) = norme.dimensions();
    let mut res = ImageBinaire::nouvelle(l, h, false);
    let mut pile: Vec<Point> = Vec::new();

    // Graines : les pixels forts.
    for (x, y, &v) in norme.pixels() {
        if v > haut {
            res[(x, y)] = true;
            pile.push((x, y));
        }
    }

    // Propagation : les pixels > bas voisins d'un pixel retenu sont retenus à leur tour.
    let voisins = Connexite::Huit.voisins();
    while let Some((x, y)) = pile.pop() {
        for &(dx, dy) in voisins {
            let (nx, ny) = (x as isize + dx, y as isize + dy);
            if !norme.contient(nx, ny) {
                continue;
            }
            let q = (nx as usize, ny as usize);
            if !res[q] && norme[q] > bas {
                res[q] = true;
                pile.push(q);
            }
        }
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Construit une image binaire à partir de lignes de texte ('#' = objet).
    fn img(lignes: &[&str]) -> ImageBinaire {
        let h = lignes.len();
        let l = lignes[0].len();
        let d = lignes
            .iter()
            .flat_map(|s| s.bytes().map(|c| c == b'#'))
            .collect();
        ImageBinaire::depuis_vec(l, h, d).unwrap()
    }

    #[test]
    fn distances() {
        assert_eq!(distance_d4((1, 1), (4, 5)), 7);
        assert_eq!(distance_d8((1, 1), (4, 5)), 4);
        assert!((distance_euclidienne((1, 1), (4, 5)) - 5.0).abs() < 1e-12);
    }

    #[test]
    fn carte_de_distances() {
        let i = img(&["#####", "#####", "#####", "#####", "#####", "..#.."]);
        let c4 = carte_distances(&i, Distance::D4);
        let c8 = carte_distances(&i, Distance::D8);
        assert_eq!(c4[(0, 0)], 5);
        assert_eq!(c4[(2, 4)], 2);
        assert_eq!(c4[(2, 0)], 6);
        assert_eq!(c4[(2, 5)], 1);
        assert_eq!(c4[(0, 5)], 0);
        assert_eq!(c8[(0, 0)], 5);
        assert_eq!(c8[(2, 4)], 1);
        assert_eq!(c8[(2, 2)], 3);
    }

    #[test]
    fn freeman_carre() {
        let i = img(&["....", ".##.", ".##.", "...."]);
        let c4 = freeman(&i, Connexite::Quatre).unwrap();
        assert_eq!(c4.depart, (1, 1));
        assert_eq!(c4.codes, vec![0, 3, 2, 1]);
        let c8 = freeman(&i, Connexite::Huit).unwrap();
        assert_eq!(c8.codes, vec![0, 6, 4, 2]);
        let pts = decoder_freeman(&c8);
        assert_eq!(pts, vec![(1, 1), (2, 1), (2, 2), (1, 2), (1, 1)]);
    }

    #[test]
    fn freeman_cas_limites() {
        assert!(freeman(&img(&["...", "..."]), Connexite::Huit).is_none());
        let seul = freeman(&img(&["...", ".#.", "..."]), Connexite::Quatre).unwrap();
        assert!(seul.codes.is_empty());
        // Un L : le décodage doit refermer le contour.
        let l = img(&["#..", "#..", "###"]);
        for c in [Connexite::Quatre, Connexite::Huit] {
            let code = freeman(&l, c).unwrap();
            let pts = decoder_freeman(&code);
            assert_eq!(pts.first(), pts.last());
        }
    }

    #[test]
    fn etiquetage_u_et_diagonales() {
        // Le U oblige à fusionner deux étiquettes.
        let u = img(&["#.#", "#.#", "###"]);
        assert_eq!(compter_objets(&u, Connexite::Quatre), 1);
        let d = img(&["#..", ".#.", "..#"]);
        assert_eq!(compter_objets(&d, Connexite::Quatre), 3);
        assert_eq!(compter_objets(&d, Connexite::Huit), 1);
    }

    #[test]
    fn les_deux_methodes_concordent() {
        let i = img(&[
            "##..#.##", "#..##..#", "#.#...##", "#.#.##..", "###..#.#", "....#..#",
        ]);
        for c in [Connexite::Quatre, Connexite::Huit] {
            assert_eq!(etiqueter(&i, c), etiqueter_une_passe(&i, c));
        }
    }

    #[test]
    fn petits_objets() {
        let i = img(&["##..#", "##...", "....#", "..###"]);
        let (et, n) = etiqueter(&i, Connexite::Quatre);
        assert_eq!(n, 3);
        assert_eq!(surfaces(&et), vec![11, 4, 1, 4]);
        let net = supprimer_petits_objets(&i, 2, Connexite::Quatre);
        assert_eq!(net.nb_pixels_objet(), 8);
        assert!(!net[(4, 0)]);
    }

    #[test]
    fn hysteresis() {
        let n = ImageReelle::depuis_vec(5, 1, vec![9.0, 5.0, 1.0, 5.0, 5.0]).unwrap();
        let r = seuillage_hysteresis(&n, 4.0, 8.0).unwrap();
        assert_eq!(r.donnees(), &[true, true, false, false, false]);
    }
}
