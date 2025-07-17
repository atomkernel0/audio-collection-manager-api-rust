use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Display, EnumString, Serialize, Deserialize, Clone)]
pub enum MusicGenre {
    #[strum(serialize = "RAC")]
    Rac,
    #[strum(serialize = "NSBM")]
    Nsbm,
    #[strum(serialize = "OI")]
    Oi,
    #[strum(serialize = "RAP")]
    Rap,
    #[strum(serialize = "BALLADS")]
    Ballads,
    #[strum(serialize = "CHANT_MILITAIRE")]
    ChantMilitaire,
    #[strum(serialize = "PSYCHEDELIC_ROCK")]
    PsychedelicRock,
}
