extern crate ndarray;
extern crate rand;

mod state;

use ndarray::ArrayView1;
use numpy::{IntoPyArray, PyArray1};
use pyo3::prelude::{
    pyclass, pymethods, pymodule, Py, PyModule, PyObject, PyRawObject, PyResult, Python, ToPyObject,
};
use state::{Color, IllegalMoves, State, Value};

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
        match self.state.play_discard(position) {
            Ok(_) => "".to_string(),
            Err(err) => format!("{:?}", err),
        }
    }

    fn clue(&mut self, py: Python, target: usize, info: PyObject) -> Option<String> {
        match if let Ok(value) = info.extract::<usize>(py) {
            if 1 <= value && value <= 5 {
                self.state.clue_value(target, Value::new(value - 1))
            } else {
                Err(IllegalMoves::Error)
            }
        } else if let Ok(color) = info.extract::<&str>(py) {
            if "rgbyp".to_string().contains(color) {
                self.state.clue_color(target, Color::from_str(color))
            } else {
                Err(IllegalMoves::Error)
            }
        } else {
            Ok(())
        } {
            Ok(_) => None,
            Err(err) => Some(format!("{:?}", err)),
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
        *self.state.turn()
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
        self.state
            .history()
            .iter()
            .map(|x| format!("{}", x))
            .collect()
    }
}
