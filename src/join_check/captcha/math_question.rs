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

    const fn eval(self, lhs: u8, rhs: u8) -> Answer {
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
    /// Generate a question
    pub fn generate_question<const N: usize>() -> (Self, [Answer; N]) {
        debug_assert!(
            N < MAX_ANSWER as usize,
            "N shouldn't be bigger then {MAX_ANSWER}"
        );
        // in best case we should create the thread rng one
        // time and then use it in the entire program...
        let mut rng = thread_rng();
        let lhs = rng.gen_range(MIN..MAX);
        let operator = *Operator::LIST.choose(&mut rng).unwrap();
        // if we want to sub the numbers, then the first number
        // need to be always greater or equal to the second one.
        let rhs = rng.gen_range(MIN..=if operator == Operator::Sub { lhs } else { MAX });

        let answer = operator.eval(lhs, rhs);

        let range = Uniform::new_inclusive(MIN, MAX_ANSWER);
        // we do this because we want a fully random list of numbers
        // it might get a little slower sometimes, but its a better
        // choice security wise...
        let mut answers = [0; N];
        answers[N - 1] = answer;
        for i in 0..N - 1 {
            loop {
                let r = rng.sample(range);
                if r != answer && !answers.contains(&r) {
                    answers[i] = r;
                    break;
                }
            }
        }
        // we shuffle 1..N because we dont want the answer
        // to end up being at start of list.
        answers[1..N].shuffle(&mut rng);

        (Self { lhs, operator, rhs }, answers)
    }

    pub const fn validate_answer(self, answer: Answer) -> bool {
        self.operator.eval(self.lhs, self.rhs) == answer
    }
}

impl Display for MathQuestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}‌ {}‌ {}", self.lhs, self.operator, self.rhs)
    }
}

#[cfg(test)]
mod test {
    use super::MathQuestion;

    fn generic_question<const N: usize>() {
        for _ in 0..10 {
            let (question, answers) = MathQuestion::generate_question::<N>();

            let mut found_answer = false;
            for (idx, answer) in answers.into_iter().enumerate() {
                if question.validate_answer(answer) {
                    assert_ne!(idx, 0, "Answer never should be the first value.");
                    found_answer = true;
                }
            }

            assert!(found_answer, "Didn't found the answer in math question.");
        }
    }

    #[test]
    fn generate_question() {
        generic_question::<4>();
        generic_question::<5>();
        generic_question::<10>();
    }
}
