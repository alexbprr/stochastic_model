use std::path::Path;

use expr_evaluator::{expr::{ExprContext, LeafNode, Node, NodeType, Operator}, lexer::{self, *}};
use expr_evaluator::expr::Operator::*;

use super::{Reaction, State, StochasticModel};

pub enum ParserError {
    FunctionNameNotFoundError
}

#[derive(Clone,Debug)]
pub struct StoParser {
    tokens: Vec<Token>,
    c_token: Token,
    last_token: Token,
    index: usize,
    reaction_id: usize,
    model: StochasticModel,
}

impl StoParser {

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            c_token: Token::new(0, 0, 0, TokenKind::Error(String::from(""))),
            last_token: Token::new(0, 0, 0, TokenKind::Error(String::from(""))),
            index: 0,
            reaction_id: 0,
            model: StochasticModel::new()
        }
    }

    fn next_token(&mut self) -> Token {
        if self.index < self.tokens.len() {
            self.last_token = self.tokens.get(self.index).unwrap().clone(); 
            self.index += 1;
        }        
        return self.last_token.clone();
    }

    fn back_token(&mut self) {
        self.index -= 1;
    }

    fn expr(&mut self) -> Box<Node>{
        let node: Box<Node> = self.termo();
        return self.adicao_opc(node);
    }

    fn termo(&mut self) -> Box<Node>{
        let node = self.fator();
        return self.termo_opc(node);
    }

    fn adicao_opc(&mut self, node: Box<Node>) -> Box<Node>{
        self.c_token = self.next_token();

        match self.c_token.token_type.clone() {            
            TokenKind::Operators(Operators::Plus) => { 
                let right_node = self.termo();
                let binary_node = Box::new(Node::BinaryExpr { op: Operator::Plus, left_expr: node, right_expr: right_node });
                return self.adicao_opc(binary_node);
            },
            TokenKind::Operators(Operators::Minus) => {
                let right_node = self.termo();
                let binary_node = Box::new(Node::BinaryExpr { op: Operator::Minus, left_expr: node, right_expr: right_node });
                return self.adicao_opc(binary_node);
            },
            _ => {
                self.back_token();
                return node
            }
        }
    }

    fn termo_opc(&mut self, node: Box<Node>) -> Box<Node>{
        self.c_token = self.next_token();
        match self.c_token.token_type.clone() {
            TokenKind::Operators(Operators::Multiplication) => {
                let right_node = self.fator();
                let binary_node = Box::new(Node::BinaryExpr { op: Operator::Mult, left_expr: node, right_expr: right_node });
                return self.termo_opc(binary_node);
            },
            TokenKind::Operators(Operators::Division) => {
                let right_node = self.fator();
                let binary_node = Box::new(Node::BinaryExpr { op: Operator::Div, left_expr: node, right_expr: right_node });
                return self.termo_opc(binary_node);
            },
            _ => {
                self.back_token();
                return node
            } 
        }
    }

    fn fator(&mut self) -> Box<Node>{        
        let mut is_unary: bool = false;
        let mut is_minus: bool = false;
        self.c_token = self.next_token();

        match self.c_token.token_type.clone(){
            TokenKind::Operators(Operators::Minus) => {
                is_unary = true;
                is_minus = true;
            }
            TokenKind::Operators(Operators::Plus) => {
                is_unary = true;
            }
            _ => (),
        }
        
        let node: Box<Node> = self.fator2(is_unary);
        if is_minus {
            return Box::new(Node::UnaryExpr { op: Minus, expr: node });            
        }
        else {
            return node;
        }
    }

    fn fator2(&mut self, is_unary: bool) -> Box<Node>{
        if is_unary {
            self.c_token = self.next_token();
        }
            
        match self.c_token.token_type.clone() {
            TokenKind::Identifier(lexeme) =>  {
                self.c_token = self.next_token(); 
                match self.c_token.token_type.clone(){
                    TokenKind::Punctuation(Punctuation::LParen) =>{
                        return self.chamada_funcao(lexeme);
                    },
                    _ => {                         
                        self.back_token();
                        
                        match self.model.states.get_mut(&lexeme) {
                            Some(state) => state.reactions.push(self.reaction_id),
                            None => (),
                        }                        

                        return Box::new(Node::Leaf(LeafNode { node_type: NodeType::Var, name: lexeme, value: 0.0, args: vec![] }))
                    }
                }
            },
            TokenKind::FloatConst(value) => {
                return Box::new(Node::Leaf(LeafNode { node_type: NodeType::Constant, name: value.to_string(), value: value, args: vec![] }))
            },
            TokenKind::IntConst(value) => {
                return Box::new(Node::Leaf(LeafNode { node_type: NodeType::Constant, name: value.to_string(), value: value as f64, args: vec![] }))
            },
            TokenKind::Punctuation(Punctuation::LParen) => {
                let node = self.expr();
                self.c_token = self.next_token(); 
                if self.c_token.token_type != TokenKind::Punctuation(Punctuation::RParen) {
                    println!("Erro sintatico na linha {}. ) esperado na entrada.", self.c_token.line_number);
                }
                return node;
            },
            _ => {
                println!("Erro sintatico na linha {}. Id, ( ou constante float esperados na entrada.", self.c_token.line_number);
                return Box::new(Node::Leaf(LeafNode::new(NodeType::Var, String::from("Error"))))
            }
        }
    }

    fn chamada_funcao(&mut self, function_name: String) -> Box<Node>{

        let mut function_node = LeafNode::new(NodeType::Function, function_name);
        let args = self.lista_args();
        function_node.args = args;
        
        if self.c_token.token_type != TokenKind::Punctuation(Punctuation::RParen) {
            println!("Erro sintatico na linha {}. ) esperado na entrada.", self.c_token.line_number);
        }
        
        return Box::new(Node::Leaf(function_node)) 
    }

    fn lista_args(&mut self) -> Vec<Box<Node>>{
        let mut args: Vec<Box<Node>> = vec![];
        self.c_token = self.next_token(); 
        match self.c_token.token_type.clone() {
            TokenKind::Operators(Operators::Plus) | TokenKind::Operators(Operators::Minus) | TokenKind::Identifier(_)
            | TokenKind::FloatConst(_) | TokenKind::IntConst(_)  | TokenKind::Punctuation(Punctuation::LParen) => {
                self.back_token();
                let node = self.expr();
                args.push(node);
                args = self.lista_args2(args);
            }            
            _ => ()            
        }

        return args
    }

    fn lista_args2(&mut self, mut args: Vec<Box<Node>>) -> Vec<Box<Node>>{
        self.c_token = self.next_token();
        match self.c_token.token_type.clone() {
            TokenKind::Punctuation(Punctuation::Comma) => {
                let node = self.expr();
                args.push(node);
                args = self.lista_args2(args);
            },
            _ => () 
        }
        return args
    }

    pub fn id_list(&mut self, reaction: &mut Reaction){
        
        self.c_token = self.next_token();
        match self.c_token.token_type.clone() {
            TokenKind::Operators(Operators::Minus) => {
                
                self.c_token = self.next_token();
                match self.c_token.token_type.clone() {
                    TokenKind::Identifier(id) => {
                        reaction.updates.insert(id, -1);

                        self.c_token = self.next_token();
                        match self.c_token.token_type.clone() {
                            TokenKind::Punctuation(Punctuation::Comma) => {
                                self.id_list(reaction);
                            }
                            _ => self.back_token(),
                        }
                    }
                    _ => ()
                }
            }
            TokenKind::Operators(Operators::Plus) => {
               
                self.c_token = self.next_token();
                match self.c_token.token_type.clone() {
                    TokenKind::Identifier(id) => {
                        reaction.updates.insert(id, 1);

                        self.c_token = self.next_token();
                        match self.c_token.token_type.clone() {
                            TokenKind::Punctuation(Punctuation::Comma) => {
                                self.id_list(reaction);
                            }
                            _ => self.back_token(),
                        }
                    }
                    TokenKind::Operators(Operators::Minus) => {
                        self.c_token = self.next_token();
                        match self.c_token.token_type.clone() {
                            TokenKind::Identifier(id) => {
                                reaction.updates.insert(id, 0);

                                self.c_token = self.next_token();
                                match self.c_token.token_type.clone() {
                                    TokenKind::Punctuation(Punctuation::Comma) => {
                                        self.id_list(reaction);
                                    }
                                    _ => self.back_token(),
                                }
                            }
                            _ => self.back_token(),
                        }
                    }
                    _ => ()
                }
            }
            _ => ()
        }   

    }

    pub fn reaction_list(&mut self){
        //println!("---------reaction_list--------");

        let mut reaction = Reaction::new();
        reaction.expr.ast = Some(self.expr());
        
        self.c_token = self.next_token();
        match self.c_token.token_type.clone() {
            TokenKind::Operators(Operators::Arrow)=>{
                
                self.id_list(&mut reaction);
                //println!("token: {:?}", self.c_token.token_type);

                if self.c_token.token_type != TokenKind::Punctuation(Punctuation::Semicolon) {
                    println!("Erro sintatico na linha {}. \";\" esperado na entrada.", self.c_token.line_number);
                }
                reaction.id = self.reaction_id;
                self.model.reactions.insert(self.reaction_id, reaction);
                self.reaction_id += 1;
            }
            _ => ()
        }

        self.c_token = self.next_token();
        self.c_token = self.next_token();  
        //println!("next token: {:?}", self.c_token.token_type);
        match self.c_token.token_type.clone() {
            TokenKind::Identifier(_) | TokenKind::FloatConst(_) | TokenKind::IntConst(_) | TokenKind::Punctuation(Punctuation::LParen) => {                  
                self.back_token();      
                self.reaction_list();
            }
            _ => ()
        }
    }

    pub fn assign(&mut self, is_states: bool){

        self.c_token = self.next_token();
        match self.c_token.token_type.clone() {
            TokenKind::Identifier(id) => {
                self.c_token = self.next_token();
                if self.c_token.token_type == TokenKind::Operators(Operators::Assign) {
                    
                    self.c_token = self.next_token();
                    match self.c_token.token_type.clone() {
                        TokenKind::IntConst(v) => {
                            if is_states {
                                let name = id.clone();
                                self.model.states.insert(name.clone(),State::new(name.clone(), v));                               
                            }
                            else {
                                self.model.parameters.insert(id, v as f64);
                            }
                        }
                        TokenKind::FloatConst(v) => {
                            if is_states {
                                let name = id.clone();
                                self.model.states.insert(name.clone(),State::new(name.clone(), v as i32));
                            }
                            else {
                                self.model.parameters.insert(id, v);
                            }                            
                        }
                        _ => ()
                    }
                    
                    self.c_token = self.next_token();
                    if ! (self.c_token.token_type == TokenKind::Punctuation(Punctuation::Semicolon)) {
                        println!("Erro sintático! ; esperado na linha {:?}", self.c_token.line_number);
                        self.back_token();
                    }
                }
            }
            _ => ()
        }
    }

    pub fn assign_list(&mut self, is_states: bool){
        self.c_token = self.next_token();
        match self.c_token.token_type.clone() {
            TokenKind::Identifier(_id) => {
                self.back_token();
                self.assign(is_states);
                self.assign_list(is_states);
            }
            _ => ()
        }
    }

    pub fn parse(&mut self) {
        self.model = StochasticModel::new();
        self.c_token = self.next_token();        
        
        match self.c_token.token_type.clone() {
            TokenKind::ReservedWords(ReservedWords::Populations) => {
                self.assign_list(true);
            }
            _ => () 
        }        
        
        match self.c_token.token_type.clone() {
            TokenKind::ReservedWords(ReservedWords::Params) => {
                self.assign_list(false);
            }
            _ => () 
        }

        match self.c_token.token_type.clone() {
            TokenKind::ReservedWords(ReservedWords::Reactions) => {
                self.reaction_list();
            }
            _ => () 
        }       

    }    
}

pub fn parse_input(fname: String) -> StochasticModel{
    let tokens = lexer::tokenize_file(Path::new(&fname));
    let mut parser = StoParser::new(tokens);
    parser.parse();
    //parser.model.save_model(&Path::new(&String::from(fname.to_string() + ".json")));
    return parser.model;
}
