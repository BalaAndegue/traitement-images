//! Chapitre 6 : segmentation.
//!
//! Responsable : Nkamla Chedjou Johan.
//!
//! Seuillage (simple, multiple, automatique, adaptatif), k-moyennes,
//! division-fusion et croissance de régions.

use std::collections::{HashSet, VecDeque};

use crate::alea::Alea;
use crate::erreur::{Erreur, Resultat};
use crate::image::{Connexite, ImageBinaire, ImageEtiquettes, ImageGris, ImageRvb, Rvb};

// ---------------------------------------------------------------------------
// Outils internes
// ---------------------------------------------------------------------------

/// Histogramme H(k) d'un ensemble de pixels.
///
/// `ponctuel::histogramme` (Jeff) n'est pas encore écrit sur `main` : on garde
/// cette version privée en attendant. Elle pourra être remplacée par
/// `crate::ponctuel::histogramme(img)` pour l'image entière.
fn histogramme_pixels<'a>(pixels: impl Iterator<Item = &'a u8>) -> [u32; 256] {
    let mut h = [0u32; 256];
    for &p in pixels {
        h[p as usize] += 1;
    }
    h
}

fn histogramme_image(img: &ImageGris) -> [u32; 256] {
    histogramme_pixels(img.donnees().iter())
}

/// Parmi les valeurs `cout[t]` définies, renvoie le milieu du premier palier
/// de valeurs minimales. Quand plusieurs seuils sont également bons (par exemple
/// tous ceux situés dans le creux entre deux modes séparés), on prend celui du
/// milieu plutôt que le premier.
fn milieu_palier_minimal(cout: &[Option<f64>]) -> Option<usize> {
    let min = cout.iter().flatten().copied().fold(f64::INFINITY, f64::min);
    if !min.is_finite() {
        return None;
    }
    let tol = 1e-9 * (1.0 + min.abs());
    let debut = cout
        .iter()
        .position(|c| matches!(c, Some(v) if *v <= min + tol))?;
    let mut fin = debut;
    while fin + 1 < cout.len() && matches!(cout[fin + 1], Some(v) if v <= min + tol) {
        fin += 1;
    }
    Some((debut + fin) / 2)
}

/// Statistiques d'une classe [a, b) de l'histogramme : (effectif, moyenne, variance).
fn stats_classe(h: &[u32; 256], a: usize, b: usize) -> (f64, f64, f64) {
    let mut n = 0.0;
    let mut s = 0.0;
    let mut s2 = 0.0;
    for (i, &c) in h.iter().enumerate().take(b).skip(a) {
        let c = c as f64;
        let i = i as f64;
        n += c;
        s += c * i;
        s2 += c * i * i;
    }
    if n == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let mu = s / n;
    (n, mu, (s2 / n - mu * mu).max(0.0))
}

/// Seuil d'Otsu calculé sur un histogramme.
fn otsu_histogramme(h: &[u32; 256]) -> u8 {
    let total: f64 = h.iter().map(|&c| c as f64).sum();
    if total == 0.0 {
        return 0;
    }
    // cout[t] = sigma_w²(t) pour T = t, classe 1 = [0, T - 1], classe 2 = [T, 255].
    let mut cout = vec![None; 256];
    for (t, c) in cout.iter_mut().enumerate().skip(1) {
        let (n1, _, v1) = stats_classe(h, 0, t);
        let (n2, _, v2) = stats_classe(h, t, 256);
        if n1 == 0.0 || n2 == 0.0 {
            continue;
        }
        *c = Some(n1 / total * v1 + n2 / total * v2);
    }
    match milieu_palier_minimal(&cout) {
        Some(t) => t as u8,
        // Image constante : un seul niveau, on le prend comme seuil.
        None => h.iter().position(|&c| c > 0).unwrap_or(0) as u8,
    }
}

fn moyenne_variance(pixels: &[u8]) -> (f64, f64) {
    if pixels.is_empty() {
        return (0.0, 0.0);
    }
    let n = pixels.len() as f64;
    let s: f64 = pixels.iter().map(|&p| p as f64).sum();
    let s2: f64 = pixels.iter().map(|&p| (p as f64) * (p as f64)).sum();
    let mu = s / n;
    (mu, (s2 / n - mu * mu).max(0.0))
}

// ---------------------------------------------------------------------------
// Seuillage
// ---------------------------------------------------------------------------

/// Seuillage simple : 1 si f(x, y) >= T, 0 sinon.
pub fn seuillage(img: &ImageGris, seuil: u8) -> ImageBinaire {
    img.map(|&p| p >= seuil)
}

/// Multi-seuils : n seuils triés donnent n + 1 classes numérotées 0..=n.
pub fn seuillage_multiple(img: &ImageGris, seuils: &[u8]) -> Resultat<ImageEtiquettes> {
    if seuils.is_empty() {
        return Err(Erreur::Parametre(
            "il faut au moins un seuil pour le seuillage multiple".into(),
        ));
    }
    if seuils.windows(2).any(|w| w[0] >= w[1]) {
        return Err(Erreur::Parametre(format!(
            "les seuils doivent être strictement croissants : {seuils:?}"
        )));
    }
    // Classe d'un pixel = nombre de seuils qu'il atteint (f >= T_i).
    let mut lut = [0u32; 256];
    for (v, classe) in lut.iter_mut().enumerate() {
        *classe = seuils.iter().filter(|&&t| v >= t as usize).count() as u32;
    }
    Ok(img.map(|&p| lut[p as usize]))
}

/// Seuil = moyenne des niveaux de gris.
pub fn seuil_moyenne(img: &ImageGris) -> u8 {
    moyenne_variance(img.donnees()).0.round() as u8
}

/// Seuil = milieu entre le niveau min et le niveau max.
pub fn seuil_milieu(img: &ImageGris) -> u8 {
    let min = img.donnees().iter().copied().min().unwrap_or(0) as f64;
    let max = img.donnees().iter().copied().max().unwrap_or(0) as f64;
    ((min + max) / 2.0).round() as u8
}

/// Seuil qui équilibre les deux parties de l'histogramme (médiane des pixels).
pub fn seuil_median(img: &ImageGris) -> u8 {
    let h = histogramme_image(img);
    let total = img.nb_pixels() as f64;
    if total == 0.0 {
        return 0;
    }
    // cout[t] = | #{f < t} - #{f >= t} | ; on prend le milieu du meilleur palier.
    let mut cout = vec![None; 256];
    let mut dessous = 0.0;
    for (t, c) in cout.iter_mut().enumerate() {
        *c = Some((dessous - (total - dessous)).abs());
        dessous += h[t] as f64;
    }
    milieu_palier_minimal(&cout).unwrap_or(0) as u8
}

/// Algorithme d'Otsu : le seuil qui minimise la variance intra-classes
/// sigma_w² = P1 sigma1² + P2 sigma2².
pub fn seuil_otsu(img: &ImageGris) -> u8 {
    otsu_histogramme(&histogramme_image(img))
}

/// Bonus : méthode de Kittler (erreur minimale, modèle gaussien).
pub fn seuil_kittler(img: &ImageGris) -> u8 {
    let h = histogramme_image(img);
    let total = img.nb_pixels() as f64;
    if total == 0.0 {
        return 0;
    }
    // J(T) = 1 + 2 (P1 ln s1 + P2 ln s2) - 2 (P1 ln P1 + P2 ln P2)
    let mut cout = vec![None; 256];
    for (t, c) in cout.iter_mut().enumerate().skip(1) {
        let (n1, _, v1) = stats_classe(&h, 0, t);
        let (n2, _, v2) = stats_classe(&h, t, 256);
        if n1 == 0.0 || n2 == 0.0 || v1 <= 0.0 || v2 <= 0.0 {
            continue;
        }
        let (p1, p2) = (n1 / total, n2 / total);
        *c = Some(
            1.0 + 2.0 * (p1 * v1.sqrt().ln() + p2 * v2.sqrt().ln())
                - 2.0 * (p1 * p1.ln() + p2 * p2.ln()),
        );
    }
    match milieu_palier_minimal(&cout) {
        Some(t) => t as u8,
        // Aucune coupure avec deux classes de variance non nulle (image à un ou
        // deux niveaux) : le modèle gaussien ne s'applique pas, on prend Otsu.
        None => otsu_histogramme(&h),
    }
}

/// Seuillage local adaptatif : on découpe l'image en blocs `taille_bloc` x `taille_bloc`,
/// chaque bloc est seuillé avec son propre seuil d'Otsu, sauf si sa variance est
/// inférieure à `variance_min` (100 dans le cours) : il est alors entièrement fond ou objet
/// selon sa moyenne comparée au seuil global.
pub fn seuillage_adaptatif(
    img: &ImageGris,
    taille_bloc: usize,
    variance_min: f64,
) -> Resultat<ImageBinaire> {
    if taille_bloc == 0 {
        return Err(Erreur::Parametre(
            "la taille des blocs doit être au moins 1".into(),
        ));
    }
    if variance_min.is_nan() || variance_min < 0.0 {
        return Err(Erreur::Parametre(format!(
            "variance minimale invalide : {variance_min}"
        )));
    }
    let (l, h) = img.dimensions();
    let seuil_global = seuil_otsu(img) as f64;
    let mut sortie = ImageBinaire::nouvelle(l, h, false);
    let mut bloc = Vec::with_capacity(taille_bloc * taille_bloc);
    for y0 in (0..h).step_by(taille_bloc) {
        for x0 in (0..l).step_by(taille_bloc) {
            let (x1, y1) = ((x0 + taille_bloc).min(l), (y0 + taille_bloc).min(h));
            bloc.clear();
            for y in y0..y1 {
                bloc.extend_from_slice(&img.ligne(y)[x0..x1]);
            }
            let (moyenne, variance) = moyenne_variance(&bloc);
            let decision: Box<dyn Fn(u8) -> bool> = if variance < variance_min {
                let objet = moyenne >= seuil_global;
                Box::new(move |_| objet)
            } else {
                let t = otsu_histogramme(&histogramme_pixels(bloc.iter()));
                Box::new(move |p| p >= t)
            };
            for y in y0..y1 {
                for x in x0..x1 {
                    sortie[(x, y)] = decision(img[(x, y)]);
                }
            }
        }
    }
    Ok(sortie)
}

// ---------------------------------------------------------------------------
// k-moyennes
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct KMoyennes<C> {
    /// Étiquette 1..=k de chaque pixel.
    pub etiquettes: ImageEtiquettes,
    /// Centre de chaque groupe (indice i -> étiquette i + 1).
    pub centres: Vec<C>,
    pub iterations: usize,
}

fn verifier_kmoyennes(k: usize, iterations_max: usize, distincts: usize) -> Resultat<()> {
    if k == 0 {
        return Err(Erreur::Parametre("k doit être au moins 1".into()));
    }
    if iterations_max == 0 {
        return Err(Erreur::Parametre(
            "il faut au moins une itération pour les k-moyennes".into(),
        ));
    }
    if distincts < k {
        return Err(Erreur::Parametre(format!(
            "{k} groupes demandés mais l'image n'a que {distincts} valeurs distinctes"
        )));
    }
    Ok(())
}

/// Tire `k` indices distincts parmi `0..n` (mélange de Fisher-Yates partiel).
fn tirer_indices(n: usize, k: usize, graine: u64) -> Vec<usize> {
    let mut alea = Alea::new(graine);
    let mut indices: Vec<usize> = (0..n).collect();
    for i in 0..k {
        let j = i + alea.entier(n - i);
        indices.swap(i, j);
    }
    indices.truncate(k);
    indices
}

/// k-moyennes sur l'intensité.
pub fn kmoyennes_gris(
    img: &ImageGris,
    k: usize,
    iterations_max: usize,
    graine: u64,
) -> Resultat<KMoyennes<f64>> {
    // Tous les pixels d'un même niveau tombent dans le même groupe : on travaille
    // sur l'histogramme (256 entrées) au lieu des M x N pixels.
    let h = histogramme_image(img);
    let niveaux: Vec<usize> = (0..256).filter(|&v| h[v] > 0).collect();
    verifier_kmoyennes(k, iterations_max, niveaux.len())?;

    // Étape 1 : k niveaux distincts tirés au hasard comme centres initiaux.
    let mut centres: Vec<f64> = tirer_indices(niveaux.len(), k, graine)
        .into_iter()
        .map(|i| niveaux[i] as f64)
        .collect();
    let mut groupe = [usize::MAX; 256];
    let mut iterations = 0;
    while iterations < iterations_max {
        iterations += 1;
        // Étape 3 : affectation au centre le plus proche.
        let mut change = false;
        for &v in &niveaux {
            let g = plus_proche(&centres, |c| (v as f64 - c).abs());
            if groupe[v] != g {
                groupe[v] = g;
                change = true;
            }
        }
        if !change {
            break;
        }
        // Étape 2 : nouveaux centres = moyenne des pixels du groupe.
        let mut somme = vec![0.0; k];
        let mut nb = vec![0.0; k];
        for &v in &niveaux {
            somme[groupe[v]] += h[v] as f64 * v as f64;
            nb[groupe[v]] += h[v] as f64;
        }
        for i in 0..k {
            if nb[i] > 0.0 {
                centres[i] = somme[i] / nb[i];
            }
        }
    }

    // Groupes numérotés du plus sombre au plus clair.
    let mut ordre: Vec<usize> = (0..k).collect();
    ordre.sort_by(|&a, &b| centres[a].total_cmp(&centres[b]));
    let mut rang = vec![0u32; k];
    for (r, &i) in ordre.iter().enumerate() {
        rang[i] = r as u32 + 1;
    }
    let etiquettes = img.map(|&p| rang[groupe[p as usize]]);
    Ok(KMoyennes {
        etiquettes,
        centres: ordre.iter().map(|&i| centres[i]).collect(),
        iterations,
    })
}

fn plus_proche<C: Copy>(centres: &[C], distance: impl Fn(C) -> f64) -> usize {
    let mut meilleur = 0;
    let mut dmin = f64::INFINITY;
    for (i, &c) in centres.iter().enumerate() {
        let d = distance(c);
        if d < dmin {
            dmin = d;
            meilleur = i;
        }
    }
    meilleur
}

fn distance2_rvb(p: Rvb, c: [f64; 3]) -> f64 {
    let dr = p.r as f64 - c[0];
    let dv = p.v as f64 - c[1];
    let db = p.b as f64 - c[2];
    dr * dr + dv * dv + db * db
}

/// k-moyennes sur la couleur (distance euclidienne dans RVB).
pub fn kmoyennes_couleur(
    img: &ImageRvb,
    k: usize,
    iterations_max: usize,
    graine: u64,
) -> Resultat<KMoyennes<[f64; 3]>> {
    let mut distinctes: Vec<Rvb> = img
        .donnees()
        .iter()
        .copied()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    // Ordre fixe pour que la graine donne toujours le même résultat.
    distinctes.sort_by_key(|c| (c.r, c.v, c.b));
    verifier_kmoyennes(k, iterations_max, distinctes.len())?;

    let mut centres: Vec<[f64; 3]> = tirer_indices(distinctes.len(), k, graine)
        .into_iter()
        .map(|i| {
            let c = distinctes[i];
            [c.r as f64, c.v as f64, c.b as f64]
        })
        .collect();
    let mut groupe = vec![usize::MAX; img.nb_pixels()];
    let mut iterations = 0;
    while iterations < iterations_max {
        iterations += 1;
        let mut change = false;
        for (g, &p) in groupe.iter_mut().zip(img.donnees()) {
            let n = plus_proche(&centres, |c| distance2_rvb(p, c));
            if *g != n {
                *g = n;
                change = true;
            }
        }
        if !change {
            break;
        }
        let mut somme = vec![[0.0; 3]; k];
        let mut nb = vec![0.0; k];
        for (&g, &p) in groupe.iter().zip(img.donnees()) {
            somme[g][0] += p.r as f64;
            somme[g][1] += p.v as f64;
            somme[g][2] += p.b as f64;
            nb[g] += 1.0;
        }
        for i in 0..k {
            if nb[i] > 0.0 {
                centres[i] = [
                    somme[i][0] / nb[i],
                    somme[i][1] / nb[i],
                    somme[i][2] / nb[i],
                ];
            }
        }
    }

    // Groupes numérotés par luminance croissante (0,299 R + 0,587 V + 0,114 B).
    let lum = |c: &[f64; 3]| 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
    let mut ordre: Vec<usize> = (0..k).collect();
    ordre.sort_by(|&a, &b| lum(&centres[a]).total_cmp(&lum(&centres[b])));
    let mut rang = vec![0u32; k];
    for (r, &i) in ordre.iter().enumerate() {
        rang[i] = r as u32 + 1;
    }
    let etiquettes = ImageEtiquettes::depuis_vec(
        img.largeur(),
        img.hauteur(),
        groupe.iter().map(|&g| rang[g]).collect(),
    )?;
    Ok(KMoyennes {
        etiquettes,
        centres: ordre.iter().map(|&i| centres[i]).collect(),
        iterations,
    })
}

/// Remplace chaque pixel par le centre de son groupe (image « posterisée »).
pub fn image_kmoyennes_couleur(resultat: &KMoyennes<[f64; 3]>) -> ImageRvb {
    let vers_u8 = |v: f64| v.round().clamp(0.0, 255.0) as u8;
    resultat.etiquettes.map(|&e| {
        match (e as usize)
            .checked_sub(1)
            .and_then(|i| resultat.centres.get(i))
        {
            Some(c) => Rvb::new(vers_u8(c[0]), vers_u8(c[1]), vers_u8(c[2])),
            None => Rvb::default(),
        }
    })
}

// ---------------------------------------------------------------------------
// Division-fusion
// ---------------------------------------------------------------------------

/// Prédicat d'homogénéité d'un bloc ou d'une région.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Homogeneite {
    /// Variance <= valeur.
    Variance(f64),
    /// max - min <= valeur.
    MaxMin(u8),
}

/// Statistiques cumulables d'une région : on peut tester l'homogénéité de
/// l'union de deux régions sans reparcourir leurs pixels.
#[derive(Clone, Copy, Debug)]
struct Stats {
    n: f64,
    somme: f64,
    somme2: f64,
    min: u8,
    max: u8,
}

impl Stats {
    fn bloc(img: &ImageGris, (x0, y0, l, h): (usize, usize, usize, usize)) -> Stats {
        let mut s = Stats {
            n: 0.0,
            somme: 0.0,
            somme2: 0.0,
            min: u8::MAX,
            max: 0,
        };
        for y in y0..y0 + h {
            for &p in &img.ligne(y)[x0..x0 + l] {
                let v = p as f64;
                s.n += 1.0;
                s.somme += v;
                s.somme2 += v * v;
                s.min = s.min.min(p);
                s.max = s.max.max(p);
            }
        }
        s
    }

    fn union(&self, o: &Stats) -> Stats {
        Stats {
            n: self.n + o.n,
            somme: self.somme + o.somme,
            somme2: self.somme2 + o.somme2,
            min: self.min.min(o.min),
            max: self.max.max(o.max),
        }
    }

    fn moyenne(&self) -> f64 {
        self.somme / self.n.max(1.0)
    }

    fn homogene(&self, critere: Homogeneite) -> bool {
        if self.n == 0.0 {
            return true;
        }
        match critere {
            Homogeneite::Variance(v) => {
                let mu = self.moyenne();
                (self.somme2 / self.n - mu * mu).max(0.0) <= v
            }
            Homogeneite::MaxMin(m) => self.max - self.min <= m,
        }
    }
}

fn verifier_critere(critere: Homogeneite) -> Resultat<()> {
    if let Homogeneite::Variance(v) = critere {
        if v.is_nan() || v < 0.0 {
            return Err(Erreur::Parametre(format!(
                "seuil de variance invalide : {v}"
            )));
        }
    }
    Ok(())
}

/// Division-fusion (split and merge) : division récursive en 4 (quadtree) des blocs
/// non homogènes jusqu'à `taille_min`, puis fusion des blocs adjacents dont l'union
/// reste homogène.
pub fn division_fusion(
    img: &ImageGris,
    critere: Homogeneite,
    taille_min: usize,
) -> Resultat<ImageEtiquettes> {
    verifier_critere(critere)?;
    if taille_min == 0 {
        return Err(Erreur::Parametre(
            "la taille minimale des blocs doit être au moins 1".into(),
        ));
    }
    let (l, h) = img.dimensions();
    let blocs = division(img, critere, taille_min);

    // Numéro de bloc de chaque pixel.
    let mut num = vec![0usize; l * h];
    for (i, &(x0, y0, bl, bh)) in blocs.iter().enumerate() {
        for y in y0..y0 + bh {
            num[y * l + x0..y * l + x0 + bl].fill(i);
        }
    }
    // Paires de blocs adjacents (4-connexité).
    let mut voisins = HashSet::new();
    for y in 0..h {
        for x in 0..l {
            let a = num[y * l + x];
            if x + 1 < l && num[y * l + x + 1] != a {
                voisins.insert((a.min(num[y * l + x + 1]), a.max(num[y * l + x + 1])));
            }
            if y + 1 < h && num[(y + 1) * l + x] != a {
                voisins.insert((a.min(num[(y + 1) * l + x]), a.max(num[(y + 1) * l + x])));
            }
        }
    }
    let stats_blocs: Vec<Stats> = blocs.iter().map(|&b| Stats::bloc(img, b)).collect();
    // On fusionne d'abord les blocs les plus semblables (moyennes proches).
    let mut aretes: Vec<(usize, usize)> = voisins.into_iter().collect();
    aretes.sort_by(|&(a, b), &(c, d)| {
        let e1 = (stats_blocs[a].moyenne() - stats_blocs[b].moyenne()).abs();
        let e2 = (stats_blocs[c].moyenne() - stats_blocs[d].moyenne()).abs();
        e1.total_cmp(&e2).then((a, b).cmp(&(c, d)))
    });

    // Union-find des régions, avec les statistiques de chaque racine.
    let mut parent: Vec<usize> = (0..blocs.len()).collect();
    let mut stats = stats_blocs;
    fn racine(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    loop {
        let mut fusion = false;
        for &(a, b) in &aretes {
            let (ra, rb) = (racine(&mut parent, a), racine(&mut parent, b));
            if ra == rb {
                continue;
            }
            let u = stats[ra].union(&stats[rb]);
            if u.homogene(critere) {
                let (garde, absorbe) = (ra.min(rb), ra.max(rb));
                parent[absorbe] = garde;
                stats[garde] = u;
                fusion = true;
            }
        }
        if !fusion {
            break;
        }
    }

    // Étiquettes 1..n dans l'ordre de première apparition (balayage ligne par ligne).
    let mut etiquette = vec![0u32; blocs.len()];
    let mut suivante = 0u32;
    let mut donnees = Vec::with_capacity(l * h);
    for &b in &num {
        let r = racine(&mut parent, b);
        if etiquette[r] == 0 {
            suivante += 1;
            etiquette[r] = suivante;
        }
        donnees.push(etiquette[r]);
    }
    ImageEtiquettes::depuis_vec(l, h, donnees)
}

/// Résultat de la phase de division seule (pour illustrer le quadtree dans le rapport) :
/// liste des blocs (x, y, largeur, hauteur).
///
/// Un bloc non homogène n'est divisé que si ses quatre sous-blocs gardent au moins
/// `taille_min` pixels de côté (une `taille_min` nulle compte comme 1). Les blocs
/// sont renvoyés dans l'ordre du parcours en profondeur du quadtree.
pub fn division(
    img: &ImageGris,
    critere: Homogeneite,
    taille_min: usize,
) -> Vec<(usize, usize, usize, usize)> {
    let taille_min = taille_min.max(1);
    let mut blocs = Vec::new();
    if img.nb_pixels() == 0 {
        return blocs;
    }
    let mut pile = vec![(0, 0, img.largeur(), img.hauteur())];
    while let Some(b @ (x, y, l, h)) = pile.pop() {
        let divisible = l >= 2 * taille_min && h >= 2 * taille_min;
        if !divisible || Stats::bloc(img, b).homogene(critere) {
            blocs.push(b);
            continue;
        }
        let (l1, h1) = (l / 2, h / 2);
        // Empilés à l'envers pour sortir dans l'ordre NO, NE, SO, SE.
        pile.push((x + l1, y + h1, l - l1, h - h1));
        pile.push((x, y + h1, l1, h - h1));
        pile.push((x + l1, y, l - l1, h1));
        pile.push((x, y, l1, h1));
    }
    blocs
}

// ---------------------------------------------------------------------------
// Croissance de régions
// ---------------------------------------------------------------------------

/// Croissance de régions à partir de germes : un voisin est ajouté si
/// |niveau - moyenne de la région| <= tolerance. Germe i -> étiquette i + 1.
///
/// Les régions grandissent en même temps (une seule file pour tous les germes) :
/// un pixel appartient à la première région qui l'atteint. Un germe déjà pris par
/// une région précédente (germe en double) ne crée pas de région.
pub fn croissance_regions(
    img: &ImageGris,
    germes: &[(usize, usize)],
    tolerance: f64,
    connexite: Connexite,
) -> Resultat<ImageEtiquettes> {
    if germes.is_empty() {
        return Err(Erreur::Parametre("il faut au moins un germe".into()));
    }
    if tolerance.is_nan() || tolerance < 0.0 {
        return Err(Erreur::Parametre(format!(
            "tolérance invalide : {tolerance}"
        )));
    }
    let (l, h) = img.dimensions();
    if let Some(&(x, y)) = germes.iter().find(|&&(x, y)| x >= l || y >= h) {
        return Err(Erreur::Parametre(format!(
            "germe ({x}, {y}) hors de l'image {l}x{h}"
        )));
    }
    let mut etiquettes = ImageEtiquettes::nouvelle(l, h, 0);
    let mut somme = vec![0.0; germes.len()];
    let mut nb = vec![0.0; germes.len()];
    let mut file = VecDeque::new();
    for (i, &(x, y)) in germes.iter().enumerate() {
        if etiquettes[(x, y)] == 0 {
            etiquettes[(x, y)] = i as u32 + 1;
            somme[i] += img[(x, y)] as f64;
            nb[i] += 1.0;
            file.push_back((x, y));
        }
    }
    while let Some((x, y)) = file.pop_front() {
        let r = etiquettes[(x, y)] as usize - 1;
        for &(dx, dy) in connexite.voisins() {
            let (vx, vy) = (x as isize + dx, y as isize + dy);
            if !etiquettes.contient(vx, vy) {
                continue;
            }
            let (vx, vy) = (vx as usize, vy as usize);
            if etiquettes[(vx, vy)] != 0 {
                continue;
            }
            let v = img[(vx, vy)] as f64;
            if (v - somme[r] / nb[r]).abs() <= tolerance {
                etiquettes[(vx, vy)] = r as u32 + 1;
                somme[r] += v;
                nb[r] += 1.0;
                file.push_back((vx, vy));
            }
        }
    }
    Ok(etiquettes)
}

/// Norme du gradient de Sobel |Gx| + |Gy| (bord répliqué).
///
/// `contours::gradient` (Pio) n'est pas encore écrit sur `main` : version privée
/// minimale pour le choix des germes.
fn norme_sobel(img: &ImageGris) -> Vec<f64> {
    let (l, h) = img.dimensions();
    let p = |x: isize, y: isize| -> f64 {
        let xc = x.clamp(0, l as isize - 1) as usize;
        let yc = y.clamp(0, h as isize - 1) as usize;
        img[(xc, yc)] as f64
    };
    let mut g = Vec::with_capacity(l * h);
    for y in 0..h as isize {
        for x in 0..l as isize {
            let gx = (p(x + 1, y - 1) + 2.0 * p(x + 1, y) + p(x + 1, y + 1))
                - (p(x - 1, y - 1) + 2.0 * p(x - 1, y) + p(x - 1, y + 1));
            let gy = (p(x - 1, y + 1) + 2.0 * p(x, y + 1) + p(x + 1, y + 1))
                - (p(x - 1, y - 1) + 2.0 * p(x, y - 1) + p(x + 1, y - 1));
            g.push(gx.abs() + gy.abs());
        }
    }
    g
}

/// Choix automatique de germes : minima locaux de la norme du gradient, espacés d'au
/// moins `espacement` pixels.
///
/// Les minima locaux (pixel <= ses 8 voisins) sont pris du plus petit gradient au
/// plus grand ; un candidat est gardé s'il est à une distance d'échiquier
/// max(|dx|, |dy|) d'au moins `espacement` de tous les germes déjà gardés.
pub fn germes_automatiques(img: &ImageGris, espacement: usize) -> Vec<(usize, usize)> {
    let (l, h) = img.dimensions();
    if l == 0 || h == 0 {
        return Vec::new();
    }
    let espacement = espacement.max(1);
    let g = norme_sobel(img);
    let mut candidats = Vec::new();
    for y in 0..h {
        for x in 0..l {
            let v = g[y * l + x];
            let minimum = Connexite::Huit.voisins().iter().all(|&(dx, dy)| {
                let (vx, vy) = (x as isize + dx, y as isize + dy);
                !img.contient(vx, vy) || v <= g[vy as usize * l + vx as usize]
            });
            if minimum {
                candidats.push((v, x, y));
            }
        }
    }
    // Tri stable : à gradient égal, l'ordre de balayage est conservé.
    candidats.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut interdit = vec![false; l * h];
    let mut germes = Vec::new();
    let r = espacement - 1;
    for (_, x, y) in candidats {
        if interdit[y * l + x] {
            continue;
        }
        germes.push((x, y));
        for yy in y.saturating_sub(r)..(y + r + 1).min(h) {
            interdit[yy * l + x.saturating_sub(r)..yy * l + (x + r + 1).min(l)].fill(true);
        }
    }
    germes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gris(l: usize, h: usize, v: &[u8]) -> ImageGris {
        ImageGris::depuis_vec(l, h, v.to_vec()).unwrap()
    }

    /// Image 8x8 : moitié gauche à 50, moitié droite à 200.
    fn deux_moities() -> ImageGris {
        ImageGris::depuis_fonction(8, 8, |x, _| if x < 4 { 50 } else { 200 })
    }

    #[test]
    fn seuillage_simple() {
        let img = gris(4, 1, &[0, 99, 100, 255]);
        let b = seuillage(&img, 100);
        assert_eq!(b.donnees(), &[false, false, true, true]);
        // Seuil 0 : tout est objet.
        assert_eq!(seuillage(&img, 0).nb_pixels_objet(), 4);
    }

    #[test]
    fn seuillage_multiple_trois_classes() {
        let img = gris(5, 1, &[10, 99, 100, 150, 200]);
        let e = seuillage_multiple(&img, &[100, 200]).unwrap();
        assert_eq!(e.donnees(), &[0, 0, 1, 1, 2]);
    }

    #[test]
    fn seuillage_multiple_refuse_les_seuils_non_tries() {
        let img = gris(2, 1, &[0, 255]);
        assert!(seuillage_multiple(&img, &[200, 100]).is_err());
        assert!(seuillage_multiple(&img, &[100, 100]).is_err());
        assert!(seuillage_multiple(&img, &[]).is_err());
    }

    #[test]
    fn seuils_moyenne_milieu_median() {
        // Moyenne = (10 + 20 + 30 + 240) / 4 = 75 ; milieu = (10 + 240) / 2 = 125.
        let img = gris(4, 1, &[10, 20, 30, 240]);
        assert_eq!(seuil_moyenne(&img), 75);
        assert_eq!(seuil_milieu(&img), 125);
        // Médiane : 2 pixels de chaque côté pour tout T dans ]20, 30] -> milieu 25.
        assert_eq!(seuil_median(&img), 25);
        let b = seuillage(&img, seuil_median(&img));
        assert_eq!(b.nb_pixels_objet(), 2);
    }

    #[test]
    fn seuil_median_nombre_impair() {
        // [10, 20, 30] : T = 20 est la médiane.
        assert_eq!(seuil_median(&gris(3, 1, &[30, 10, 20])), 20);
    }

    #[test]
    fn otsu_deux_niveaux() {
        // Tout T dans ]50, 200] donne sigma_w = 0 : milieu du palier = (51 + 200) / 2.
        let img = deux_moities();
        assert_eq!(seuil_otsu(&img), 125);
        let b = seuillage(&img, seuil_otsu(&img));
        assert_eq!(b.nb_pixels_objet(), 32);
        assert!(b[(4, 0)] && !b[(3, 0)]);
    }

    #[test]
    fn otsu_deux_modes_etales() {
        // Deux modes 40..=60 et 190..=210 : le seuil tombe entre les deux.
        let img =
            ImageGris::depuis_fonction(
                21,
                2,
                |x, y| {
                    if y == 0 {
                        40 + x as u8
                    } else {
                        190 + x as u8
                    }
                },
            );
        let t = seuil_otsu(&img);
        assert!(t > 60 && t <= 190, "seuil {t}");
        assert_eq!(seuillage(&img, t).nb_pixels_objet(), 21);
    }

    #[test]
    fn otsu_image_constante() {
        assert_eq!(seuil_otsu(&ImageGris::nouvelle(3, 3, 77)), 77);
    }

    #[test]
    fn kittler_deux_modes() {
        let img =
            ImageGris::depuis_fonction(
                21,
                2,
                |x, y| {
                    if y == 0 {
                        40 + x as u8
                    } else {
                        190 + x as u8
                    }
                },
            );
        let t = seuil_kittler(&img);
        assert!(t > 60 && t <= 190, "seuil {t}");
        // Deux niveaux seulement : variances nulles, on retombe sur Otsu.
        assert_eq!(seuil_kittler(&deux_moities()), 125);
    }

    #[test]
    fn adaptatif_eclairage_variable() {
        // Gauche sombre (objet 60 sur fond 20), droite claire (objet 230 sur fond 160).
        // Un seuil global ne peut pas marcher : 60 < 160.
        let img = ImageGris::depuis_fonction(8, 4, |x, y| {
            let objet = y % 2 == 0;
            match (x < 4, objet) {
                (true, true) => 60,
                (true, false) => 20,
                (false, true) => 230,
                (false, false) => 160,
            }
        });
        let b = seuillage_adaptatif(&img, 4, 100.0).unwrap();
        let attendu = ImageBinaire::depuis_fonction(8, 4, |_, y| y % 2 == 0);
        assert_eq!(b, attendu);
    }

    #[test]
    fn adaptatif_bloc_uniforme_non_traite() {
        // Bloc gauche uniforme à 220 (variance 0 < 100) : objet entier car 220 >= seuil
        // global ; bloc droit bimodal seuillé normalement.
        let img = ImageGris::depuis_fonction(4, 2, |x, y| match (x < 2, y) {
            (true, _) => 220,
            (false, 0) => 0,
            (false, _) => 255,
        });
        let b = seuillage_adaptatif(&img, 2, 100.0).unwrap();
        assert_eq!(
            b.donnees(),
            &[true, true, false, false, true, true, true, true]
        );
        assert!(seuillage_adaptatif(&img, 0, 100.0).is_err());
        assert!(seuillage_adaptatif(&img, 2, -1.0).is_err());
    }

    #[test]
    fn kmoyennes_gris_trois_groupes() {
        let img = gris(6, 1, &[10, 12, 100, 104, 250, 246]);
        for graine in 0..5 {
            let r = kmoyennes_gris(&img, 3, 20, graine).unwrap();
            assert_eq!(r.centres, vec![11.0, 102.0, 248.0]);
            assert_eq!(r.etiquettes.donnees(), &[1, 1, 2, 2, 3, 3]);
            assert!(r.iterations <= 20);
        }
    }

    #[test]
    fn kmoyennes_parametres_invalides() {
        let img = gris(2, 1, &[0, 255]);
        assert!(kmoyennes_gris(&img, 0, 10, 1).is_err());
        assert!(kmoyennes_gris(&img, 3, 10, 1).is_err());
        assert!(kmoyennes_gris(&img, 2, 0, 1).is_err());
    }

    #[test]
    fn kmoyennes_couleur_deux_groupes() {
        let rouge = Rvb::new(200, 10, 10);
        let rouge2 = Rvb::new(210, 20, 10);
        let bleu = Rvb::new(10, 10, 200);
        let img = ImageRvb::depuis_vec(4, 1, vec![rouge, bleu, rouge2, bleu]).unwrap();
        let r = kmoyennes_couleur(&img, 2, 10, 7).unwrap();
        // Bleu plus sombre (luminance) que rouge : étiquette 1.
        assert_eq!(r.etiquettes.donnees(), &[2, 1, 2, 1]);
        assert_eq!(r.centres, vec![[10.0, 10.0, 200.0], [205.0, 15.0, 10.0]]);
        let poster = image_kmoyennes_couleur(&r);
        assert_eq!(poster[(0, 0)], Rvb::new(205, 15, 10));
        assert_eq!(poster[(1, 0)], bleu);
        assert!(kmoyennes_couleur(&img, 4, 10, 7).is_err());
    }

    #[test]
    fn image_kmoyennes_etiquette_zero_noire() {
        let r = KMoyennes {
            etiquettes: ImageEtiquettes::depuis_vec(2, 1, vec![0, 1]).unwrap(),
            centres: vec![[1.4, 2.6, 255.9]],
            iterations: 1,
        };
        let img = image_kmoyennes_couleur(&r);
        assert_eq!(img.donnees(), &[Rvb::default(), Rvb::new(1, 3, 255)]);
    }

    /// 4x4 : quart haut-gauche à 200, le reste à 10.
    fn quart_clair() -> ImageGris {
        ImageGris::depuis_fonction(4, 4, |x, y| if x < 2 && y < 2 { 200 } else { 10 })
    }

    #[test]
    fn division_quadtree() {
        let img = quart_clair();
        // Image non homogène -> 4 blocs 2x2, chacun homogène.
        let blocs = division(&img, Homogeneite::Variance(0.0), 1);
        assert_eq!(
            blocs,
            vec![(0, 0, 2, 2), (2, 0, 2, 2), (0, 2, 2, 2), (2, 2, 2, 2)]
        );
        // Image homogène : un seul bloc.
        let uni = ImageGris::nouvelle(4, 4, 9);
        assert_eq!(
            division(&uni, Homogeneite::MaxMin(0), 1),
            vec![(0, 0, 4, 4)]
        );
        // taille_min = 4 : on ne peut pas descendre à 2x2.
        assert_eq!(
            division(&img, Homogeneite::MaxMin(0), 4),
            vec![(0, 0, 4, 4)]
        );
    }

    #[test]
    fn division_jusquau_pixel() {
        let img = gris(2, 2, &[0, 50, 100, 150]);
        let blocs = division(&img, Homogeneite::MaxMin(10), 1);
        assert_eq!(blocs.len(), 4);
        assert!(blocs.iter().all(|&(_, _, l, h)| l == 1 && h == 1));
    }

    #[test]
    fn division_fusion_deux_regions() {
        let img = quart_clair();
        let e = division_fusion(&img, Homogeneite::MaxMin(5), 1).unwrap();
        // Les 3 blocs sombres fusionnent : 2 régions.
        assert_eq!(e.etiquette_max(), 2);
        assert_eq!(e[(0, 0)], 1);
        assert_eq!(e[(1, 1)], 1);
        assert_eq!(e[(3, 0)], 2);
        assert_eq!(e[(0, 3)], 2);
        assert_eq!(e[(3, 3)], 2);
        assert!(division_fusion(&img, Homogeneite::Variance(-1.0), 1).is_err());
        assert!(division_fusion(&img, Homogeneite::Variance(1.0), 0).is_err());
    }

    #[test]
    fn division_fusion_regions_non_adjacentes_separees() {
        // Deux coins clairs opposés : même niveau, mais non adjacents.
        let img = ImageGris::depuis_fonction(4, 4, |x, y| {
            if (x < 2 && y < 2) || (x >= 2 && y >= 2) {
                200
            } else {
                10
            }
        });
        let e = division_fusion(&img, Homogeneite::Variance(1.0), 1).unwrap();
        // Les deux blocs sombres (haut-droite et bas-gauche) ne se touchent que par un
        // coin : en 4-connexité ce sont aussi deux régions -> 4 régions.
        assert_eq!(e.etiquette_max(), 4);
    }

    #[test]
    fn croissance_un_germe() {
        // Région claire en L entourée de sombre.
        let img = gris(
            4,
            3,
            &[
                200, 205, 10, 10, //
                198, 10, 10, 10, //
                202, 201, 199, 10,
            ],
        );
        let e = croissance_regions(&img, &[(0, 0)], 10.0, Connexite::Quatre).unwrap();
        let attendu = [1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0];
        assert_eq!(e.donnees(), &attendu);
    }

    #[test]
    fn croissance_plusieurs_germes_et_connexite() {
        // Deux pixels clairs qui ne se touchent que par un coin.
        let img = gris(2, 2, &[200, 10, 10, 200]);
        let e4 = croissance_regions(&img, &[(0, 0)], 5.0, Connexite::Quatre).unwrap();
        assert_eq!(e4.donnees(), &[1, 0, 0, 0]);
        let e8 = croissance_regions(&img, &[(0, 0)], 5.0, Connexite::Huit).unwrap();
        assert_eq!(e8.donnees(), &[1, 0, 0, 1]);
        // Germe sombre (1, 0) : en 4-connexité il n'atteint pas (0, 1),
        // en 8-connexité il l'atteint par la diagonale.
        let deux = croissance_regions(&img, &[(0, 0), (1, 0)], 5.0, Connexite::Quatre).unwrap();
        assert_eq!(deux.donnees(), &[1, 2, 0, 0]);
        let deux8 = croissance_regions(&img, &[(1, 0), (0, 0)], 5.0, Connexite::Huit).unwrap();
        assert_eq!(deux8.donnees(), &[2, 1, 1, 2]);
    }

    #[test]
    fn croissance_parametres_invalides() {
        let img = gris(2, 2, &[0; 4]);
        assert!(croissance_regions(&img, &[], 5.0, Connexite::Quatre).is_err());
        assert!(croissance_regions(&img, &[(2, 0)], 5.0, Connexite::Quatre).is_err());
        assert!(croissance_regions(&img, &[(0, 0)], -1.0, Connexite::Quatre).is_err());
    }

    #[test]
    fn germes_dans_les_zones_plates() {
        // Deux zones plates 10x10 séparées par une marche verticale en x = 10.
        let img = ImageGris::depuis_fonction(20, 10, |x, _| if x < 10 { 30 } else { 220 });
        let germes = germes_automatiques(&img, 5);
        assert!(!germes.is_empty());
        // Aucun germe sur la marche (gradient fort en x = 9 et x = 10).
        assert!(germes.iter().all(|&(x, _)| x != 9 && x != 10));
        // Au moins un germe de chaque côté.
        assert!(germes.iter().any(|&(x, _)| x < 9));
        assert!(germes.iter().any(|&(x, _)| x > 10));
        // Espacement respecté.
        for (i, a) in germes.iter().enumerate() {
            for b in &germes[i + 1..] {
                let d = a.0.abs_diff(b.0).max(a.1.abs_diff(b.1));
                assert!(d >= 5, "{a:?} et {b:?} trop proches");
            }
        }
        // Les germes automatiques donnent bien les deux régions.
        let e = croissance_regions(&img, &germes, 10.0, Connexite::Quatre).unwrap();
        assert!(e.donnees().iter().all(|&v| v > 0));
        assert_ne!(e[(0, 0)], e[(19, 0)]);
    }
}
