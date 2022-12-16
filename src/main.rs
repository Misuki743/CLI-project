use std::path::Path;
use std::{cmp, env, fs};
use std::collections::{HashMap, HashSet};
use rand::distributions::{Distribution, Uniform};
use serde_json::Value;
use serde::{Deserialize, Serialize};
use crate::DTOs::*;

#[allow(non_snake_case)]
pub mod DTOs {
  use std::path::Path;
  use std::fs;
  use std::time::Duration;
  use std::thread::sleep;
  use serde_json::Value;
  use serde::{Deserialize, Serialize};
  use spinner::SpinnerBuilder;

  #[derive(Serialize, Deserialize)]
  pub struct ProblemDTO {
    pub contest_id: i64,
    pub index: String,
    pub name: String,
    pub rating: i64,
  }

  #[allow(non_snake_case)]
  pub fn update_problemDTOs() {
    let spin = SpinnerBuilder::new("fetching problem data...".into()).start();
    sleep(Duration::from_secs(2));
    let response = reqwest::blocking::get("https://codeforces.com/api/problemset.problems").unwrap();
    spin.close();
    print!("\r                                   \r");
    let res: Value = response.json().unwrap();  
    fs::write("problems", res.to_string()).ok();
  }

  #[allow(non_snake_case)]
  pub fn get_problemDTOs() -> Vec<ProblemDTO> {
    if !Path::new("problems").exists() {
      update_problemDTOs();
    }
    let res: Value = serde_json::from_str(&fs::read_to_string("problems").expect("read problems")).expect("convert str to json");
    let mut problemDTOs: Vec<ProblemDTO> = Vec::new();
    for element in res["result"]["problems"].as_array().unwrap() {
      if element["rating"].is_null() || element["contestId"].is_null() {
        continue;
      }
      let tmp = ProblemDTO {
        contest_id: element["contestId"].as_i64().unwrap(),
        index: element["index"].as_str().unwrap().to_string(),
        name: element["name"].as_str().unwrap().to_string(),
        rating: element["rating"].as_i64().unwrap(),
      };
      problemDTOs.push(tmp);
    } 

    return problemDTOs;
  }

  #[derive(Serialize, Deserialize)]
  pub struct ContestDTO {
    pub id: i64,
    pub name: String,
    pub contest_type: String,
  }

  impl ContestDTO {
    pub fn clone(&self) -> Self {
      Self {
        id: self.id,
        name: self.name.clone(),
        contest_type: self.contest_type.clone(),
      }
    }
  }

  #[allow(non_snake_case)]
  pub fn update_contestDTOs() {
    let spin = SpinnerBuilder::new("fetching contest data...".into()).start();
    sleep(Duration::from_secs(2));
    let response = reqwest::blocking::get("https://codeforces.com/api/contest.list").unwrap();
    spin.close();
    print!("\r                                   \r");
    let res: Value = response.json().unwrap();
    fs::write("contests", res.to_string()).ok();
  }

  #[allow(non_snake_case)]
  pub fn get_contestDTOs() -> Vec<ContestDTO> {
    if !Path::new("contests").exists() {
      update_contestDTOs();
    }
    let res: Value = serde_json::from_str(&fs::read_to_string("contests").expect("read contests")).expect("convert str to json");
    let mut contestDTOs: Vec<ContestDTO> = Vec::new();
    for element in res["result"].as_array().unwrap() {
      let tmp = ContestDTO {
        id: element["id"].as_i64().unwrap(),
        name: element["name"].as_str().unwrap().to_string(),
        contest_type: element["type"].as_str().unwrap().to_string(),
      };
      contestDTOs.push(tmp);
    }

    return contestDTOs;
  }

  pub struct SubmissionDTO {
    pub problem: ProblemDTO,
    pub verdict: String,
  }

  #[allow(non_snake_case)]
  pub fn update_submissionDTOs(handle: &String) {
    let spin = SpinnerBuilder::new("fetching submission data...".into()).start();
    sleep(Duration::from_secs(2));
    let response = reqwest::blocking::get("https://codeforces.com/api/user.status?handle=".to_owned() + &handle).unwrap();
    spin.close();
    print!("\r                                   \r");
    let res: Value = response.json().unwrap();  
    fs::write(&handle, res.to_string()).ok();
  }

  #[allow(non_snake_case)]
  pub fn get_submissionDTOs(handle: &String) -> Vec<SubmissionDTO> {
    if !Path::new(handle).exists() {
      update_submissionDTOs(&handle);
    }
    let res: Value = serde_json::from_str(&fs::read_to_string(handle).expect("read submissions")).expect("convert str to json");
    let mut submissionDTOs: Vec<SubmissionDTO> = Vec::new();
    for element in res["result"].as_array().unwrap() {
      if element["problem"]["rating"].is_null() || element["problem"]["contestId"].is_null() {
        continue;
      }
      let prob = ProblemDTO {
        contest_id: element["problem"]["contestId"].as_i64().unwrap(),
        index: element["problem"]["index"].as_str().unwrap().to_string(),
        name: element["problem"]["name"].as_str().unwrap().to_string(),
        rating: element["problem"]["rating"].as_i64().unwrap(),
      };
      let tmp = SubmissionDTO {
        problem: prob,
        verdict: element["verdict"].as_str().unwrap().to_string(),
      };
      submissionDTOs.push(tmp);
    } 

    return submissionDTOs;
  }

  #[allow(dead_code)]
  pub struct UserInfoDTO {
    pub handle: String,
    pub rank: String, 
    pub rating: i64,
    pub max_rank: String,
    pub max_rating: i64,
  }

  impl UserInfoDTO {
    pub fn update(handle: &String) {
      let file_name = "user_info";
      let spin = SpinnerBuilder::new("fetching userInfo data...".into()).start();
      sleep(Duration::from_secs(2));
      let response = reqwest::blocking::get("https://codeforces.com/api/user.info?handles=".to_owned() + &handle).unwrap();
      spin.close();
      print!("\r                                   \r");
      let res: Value = response.json().unwrap();  
      fs::write(&file_name, res.to_string()).ok();
    }
    pub fn new(handle: &String) -> UserInfoDTO {
      let file_name = "user_info";
      if !Path::new(&file_name).exists() {
        Self::update(&handle);
      }
      let res: Value = serde_json::from_str(&fs::read_to_string(&file_name).expect("read user info")).expect("convert str to json");
      
      let user_infoDTO = UserInfoDTO {
        handle: handle.clone(),
        rank: res["result"][0]["rank"].as_str().unwrap().to_string(),
        rating: res["result"][0]["rating"].as_i64().unwrap(),
        max_rank: res["result"][0]["maxRank"].as_str().unwrap().to_string(),
        max_rating: res["result"][0]["maxRating"].as_i64().unwrap(),
      };

      return user_infoDTO;
    }
  }
}

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

#[derive(Serialize, Deserialize, Copy, Clone)]
enum CompetitionSystem {
  CF,
  ICPC,
  IOI,
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
      div: if contest_name.contains("(Div. 1)") && contest_type == "CF" {
        Division::Div1
      } else if !contest_name.contains("Div. 1 + Div. 2") && contest_name.contains("Div. 2") && contest_type == "CF" {
        Division::Div2
      } else if contest_name.contains("Div. 1 + Div. 2") && contest_type == "CF" {
        Division::Div12
      } else if contest_name.contains("Global Round") && contest_type == "CF" {
        Division::GlobalRound 
      } else if contest_name.contains("Educational") && contest_type == "ICPC" {
        Division::Educational
      } else {
        Division::Other
      },
      contest_type: if contest_type == "CF" {
        CompetitionSystem::CF 
      } else if contest_type == "ICPC" {
        CompetitionSystem::ICPC  
      } else {
        CompetitionSystem::IOI
      }
    };
    problems.push(tmp);
  }
  
  return problems;
}

impl Problem {
  fn problem_url(&self) -> String {
    if self.name == String::from("") {
      let url = String::from("nan");
      return url;
    } else {
      let url = String::from("https://codeforces.com/problemset/problem/") + &self.contest_id.to_string() + "/" + &self.index;
      return url;
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
    let res = self.contest_id.to_string() + &self.index + " - " + &self.name + "\n" + &self.problem_url();
    return res;
  }
  fn combined_id(&self) -> String {
    let res = self.contest_id.to_string() + &self.index;
    return res;
  }
  fn clone(&self) -> Self {
    let res = Self {
      contest_id: self.contest_id,
      contest_name: self.contest_name.clone(),
      index: self.index.clone(),
      name: self.name.clone(),
      rating: self.rating,
      div: self.div,
      contest_type: self.contest_type,
    };
    return res;
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

    let res = User {
      handle: handle.clone(),
      max_rating: UserInfoDTO::new(&handle).max_rating,
      accepted_problems,
      excluded_problems,
    };

    return res;
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

    let tmp = UserInfoDTO::new(&handle).max_rating;
    if tmp != res.max_rating {
      let res = ProblemRecommender {
        max_rating: tmp,
        ..res
      };

      return res;
    }

    return res;
  }

  #[allow(dead_code)]
  fn to_string(&self) -> String {
    let res = String::from("handle: ") + &self.handle + "\n" +
              "max_rating: " + &self.max_rating.to_string() + "\n" +
              "recommended_diff: " + &self.recommended_diff.to_string() + "\n" +
              "bind_problem: " + &self.bind_problem.to_string() + "\n" +
              "streak: " + &self.streak.to_string();
    return res;
  }
  
  fn save(&self) {
    let file_name = "recommender";
    fs::write(file_name, serde_json::to_string(&self).unwrap()).ok();
  }

  fn generate_problem_pool(&self, problems: &Vec<Problem>) -> Vec<Problem> {
    let user = Some(User::new(&self.handle));
    let request_diff = if self.streak <= -2 {
      self.recommended_diff - 100
    } else if self.streak >= 2 {
      self.recommended_diff + 100
    } else {
      self.recommended_diff
    };
    //consider recent div. 1 problems
    let tmp: Vec<Division> = vec![Division::Div1, Division::Div12, Division::GlobalRound];
    let mut problem_pool = filter_problems(problems, request_diff - 50, request_diff + 50, 1480, &tmp, &user);
    //consider all recent problems
    if problem_pool.len() == 0 {
      let tmp: Vec<Division> = vec![Division::Div2];
      problem_pool = filter_problems(problems, request_diff + 50, request_diff + 150, 1480, &tmp, &user)
    }

    return problem_pool;
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
  return rating + ((k_factor * (score - expected_score)).round() as i64);
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

fn filter_problems(problems: &Vec<Problem>, min_diff: i64, max_diff: i64, oldest_round: i64, div: &Vec<Division>, user: &Option<User>) -> Vec<Problem> {
  let mut res: Vec<Problem> = Vec::new();

  let mut div1 = false;
  let mut div12 = false;
  let mut div2 = false;
  let mut edu = false;
  let mut gl = false;
  let mut other = false;

  for element in div {
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
    let match_div = match problem.div {
      Division::Div1 => div1,
      Division::Div12 => div12,
      Division::Div2 => div2,
      Division::Educational => edu,
      Division::GlobalRound => gl,
      Division::Other => other,
    };

    let unsolved = if let Some(tmp) = user {
      !tmp.accepted_problems.contains(&problem.combined_id()) && !tmp.excluded_problems.contains(&problem.combined_id())
    } else {
      true
    };

    if min_diff <= problem.rating && problem.rating <= max_diff && problem.contest_id >= oldest_round && match_div && unsolved {
      res.push(problem.clone());
    }
  }

  return res;
}

fn main() {
  let problemDTOs = get_problemDTOs();
  let contestDTOs = get_contestDTOs();

  let user_handle = String::from("__Shioko");
  let problems = get_problems(&problemDTOs, &contestDTOs);

  let args: Vec<String> = env::args().collect();
  if args.len() == 1 || (args.len() == 2 && args[1] == "help") {
    print_description();
  } else if args[1] == "bind" {
    ProblemRecommender::new(&user_handle).bind_problem(&problems);
  } else if args[1] == "solved" {
    ProblemRecommender::new(&user_handle).solve_problem();
  } else if args[1] == "unsolved" {
    ProblemRecommender::new(&user_handle).unsolve_problem();
  } else if args[1] == "drop" {
    ProblemRecommender::new(&user_handle).drop_problem();
  } else if args[1] == "update" {
    update_problemDTOs();
    update_contestDTOs();
    update_submissionDTOs(&user_handle);
    UserInfoDTO::update(&user_handle);
  } else if args.len() >= 3 && args[1] == "query" {

    let mut div: Vec<Division> = Vec::new();
    let mut round = 1480;
    let diff = args[2].parse::<i64>().unwrap();
    for element in &args {
      if element == "-d1" {
        div.push(Division::Div1);
      } else if element == "-d2" {
        div.push(Division::Div2);
      } else if element == "-d12" {
        div.push(Division::Div12);
      } else if element == "-gl" {
        div.push(Division::GlobalRound);
      } else if element == "-edu" {
        div.push(Division::Educational);
      } else if element == "-old" {
        round = 1364;
      }
    }

    if div.is_empty() {
      div = vec![Division::Div1, Division::Div12, Division::GlobalRound];
    }
    
    let user = Some(User::new(&user_handle));
    let mut count: i64 = 0;
    for element in filter_problems(&problems, diff, diff, round, &div, &user) {
       println!("{}", element.to_string());
       count += 1;
    }

    println!("problem count: {}", count);
    
  } else {
    println!("Invalid arguments!");
    println!("You may enter \"rec help\" for help.");
  }
}
