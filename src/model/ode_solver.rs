use ode_solvers::dop853::*;
use ode_solvers::*;
use std::{fs::File, io::{BufWriter, Write}, path::Path};

pub type State = DVector<f64>;

pub struct OdeProblem {
    constants: State,
}

impl OdeProblem {
    pub fn solve(t_ini: f64, t_final: f64, dt: f64, y0: State, constants: State) -> Vec<State> {
        // Create the structure containing the problem specific constant and equations.
        let system: OdeProblem = OdeProblem {
            constants: constants,
        };
        // Create a stepper and run the integration.
        let mut stepper = 
                Dop853::new(system, t_ini, t_final, dt, y0, 1.0e-8, 1.0e-8);
        let res = stepper.integrate();

        // Handle result
        match res {
            Ok(stats) => {
                let result = stepper.y_out().to_vec();
                return result;
            }
            Err(e) => {println!("An error occured: {}", e); return vec![]; } ,
        }        
    }

    pub fn save(times: &Vec<f64>, states: &Vec<State>, filename: &Path) {
        // Create or open file
        let file = match File::create(filename) {
            Err(e) => {
                println!("Could not open file. Error: {:?}", e);
                return;
            }
            Ok(buf) => buf,
        };
        let mut buf = BufWriter::new(file);

        // Write time and state vector in a csv format
        for (i, state) in states.iter().enumerate() {
            buf.write_fmt(format_args!("{:.6}", times[i])).unwrap();
            for val in state.iter() {
                buf.write_fmt(format_args!(", {}", val)).unwrap();
            }
            buf.write_fmt(format_args!("\n")).unwrap();
        }
        if let Err(e) = buf.flush() {
            println!("Could not write to file. Error: {:?}", e);
        }
    }
}

impl ode_solvers::System<f64, State> for OdeProblem {
    fn system(&self, _t: f64, y: &State, dy: &mut State) {
        let alpha = self.constants[0];
        let gamma = self.constants[1];
        let beta = self.constants[2];
        let s = y[0];
        let i = y[1];
        let r = y[2];
        dy[0] = -alpha*s*i + beta*r;
        dy[1] = alpha*s*i - gamma*i;
        dy[2] = gamma*i - beta*r;
    }
}