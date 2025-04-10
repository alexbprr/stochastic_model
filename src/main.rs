mod stochastic_model;
use ::stochastic_model::simulate;
use stochastic_model::csvdata::CSVData;

pub fn main(){
    simulate("./tests/models/predatorprey_model.txt","./tests/results/",50.0, 5);
    let data_loaded: CSVData = CSVData::load_data("./tests/results/").unwrap();
    data_loaded.plot_all();
}