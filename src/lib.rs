pub mod stochastic_model;
use std::{fs, path::Path};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use stochastic_model::{csvdata::CSVData, sto_parser::parse_input};

pub fn simulate(model_name: &str, path: &str, t_final: f64, num_execs: usize){
    let dir: &Path = &Path::new(path);
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            fs::remove_file(entry.unwrap().path()).unwrap();
        }
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .unwrap();

    pool.install(|| {
        rayon::scope(|s| {
            for k in 0..num_execs { 
                s.spawn(move |_| {
                    let mut model = parse_input(String::from(model_name)); 
                    model.gillespie(t_final, path.to_string(), k);
                    model.plot(k);   
                });    
            }
        });
    });
}


#[test]
pub fn test(){
    simulate("./tests/models/viral_infection.txt","./tests/results/",30.0, 4);
    let data_loaded: CSVData = CSVData::load_data("./tests/results/").unwrap();
    data_loaded.plot_all();
}