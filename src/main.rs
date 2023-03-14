use clap::Parser;
use isahc::prelude::*;
use treexml::Document;
use serialport;
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
    let mut rotel = serialport::new(args.rotel_rs232_port, args.rotel_rs232_baud_rate).open().expect("Failed to open Rotel's port");
    println!("Connected to Rotel...");

    loop {
        println!("Getting State from Node...");

        //connecto to Node
        let mut response = isahc::get(format!("http://{}:{}/Status", args.node_ip_address, args.node_port))?;
        if !response.status().is_success() {
            panic!("Failed to get a successful response status from Node!");
        }

        println!("Parsing response from Node...");
        let node_response_raw = format!("{}", response.text()?);
        let node_response = Document::parse(node_response_raw.as_bytes()).unwrap();
        let node_response_root = node_response.root.unwrap();
        let node_state_node = node_response_root.find_child(|tag| tag.name == "state").unwrap().clone();
        let mode_state = format!("{}", node_state_node.text.unwrap());
    
        //Node is streaming/playing
        if ["stream", "play"].contains(&mode_state.as_str()) {
            println!("Node is streaming. State: {}", mode_state.as_str());
            println!("Asking Rotel's status...");
            rotel.write("power?".as_bytes()).expect("Write to Rotel is failed!");
            thread::sleep(time::Duration::from_millis(200));
            let mut rotel_response_bufer: Vec<u8> = vec![0; 64];
            rotel.read(rotel_response_bufer.as_mut_slice());//not using the Result intentionally
            let rote_response_data = core::str::from_utf8(&rotel_response_bufer).unwrap();// Doesn't panic
            println!("Rotel responded: {}", rote_response_data);
            if rote_response_data.contains("power=standby$") {
                println!("Rotel is in stand by mode...");
                println!("Turning on Rotel...");
                rotel.write("aux1!".as_bytes()).expect("Write to Rotel is failed!");
            }
        } else {
            println!("Node is NOT streaming. State: {}", mode_state.as_str());
            //TODO think about auto off
        }

        println!("Waiting...");
        thread::sleep(time::Duration::from_secs(1));
        break;
    }

    Ok(())
}