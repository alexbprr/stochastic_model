#[derive(Clone,Debug)]
pub enum Operator {
    Plus,
    Minus,
    Times,
    Div,
}

#[derive(Clone,Debug)] 
pub struct LeafNode {
    pub name: String,
    pub value: f64,
}

#[derive(Clone,Debug)] 
//cada nó precisa ter um nome para que o valor dele possa ser atualizado a cada passo de tempo
//as atualizacoes ocorrem somente nos nós folhas que representam uma populacao
pub enum Node {
    Leaf(LeafNode), //can be a constant or population
    UnaryExpr {
        op: Operator,
        child: Box<Node>,
    },
    BinaryExpr {
        op: Operator,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}

impl Node {
    //eval the term associated to this node
    pub fn eval(&self) -> f64 {
        match self {
            Node::Leaf(n) => (*n).value,
            Node::UnaryExpr { op, child } => {
                let child_value = child.eval();
                match op {
                    Operator::Plus => child_value,
                    Operator::Minus => - child_value,
                    _ => child_value,
                }
            }
            Node::BinaryExpr {op, lhs, rhs } => {
                let lhs_value: f64 = lhs.eval();
                let rhs_value: f64 = rhs.eval();
                match op {
                    Operator::Plus => lhs_value + rhs_value,
                    Operator::Minus => lhs_value - rhs_value,
                    Operator::Times => lhs_value * rhs_value,
                    Operator::Div => lhs_value / rhs_value,
                }
            }
        }
    }

    pub fn update_node(p_name: String){
        
    }

}