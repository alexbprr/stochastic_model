mod stochastic_model;
mod settings;
use anyhow::Error;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use stochastic_model::{csvdata::CSVData, sto_parser::parse_input};
use settings::Settings;

pub fn simulate(model_name: &str, t_final: f64, num_execs: usize){
    (1..num_execs+1).into_par_iter().for_each( |k| {
        let mut model = parse_input(String::from(model_name)); 
        model.gillespie(t_final, String::from("./tests/results/"), k);
        model.plot_results_manyplots(k);
        
    });
}

fn main() -> Result<(),Error> {
    simulate("./tests/models/predatorprey_model.txt",30.0, 10);
    let data_loaded: CSVData = CSVData::load_data("./tests/results/").unwrap();
    data_loaded.plot_all_data();

    Ok(())
}
