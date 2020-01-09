extern crate ndarray;
extern crate rand;

mod state;

use ndarray::ArrayView1;
use numpy::{IntoPyArray, PyArray1};
use pyo3::prelude::{
    pyclass, pymethods, pymodule, Py, PyModule, PyObject, PyRawObject, PyResult, Python, ToPyObject,
};
use state::{Color, State, Value};

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

    fn play(&mut self, position: usize) -> String {
        match self.state.play(position) {
            Ok(_) => "".to_string(),
            Err(err) => format!("{:?}", err),
        }
    }

    fn discard(&mut self, position: usize) -> String {
        match self.state.discard(position) {
            Ok(_) => "".to_string(),
            Err(err) => format!("{:?}", err),
        }
    }

    fn clue_value(&mut self, target: usize, value: usize) -> String {
        match self.state.clue_value(target, Value::new(value)) {
            Ok(_) => "".to_string(),
            Err(err) => format!("{:?}", err),
        }
    }

    fn clue_color(&mut self, target: usize, color: &str) -> String {
        match self.state.clue_color(target, Color::from_str(color)) {
            Ok(_) => "".to_string(),
            Err(err) => format!("{:?}", err),
        }
    }

    fn encode(&self, py: Python) -> Py<PyArray1<f32>> {
        self.state.encode().into_pyarray(py).to_owned()
    }

    fn decode(&mut self, py: Python, x: &PyArray1<f32>) -> PyObject {
        let x: ArrayView1<f32> = x.as_array();
        match self.state.decode(&x) {
            Ok(score) => score.to_object(py),
            Err(err) => format!("{:?}", err).to_object(py),
        }
    }

    #[getter]
    fn get_turn(&self) -> usize {
        self.state.turn()
    }

    #[getter]
    fn get_score(&self) -> usize {
        self.state.score()
    }

    #[getter]
    fn get_deck(&self) -> Vec<String> {
        self.state.deck().iter().map(|x| format!("{}", x)).collect()
    }

    #[getter]
    fn get_history(&self) -> Vec<String> {
        self.state.history().iter().map(|x| format!("{}", x)).collect()
    }
}
