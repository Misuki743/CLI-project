use std::path::Path;
use std::{cmp, env, fs};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use rand::distributions::{Distribution, Uniform};
use serde_json::Value;
use serde::{Deserialize, Serialize};
use crate::DTOs::DTOs::*;
use strum_macros::{EnumString, Display};

#[derive(Serialize, Deserialize)]
pub struct Problem {
  pub contest_id: i64,
  pub contest_name: String,
  pub contest_type: CompetitionSystem, 
  pub div: Division,
  pub index: String,
  pub name: String,
  pub rating: i64,
}

#[derive(Serialize, Deserialize, Copy, Clone, Display)]
pub enum Division {
  #[strum(serialize = "Div. 1")]
  Div1,
  #[strum(serialize = "Div. 2")]
  Div2,
  #[strum(serialize = "Div. 1 + 2")]
  Div12,
  #[strum(serialize = "Global Round")]
  GlobalRound,
  #[strum(serialize = "Educational")]
  Educational,
  #[strum(serialize = "Other")]
  Other,
}

#[derive(Serialize, Deserialize, Copy, Clone, EnumString)]
pub enum CompetitionSystem {
  CF,
  ICPC,
  IOI,
}

impl Division {
  pub fn determine_division(contest_name: &String, contest_type: &CompetitionSystem) -> Division {
    match contest_type {
      CompetitionSystem::CF if contest_name.contains("(Div. 1)") => Division::Div1,
      CompetitionSystem::CF if contest_name.contains("Div. 1 + Div. 2") => Division::Div12,
      CompetitionSystem::CF if contest_name.contains("Div. 2") => Division::Div2,
      CompetitionSystem::CF if contest_name.contains("Global Round") => Division::GlobalRound,
      CompetitionSystem::ICPC if contest_name.contains("Educational") => Division::Educational,
      _ => Division::Other,
    } 
  }
}

pub fn get_problems(probDTOs: &Vec<ProblemDTO>, contDTOs: &Vec<ContestDTO>) -> Vec<Problem> {
  let mut problems: Vec<Problem> = Vec::new();  
  let mut contestDTO_map: HashMap<i64, ContestDTO> = HashMap::new();
  for element in contDTOs {
    contestDTO_map.insert(element.id, element.clone());
  }
  for element in probDTOs {
    let contest_name = contestDTO_map.get(&element.contest_id).unwrap().name.clone();
    let contest_type = contestDTO_map.get(&element.contest_id).unwrap().contest_type.clone();
    let tmp = Problem {
      contest_id: element.contest_id,
      contest_name: contest_name.to_string(),
      index: element.index.clone(),
      name: element.name.clone(),
      rating: element.rating,
      div: Division::determine_division(&contest_name, &CompetitionSystem::from_str(&contest_type).unwrap()),
      contest_type: CompetitionSystem::from_str(&contest_type).unwrap(),
    };
    problems.push(tmp);
  }
  
  problems
}

impl Problem {
  pub fn problem_url(&self) -> String {
    if self.name == String::from("") {
      String::from("nan")
    } else {
      String::from("https://codeforces.com/problemset/problem/") + &self.contest_id.to_string() + "/" + &self.index
    }
  }
  pub fn unit() -> Self {
    Self {
      contest_id: 0,
      contest_name: String::from(""),
      index: String::from(""),
      name: String::from(""),
      rating: 0,
      div: Division::Other,
      contest_type: CompetitionSystem::CF,
    }
  }
  pub fn to_string(&self) -> String {
    self.contest_id.to_string() + &self.index + " - " + &self.name + "\n" + &self.problem_url()
  }
  pub fn combined_id(&self) -> String {
    self.contest_id.to_string() + &self.index
  }
  pub fn clone(&self) -> Self {
    Self {
      contest_id: self.contest_id,
      contest_name: self.contest_name.clone(),
      index: self.index.clone(),
      name: self.name.clone(),
      rating: self.rating,
      div: self.div,
      contest_type: self.contest_type,
    }
  }
}

#[allow(dead_code)]
pub struct User {
  pub handle: String,
  pub max_rating: i64,
  pub accepted_problems: HashSet<String>,
  pub excluded_problems: HashSet<String>,
}

impl User {
  pub fn new(handle: &String) -> User {
    let mut accepted_problems: HashSet<String> = HashSet::new();
    let submissionDTOs = get_submissionDTOs(&handle);
    for element in &submissionDTOs {
      if element.verdict == "OK" {
        accepted_problems.insert(element.problem.contest_id.to_string() + &element.problem.index);
      }
    }

    let mut excluded_problems: HashSet<String> = HashSet::new();
    if Path::new(&("excluded")).exists() {
      let res: Value = serde_json::from_str(&fs::read_to_string(&("excluded")).expect("read excluded list"))
                       .expect("convert str to json");
      for element in res.as_array().unwrap() {
        excluded_problems.insert(element.as_str().unwrap().to_string());  
      }
    }

    User {
      handle: handle.clone(),
      max_rating: UserInfoDTO::new(&handle).max_rating,
      accepted_problems,
      excluded_problems,
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct ProblemRecommender {
  handle: String,
  max_rating: i64,
  recommended_diff: i64,
  bind_problem: Problem,
  streak: i64,
}

impl ProblemRecommender {
  pub fn new(handle: &String) -> ProblemRecommender {
    let file_name = "recommender";
    if !Path::new(&file_name).exists() {
      let res = ProblemRecommender {
        handle: handle.clone(),
        max_rating: UserInfoDTO::new(&handle).max_rating,
        recommended_diff: UserInfoDTO::new(&handle).max_rating + 200, 
        bind_problem: Problem::unit(),
        streak: 0,
      };
      fs::write(&file_name, serde_json::to_string(&res).unwrap()).ok();
    }

    let res: ProblemRecommender = serde_json::from_str(&fs::read_to_string(&file_name).expect("read problem recommender"))
             .expect("convert str to json");
    return res;
  }

  #[allow(dead_code)]
  pub fn to_string(&self) -> String {
    String::from("handle: ") + &self.handle + "\n" +
    "max_rating: " + &self.max_rating.to_string() + "\n" +
    "recommended_diff: " + &self.recommended_diff.to_string() + "\n" +
    "bind_problem: " + &self.bind_problem.to_string() + "\n" +
    "streak: " + &self.streak.to_string()
  }
  
  pub fn save(&self) {
    let file_name = "recommender";
    fs::write(file_name, serde_json::to_string(&self).unwrap()).ok();
  }

  pub fn generate_problem_pool(&self, problems: &Vec<Problem>) -> Vec<Problem> {
    let request_diff = if self.streak <= -2 {
      self.recommended_diff - 100
    } else if self.streak >= 2 {
      self.recommended_diff + 100
    } else {
      self.recommended_diff
    };
    //consider recent div. 1 problems
    let filter_options = FilterOptions {
      min_diff: request_diff - 50,
      max_diff: request_diff + 50,
      oldest_round: Some(1480),
      div: vec![Division::Div1, Division::Div12, Division::GlobalRound],
      user: Some(User::new(&self.handle)),
      pool_size: None,
    };
    let mut problem_pool = filter_problems(problems, &filter_options);
    //consider all recent problems
    if problem_pool.len() == 0 {
      let filter_options = FilterOptions {
        min_diff: request_diff + 50,
        max_diff: request_diff + 150,
        oldest_round: Some(1480),
        div: vec![Division::Div2],
        user: Some(User::new(&self.handle)),
        pool_size: None,
      };
      problem_pool = filter_problems(problems, &filter_options);
    }

    problem_pool
  }

  pub fn bind_problem(&mut self, problems: &Vec<Problem>) {
    if self.bind_problem.name != String::from("") {
      println!("Already have a binded problem: {}", self.bind_problem.to_string());
    } else {
      let problem_pool = self.generate_problem_pool(problems);
      let mut rng = rand::thread_rng();
      let unif = Uniform::from(0..problem_pool.len());
      self.bind_problem = problem_pool[unif.sample(&mut rng)].clone();
      println!("Binded problem: {}", self.bind_problem.to_string());
      self.save();
    }
  }

  pub fn solve_problem(&mut self) {
    if self.bind_problem.name == String::from("") {
      println!("Don't have a binded problem!");
    } else {
      self.rating_change(true);
      self.bind_problem = Problem::unit();
      self.streak = cmp::max(self.streak + 1, 1);
      self.save();
      println!("Unbind the problem, rating change sucessfully!");
    }
  }

  pub fn unsolve_problem(&mut self) {
    if self.bind_problem.name == String::from("") {
      println!("Don't have a binded problem!");
    } else {
      self.rating_change(false);
      self.bind_problem = Problem::unit();
      self.streak = cmp::min(self.streak - 1, -1);
      self.save();
      println!("Unbind the problem, rating change sucessfully!");
    }
  }

  pub fn drop_problem(&mut self) {
    if self.bind_problem.name == String::from("") {
      println!("Don't have a binded problem!");
    } else {
      self.bind_problem = Problem::unit();
      self.save();
      println!("Unbind the problem.");
    }
  }

  fn rating_change(&mut self, solved: bool) {
    let k_factor = 24.0;
    let score = if solved { 1.0 } else { 0.0 };
    let expected_score = 1.0 / (1.0 + (10.0 as f64).powf((self.bind_problem.rating - self.recommended_diff) as f64 / 400.0));

    self.recommended_diff += (k_factor * (score - expected_score)).round() as i64;
  }
}

pub struct FilterOptions {
  pub min_diff: i64,
  pub max_diff: i64,
  pub oldest_round: Option<i64>,
  pub div: Vec<Division>,
  pub user: Option<User>,
  pub pool_size: Option<i64>,
}

pub fn filter_problems(problems: &Vec<Problem>, options: &FilterOptions) -> Vec<Problem> {
  let mut res: Vec<Problem> = Vec::new();

  let mut div1 = false;
  let mut div12 = false;
  let mut div2 = false;
  let mut edu = false;
  let mut gl = false;
  let mut other = false;

  for element in &options.div {
    match element {
      Division::Div1 => div1 = true,
      Division::Div12 => div12 = true,
      Division::Div2 => div2 = true,
      Division::Educational => edu = true,
      Division::GlobalRound => gl = true,
      Division::Other => other = true,
    }
  }

  for problem in problems {
    let mut valid = true;

    valid = valid && options.min_diff <= problem.rating && problem.rating <= options.max_diff;
    valid = valid && if let Some(oldest_round) = options.oldest_round {
      problem.contest_id >= oldest_round
    } else {
      true
    };
    valid = valid && match problem.div {
      Division::Div1 => div1,
      Division::Div12 => div12,
      Division::Div2 => div2,
      Division::Educational => edu,
      Division::GlobalRound => gl,
      Division::Other => other,
    };
    valid = valid && if let Some(tmp) = &options.user {
      !tmp.accepted_problems.contains(&problem.combined_id()) && !tmp.excluded_problems.contains(&problem.combined_id())
    } else {
      true
    };
    valid = valid && if let Some(pool_size) = options.pool_size {
      res.len() < pool_size.try_into().unwrap()
    } else {
      true
    };
  
    if valid {
      res.push(problem.clone());
    }
  }

  res
}
