mod DTOs;
mod problem;

use std::path::Path;
use std::{cmp, env, fs};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use rand::distributions::{Distribution, Uniform};
use serde_json::Value;
use serde::{Deserialize, Serialize};
use crate::DTOs::DTOs::*;
use strum_macros::EnumString;
use crate::problem::*;
use terminal_link::Link;

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
  println!("  bind                           bind a new problem.");
  println!("  solved                         tell the program you solved the binded problem, and to unbind it.");
  println!("  unsolved                       tell the program you didn't solve the binded problem, and to unbind it.");
  println!("  drop                           unbind the problem, this will not change your Elo rating of practice.");
  println!("  update                         pull data from codeforces API, this may take a while.");
  println!("  query difficulty [flags...]    query problems satisfy the requirement and certain integer difficulty.");
  println!();
  println!("Some flags for query command:");
  println!("  -d1       query div. 1 problems.");
  println!("  -d2       query div. 2 problems.");
  println!("  -d12      query div. 1 + 2 problems.");
  println!("  -edu      query educational problems.");
  println!("  -gl       query global round problems.");
  println!("  -other    query problems that are not fall into the above categories.");
  println!("  -old      allow query old problems.");
  println!("  -rec      query the most recent 10 problems satisfy the requirement.");
}

fn print_guide() {
  println!("Invalid arguments!");
  println!("You may enter \"rec help\" for help.");
}

fn update_all_DTOs(user_handle: &String) {
  update_problemDTOs();
  update_contestDTOs();
  update_submissionDTOs(&user_handle);
  UserInfoDTO::update(&user_handle);
}

fn query_problems(args: &Vec<String>, problems: &Vec<Problem>, user_handle: &String) -> Vec<Problem> {
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
        Flag::Other => div.push(Division::Other),
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

  filter_problems(&problems, &filter_options)
}

fn print_problems(problems: &Vec<Problem>) {
  println!();
  println!("|            problem name            | rating | contest name");
  for element in problems {
    let contest_id = "#".to_owned() + &element.contest_id.to_string();
    let url = element.problem_url();
    let problem_name = format!("{:^36}", element.name);
    let hyper_link = Link::new(&problem_name[..], &url);
    println!("|{:^36}|{:^8}| {}", hyper_link, element.rating, element.contest_name);
  }
  println!();
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
  #[strum(serialize = "-other")]
  Other,
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
      Command::Query if args.len() >= 3 => print_problems(&query_problems(&args, &problems, &user_handle)),
      _ => print_guide(),
    }, 
    Err(error) => print_guide(),
  }
}
