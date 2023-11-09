//! Lindenmayer system algorithms.

#[derive(Clone, Debug)]
pub struct LSysItem {
    symbol: String,
    definition: String,
}

#[rustfmt::skip]
pub enum Symbols {
    A, B, C, D, E,
    F, G, H, I, J,
    K, L, M, N, O,
    P, Q, R, S, T,
    U, V, W, X, Y,
    Z,
}

impl LSysItem {
    pub fn new(symbol: &str, definition: &str) -> Self {
        Self {
            symbol: String::from(symbol),
            definition: String::from(definition),
        }
    }
}

pub struct LSys {
    items: Vec<LSysItem>,
    axiom: String,
}
