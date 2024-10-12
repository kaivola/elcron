use std::{
    fmt::Display, fs::File, io::{BufRead, BufReader, Write}, process::exit
};

use log::{error, info};

#[derive(Debug, PartialEq)]
pub enum TriggerCondition {
    Above,
    Below,
}

#[derive(Debug)]
pub struct Job {
    pub price_threshold: u16,
    pub condition: TriggerCondition,
    pub command: String,
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Job(price_threshold={}, condition={:?}, command={})", self.price_threshold, self.condition, self.command)
    }
}

impl Job {
    fn new(price_threshold: u16, condition: TriggerCondition, command: String) -> Self {
        Self {
            price_threshold,
            condition,
            command,
        }
    }
    fn from_elcron_line(line: &str) -> Self {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            panic!("Invalid number of parts in elcron line: {}", line);
        }
        let price = get_price(parts[0]);
        let condition = get_condition(parts[1]);
        let command = parts[2].trim().to_string();
        Self::new(price, condition, command)
    }
    pub fn should_execute(&self, price: f64) -> bool {
        match self.condition {
            TriggerCondition::Above => price > f64::from(self.price_threshold),
            TriggerCondition::Below => price < f64::from(self.price_threshold),
        }
    }
    pub fn execute(&self) {
        info!("Activating job: {}", self);
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(self.command.clone())
            .output()
            .expect("Failed to execute command");
        info!("Output: {}", String::from_utf8_lossy(&output.stdout).to_string().trim());
    }
}

pub fn parse_elcron_file(filename: &str) -> Vec<Job> {
    let file = open_elcron_file(filename);
    let lines = read_elcron_lines(&file);
    if lines.is_empty() {
        panic!("No valid lines found in elcron file");
    }
    
    parse_lines(&lines)
}

fn parse_lines(lines: &[String]) -> Vec<Job> {
    let mut jobs = vec![];
    for line in lines {
        let job = Job::from_elcron_line(line);
        jobs.push(job);
    }
    info!("Found {} jobs in elcron file", jobs.len());
    jobs
}

fn get_price(price: &str) -> u16 {
    match price.trim().parse::<u16>() {
        Ok(p) => p,
        Err(_e) => panic!("Invalid price: {}", price)
    }
}

fn get_condition(condition: &str) -> TriggerCondition {
    match condition.trim().to_lowercase().as_str() {
        "above" => TriggerCondition::Above,
        "below" => TriggerCondition::Below,
        _ => panic!("Invalid condition: {}", condition)
    }
}

fn read_elcron_lines(file: &File) -> Vec<String> {
    let reader = BufReader::new(file);
    let mut res = vec![];
    for l in reader.lines() {
        let line = l.unwrap();
        if !line.starts_with('#') && !line.is_empty() {
            res.push(line);
        }
    }
    info!("Read {} lines from elcron file", res.len());
    res
}

fn open_elcron_file(filename: &str) -> File {
    info!("Reading elcron file: {}", filename);
    
    File::open(filename).unwrap_or_else(|_e| {
        info!("File: {} not found! Creating it and exiting", filename);
        let mut new_file = File::create(filename).unwrap();
        print_elcron_file_template(&mut new_file);
        exit(1);
    })
}

fn print_elcron_file_template(file: &mut File) {
    let template = r#"# This file is used to define jobs that will be executed when the price of electricity is above or below a certain threshold

# The file is in the following format with columns separated by comma:
# price, condition, command

# price: The price of electricity in c/kWh that will trigger the job
# condition: Can be set to above or below. The condition determines if the job will be triggered when the price is above or below the threshold
# when the price is above or below the threshold
# command: The command that will be executed when the conditions are met

# Example:
# price,    condition,  command
# 5,        above,      echo "Price of electricity is above 5"
# 10,       below,      echo "Price of electricity is below 10"
"#;
    if let Err(e) = file.write_all(template.as_bytes()) { error!("Error writing to file: {}", e) }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_lines() {
        let lines = vec![
            "5, above, echo \"Price of electricity is above 5\"".to_string(),
            "10, below, echo \"Price of electricity is below 10\"".to_string()
        ];
        let jobs = parse_lines(&lines);
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].price_threshold, 5);
        assert_eq!(jobs[0].condition, TriggerCondition::Above);
        assert_eq!(jobs[0].command, "echo \"Price of electricity is above 5\"");

        assert_eq!(jobs[1].price_threshold, 10);
        assert_eq!(jobs[1].condition, TriggerCondition::Below);
        assert_eq!(jobs[1].command, "echo \"Price of electricity is below 10\"");
    }

    #[test]
    #[should_panic]
    fn test_parse_lines_invalid() {
        let lines = vec![
            "5, above, 2, echo \"Price of electricity is above 5\"".to_string(),
            "10, invalid, 3, echo \"Price of electricity is below 10\"".to_string()
        ];
        parse_lines(&lines);
    }

    #[test]
    #[should_panic]
    fn test_parse_lines_invalid_number_of_parts() {
        let lines = vec![
            "5, above, 2, echo \"Price of electricity is above 5\"".to_string(),
            "10, below, 3".to_string()
        ];
        parse_lines(&lines);
    }

    #[test]
    fn test_validate_price() {
        assert_eq!(get_price("5"), 5);
        assert_eq!(get_price("10"), 10);
    }

    #[test]
    #[should_panic]
    fn test_validate_price_invalid() {
        get_price("invalid");
    }

    #[test]
    fn test_validate_condition() {
        assert_eq!(get_condition("above"), TriggerCondition::Above);
        assert_eq!(get_condition("below"), TriggerCondition::Below);
    }

    #[test]
    #[should_panic]
    fn test_validate_condition_invalid() {
        get_condition("invalid");
    }

    #[test]
    fn test_job_should_execute() {
        let job = Job::new(5, TriggerCondition::Above, "echo \"test\"".to_string());
        assert!(job.should_execute(6.0));
        assert!(!job.should_execute(4.0));
    }

    #[test]
    fn test_job_execute() {
        let job = Job::new(5, TriggerCondition::Above, "echo \"test\"".to_string());
        job.execute();
    }

    #[test]
    fn test_job_from_elcron_line() {
        let job = Job::from_elcron_line("5, above, echo \"test\"");
        assert_eq!(job.price_threshold, 5);
        assert_eq!(job.condition, TriggerCondition::Above);
        assert_eq!(job.command, "echo \"test\"");
    }
}