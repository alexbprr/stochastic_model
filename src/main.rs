use std::path::Path;

use crate::model::Model;

mod model;
//to do: usar expressão regular para procurar numeros nas expressoes do lado esquerdo e direito 

fn main(){
    let mut stochastic_model: Model = Model::new();
    stochastic_model.parse_input(Path::new("./src/tests/predator_prey.txt"));
    println!("{:#?}", stochastic_model);
    stochastic_model.print_reactions();
    stochastic_model.calculate_rates();
}