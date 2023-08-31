use rand::{distributions::Uniform, seq::SliceRandom, thread_rng, Rng};

pub type Question = String;
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

impl TryFrom<&str> for Operators {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Sub),
            "*" => Ok(Self::Mul),
            _ => Err("Unknown operator"),
        }
    }
}

impl std::fmt::Display for Operators {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => '+',
            Self::Sub => '-',
            Self::Mul => '*',
        }
        .fmt(f)
    }
}

pub fn generate_captcha() -> (Question, [Answer; 4]) {
    // in best case we should create the thread rng one time and then use it in entire program...
    let mut rng = thread_rng();
    let num1 = rng.gen_range(MIN..MAX);
    let operator = Operators::LIST.choose(&mut rng).unwrap();
    let num2 = rng.gen_range(
        MIN..=if *operator == Operators::Sub {
            num1 // if we want to sub the numbers the first number need to be always bigger then second one
        } else {
            MAX
        },
    );

    let question = format!("{num1} {operator} {num2}");
    let answer = operator.eval(num1, num2);

    let range = Uniform::new_inclusive(MIN, MAX_ANSWER);
    let mut answers = [
        rng.sample(range),
        rng.sample(range),
        rng.sample(range),
        answer,
    ];
    answers.shuffle(&mut rng);

    (question, answers)
}

pub fn validate_captcha_answer(question: Question, answer: Answer) -> bool {
    let split: Vec<_> = question.split_whitespace().collect();
    if split.len() != 3 {
        log::warn!(
            "Found unsupported number of element when parsing the captcha question in validation."
        );
        return false;
    }

    let (Ok(num1), Ok(num2)) = (split[0].parse::<u8>(), split[2].parse::<u8>()) else {
        log::warn!("Failed to parse question numbers back to u8 when validating captcha");
        return false
    };

    Operators::try_from(split[1]).is_ok_and(|o| answer == o.eval(num1, num2))
}
