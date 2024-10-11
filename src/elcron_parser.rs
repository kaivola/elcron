use std::{
    fs::File, io::{BufRead, BufReader, Write}, process::exit
};

use log::{error, info};

#[derive(Debug, PartialEq)]
pub enum ActivateOn {
    Above,
    Below,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Job {
    pub price_threshold: u16,
    pub activate_on: ActivateOn,
    pub activation_duration: u8,
    pub command: String,
}

impl Job {
    pub fn new(price_threshold: u16, activate_on: ActivateOn, activation_duration: u8, command: String) -> Job {
        Job {
            price_threshold,
            activate_on,
            activation_duration,
            command,
        }
    }
    
}

pub fn parse_elcron_file(filename: &str) -> Vec<Job> {
    let file = open_elcron_file(filename);
    let lines = read_elcron_lines(&file);
    if lines.is_empty() {
        error!("No valid lines found in elcron file");
    }
    let jobs = parse_lines(&lines);
    jobs
}

fn parse_lines(lines: &Vec<String>) -> Vec<Job> {
    let mut jobs = vec![];
    for (index, line) in lines.iter().enumerate() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            error!("Invalid line in elcron file: {}", line);
            continue;
        }
        let price = match parts[0].trim().parse::<u16>() {
            Ok(p) => p,
            Err(e) => {
                error!("Error parsing price in line {}: {}", index, e);
                continue;
            }
        };
        let direction = match parts[1].trim().to_lowercase().as_str() {
            "above" => ActivateOn::Above,
            "below" => ActivateOn::Below,
            _ => {
                error!("Invalid direction {} in line {}", parts[1], index);
                continue;
            }
        };
        let duration = match parts[2].trim().parse::<u8>() {
            Ok(p) => p,
            Err(e) => {
                error!("Error parsing price in line {}: {}", index, e);
                continue;
            }
        };
        let command = parts[3].trim().to_string();
        jobs.push(Job::new(price, direction, duration, command));
    }
    info!("Parsed {} valid jobs from elcron file", jobs.len());
    jobs
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
    let file = File::open(filename).unwrap_or_else(|e| {
        error!("Error opening file: {}", e);
        info!("Creating {} file and exiting", filename);
        let mut new_file = File::create(filename).unwrap();
        print_elcron_file_template(&mut new_file);
        exit(1);
    });
    return file;
}

fn print_elcron_file_template(file: &mut File) {
    let template = r#"# This file is used to define jobs that will be executed when the price of electricity is above or below a certain threshold

# The file is in the following format with columns separated by comma:
# price, activate on, duration, command
# 
# price: The price of electricity that will trigger the job
# activate on: Defines if the job will be triggered when the price is above or below the threshold
# duration: The number of hours the price has to be above or below the threshold before for the job to be triggered
# command: The command that will be executed when the conditions are met

# Example:
# 5, above, 2, echo "Price of electricity is above 5 for 2 hours"
# 10, below, 3, echo "Price of electricity is below 10 for 3 hours"
"#;
    match file.write_all(template.as_bytes()) {
        Err(e) => error!("Error writing to file: {}", e),
        Ok(_) => (),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_lines() {
        let lines = vec![
            "5, above, 2, echo \"Price of electricity is above 5 for 2 hours\"".to_string(),
            "10, below, 3, echo \"Price of electricity is below 10 for 3 hours\"".to_string(),
            "invalid line".to_string(),
        ];
        let jobs = parse_lines(&lines);
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].price_threshold, 5);
        assert_eq!(jobs[0].activate_on, ActivateOn::Above);
        assert_eq!(jobs[0].activation_duration, 2);
        assert_eq!(jobs[0].command, "echo \"Price of electricity is above 5 for 2 hours\"");

        assert_eq!(jobs[1].price_threshold, 10);
        assert_eq!(jobs[1].activate_on, ActivateOn::Below);
        assert_eq!(jobs[1].activation_duration, 3);
        assert_eq!(jobs[1].command, "echo \"Price of electricity is below 10 for 3 hours\"");
    }
}