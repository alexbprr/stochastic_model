use std::thread;
use crate::model::Model;

mod model;
//to do: usar expressão regular para procurar numeros nas expressoes do lado esquerdo e direito 

fn main(){
    let handle = thread::spawn(|| {
        for i in 1..5 {
            println!("hi number {} from the spawned thread!", i);
            let mut f_name: String = format!("./src/tests/pp_input{i}");
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