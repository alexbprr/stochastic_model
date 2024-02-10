use node::Node;
use rand_distr::{Exp, Distribution};
mod node;

#[feature(entry_and_modify)]
use std::collections::HashMap;
use std::{thread, time::Duration}; 
use std::sync::{Arc, Mutex};

use crate::node::LeafNode;

#[derive(Clone,Debug)]
struct StochasticSimulation {
    initials: Vec<i32>, 
    propensities: Vec<Node>, //cada expressao é representada por um nó 
    stoichiometry: Vec<Vec<i32>>,
}

impl StochasticSimulation {

    pub fn new()-> Self {
        Self {
            initials: vec![],
            propensities: vec![],
            stoichiometry: vec![],            
        }
    }

    pub fn gillespie(&mut self, duration: f64){
        let mut times: Vec<f64> = vec![];
        times.push(0.0);
        println!("{:#?}", self);
        let mut values: Vec<Vec<i32>> = vec![self.initials.clone()];
        while times.last().unwrap() < &duration {
            //# get current state
            let state = values.last().unwrap();
            //Dado o estado atual, calcula as probabilidades de cada termo 
            let rates: Vec<f64> = vec![1.0];

            //escolhe e executa uma das reações 


            //atualiza o tempo e populations
            let exp = Exp::new(rates.iter().sum()).unwrap();
            let dt: f64 = exp.sample(&mut rand::thread_rng());            
            times.push(times.last().unwrap() + dt);                       
        }
        /*
            rates = [prop(*state) for prop in propensities]
    
            # stop loop if no transitions available
            if all(r == 0 for r in rates):
                break
    
            # randomly draw one transition
            transition = random.choices(stoichiometry, weights=rates)[0]
            next_state = [a + b for a, b in zip(state, transition)]
    
            # draw next time increment from random exponential distribution
            # dt = math.log(1.0 / random.random()) / sum(weights)
            dt = random.expovariate(sum(rates))
    
            # append new values
            times.append(times[-1] + dt)
            counts.append(next_state)
    
        return times, counts*/
    }

}


/*match simulator.populations.get("prey").unwrap().clone().as_mut() {
    Node::Leaf(value) => {
        *value = 100.0;
    },
    Node::UnaryExpr { op, child } => todo!(),
    Node::BinaryExpr { op, lhs, rhs } => todo!(),
}*/

fn main() {    
    let mut sto_simul: StochasticSimulation = StochasticSimulation::new();
    let duration: f64 = 10.0;

    let initials: Vec<f64> = vec![50.0, 20.0];
    let prey = Box::new(Node::Leaf(LeafNode { name: String::from("prey"), value: initials[0]}));
    let predator  = Box::new(Node::Leaf(LeafNode { name: String::from("predator"), value: initials[1]}));
    // sto_simul.populations.insert(String::from("predator"), predator.clone()) ;

    sto_simul.initials = initials.iter().map(|v| *v as i32).collect();

    let r = Box::new(Node::Leaf(LeafNode {name: String::from("r"), value: 0.05} ));
    let a = Box::new(Node::Leaf(LeafNode {name: String::from("a"), value: 0.01}));
    let m = Box::new(Node::Leaf(LeafNode {name: String::from("r"), value: 0.25}));

    sto_simul.propensities.push(Node::BinaryExpr { op: (node::Operator::Times), lhs: r, rhs: prey.clone() });
    let prey_x_predator = Node::BinaryExpr { op: (node::Operator::Times), lhs: prey, rhs: predator.clone() };
    sto_simul.propensities.push(Node::BinaryExpr { op: (node::Operator::Times), lhs: a, rhs: Box::new(prey_x_predator) });
    sto_simul.propensities.push(Node::BinaryExpr { op: (node::Operator::Times), lhs: m, rhs: predator });

    println!("eval result: {:?}", sto_simul.propensities[0].eval());

    sto_simul.stoichiometry = vec![
                    vec![1, 0], 
                    vec![-1, 1],
                    vec![0, -1]];

    sto_simul.gillespie(duration);

    /*const N_RUNS: usize = 1;
    let shared_state = Arc::new(Mutex::new(vec![sto_simul]));
    let mut thread_handles = vec![];
    
    for thread_id in 0..N_RUNS {    
        let thread_data: Arc<Mutex<Vec<StochasticSimulation>>> = Arc::clone(&shared_state);    
        let handle = thread::spawn( move || {
            let state: std::sync::MutexGuard<'_, Vec<StochasticSimulation>> = thread_data.lock().unwrap();
            println!("Executing thread {:?}", thread_id);
            state.get(thread_id).unwrap().gillespie(duration);            
        });
        thread_handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in thread_handles {
        handle.join().unwrap();
    }*/
}
