use std::{collections::HashMap, fs, path::Path};
use colorgrad::Gradient;
use plotpy::{Curve, Legend, Plot};

#[derive(Default, Debug, Clone)]
pub struct CSVData {
    pub labels: Vec<String>,
    pub states: HashMap<String,Vec<f64>>,
    pub all_data: HashMap<String,Vec<Vec<f64>>>,
    pub times: Vec<Vec<f64>>,
}

impl CSVData {

    pub fn load_data(dir_path: &str) -> std::io::Result<Self> {
        let dir: &Path = &Path::new(dir_path);
        let mut data = CSVData::default();
        
        let mut index = 0;
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                match csv::Reader::from_path(path.clone()) {
                    Ok(mut reader) => {

                        let mut times: Vec<f64> = vec![];
                        //println!("path = {:?}", path);
                        if reader.has_headers() {
                            data.labels = reader
                                    .headers()
                                    .unwrap()
                                    .iter()
                                    .map(|v| v.trim().to_string())
                                    .collect(); 
                            
                            if index == 0 {
                                for label in data.labels.iter(){                                
                                    data.all_data.insert(label.trim().to_string(), vec![]);                                    
                                }
                            }
                            for label in data.labels.iter(){
                                data.states.insert(label.trim().to_string(), vec![]);
                            }
                            
                        }
                        let n_cols = data.labels.len();                                                
                        
                        let records_iter = reader.records();
                        for record in records_iter {
                            let str_record = record.unwrap();
                            
                            let mut j = 0;
                            for v in str_record.iter(){
                                let value = v.trim();
                                let label = &data.labels[j];
                                if j == 0 {
                                    times.push(value.parse::<f64>().unwrap());
                                }
                                else {
                                    let data_entry = data.states.get_mut(label).unwrap();
                                    data_entry.push(value.parse::<f64>().unwrap());       
                                }
                                j += 1;
                            }
                        }

                        for j in 1..n_cols {
                            let label = &data.labels[j];
                            let values = data.all_data.get_mut(label).unwrap();
                            values.push(data.states.get(label).unwrap().to_vec());
                        }

                        data.times.push(times);
                        data.labels.remove(0);
                                                
                    },
                    Err(e) => { println!("An error ocurred {:?}", e); },
                } 

                index += 1;               
            }
        }
    
        //println!("all data: {:#?}", data.all_data);
        //println!("times: {:#?}", data.times);
        Ok(data)
    }

    pub fn plot_all_data(&self){

        let grad = colorgrad::GradientBuilder::new()
                .html_colors(&["deeppink", "gold", "darkgreen"])
                .build::<colorgrad::CatmullRomGradient>().unwrap();
        println!("gradient = {:#?}", grad);

        let p_size = self.labels.len();        
        let n_execs: usize = self.times.len();
        
        for label in self.labels.iter() {

            let mut color_ind: f32 = 0.0;
            let color_inc: f32 = 1.0/(n_execs as f32);

            let mut plot = Plot::new();
            plot.set_figure_size_points(900.0, 600.0);
            plot.set_labels("time (days)", label);
    
            for i in 0..n_execs {
                
                let rgba = grad.at(color_ind);
                color_ind += color_inc;
                let color = rgba.to_hex_string();           
    
                let mut curve = Curve::new();
                curve.set_line_width(1.3);    
                curve.set_line_color(&color);
                let name = format!("{}_{}", label, i+1);
                curve.set_label(&name);
                
                match self.all_data.get(&label.clone()) {
                    Some(vec) => {
                        //println!("{:?}", vec[i]);
                        curve.draw(&self.times[i], &vec[i]);
                        plot.add(&curve);
                    },
                    None => {
                        println!("Error! Data not found for {:?}", label);
                    },
                }
                
            }
            let mut legend = Legend::new();
            legend.set_fontsize(11.0)
                .set_handle_len(1.0)
                .set_num_col(n_execs/5 + 1)
                .set_outside(true)
                .set_show_frame(false);
            legend.draw();

            plot.add(&legend);
            plot.save(&format!("{}{}{}", String::from("./tests/imgs/plot_"), label, ".png")).unwrap();
        }
            
    }
}
