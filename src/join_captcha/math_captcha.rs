use std::fmt::Display;

use rand::{distributions::Uniform, seq::SliceRandom, thread_rng, Rng};

pub type Answer = u8;

const MIN: u8 = 1;
const MAX: u8 = 10;
const MAX_ANSWER: u8 = 100;

#[derive(Clone, Copy, PartialEq)]
enum Operators {
    Add,
    Sub,
    Mul,
}

impl Operators {
    const LIST: [Operators; 3] = [Self::Add, Self::Sub, Self::Mul];

    fn eval(&self, num1: u8, num2: u8) -> u8 {
        match self {
            Self::Add => num1 + num2,
            Self::Sub => num1 - num2,
            Self::Mul => num1 * num2,
        }
    }
}

impl Display for Operators {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => '+',
            Self::Sub => '-',
            Self::Mul => '*',
        }
        .fmt(f)
    }
}

#[derive(Clone)]
pub struct Question {
    num1: u8,
    operator: Operators,
    num2: u8,
}

impl Question {
    pub fn generate_question() -> (Self, [Answer; 4]) {
        // in best case we should create the thread rng one time and then use it in entire program...
        let mut rng = thread_rng();
        let num1 = rng.gen_range(MIN..MAX);
        let operator = *Operators::LIST.choose(&mut rng).unwrap();
        let num2 = rng.gen_range(
            MIN..=if operator == Operators::Sub {
                num1 // if we want to sub the numbers the first number need to be always bigger then second one
            } else {
                MAX
            },
        );

        let answer = operator.eval(num1, num2);

        let range = Uniform::new_inclusive(MIN, MAX_ANSWER);
        let mut answers = [
            rng.sample(range),
            rng.sample(range),
            rng.sample(range),
            answer,
        ];
        answers.shuffle(&mut rng);

        (Self { num1, operator, num2 }, answers)
    }

    pub fn validate_question(&self, answer: Answer) -> bool {
        self.operator.eval(self.num1, self.num2) == answer
    }
}

impl Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.num1, self.operator, self.num2)
    }
}
