mod stochastic_model;
mod settings;
use anyhow::Error;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use stochastic_model::sto_parser::parse_input;
use std::path::Path;
use settings::Settings;

pub fn simulate(t_final: f64, num_execs: usize){
    (1..num_execs+1).into_par_iter().for_each( |k| {
        let mut model = parse_input(&String::from("./src/tests/hiv_model.txt"));
        model.gillespie(t_final, String::from("./src/tests/results_"), k);
        model.plot_results(k);
        model.plot_results_manyplots(k);        
    });
}

fn main() -> Result<(),Error> {
    simulate(1.0, 1);    
    
    Ok(())
}