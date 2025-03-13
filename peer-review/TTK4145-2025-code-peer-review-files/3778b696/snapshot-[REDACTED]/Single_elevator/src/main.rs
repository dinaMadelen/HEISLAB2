use clap::builder::TypedValueParser;
use crossbeam_channel as cbc;
use driver_rust::elevio;
use driver_rust::elevio::elev as e;
use std::thread::*;
use std::time::*;
use clap::Parser;




//use Single_elevator::elevator::*;
//use Single_elevator::fsm::*;
//use Single_elevator::requests::*;
//mod tcp_elevator;
mod client;
mod tcp_elevator;
mod fsm;
mod elevator;
mod requests;
mod elevator_buttons;
mod tcp_definitions;


/// program for controlling multiple lifts.
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args{
    /// The address and port to find the master
    #[arg(short,long,default_value="localhost:12345")]
    master_address : String,
    /// the address and port of the physical elevator
    #[arg(short,long,default_value="localhost:15657")]
    elevator_address : String,
    /// identifier of the client. id none it uses the ip
    #[arg(short,long,default_value_t=0)]
    id : u32,


}

fn main() -> std::io::Result<()> {
    let elevator_id: u32 = 1; 
          
    let args= Args::parse();

    let elev_num_floors = 4;
    let hw_elevator = e::Elevator::init(&args.elevator_address, elev_num_floors)?;
    println!("Elevator started:\n{:#?}", hw_elevator);

    let poll_period = Duration::from_millis(25);

    // Orders from Master/Server
    // if this is a master node. start a server.
    if (args.master_address.contains("localhost")){
        println!("setting up master server");
    let address= args.master_address.clone();
    let server_handle = std::thread::spawn(move || tcp_elevator::tcp_server(address));
    }
    //Orders from Master/Server
    let (tcp_order_tx, tcp_order_rx) = cbc::unbounded::<tcp_definitions::tcp_message>();

    {
        let tcp_order_tx = tcp_order_tx.clone();
        let address= args.master_address.clone();
       // spawn(move || client::receive_orders("127.0.0.1:8080", tcp_order_tx,call_button_rx_clone, call_button_main_tx, elevator_id));
       

    }



    let (call_button_tx, call_button_rx) = cbc::unbounded::<elevio::poll::CallButton>();

    // Need two call_button channels due to race conditions
    let (call_button_main_tx, call_button_main_rx) = cbc::unbounded::<elevio::poll::CallButton>();

    {
        let hw_elevator = hw_elevator.clone();
        spawn(move || elevio::poll::call_buttons(hw_elevator, call_button_tx, poll_period));
    }

    {
        let tcp_order_tx = tcp_order_tx.clone();
        let call_button_rx_clone = call_button_rx.clone();
        let call_button_main_tx = call_button_main_tx.clone();
        let address=args.master_address.clone();
        spawn(move || client::receive_orders(&address, tcp_order_tx, call_button_rx_clone, call_button_main_tx, elevator_id));
    }

    let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
    {
        let hw_elevator = hw_elevator.clone();
        spawn(move || elevio::poll::floor_sensor(hw_elevator, floor_sensor_tx, poll_period));
    }

    let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
    {
        let hw_elevator = hw_elevator.clone();
        spawn(move || elevio::poll::stop_button(hw_elevator, stop_button_tx, poll_period));
    }

    let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
    {
        let hw_elevator = hw_elevator.clone();
        spawn(move || elevio::poll::obstruction(hw_elevator, obstruction_tx, poll_period));
    }
    println!("finished handlers");

    let mut system_elevator = elevator::Elevator::uninitialized(); // Starts at floor 0, dirn stop, idle
    fsm::fsm_init_lights(&hw_elevator);
    println!("Starting initialization");
    fsm::fsm_init(&mut system_elevator, &hw_elevator, &floor_sensor_rx); // Go to floor 0

    let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
    {
        let hw_elevator = hw_elevator.clone();
        spawn(move || elevio::poll::floor_sensor(hw_elevator, floor_sensor_tx, poll_period));
    }
    println!("setup finished");
    
    
   

    //Main loop
    loop {
        cbc::select!{
            //Order from master, Only HALLUp And HALLDown
            recv(tcp_order_rx) -> a => { 
                if let Ok(tcp_definitions::tcp_message::set_order { order }) = a {
                    if order.button_type as i32 != 2 {println!(
                        "Received TCP order: Floor {}, Button {:?}",
                        order.floor,
                        order.button_type
                        
                    );
                   fsm::fsm_handle_new_order(&mut system_elevator, &hw_elevator, order.floor as i32, order.button_type as i32);}
                    
                }
            },
            recv(call_button_main_rx) -> a => { // Only cab calls
                let call_button_main = a.unwrap(); 
                println!("call_button floor: {}, CALL: {}", call_button_main.floor, call_button_main.call);
                if call_button_main.call == 2 {fsm::fsm_handle_new_order(&mut system_elevator, &hw_elevator, call_button_main.floor as i32, call_button_main.call as i32);}
            },
            recv(call_button_rx) -> a => { // Only cab calls
                println!("fads");
                let call_button = a.unwrap(); 
                client::send_order(&args.master_address,&call_button,&args.id);

                if call_button.call == 2 {fsm::fsm_handle_new_order(&mut system_elevator, &hw_elevator, call_button.floor as i32, call_button.call as i32);}
             },
            
            /* 
             //single elevator local
            recv(call_button_rx) -> a => {
                let call_button = a.unwrap();
                let local_call = tcp_elevator::TCP_lift_order_t {
                    button_type:ButtonType::from_u8(call_button.call),
                    floor: call_button.floor as u32,
                    elevator_id: elevator_id,
                };
                
               // tcp_order_tx.send(call_button);
                //fsm_handle_new_order(&mut system_elevator, &hw_elevator, call_button.floor as i32, call_button.call as i32);
            },*/
            recv(floor_sensor_rx) -> a => {
                let floor = a.unwrap();
                fsm::fsm_on_floor_arrival(&mut system_elevator, &hw_elevator, floor as i32);
            },
            recv(stop_button_rx) -> _ => {
                println!("Stop button pressed!");
                hw_elevator.stop_button_light(true);
                std::thread::sleep(Duration::from_secs(3));
                hw_elevator.stop_button_light(false);
                println!("Elevator ready");
            },
            recv(obstruction_rx) -> a => {
                let obstr = a.unwrap();
                if obstr {
                    println!("Obstruction detected! Stopping elevator.");
                    hw_elevator.motor_direction(e::DIRN_STOP);
                } else {
                    system_elevator.set_motor_direction(&hw_elevator);
                }
            },
        }
    }
}





/*

let poll_period = Duration::from_millis(25);

let (call_button_tx, call_button_rx) = cbc::unbounded::<elevio::poll::CallButton>();
{
    let elevator = elevator.clone();
    spawn(move || elevio::poll::call_buttons(elevator, call_button_tx, poll_period));
}

let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
{
    let elevator = elevator.clone();
    spawn(move || elevio::poll::floor_sensor(elevator, floor_sensor_tx, poll_period));
}

let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
{
    let elevator = elevator.clone();
    spawn(move || elevio::poll::stop_button(elevator, stop_button_tx, poll_period));
}

let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
{
    let elevator = elevator.clone();
    spawn(move || elevio::poll::obstruction(elevator, obstruction_tx, poll_period));
}

let mut dirn = e::DIRN_DOWN;
if elevator.floor_sensor().is_none() {
    elevator.motor_direction(dirn);
}

loop {
    cbc::select! {
        recv(call_button_rx) -> a => {
            let call_button = a.unwrap();
            println!("{:#?}", call_button);
            elevator.call_button_light(call_button.floor, call_button.call, true);
        },
        recv(floor_sensor_rx) -> a => {
            let floor = a.unwrap();
            println!("Floor: {:#?}", floor);
            dirn =
                if floor == 0 {
                    e::DIRN_UP
                } else if floor == elev_num_floors-1 {
                    e::DIRN_DOWN
                } else {
                    dirn
                };
            elevator.motor_direction(dirn);
        },
        recv(stop_button_rx) -> a => {
            let stop = a.unwrap();
            println!("Stop button: {:#?}", stop);
            for f in 0..elev_num_floors {
                for c in 0..3 {
                    elevator.call_button_light(f, c, false);
                }
            }
        },
        recv(obstruction_rx) -> a => {
            let obstr = a.unwrap();
            println!("Obstruction: {:#?}", obstr);
            elevator.motor_direction(if obstr { e::DIRN_STOP } else { dirn });
        },
    }
}
*/

