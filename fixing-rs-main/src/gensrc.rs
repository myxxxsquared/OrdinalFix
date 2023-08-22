use crate::grammars::SupportedGrammar;
use clap::{Parser, ValueEnum};
use fixing_rs_base::gensrc::{gen_g_src, gen_s_src, gen_s_symbolic_src};

#[derive(Parser)]
pub struct GenSrcCmd {
    #[clap(value_enum)]
    lang: SupportedGrammar,
    #[clap(value_enum)]
    ty: GenSrcType,
}

#[derive(ValueEnum, Clone)]
pub enum GenSrcType {
    G,
    S,
    SS,
}

impl GenSrcCmd {
    pub fn run(&self) {
        let grammar = self.lang.fixing_info();
        match self.ty {
            GenSrcType::G => {
                let result = gen_g_src(grammar).unwrap();
                println!("{}", result);
            }
            GenSrcType::S => {
                let result = gen_s_src(grammar).unwrap();
                println!("{}", result);
            }
            GenSrcType::SS => {
                let result = gen_s_symbolic_src(grammar).unwrap();
                println!("{}", result);
            }
        }
    }
}
