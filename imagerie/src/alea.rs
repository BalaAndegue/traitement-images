//! Petit générateur pseudo-aléatoire (xorshift64*), pour le bruit et les k-moyennes.
//!
//! Une graine fixe donne toujours la même suite : les tests et les figures du
//! rapport restent reproductibles.

#[derive(Clone, Debug)]
pub struct Alea {
    etat: u64,
}

impl Alea {
    pub fn new(graine: u64) -> Self {
        // L'état ne doit jamais valoir 0.
        Alea {
            etat: graine ^ 0x9E37_79B9_7F4A_7C15 | 1,
        }
    }

    pub fn suivant_u64(&mut self) -> u64 {
        let mut x = self.etat;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.etat = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Réel uniforme dans [0, 1).
    pub fn uniforme(&mut self) -> f64 {
        (self.suivant_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Entier uniforme dans [0, n).
    pub fn entier(&mut self, n: usize) -> usize {
        (self.uniforme() * n as f64) as usize
    }

    /// Tirage gaussien centré réduit (Box-Muller).
    pub fn gaussien(&mut self) -> f64 {
        let u1 = self.uniforme().max(f64::MIN_POSITIVE);
        let u2 = self.uniforme();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reproductible_et_borne() {
        let mut a = Alea::new(42);
        let mut b = Alea::new(42);
        for _ in 0..1000 {
            let u = a.uniforme();
            assert_eq!(u, b.uniforme());
            assert!((0.0..1.0).contains(&u));
        }
    }
}
