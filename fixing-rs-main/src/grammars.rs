use crate::{c::fixing::C_FIXING_INFO, mj::fixing::MJ_FIXING_INFO};
use clap::ValueEnum;
use fixing_rs_base::fixing_info::FixingInfo;

#[derive(ValueEnum, Clone, Copy, PartialEq, Eq)]
pub enum SupportedGrammar {
    MJ,
    C,
}

impl SupportedGrammar {
    pub fn fixing_info(self) -> &'static FixingInfo {
        match self {
            Self::MJ => &MJ_FIXING_INFO,
            Self::C => &C_FIXING_INFO,
        }
    }
}
