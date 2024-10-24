mod stochastic_model;
use ::stochastic_model::simulate;
use stochastic_model::csvdata::CSVData;

pub fn main(){
    simulate("./tests/models/viral_infection.txt","./tests/results/",30.0, 4);
    let data_loaded: CSVData = CSVData::load_data("./tests/results/").unwrap();
    data_loaded.plot_all();
}