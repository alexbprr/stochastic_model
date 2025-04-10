use std::{collections::{BTreeMap, HashMap, HashSet}, fs::File, io::{BufWriter, Write}, path::Path, thread::Thread};
use expr_evaluator::expr::{ExprContext, Expression};
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Exp};
use plotpy::{Curve, Legend, Plot};
use colorgrad::Gradient;
use serde::{Serialize,Deserialize};
pub mod sto_parser;
pub mod plot;
pub mod csvdata;

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Reaction {
    pub id: usize,
    pub updates: HashMap<String,i32>,
    pub expr: Expression,
    pub rate: f64,
    pub time: f64,
    pub updated: bool,    
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct LogData {
    pub reactions: Vec<Reaction>,
    //pub n_executions: HashMap<usize,usize>
}

impl Reaction {

    pub fn new() -> Self {
        Self {
            id: 0,
            updates: HashMap::new(),
            expr: Expression::new(),
            rate: 0.0,
            time: 100.0,      
            updated: false      
        }
    }

    pub fn update_rate(&mut self) {
        
        match self.expr.eval() {
            Ok(v) => self.rate = v,
            Err(e) => {println!("An error ocurred during expression evaluation: {:?}", e); self.rate = 0.0; },
        };
    }

    pub fn update_time(&mut self, rng: &mut ThreadRng){
        
        match Exp::new(f64::abs(self.rate)) {
            Ok(exp) => {
                self.time = exp.sample(rng);
            },
            Err(e) => {
                println!("An error ocurred: {:?}", e); 
                self.time = 100.0;
            },
        }
    }

    pub fn update_rate_and_time(&mut self, rng: &mut ThreadRng) {        

        match self.expr.eval() {
            Ok(v) => self.rate = v,
            Err(e) => {println!("An error ocurred during expression evaluation: {:?}", e); self.rate = 0.0; },
        };
        let rate = f64::abs(self.rate);
        
        match Exp::new(rate) {
            Ok(exp) => {
                self.time = exp.sample(rng);
            },
            Err(e) => {
                println!("An error ocurred: {:?}", e); 
                self.time = 100.0;
            },
        } 
    }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct State {    
    name: String,
    initial_value: i32,
    value: i32,
    values: Vec<i32>, 
    reactions: Vec<usize>, 
}

impl State {
    pub fn new(name: String, value: i32) -> Self{
        let mut values = Vec::with_capacity(100);
        values.push(value);
        Self{ 
            name, 
            initial_value: value, 
            value: value, 
            values,
            reactions: vec![]
        }
    }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct StochasticModel {
    pub name: String,
    pub states: BTreeMap<String,State>,
    pub reactions: HashMap<usize,Reaction>,
    pub parameters: BTreeMap<String,f64>,     
    pub times: Vec<f64>
}

impl StochasticModel {

    pub fn new() -> Self {
        Self { 
            name: String::from("StochasticModel"), 
            states: BTreeMap::new(), 
            reactions: HashMap::new(), 
            parameters: BTreeMap::new(),             
            times: vec![] 
        }
    }

    pub fn set_initial_context(&mut self, rng: &mut ThreadRng){
        let mut context = ExprContext::new();
        for s in self.states.values(){
            context.insert_var(s.name.clone(), s.initial_value as f64);
        }
        for p in self.parameters.iter() {
            context.insert_var(p.0.to_string(), *p.1);        
        }
        for reaction in self.reactions.values_mut() {
            reaction.expr.context = context.clone();
            reaction.update_rate_and_time(rng);
        }
    }

    pub fn choose_reaction(&self) -> Reaction {
        let mut choice = Reaction::new();
        let mut min_time: f64 = 1000.0;
        for reaction in self.reactions.values() {
            if reaction.time < min_time {
                min_time = reaction.time;
                choice = reaction.clone();
            }
        }
        return choice
    }

    pub fn gillespie(&mut self, t_final: f64, fname: String, exec_index: usize){        

        let mut reactions_executed: Vec<Reaction> = vec![];
        let mut t: f64 = 0.0;
        self.times.push(t);

        let rng: &mut ThreadRng = &mut rand::thread_rng();
        self.set_initial_context(rng);
        
        let mut reaction_chosen = self.choose_reaction();        

        while t < t_final {            

            reactions_executed.push(reaction_chosen.clone());

            for (_, reaction) in self.reactions.iter_mut(){
                reaction.updated = false;
            }
            
            for state in self.states.values_mut(){
                state.values.push(state.value);
            }

            let mut states_to_update: Vec<State> = vec![];
            for (name, v) in reaction_chosen.updates.iter(){
                //atualiza o valor do state correspondente                 
                let state = self.states.get_mut(name).unwrap();
                if *v == 0 && reaction_chosen.rate < 0.0 {
                    state.value -= 1;                    
                }
                else if *v == 0 && reaction_chosen.rate > 0.0 {
                    state.value += 1;                    
                }
                else {
                    state.value += v;
                }
                
                let length = state.values.len();
                state.values[length - 1] = state.value;
                states_to_update.push(state.clone());
            }

            let mut reactions_to_update: Vec<usize> = vec![];
            //atualiza o contexto e reavalia somente as reacoes afetadas
            for state in states_to_update.iter(){
                for pos in state.reactions.iter(){
                    let r = self.reactions.get_mut(pos).unwrap();
                    r.expr.context.set_var(&state.name, state.value as f64);
                    reactions_to_update.push(*pos);                    
                }
            }

            for pos in reactions_to_update.iter(){
                let r = self.reactions.get_mut(pos).unwrap();
                r.update_rate_and_time(rng);
                r.updated = true;
            }
        
            reaction_chosen = self.choose_reaction();
            
            let mut dt = reaction_chosen.time;            
            
            if dt < f64::powi(10.0, -4) {
                dt = f64::powi(10.0, -4);
            }
            t += dt;
            self.times.push(t); 
            println!("t = {:?}", t);

            if t > t_final + 1.0 {
                self.times.remove(self.times.len() - 1);
                for state in self.states.values_mut(){
                    state.values.remove(state.values.len()-1);
                }
            }
        }

        StochasticModel::save_log(&LogData { reactions: reactions_executed}, &String::from(format!("{}{}{}", String::from("./tests/logs/"), exec_index, "_log.txt")));
        self.save_results( Path::new(&format!("{}{}{}", fname, exec_index, ".csv")));
    }    
    
    pub fn plot_results(&self, exec_index: usize) {
        let mut plot = Plot::new();
        plot.set_figure_size_points(900.0, 600.0);
        plot.set_labels("time (days)", "population");

        let grad = colorgrad::GradientBuilder::new()
            .html_colors(&["deeppink", "gold", "seagreen"])
            .build::<colorgrad::CatmullRomGradient>().unwrap();

        let p_size = self.states.len();    
        let mut color_ind: f32 = 0.0;        
        let color_inc: f32 = 1.0/(p_size as f32);
                
        for state in self.states.values() {
            
            let rgba = grad.at(color_ind);
            color_ind += color_inc;
            let color = rgba.to_hex_string();           

            let mut curve = Curve::new();
            curve.set_line_width(1.3);    
            curve.set_line_color(&color);
            curve.set_label(&state.name);

            let values = state.clone().values.into_iter().map(|x| x as f64).collect();
            curve.draw(&self.times, &values);
            //plot.set_ymin(-1.0);
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
        plot.save(&format!("{}{}{}", String::from("./tests/imgs/plot_"), exec_index, ".png")).unwrap();

    }

    /*let grad = colorgrad::GradientBuilder::new()
                .html_colors(&["deeppink", "gold", "seagreen"])
                .build::<colorgrad::CatmullRomGradient>().unwrap();*/
    pub fn plot(&self, exec_index: usize) {        
        let grad = colorgrad::GradientBuilder::new()
                .html_colors(&["deeppink", "gold", "seagreen"])
                .build::<colorgrad::CatmullRomGradient>().unwrap();

        let p_size = self.states.len();
        let mut color_ind: f32 = 0.0;
        let color_inc: f32 = 1.0/(p_size as f32);
        
        for state in self.states.values() {
            let mut plot = Plot::new();
            plot.set_figure_size_points(900.0, 600.0);
            plot.set_labels("time (days)", &state.name);   
            plot.set_cross(0.0, 0.0, "grey", "solid", 1.0);         
            
            let rgba = grad.at(color_ind);
            color_ind += color_inc;
            let color = rgba.to_hex_string();           

            let mut curve = Curve::new();
            curve.set_line_width(1.3);    
            curve.set_line_color(&color);
            curve.set_label(&state.name);

            let values = state.clone().values.into_iter().map(|x| x as f64).collect();
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
            plot.save(&format!("{}_{}_{}{}", String::from("./tests/imgs/plot"), state.name, exec_index, ".png")).unwrap();
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
        for state in self.states.iter() {
            write!(buf,", {}", state.0).unwrap();
        }        
        write!(buf, "\n").unwrap();

        let mut results: Vec<Vec<f64>> = vec![vec![0.0; self.states.len()]; self.times.len()];
        
        let mut j = 0;
        for state in self.states.values() { 
            let mut i: usize = 0;
            
            for v in state.values.iter() {
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

    pub fn save_log<P: AsRef<Path>>(log: &LogData, path: &P) {
        let file = match File::create(path) {
            Err(e) => {
                println!("Could not open file. Error: {:?}", e);
                return;
            }
            Ok(buf) => buf,
        };
        let mut buf = BufWriter::new(file);
    
        for reaction in log.reactions.iter() {
            write!(buf,"{}, ", reaction.id).unwrap();
        }
        
        if let Err(e) = buf.flush() {
            println!("Could not write to file. Error: {:?}", e);
        }
    }
}