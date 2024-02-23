use std::{io::Read, path::Path};

use csv::{Writer, WriterBuilder};

use super::Model;

#[derive(Default, Debug)]
pub struct CSVData {
    pub labels: Vec<String>,
    pub lines: Vec<Vec<f64>>,
    pub time: Vec<f64>,
}

impl CSVData {

    pub fn load_data(reader: impl Read) -> std::io::Result<Self> {
        let mut rdr = csv::Reader::from_reader(reader);

        let mut data = CSVData::default();

        if rdr.has_headers() {
            data.labels = rdr
                .headers()
                .unwrap()
                .iter()
                .map(|v| v.to_string())
                .collect();
        }

        let n_cols = data.labels.len();

        let mut populations: Vec<Vec<f64>> = (0..n_cols).map(|_| Vec::new()).collect();

        for record in rdr.records() {
            println!("record = {:?}", record);
            record?
                .into_iter()
                .map(|v| v.trim())
                .map(|v| v.parse())
                .zip(populations.iter_mut())
                .try_for_each(|(value, population)| {
                    println!("value: {:?}", value);
                    population.push(value?);
                    Result::<(), std::num::ParseFloatError>::Ok(())
                })
                .unwrap();
        }

        data.time = populations.remove(0);
        data.lines = populations;
        data.labels.remove(0);

        Ok(data)
    }

    pub fn save_data(model: &Model) -> Result<(), Box<dyn std::error::Error>>{
        let writer_result = Writer::from_path(Path::new("./src/tests/simulation_result.csv"));
        let mut writer = match writer_result {
            Ok(writer) => writer,
            Err(err) => return Err(Box::new(err)),
        };
        writer.write_record(&model.populations).unwrap();
        let mut index = 0;
        for state in model.states.iter() {
            //let time_state: Vec<f64> = state.insert(0, model.times[index]);

            let states_record: Vec<String> = state.iter().map(|v| v.to_string()).collect();
            let record_result = match writer.write_record(states_record) {
                 Ok(record_result) => record_result,
                 Err(err) => return Err(Box::new(err)),            
            };
            index += 1;
        }
        Ok(())
    }

    pub fn population_count(&self) -> usize {
        self.lines[0].len()
    }
}