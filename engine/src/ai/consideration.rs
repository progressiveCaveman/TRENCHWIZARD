use serde::Serialize;
use serde::Deserialize;

use super::input::InputType;


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Consideration {
    pub name: String,
    pub input_type: InputType,
    pub params: ConsiderationParam,
}

impl Consideration {
    pub fn new(name: String, input_type: InputType, params: ConsiderationParam) -> Consideration {
        Consideration {
            name: name,
            input_type: input_type,
            params: params,
        }
    }

    pub fn get_score(&self, input: f32) -> f32 {
        let t = &self.params.t;
        let m = self.params.m;
        let k = self.params.k;
        let c = self.params.c;
        let b = self.params.b;

        let score = match t {
            ResponseCurveType::Const => m * input,
            ResponseCurveType::Quadratic | ResponseCurveType::Linear => m * (input - c).powf(k) + b,
            ResponseCurveType::Logistic => {
                let e = std::f64::consts::E as f32;
                k * 1. / (1. + (1000. * e * m).powf(-1. * input + c)) + b
            }
            ResponseCurveType::GreaterThan => {
                if input > m {
                    1.
                } else {
                    0.
                }
            }
            ResponseCurveType::LessThan => {
                if input < m {
                    1.
                } else {
                    0.
                }
            }
        };

        return score.clamp(0., 1.);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConsiderationParam {
    pub t: ResponseCurveType,
    pub m: f32,
    pub k: f32,
    pub c: f32,
    pub b: f32,
}

impl ConsiderationParam {
    pub fn new_const(v: f32) -> ConsiderationParam {
        ConsiderationParam {
            t: ResponseCurveType::Const,
            m: v,
            k: 0.,
            c: 0.,
            b: 0.,
        }
    }
}

/*
for types Const, GreaterThan, and LessThan, only m is considered
Linear
Quadratic
logisitic
Logit

Paramters - m,k,c,b

Linear/quad: y=m*(x-c)^k + b
m = slope
k = exponent
b = vert shift
c = horiz shift

Logistic: y = (k * (1/(1+1000em^(-1x+c)))) + b
m=slope of inflection
k=vertical size of curve
b=vert shift
c=horiz shift
*/
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ResponseCurveType {
    Const,
    GreaterThan,
    LessThan,
    Linear,
    Quadratic,
    Logistic,
}