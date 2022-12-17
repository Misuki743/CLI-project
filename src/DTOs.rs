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

    problemDTOs
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

    contestDTOs
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

    submissionDTOs
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

      user_infoDTO
    }
  }
}
