# About
This is a simple application, which scans status of the Bluesound Node/PowerNode streamer available in your network and truns on the Rotel Amplifier via RS232 port ASCII command if Node Streamer is straming content.

# Usage
    1. Compile the source code with the usage of RUST Compiler
    2. Run the app (preferably with a supervisord)

# Input params
    --node-ip-address - the IP adress of the Bluesound Node/PowerNode device. Example: --node-ip-address 192.168.1.100
    --node-port - the port of the Bluesound Node/PowerNode device. Example: --node-port 11000
    --rotel-rs232-port - the address of the RS232 port connected to the Rotel Aplifier. Example: --rotel-rs232-port /dev/ttyUSB0
    --rotel-rs232-baud-rate - the speed of the RD232 port connected to the Rotel Aplifier. Example: --rotel-rs232-baud-rate 115200