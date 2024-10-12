# elcron

elcron is a simple scheduler for executing shell commands based on electricity prices. The application uses the [Entso-e Transparency Platform](https://transparency.entsoe.eu/) to get the electricity prices for the next 24 hours and schedules the commands based on the prices.

Scheduling is done by using [.elcron](.elcron) file. The file contains the commands to be executed and the price thresholds for each command. In addition each job defined has condition, which can be either `above` or `below`. The condition defines whether the command is executed when the price is above or below the threshold.

## Example .elcron file

```
#This file is used to define jobs that will be executed when the price of electricity is above or below a certain threshold

# The file is in the following format with columns separated by comma:
# price, condition, command

# price: The price of electricity in c/kWh that will trigger the job
# condition: Can be set to above or below. The condition determines if the job will 
#   be triggered when the price is above or below the threshold
# command: The command that will be executed when the conditions are met

# Example:
# price,    condition,  command
# 5,        above,      echo "Price of electricity is above 5"
# 10,       below,      echo "Price of electricity is below 10"
```

## Installation

To compile the application, you need to have Rust installed. You can install Rust by following the instructions on the [Rust website](https://www.rust-lang.org/tools/install).
```bash
git clone
cd elcron
cargo build --release
cp target/release/elcron .
```
After the compilation is done, you need to create .env file in the root directory of the project. The file should contain the following variables:
```bash
API_KEY=YOUR_API_KEY
AREA=ENTSO-E_AREA_CODE
```
You can get the API key by registering on the [Entso-e Transparency Platform](https://transparency.entsoe.eu/) and requesting an API key by email. 

Area codes can be found here: [Transparency Platform RESTful API - user guide](https://transparency.entsoe.eu/content/static_content/Static%20content/web%20api/Guide.html#_areas)

For example, the area code for Finland is: `10YFI-1--------U`



## Usage

**Note: When the application is started for the first time, it will create a `.elcron` file in the current directory and exit.**


Run the application by executing the following command:
```bash
./elcron
```

