use clap::Parser;
use isahc::prelude::*;
use treexml::Document;
use serialport::{self, SerialPort};
use std::{thread, time};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP adress of the Bluesound Node/PowerNode device
    #[arg(long)]
    node_ip_address: String,

    /// Port of the Bluesound Node/PowerNode device
    #[arg(long, default_value = "11000")]
    node_port: String,

    /// Address of the RS232 port connected to the Rotel Aplifier
    #[arg(long)]
    rotel_rs232_port: String,

    /// speed of the RD232 port connected to the Rotel Aplifier
    #[arg(long, default_value = "115200")]
    rotel_rs232_baud_rate: u32,
}

fn main() -> Result<(), isahc::Error> {
    let args = Args::parse();

    //connect to Rotel
    let mut rotel = serialport::new(args.rotel_rs232_port, args.rotel_rs232_baud_rate)
        .open().expect("Failed to open Rotel's port");
    println!("Connected to Rotel...");

    let mut last_streaming_duration_time = 0;
    let mut power_off_rotel_if_node_is_not_streaming_after = 30;
    let mut loop_counter = 300;//5 minutes per run
    loop {
        println!("Getting State from Node...");
        let mode_state = get_status_from_node(&args.node_ip_address, &args.node_port, "state".to_owned());
        println!("Node is in state: {}", mode_state.as_str());
        
        if ["stream", "play"].contains(&mode_state.as_str()) {//Node is streaming/playing
            println!("Asking Rotel's status...");
            write_to_rotel(&mut rotel, "power?".to_owned());
            thread::sleep(time::Duration::from_millis(100));
            if read_from_rotel(&mut rotel).contains("power=standby$") {
                println!("Rotel is in stand by mode...");
                println!("Turning on Rotel on AUX1...");
                write_to_rotel(&mut rotel, "aux1!".to_owned());
            }

            //Sometimes Node is hanging in a streaming state even when there is no content streamed.
            //In this case secs parameter (amount of seconds streaming) is not growing over time.
            //When situation detected, we trigger the “seek=1” command, so Node will refresh its state.
            let input_id = get_status_from_node(&args.node_ip_address, &args.node_port, "inputId".to_owned());
            let new_streaming_duration_time = get_status_from_node(&args.node_ip_address, &args.node_port, "secs".to_owned())
                .parse::<i32>().unwrap();
            if mode_state.as_str() == "stream" && input_id == "input2" &&  new_streaming_duration_time < last_streaming_duration_time{
                //straming status refresh needed
                println!("Detected Node hanging in straming state while nothing is being streamed.");
                println!("Triggering Node straming state refresh.");
                trigger_node_straming_refresh(&args.node_ip_address, &args.node_port);
            } else {
                last_streaming_duration_time = new_streaming_duration_time;
            }
        } else {
            println!("Node is NOT streaming.");

            if power_off_rotel_if_node_is_not_streaming_after > 0 {
                println!("Will attempt to turn off node in {} seconds", power_off_rotel_if_node_is_not_streaming_after);
                power_off_rotel_if_node_is_not_streaming_after -= 1;
            }

            if power_off_rotel_if_node_is_not_streaming_after == 0 {
                println!("Asking Node's input id...");
                let input_id = get_status_from_node(&args.node_ip_address, &args.node_port, "inputId".to_owned());
                println!("Node's input id is {}", input_id);

                println!("Asking Rotel's status...");
                write_to_rotel(&mut rotel, "power?".to_owned());
                thread::sleep(time::Duration::from_millis(100));
                let rotel_power_state = read_from_rotel(&mut rotel);
                write_to_rotel(&mut rotel, "source?".to_owned());
                thread::sleep(time::Duration::from_millis(100));
                let rotel_source_state = read_from_rotel(&mut rotel);

                if rotel_power_state.contains("power=on$") && rotel_source_state.contains("source=aux1$") && input_id == "input2" {
                    println!("Node is connected to Rotel, Rotel is connected to Node, Node not streaming and Rotel is powered on.");
                    println!("Truning off Rotel...");
                    write_to_rotel(&mut rotel, "power_off!".to_owned());
                }
            }
        }

        println!("Waiting 1 second...");
        thread::sleep(time::Duration::from_secs(1));
        loop_counter -= 1;
        if loop_counter <= 0 {
            break;
        }
    }

    Ok(())
}

fn write_to_rotel(rotel_connection_handler: &mut Box<dyn SerialPort>, command: String) {
    rotel_connection_handler.write(command.as_bytes()).expect("Write to Rotel is failed!");
}

fn read_from_rotel(rotel_connection_handler: &mut Box<dyn SerialPort>) -> String {
    let mut rotel_response_bufer: Vec<u8> = vec![0; 64];
    let _ = rotel_connection_handler.read(rotel_response_bufer.as_mut_slice());
    let rote_response_data = core::str::from_utf8(&rotel_response_bufer).unwrap();
    println!("Rotel responded: {}", rote_response_data);

    return rote_response_data.to_owned();
}

fn get_status_from_node(node_ip_address: &String, node_port: &String, requsted_tag_name: String) -> String {
    let mut response = isahc::get(format!("http://{}:{}/Status", node_ip_address, node_port)).unwrap();
    if !response.status().is_success() {
        panic!("Failed to get a successful response status from Node!");
    }

    let node_response_raw = format!("{}", response.text().unwrap());
    let node_response = Document::parse(node_response_raw.as_bytes()).unwrap();
    let node_response_root = node_response.root.unwrap();
    let node_node = node_response_root.find_child(|tag| tag.name == requsted_tag_name);

    if node_node.is_some() {
        return format!("{}", node_node.unwrap().clone().text.unwrap());
    }

    return format!("");
}

fn trigger_node_straming_refresh(node_ip_address: &String, node_port: &String) {
    let response = isahc::get(format!("http://{}:{}/Play?seek=1", node_ip_address, node_port)).unwrap();
    if !response.status().is_success() {
        panic!("Failed triggering straming refresh!");
    }
}
