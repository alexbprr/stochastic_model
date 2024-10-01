use std::{collections::BTreeMap, fs::File, io::{BufWriter, Write}, path::Path};
use expr_evaluator::expr::{ExprContext, Expression};
use rand::Rng;
use rand_distr::{Distribution, Exp};
use random_choice::random_choice;
use plotpy::{Curve, Legend, Plot};
use colorgrad::Gradient;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use serde::{Serialize,Deserialize};
pub mod sto_parser;
mod plot;

#[derive(Clone,Debug,Serialize,Deserialize)]
pub enum Sign {
    Negative,
    Positive,
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Reaction {
    pub input: Expression,
    pub outputs: Vec<(Sign,String)>,
    pub rate: f64,
}

impl Reaction {

    pub fn new() -> Self {
        Self { 
            input: Expression::new(), 
            outputs: vec![], 
            rate: 0.0,
        }
    }

    pub fn eval (&mut self) {
        match self.input.eval() {
            Ok(v) => self.rate = v,
            Err(e) => {println!("An error ocurred during expression evaluation: {:?}", e); self.rate = 0.0; },
        };
    }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct StochasticModel {
    pub name: String,
    pub initial_condition: BTreeMap<String,i32>,
    pub populations: BTreeMap<String,i32>,
    pub parameters: BTreeMap<String,f64>, //parameters are stored in the context of each expression that they are participating
    pub reactions: Vec<Reaction>,
    pub outputs: BTreeMap<String,Vec<i32>>,
    pub times: Vec<f64>
}

impl StochasticModel {

    pub fn new() -> Self {
        Self { 
            name: String::from("StochasticModel"), 
            initial_condition: BTreeMap::new(),
            populations: BTreeMap::new(), 
            parameters: BTreeMap::new(), 
            reactions: vec![], 
            outputs: BTreeMap::new(), 
            times: vec![] 
        }
    }

    pub fn clear_output(&mut self){
        self.outputs.clear();
        self.times.clear(); 
        self.times = vec![];      
    }

    pub fn update_context(&mut self){
        let mut context = ExprContext::new();
        for p in self.populations.iter() {        
            context.set_var(p.0.to_string(), *p.1 as f64);        
        }
        for p in self.parameters.iter() {
            context.set_var(p.0.to_string(), *p.1);        
        }
        for reaction in self.reactions.iter_mut() {
            reaction.input.set_context(context.clone());
        }
    }

    pub fn calculate_rates(&mut self) {
        for r in self.reactions.iter_mut(){
            r.eval();
        }
    } 

    pub fn save_initial_state(&mut self){
        for p in self.initial_condition.iter(){
            let mut values = vec![];
            values.push(*p.1);
            self.outputs.insert(p.0.to_string(), values);
            self.populations.insert(p.0.to_string(), *p.1);
        }
    }

    pub fn save_states(&mut self){
        for p in self.populations.iter(){
            let values = self.outputs.get_mut(p.0).unwrap();
            values.push(*p.1);
        }
    }

    pub fn gillespie(&mut self, t_final: f64, fname: String, exec_index: usize){

        let mut t: f64 = 0.0;
        self.times.push(t);
        self.save_initial_state();
        self.update_context();

        while t < t_final {

            self.calculate_rates();

            let sum: f64 = self.reactions
                            .par_iter()
                            .map(|r| r.rate)
                            .sum();                            
            //println!("sum: {:?}", sum);            
            if sum == 0.0 {
                break;
            }
            let weights: Vec<f64> = self.reactions
                                .par_iter()
                                .map(|r| r.rate/sum)
                                .collect();

            let choices = random_choice().random_choice_f64(&self.reactions, &weights, 10);

            if choices.len() == 0 {
                continue;
            }
            
            //let reaction = choices.get(0).unwrap();

            for reaction in choices.iter(){
                for output in reaction.outputs.iter() {
                    let value = self.populations.get_mut(&output.1).unwrap();
                    match output.0 {
                        Sign::Negative => *value -= 1,
                        Sign::Positive => *value += 1,
                    }
                }
            }   

            self.save_states();
            self.update_context();                
            //println!("Model output: {:#?}", self.outputs);
            let rng = &mut rand::thread_rng();
            //let random_num = rng.gen_range(0.0..1.0);
            //let dt = - f64::log(random_num, 10.0)/sum;
            
            let exp: Exp<f64> = Exp::new(sum).unwrap();
            let dt: f64 = exp.sample(rng); 
            t += dt;
            self.times.push(t);
            println!("t = {:?}", t);            
        }

        self.save_results( Path::new(&format!("{}{}{}", fname, exec_index, ".csv")) );
    }    
    
    pub fn plot_results(&self, exec_index: usize) {
        let mut plot = Plot::new();
        plot.set_figure_size_points(900.0, 600.0);
        plot.set_labels("time (days)", "population");

        /*let grad = colorgrad::GradientBuilder::new()
                .html_colors(&["deeppink", "gold", "seagreen"])
                .build::<colorgrad::CatmullRomGradient>().unwrap();*/
        let grad = colorgrad::preset::rainbow();

        let p_size = self.populations.len();    
        let mut color_ind: f32 = 0.0;        
        let color_inc: f32 = 1.0/(p_size as f32);
        
        
        for population in self.populations.keys() {        
            let results = self.outputs.get(population).unwrap();
            let mut values: Vec<f64> = vec![];
            for v in results {
                values.push(*v as f64);
            }

            let rgba = grad.at(color_ind);
            color_ind += color_inc;
            let color = rgba.to_hex_string();           

            let mut curve = Curve::new();
            curve.set_line_width(1.3);    
            curve.set_line_color(&color);
            curve.set_label(population);
            curve.draw(&self.times, &values);
            plot.set_ymin(-1.0);
            plot.add(&curve);
        }
        let mut legend = Legend::new();
        legend.set_fontsize(11.0)
            .set_handle_len(1.0)
            .set_num_col(p_size/4 + 1)
            .set_outside(true)
            .set_show_frame(false);
        legend.draw();

        plot.add(&legend);    
        plot.save(&format!("{}{}{}", String::from("./src/tests/imgs/plot_"), exec_index, ".png")).unwrap();

    }

    /*let grad = colorgrad::GradientBuilder::new()
                .html_colors(&["deeppink", "gold", "seagreen"])
                .build::<colorgrad::CatmullRomGradient>().unwrap();*/
    pub fn plot_results_manyplots(&self, exec_index: usize) {        
        let grad = colorgrad::preset::rainbow();

        let p_size = self.populations.len();
        let mut color_ind: f32 = 0.0;
        let color_inc: f32 = 1.0/(p_size as f32);
        
        for population in self.populations.keys() {
            let mut plot = Plot::new();
            plot.set_figure_size_points(900.0, 600.0);
            plot.set_labels("time (days)", "population");            
            
            let rgba = grad.at(color_ind);
            color_ind += color_inc;
            let color = rgba.to_hex_string();           

            let mut curve = Curve::new();
            curve.set_line_width(1.3);    
            curve.set_line_color(&color);
            curve.set_label(population);

            let results = self.outputs.get(population).unwrap();
            let values = results.into_iter().map(|x| *x as f64).collect();
            curve.draw(&self.times, &values);
            plot.add(&curve);

            let mut legend = Legend::new();
            legend.set_fontsize(11.0)
                .set_handle_len(1.0)
                .set_num_col(p_size/4 + 1)
                .set_outside(true)
                .set_show_frame(false);
            legend.draw();

            plot.add(&legend);
            plot.save(&format!("{}_{}_{}{}", String::from("./src/tests/imgs/plot"), population, exec_index, ".png")).unwrap();
        }

    }

    pub fn save_results(&self, fname: &Path) {
        let file = match File::create(fname) {
            Err(e) => {
                println!("Could not open file. Error: {:?}", e);
                return;
            }
            Ok(buf) => buf,
        };
        let mut buf = BufWriter::new(file);
    
        // Write time and state vector in csv format        
        write!(buf,"time").unwrap();
        for p_name in self.populations.keys() {
            write!(buf,", {}", p_name).unwrap();
        }        
        write!(buf, "\n").unwrap();

        let mut results: Vec<Vec<f64>> = vec![vec![0.0; self.populations.len()]; self.times.len()];
        
        let mut j = 0;
        for (_name, values) in self.outputs.iter() { 
            let mut i: usize = 0;
            
            for v in values.iter() {
                results[i][j] = *v as f64;
                i += 1;                
            }
            j += 1;
        }

        let mut i: usize = 0;
        for result in results.iter(){
            write!(buf,"{}", self.times[i]).unwrap();
            
            for v in result.iter() {
                write!(buf, ", {}", v).unwrap();
            }
            write!(buf, "\n").unwrap();
            i += 1;
        }
        
        if let Err(e) = buf.flush() {
            println!("Could not write to file. Error: {:?}", e);
        }
    }

    pub fn save_model<P: AsRef<Path>>(&self, path: &P) {
        let file: File = match File::create(path) {
            Ok(f) => f,
            Err(e) => {
                println!("Could not open file. Error: {:?}", e);
                return;
            }
        };
        let writer: BufWriter<File> = BufWriter::new(file);        
        serde_json::to_writer_pretty(writer, self).unwrap();
    }
}