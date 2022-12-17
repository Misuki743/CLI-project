mod DTOs;

use std::path::Path;
use std::{cmp, env, fs};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use rand::distributions::{Distribution, Uniform};
use serde_json::Value;
use serde::{Deserialize, Serialize};
use crate::DTOs::DTOs::*;
use strum_macros::EnumString;

#[derive(Serialize, Deserialize)]
struct Problem {
  contest_id: i64,
  contest_name: String,
  contest_type: CompetitionSystem, 
  div: Division,
  index: String,
  name: String,
  rating: i64,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
enum Division {
  Div1,
  Div2,
  Div12,
  GlobalRound,
  Educational,
  Other,
}

#[derive(Serialize, Deserialize, Copy, Clone, EnumString)]
enum CompetitionSystem {
  CF,
  ICPC,
  IOI,
}

fn determine_division(contest_name: &String, contest_type: &CompetitionSystem) -> Division {
  match contest_type {
    CompetitionSystem::CF if contest_name.contains("(Div. 1)") => Division::Div1,
    CompetitionSystem::CF if contest_name.contains("Div. 1 + Div. 2") => Division::Div12,
    CompetitionSystem::CF if contest_name.contains("Div. 2") => Division::Div2,
    CompetitionSystem::CF if contest_name.contains("Global Round") => Division::GlobalRound,
    CompetitionSystem::ICPC if contest_name.contains("Educational") => Division::Educational,
    _ => Division::Other,
  } 
}

fn get_problems(probDTOs: &Vec<ProblemDTO>, contDTOs: &Vec<ContestDTO>) -> Vec<Problem> {
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
      div: determine_division(&contest_name, &CompetitionSystem::from_str(&contest_type).unwrap()),
      contest_type: CompetitionSystem::from_str(&contest_type).unwrap(),
    };
    problems.push(tmp);
  }
  
  problems
}

impl Problem {
  fn problem_url(&self) -> String {
    if self.name == String::from("") {
      String::from("nan")
    } else {
      String::from("https://codeforces.com/problemset/problem/") + &self.contest_id.to_string() + "/" + &self.index
    }
  }
  fn unit() -> Self {
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
  fn to_string(&self) -> String {
    self.contest_id.to_string() + &self.index + " - " + &self.name + "\n" + &self.problem_url()
  }
  fn combined_id(&self) -> String {
    self.contest_id.to_string() + &self.index
  }
  fn clone(&self) -> Self {
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
struct User {
  handle: String,
  max_rating: i64,
  accepted_problems: HashSet<String>,
  excluded_problems: HashSet<String>,
}

impl User {
  fn new(handle: &String) -> User {
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
struct ProblemRecommender {
  handle: String,
  max_rating: i64,
  recommended_diff: i64,
  bind_problem: Problem,
  streak: i64,
}

impl ProblemRecommender {
  fn new(handle: &String) -> ProblemRecommender {
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
  fn to_string(&self) -> String {
    String::from("handle: ") + &self.handle + "\n" +
    "max_rating: " + &self.max_rating.to_string() + "\n" +
    "recommended_diff: " + &self.recommended_diff.to_string() + "\n" +
    "bind_problem: " + &self.bind_problem.to_string() + "\n" +
    "streak: " + &self.streak.to_string()
  }
  
  fn save(&self) {
    let file_name = "recommender";
    fs::write(file_name, serde_json::to_string(&self).unwrap()).ok();
  }

  fn generate_problem_pool(&self, problems: &Vec<Problem>) -> Vec<Problem> {
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

  fn bind_problem(&mut self, problems: &Vec<Problem>) {
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

  fn solve_problem(&mut self) {
    if self.bind_problem.name == String::from("") {
      println!("Don't have a binded problem!");
    } else {
      self.recommended_diff = rating_change(self.recommended_diff, self.bind_problem.rating, true);
      self.bind_problem = Problem::unit();
      self.streak = cmp::max(self.streak + 1, 1);
      self.save();
      println!("Unbind the problem, rating change sucessfully!");
    }
  }

  fn unsolve_problem(&mut self) {
    if self.bind_problem.name == String::from("") {
      println!("Don't have a binded problem!");
    } else {
      self.recommended_diff = rating_change(self.recommended_diff, self.bind_problem.rating, false);
      self.bind_problem = Problem::unit();
      self.streak = cmp::min(self.streak - 1, -1);
      self.save();
      println!("Unbind the problem, rating change sucessfully!");
    }
  }

  fn drop_problem(&mut self) {
    if self.bind_problem.name == String::from("") {
      println!("Don't have a binded problem!");
    } else {
      self.bind_problem = Problem::unit();
      self.save();
      println!("Unbind the problem.");
    }
  }
}

fn rating_change(rating: i64, problem_diff: i64, solved: bool) -> i64 {
  let k_factor = 24.0;
  let score = if solved { 1.0 } else { 0.0 };
  let expected_score = 1.0 / (1.0 + (10.0 as f64).powf((problem_diff - rating) as f64 / 400.0));

  rating + ((k_factor * (score - expected_score)).round() as i64)
}

fn print_description() {
  println!("rec - commandline codeforces problem recommender [version 1.0.0]");
  println!();
  println!("rec is a tool for practicing codeforces problems, with a classic Elo rating system to");
  println!("evaluate user's problem solving skill, and try to recommend problems that are challenging");
  println!("for the user in order to provide an effective way of training.");
  println!();
  println!("Usage:  rec subcommand");
  println!();
  println!("Some useful subcommands:");
  println!("  bind      bind a new problem.");
  println!("  solved    tell the program you solved the binded problem, and to unbind it.");
  println!("  unsolved  tell the program you didn't solve the binded problem, and to unbind it.");
  println!("  drop      unbind the problem, this will not change your Elo rating of practice.");
  println!("  update    pull data from codeforces API, this may take a while.");
}

fn print_guide() {
  println!("Invalid arguments!");
  println!("You may enter \"rec help\" for help.");
}

struct FilterOptions {
  min_diff: i64,
  max_diff: i64,
  oldest_round: Option<i64>,
  div: Vec<Division>,
  user: Option<User>,
  pool_size: Option<i64>,
}

fn filter_problems(problems: &Vec<Problem>, options: &FilterOptions) -> Vec<Problem> {
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

fn update_all_DTOs(user_handle: &String) {
  update_problemDTOs();
  update_contestDTOs();
  update_submissionDTOs(&user_handle);
  UserInfoDTO::update(&user_handle);
}

fn query_problems(args: &Vec<String>, problems: &Vec<Problem>, user_handle: &String) {
  let mut div: Vec<Division> = Vec::new();
  let mut pool_size: Option<i64> = None;
  let mut round = 1480;
  let diff = args[2].parse::<i64>().unwrap();
  for element in args {
    match Flag::from_str(&element) {
      Ok(flag) => match flag {
        Flag::Div1 => div.push(Division::Div1),
        Flag::Div2 => div.push(Division::Div2),
        Flag::Div12 => div.push(Division::Div12),
        Flag::GlobalRound => div.push(Division::GlobalRound),
        Flag::Educational => div.push(Division::Educational),
        Flag::ContainOldProblems => round = 1364,
        Flag::RecentMode => pool_size = Some(10),
      },
      Err(error) => ()
    }
  }

  if let Some(size) = pool_size {
    round = 0;
  }

  //if no specified division requirement, set the default division be rounds rated for Div.1 user
  if div.is_empty() {
    div = vec![Division::Div1, Division::Div12, Division::GlobalRound];
  }
  
  let filter_options = FilterOptions {
    min_diff: diff,
    max_diff: diff,
    oldest_round: Some(round),
    div: div,
    user: Some(User::new(&user_handle)),
    pool_size: pool_size,
  };
  let mut count: i64 = 0;
  for element in filter_problems(&problems, &filter_options) {
     println!("{}", element.to_string());
     count += 1;
  }

  println!("problem count: {}", count);
}

#[derive(EnumString)]
enum Command {
  #[strum(serialize = "help")]
  Help,
  #[strum(serialize = "bind")]
  Bind,
  #[strum(serialize = "solved")]
  Solved,
  #[strum(serialize = "unsolved")]
  Unsolved,
  #[strum(serialize = "drop")]
  Unbind,
  #[strum(serialize = "update")]
  Update,
  #[strum(serialize = "query")]
  Query,
}

#[derive(EnumString)]
enum Flag {
  #[strum(serialize = "-d1")]
  Div1,
  #[strum(serialize = "-d2")]
  Div2,
  #[strum(serialize = "-d12")]
  Div12,
  #[strum(serialize = "-gl")]
  GlobalRound,
  #[strum(serialize = "-edu")]
  Educational,
  #[strum(serialize = "-old")]
  ContainOldProblems,
  #[strum(serialize = "-rec")]
  RecentMode, //ignore the round# restriction, take the most recent 10 problems
}

fn main() {
  let problemDTOs = get_problemDTOs();
  let contestDTOs = get_contestDTOs();
  let problems = get_problems(&problemDTOs, &contestDTOs);
  let user_handle = String::from("__Shioko");
  let mut recommender = ProblemRecommender::new(&user_handle);

  let args: Vec<String> = env::args().collect();
  if args.len() == 1 {
    print_description();
    return;
  }
  match Command::from_str(&args[1]) {
    Ok(cmd) => match Command::from_str(&args[1]).unwrap() {
      Command::Help => print_description(),
      Command::Bind => recommender.bind_problem(&problems),
      Command::Solved => recommender.solve_problem(),
      Command::Unsolved => recommender.unsolve_problem(),
      Command::Unbind => recommender.drop_problem(),
      Command::Update => update_all_DTOs(&user_handle),
      Command::Query if args.len() >= 3 => query_problems(&args, &problems, &user_handle),
      _ => print_guide(),
    }, 
    Err(error) => print_guide(),
  }
}
