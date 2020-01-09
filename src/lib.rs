extern crate rand;

#[macro_use(array)]
extern crate ndarray;

mod state;

use ndarray::Array1;
use numpy::{IntoPyArray, PyArray1};
use pyo3::prelude::*;
use pyo3::prelude::{pymodule, Py, PyModule, PyResult, Python};
use state::{IllegalMoves, State, Value, Color};

#[pymodule]
fn hanabi(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Game>()?;
    Ok(())
}

#[pyclass]
struct Game {
    state: State,
}

#[pymethods]
impl Game {
    #[new]
    fn new(obj: &PyRawObject, nplayer: usize) {
        obj.init({
            Game {
                state: State::new(nplayer),
            }
        });
    }

    fn clue_value(&mut self, target: i32, value: i32) -> String {
        match self.state.clue_value(target, Value::new(value)) {
            Ok(_) => "".to_string(),
            Err(IllegalMoves::EmptyClue) => "empty clue".to_string(),
            Err(IllegalMoves::NoMoreClues) => "no clue".to_string(),
            Err(IllegalMoves::SelfClue) => "self clue".to_string(),
            Err(_) => panic!(),
        }
    }

    fn clue_color(&mut self, target: i32, color: &str) -> String {
        match self.state.clue_color(target, Color::new(color)) {
            Ok(_) => "".to_string(),
            Err(IllegalMoves::EmptyClue) => "empty clue".to_string(),
            Err(IllegalMoves::NoMoreClues) => "no clue".to_string(),
            Err(IllegalMoves::SelfClue) => "self clue".to_string(),
            Err(_) => panic!(),
        }
    }

    fn encode(&self, py: Python) -> Py<PyArray1<f32>> {
        let x: Array1<f32> = array![1.0, 2.0, 3.0, 4.0];
        x.into_pyarray(py).to_owned()
    }

    fn decode(&self, x: &PyArray1<f64>) -> String {
        let x = x.as_array();
        if x[0] > 0.0 {
            "".to_string()
        } else {
            "?".to_string()
        }
    }

    #[getter]
    fn get_turn(&self) -> i32 {
        self.state.turn()
    }
}
