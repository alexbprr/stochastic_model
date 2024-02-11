mod utils;

use std::{collections::HashMap, path::Path};
use mexprp::{Answer, Expression};
use pest_derive::Parser;
use pest::Parser;
use rand_distr::{Distribution, Exp};

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
    value_type: ValueType,
}

impl ModelValue {
    pub fn new(c_type: CType, v_type: ValueType) -> Self {
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
            println!("value = {:?}", expr_value);
            self.rate = expr_value;
            return expr_value;
        } 
        return 0.0;
    }
}

#[derive(Clone,Debug)]
pub struct Model {
    pub values: HashMap<String,ModelValue>, //constants and populations
    pub reactions: Vec<Reaction>,
    initials: Vec<i32>,
}

#[derive(Parser)]
#[grammar = "grammar/model.pest"]
pub struct ModelParser;

impl Model {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            reactions: vec![],
            initials: vec![],
        }
    }    

    pub fn parse_input(&mut self, path: &Path) {
        let mut input: String = String::from("");
        if let Ok(data) = utils::from_disk(path){
            input = data.into_iter().collect::<String>();
        }        

        let parse = ModelParser::parse(Rule::document,  input.as_str()).unwrap();

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
                                        }
                                        else if matches!(pair.as_rule(), Rule::Value){
                                            let value = pair.as_span().as_str().parse::<f64>(); 
                                            if value.is_err() == false {
                                                let v: f64 = value.unwrap();
                                                self.values.insert(key.clone(), ModelValue::new(c_type.clone(), ValueType::float(v)));
                                            }
                                            else {
                                                self.values.insert(key.clone(), ModelValue::new(c_type.clone(), ValueType::string(pair.as_span().as_str().to_string())));
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

    pub fn print_reactions(&mut self) {
        
        for reaction in self.reactions.iter_mut() {
            let mut test: String = reaction.expr_text.clone();
            for input in &reaction.inputs {
                let value_type: ValueType = self.values.get(input).unwrap().value_type.clone();
                if let ValueType::float(v) = value_type {                    
                    test = test.replace(&input.clone(), v.to_string().as_str());                    
                }
            }
            reaction.numeric_expr = test.clone();
            println!("{:?}", reaction.numeric_expr);
        }
    }

    pub fn calculate_rates(&mut self){
        for reaction in self.reactions.iter_mut() {
            reaction.calculate_rate();
        }
    }
    
    
    //itera pelas reacoes, testando se a probabilidade é menor do que rate/rate_sum para escolher a reacao a ser executada  
    pub fn choose_reaction(&mut self, probability: f64) {
    
    }   

    pub fn gillespie(&mut self){
        let mut times: Vec<f64> = vec![];
        times.push(0.0);
        
        println!("{:#?}", self);
        let mut values: Vec<Vec<i32>> = vec![self.initials.clone()];
        
        while times.last().unwrap() < &10.0 {
            //# get current state
            let state = values.last().unwrap();
            //Dado o estado atual, calcula as probabilidades de cada termo 
            let rates: Vec<f64> = vec![1.0];

            //escolhe e executa uma das reações 
            //rates = [prop(*state) for prop in propensities]

            //atualiza o tempo e populations
            let exp = Exp::new(rates.iter().sum()).unwrap();
            let dt: f64 = exp.sample(&mut rand::thread_rng());            
            times.push(times.last().unwrap() + dt);   
        }

    }
}