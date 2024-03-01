use core::num;
use std::thread;
use crate::model::Model;

mod model;
//to do: usar expressão regular para procurar numeros nas expressoes do lado esquerdo e direito 

fn main(){
    let mut number_of_threads: usize = 12;
    let n_runs: usize = 4;
    let slice: usize = 1;
    if n_runs <= number_of_threads {
        number_of_threads = n_runs;
    }
    else {
        return;
    }
    let handle = thread::spawn(move || {
        for i in 0..number_of_threads {
            let j = i + 1;
            println!("hi number {} from the spawned thread!", j);            
            let mut f_name: String = format!("./src/tests/pp_input{j}");
            let mut stochastic_model: Model = Model::new();
            stochastic_model.gillespie(f_name.clone());
            stochastic_model.build_odes();
            let results: Vec<(String,f64)> = stochastic_model.evaluate_odes();
            println!("results: {:#?}", results);
            f_name.push_str(".png");
            //stochastic_model.plot_results::<String>(f_name);
        } 
    });
    
    handle.join().unwrap();
}