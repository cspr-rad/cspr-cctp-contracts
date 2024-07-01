/// Modality of the CEP-18 contract.
#[derive(Default)]
#[odra::odra_type]
pub enum StablecoinModality {
    /// No modailities are set.
    #[default]
    None = 0,
    /// The contract can mint and burn tokens.
    MintAndBurn = 1,
}

impl StablecoinModality {
    /// Returns true if the mint and burn functionality is enabled.
    pub fn mint_and_burn_enabled(&self) -> bool {
        matches!(self, StablecoinModality::MintAndBurn)
    }
}

// implement conversion from modality into u8
impl From<StablecoinModality> for u8 {
    fn from(modality: StablecoinModality) -> u8 {
        modality as u8
    }
}
