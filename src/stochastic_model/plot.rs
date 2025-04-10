
#[derive(Default, Clone)]
pub struct PlotInfo {
    pub title: String, //titulo do plot 
    pub xlabel: String, //nome do label x 
    pub ylabel: String, //nome do label y
    pub labels: Vec<String>, //nome das populacoes 
    pub colors: Vec<String>, //cores 
    pub x_min: f64, 
    pub x_max: f64, 
    pub y_min: f64, 
    pub y_max: f64, 
    //pub x: Vec<f64>,
    //pub y: Vec<f64>,
}

impl PlotInfo {

     //ler arquivo de configuracao e carregar as informações em PlotInfo
}