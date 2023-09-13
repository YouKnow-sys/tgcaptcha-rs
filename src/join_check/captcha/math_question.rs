use std::fmt::Display;

use rand::{distributions::Uniform, seq::SliceRandom, thread_rng, Rng};

pub type Answer = u8;

const MIN: u8 = 1;
const MAX: u8 = 10;
const MAX_ANSWER: u8 = 100;

#[derive(Clone, Copy, PartialEq)]
enum Operator {
    Add,
    Sub,
    Mul,
}

impl Operator {
    const LIST: [Operator; 3] = [Self::Add, Self::Sub, Self::Mul];

    fn eval(self, lhs: u8, rhs: u8) -> Answer {
        match self {
            Self::Add => lhs + rhs,
            Self::Sub => lhs - rhs,
            Self::Mul => lhs * rhs,
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => '+',
            Self::Sub => '-',
            Self::Mul => '*',
        }
        .fmt(f)
    }
}

#[derive(Clone, Copy)]
pub struct MathQuestion {
    lhs: u8,
    operator: Operator,
    rhs: u8,
}

impl MathQuestion {
    pub fn generate_question() -> (Self, [Answer; 4]) {
        // in best case we should create the thread rng one time and then use it in the entire program...
        let mut rng = thread_rng();
        let lhs = rng.gen_range(MIN..MAX);
        let operator = *Operator::LIST.choose(&mut rng).unwrap();
        // if we want to sub the numbers,
        // then the first number need to be always greater or equal to the second one
        let rhs = rng.gen_range(MIN..=if operator == Operator::Sub { lhs } else { MAX });

        let answer = operator.eval(lhs, rhs);

        let range = Uniform::new_inclusive(MIN, MAX_ANSWER);
        let mut answers = [
            rng.sample(range),
            rng.sample(range),
            rng.sample(range),
            answer,
        ];
        answers.shuffle(&mut rng);

        (Self { lhs, operator, rhs }, answers)
    }

    pub fn validate_answer(self, answer: Answer) -> bool {
        self.operator.eval(self.lhs, self.rhs) == answer
    }
}

impl Display for MathQuestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}‌ {}‌ {}", self.lhs, self.operator, self.rhs)
    }
}
