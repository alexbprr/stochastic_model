mod utils;
mod csvdata;
use std::borrow::BorrowMut;
#[feature(entry_and_modify)]
use std::collections::HashMap;
use std::process::Output;
use std::{path::Path, vec};
use charming::component::{Axis, Title};
use charming::element::{AxisType, Color, LineStyle};
use charming::series::Line;
use charming::theme::Theme;
use charming::{ImageRenderer, ImageFormat};
use charming::{
    component::Legend,
    element::ItemStyle,
    Chart
};
use mexprp::{Answer, Expression};
use pest_derive::Parser;
use pest::Parser;
use rand_distr::{Distribution, Exp};
use random_choice::random_choice;
use regex::Regex;

use crate::model::csvdata::CSVData;

//type of a value: metadata, population or constant
#[derive(Clone,Debug)]
pub enum CType {
    Metadata = 0,
    Population = 1,
    Constant = 2,
}

#[derive(Clone,Debug)]
pub enum ValueType {
    float(f64),
    string(String)
}

#[derive(Clone,Debug)]
pub struct ModelValue {
    component_type: CType, 
    //value_type: ValueType,
    value_type: f64,
}

impl ModelValue {
    pub fn new(c_type: CType, v_type: f64) -> Self {
        Self {
            component_type: c_type,
            value_type: v_type,
        }
    }
}

#[derive(Clone,Debug)]
pub struct Reaction {
    pub expr_text: String,
    pub numeric_expr: String,
    pub inputs: Vec<String>, //cada variavél do lado esquerdo é um input 
    pub outputs: Vec<(String,i32)>, //cada variavél do lado direito é um output 
    pub rate: f64,
}

impl Reaction {

    pub fn new() -> Self {
        Self {
            expr_text: String::from(""),
            numeric_expr: String::from(""),
            inputs: vec![],
            outputs: vec![],
            rate: 0.0,
        }
    }

    pub fn calculate_rate(&mut self) -> f64 {
        let expr: Expression<f64> = Expression::parse(self.numeric_expr.as_str()).unwrap();
        let res: Result<mexprp::Answer<f64>, mexprp::MathError> = expr.eval();
        
        if let Ok(Answer::Single(expr_value)) = res {
            self.rate = expr_value;
            return expr_value;
        } 
        return 0.0;
    }
}

#[derive(Clone,Debug)]
pub struct Model {
    values: HashMap<String,ModelValue>, //constants and populations
    reactions: Vec<Reaction>,
    states: Vec<Vec<i32>>,
    populations: Vec<String>,
    times: Vec<f64>,
    odes: HashMap<String,String>,
}

#[derive(Parser)]
#[grammar = "grammar/model.pest"]
pub struct ModelParser;

impl Model {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            reactions: vec![],
            states: vec![],
            populations: vec![],
            times: vec![],
            odes: HashMap::new(),
        }
    }    

    fn parse_input<P: AsRef<Path>>(&mut self, path: P) {
        let mut input: String = String::from("");
        if let Ok(data) = utils::from_disk(path){
            input = data.into_iter().collect::<String>();
        }        

        match ModelParser::parse(Rule::document,  input.as_str()) {
            Ok(parse) => {
                for pair in parse.into_iter(){        
                    // match the rule, as the rule is an enum        
                    match pair.as_rule(){
                        Rule::document =>{
                            
                            for rule in pair.into_inner() {
        
                                let current_rule: Rule = rule.as_rule();
                                if matches!(current_rule, Rule::Reactions) == false {
        
                                    let mut c_type: CType = CType::Population;
                                    if matches!(current_rule, Rule::Metadata){
                                        c_type = CType::Metadata;
                                    }
                                    else if matches!(current_rule, Rule::Constants){
                                        c_type = CType::Constant;
                                    }
                                    
                                    for child_rule in rule.into_inner() { 
                                        let mut key: String = String::from("");
                                                                
                                        if matches!(child_rule.as_rule(), Rule::Assign){
                                            
                                            for pair in child_rule.into_inner() {
                                                if matches!(pair.as_rule(), Rule::Identifier){
                                                    key = pair.as_span().as_str().trim().to_string();
                                                    if matches!(c_type, CType::Population) {
                                                        self.populations.push(key.clone());
                                                    }
                                                }
                                                else if matches!(pair.as_rule(), Rule::Value){
                                                    let value = pair.as_span().as_str().parse::<f64>(); 
                                                    if value.is_err() == false {
                                                        let v: f64 = value.unwrap();
                                                        //self.values.insert(key.clone(), ModelValue::new(c_type.clone(), ValueType::float(v)));
                                                        self.values.insert(key.clone(), ModelValue::new(c_type.clone(), v));
                                                    }                                                                                               
                                                }
                                            }   
                                        }   
        
                                    }
                                }
                                else {
                                    
                                    let mut new_reaction: Reaction = Reaction::new();
                                    for child_rule in rule.into_inner() { //Expr level                             
                                        //test left and right expressions
                                        
                                        if matches!(child_rule.as_rule(), Rule::LeftExpr){                                
                                            new_reaction.expr_text = child_rule.as_span().as_str().trim().to_string();      
        
                                            for grand_child_rule in child_rule.into_inner() { //Fator level
                                            
                                                if matches!(grand_child_rule.as_rule(), Rule::Fator){
                                                    /*for last_rule in grand_child_rule.clone().into_inner() {
                                                        if matches!(last_rule.as_rule(), Rule::Identifier){
                                                        }
                                                        else if matches!(last_rule.as_rule(), Rule::Number){

                                                        }
                                                    }*/
                                                    new_reaction.inputs.push(grand_child_rule.as_span().as_str().trim().to_string());    
                                                }
                                            }
                                        }
                                        else if matches!(child_rule.as_rule(), Rule::RightExpr) {
                                            
                                            let mut sign: char = '+';
                                            for grand_child_rule in child_rule.into_inner() { //Identifier level                                    
                                                let right_expr: String = grand_child_rule.as_span().as_str().trim().to_string();
                                                
                                                if right_expr == "-"{
                                                    sign = '-';
                                                }
                                                else {
                                                    if sign == '-' {
                                                        new_reaction.outputs.push((right_expr,-1));
                                                    }
                                                    else {
                                                        new_reaction.outputs.push((right_expr,1));
                                                    } 
                                                    sign = '+';
                                                }                                                                                        
                                            }
                                            self.reactions.push(new_reaction); 
                                            new_reaction = Reaction::new();
                                        }  
                                                    
                                    }                         
        
                                } //end reactions                    
                            }
                        } //end document 
                        // as we have  parsed document, which is a top level rule, there
                        // cannot be anything else
                        _ => unreachable!()
                    }
                }
            }
            Err(e) => { println!("Error: the document syntax is incorrect! More details: {:?}", e);}
        }
        
    }

    pub fn update_reactions(&mut self) {
        
        for reaction in self.reactions.iter_mut() {
            let mut test: String = reaction.expr_text.clone();
            for input in &reaction.inputs {
                test = test.replace(&input.clone(), self.values.get(input).unwrap().value_type.clone().to_string().as_str());
            }
            reaction.numeric_expr = test.clone();        
        }
    }

    pub fn calculate_rates(&mut self){
        for reaction in self.reactions.iter_mut() {
            reaction.calculate_rate();
        }
    }

    fn update_population(&self) -> Vec<i32>{
        let mut state: Vec<i32> = vec![];
        for pop in self.populations.iter(){
            if let Some(value) = self.values.get(pop){
                state.push(value.value_type as i32);
            }
        }
        return state
    }

    pub fn evaluate_odes(&self) -> Vec<(String,f64)>{
        let mut results: Vec<(String,f64)> = vec![];

        for ode in self.odes.iter() {
            let mut test: String = ode.1.clone();
            let re = Regex::new(r"[-+*/]").unwrap();
        
            //re.find_iter(&test).map(|m| m.as_str().to_string()).collect();
            let operands: Vec<String> = re.split(&test).into_iter().map(|v| v.to_string()).collect(); 
            println!("operands: {:?}", operands);
            for operand in operands.iter(){
                if ! operand.is_empty() {
                    test = test.replace(&operand.clone(), self.values.get(operand).unwrap().value_type.clone().to_string().as_str());  
                }                
            }
            
            println!("test: {:?}", test);

            let expr: Expression<f64> = Expression::parse(test.as_str()).unwrap();
            let res: Result<mexprp::Answer<f64>, mexprp::MathError> = expr.eval();
        
            if let Ok(Answer::Single(expr_value)) = res {
                results.push((ode.0.clone(),expr_value));
            } 
        }
        return results
    }

    pub fn build_odes(&mut self){                  

        for pop in self.populations.iter(){
            
            let mut equation: String = String::from("");

            for reaction in self.reactions.iter(){
             
                for output in reaction.outputs.iter(){
              
                    if *pop == (*output).0 {
                        //println!("{:#?}", output);
                        //println!("{:#?}", pop);
                        //println!("reaction = {:?}", reaction);
                        if output.1 == -1 {
                            equation.push_str("-");
                        }
                        else {
                            equation.push_str("+");
                        }
                        equation.push_str(&reaction.expr_text);
                    }
                }    
            }
            self.odes.insert(pop.clone(), equation.clone());
            println!("ode for {:?}: {:?}", pop, equation);
        }
    }

    pub fn gillespie(&mut self, f_name: String){
        let mut file_name = f_name.clone();
        let mut outfile_name = f_name.clone();
        file_name.push_str(".txt");
        self.parse_input::<String>(file_name);
        
        if self.values.contains_key("t_ini"){
            self.times.push(self.values.get("t_ini").unwrap().value_type);            
        }
        else {
            self.times.push(0.0);
        }
        let mut t_final: f64 = 10.0;
        if self.values.contains_key("t_final"){
            t_final = self.values.get("t_final").unwrap().value_type;
        }
        
        self.update_reactions();
        self.states.push(self.update_population());
        
        while self.times.last().unwrap() < &t_final {
            //println!("state (t = {:?}): {:#?}", self.times.last().unwrap(), self.states.last().unwrap());
                    
            self.calculate_rates();

            let sum: f64 = self.reactions
                                .iter()
                                .fold(0.0, |acc,r| acc + r.rate);
            
            let weights: Vec<f64> = self.reactions
                                        .iter()
                                        .map(|r| r.rate/sum)
                                        .collect();

            let choices = random_choice()
                                            .random_choice_f64(&self.reactions, &weights, 1);
            for choice in choices {
                //println!("reaction chosen = {:?}", choice);
                
                for output in choice.outputs.iter() {
                    self.values
                        .entry(output.0.clone())
                        .and_modify(|v: &mut ModelValue| v.value_type = v.value_type + output.1 as f64);
                }
            } 

            //println!("values: {:#?}", self.values);           
            
            let exp: Exp<f64> = Exp::new(sum).unwrap();
            let dt: f64 = exp.sample(&mut rand::thread_rng());            
            self.times.push(self.times.last().unwrap() + dt);
            
            self.states.push(self.update_population());
            self.update_reactions();            
        }

        outfile_name.push_str("_result.csv");
        CSVData::save_data(self, &outfile_name).unwrap();
        
        //save times in a separate file 
    }    

    pub fn plot_results<P: AsRef<Path>>(&self, path: P){        
    }
}