use crossbeam_channel as cbc;
use std::{process, thread,time};
use std::net;

use network_rust::udpnet;
use super::{controller, elevator,  CTEMessageWrapper, ETCMessageWrapper,MessageWrapper, ElevatorArgument, ElevatorArgumentDataType, HallRequestDataType, HallRequestMatrix};
use crate::elevio::poll::CallButton;
use crate::interface::{CTCMessageWrapper, OrderStateDataType, InactiveStateDataType};

pub struct ElevatorUDP{
    elevator_number: u8, 
}

impl ElevatorUDP{
    /// Set up channels and distribute them to correct modules to facilitate correct communication. Runs controller and elevator main functions.
    pub fn set_up_channels_and_run_all(elevator_id: u8, elevator_port: String, bcast_port: u16, receive_port: u16, peer_listen_port:u16, peer_send_port:u16) ->  Self{
        let unit= ElevatorUDP::init(elevator_id);

        //Connectivity-channels:
        let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
        let (hierarchy_position_tx,hierarchy_position_rx) = cbc::unbounded::<u8>();

        //ETC_channels:
        let (etc_order_state_tx, etc_order_state_rx) =  cbc::unbounded::<(CallButton, bool)>();//Elevator to broadcast
        let (etc_elevator_argument_tx, etc_elevator_argument_rx) = cbc::unbounded::<(u8,ElevatorArgument)>(); //Elevator to broadcast

        let (etc_network_elevator_argument_tx, etc_network_elevator_argument_rx) =cbc::unbounded::<(u8,ElevatorArgument)>();// Elevator/Network to controller
        let (etc_network_order_state_tx, etc_network_order_state_rx) = cbc::unbounded::<(CallButton, bool)>(); //Elevator/network to controller

        //CTE-channels
        let(cte_hall_request_matrix_tx, cte_hall_request_matrix_rx) = cbc::bounded::<(u8, HallRequestMatrix)>(100); //Controller to network
        let(cte_network_hall_request_matrix_tx, cte_network_hall_request_matrix_rx) = cbc::unbounded::<(u8, HallRequestMatrix)>(); //Controller/network to elevator

        //CTC-channels
        let (ctc_reconnecting_hall_requests_tx, ctc_reconnecting_hall_requests_rx) = cbc::unbounded::<HallRequestMatrix>();//Controller to network
        let (ctc_network_hall_requests_tx, ctc_network_hall_requests_rx) = cbc::unbounded::<HallRequestMatrix>();  //Network to controller

        let(ctc_inactive_state_tx, ctc_inactive_state_rx) = cbc::unbounded::<(u8,bool)>();
        let(ctc_network_inactive_state_tx, ctc_network_inactive_state_rx) = cbc::unbounded::<(u8,bool)>();

        let (ctc_elevator_argument_tx, ctc_elevator_argument_rx) = cbc::unbounded::<(u8,ElevatorArgument)>();
        let (ctc_network_elevator_argument_tx, ctc_network_elevator_argument_rx) = cbc::unbounded::<(u8,ElevatorArgument)>();


        //UDP-threads
        {
            let etc_network_elevator_argument_tx = etc_network_elevator_argument_tx.clone();
            let etc_network_call_button_tx = etc_network_order_state_tx.clone();
            let cte_network_hall_request_matrix_tx = cte_network_hall_request_matrix_tx.clone();
            thread::Builder::new()
            .name("Broadcast".to_string())
            .spawn(move || ElevatorUDP::broadcast(
                etc_elevator_argument_rx,
                etc_network_elevator_argument_tx,
                etc_order_state_rx,
                etc_network_call_button_tx,
                hierarchy_position_rx,
                unit.elevator_number,
                cte_hall_request_matrix_rx, 
                cte_network_hall_request_matrix_tx, 
                ctc_reconnecting_hall_requests_rx, 
                ctc_inactive_state_rx,
                ctc_elevator_argument_rx,
                bcast_port
                )).unwrap();
        }

        thread::Builder::new()
        .name(("ETC_Receiver").to_string())
        .spawn(move || ElevatorUDP::reciever(
            etc_network_order_state_tx,
            etc_network_elevator_argument_tx, 
            unit.elevator_number, 
            cte_network_hall_request_matrix_tx, 
            ctc_network_hall_requests_tx, 
            ctc_network_inactive_state_tx,
            ctc_network_elevator_argument_tx,
            receive_port,
            
        )).unwrap();

        //Initiating units with channels properly distributed
        thread::Builder::new()
            .name("Elevator".to_string())
            .spawn(move || elevator::run_elevator(unit.elevator_number,
                etc_order_state_tx, 
                etc_elevator_argument_tx, 
                cte_network_hall_request_matrix_rx, 
                obstruction_tx,
                elevator_port 
            )).unwrap();
        thread::Builder::new()
            .name("Controller".to_string())
            .spawn(move || controller::run_controller(
                unit.elevator_number,
                cte_hall_request_matrix_tx, 
                etc_network_elevator_argument_rx,
                etc_network_order_state_rx, 
                obstruction_rx, 
                hierarchy_position_tx, 
                ctc_reconnecting_hall_requests_tx, 
                ctc_inactive_state_tx,
                ctc_elevator_argument_tx,
                 
                ctc_network_hall_requests_rx,
                ctc_network_inactive_state_rx,
                ctc_network_elevator_argument_rx,
                peer_listen_port,
                peer_send_port
            )).unwrap();
        unit
    }

    fn init(elevator_id: u8) -> Self{
        let adr = net::TcpStream::connect("8.8.8.8:53")//Google is helping us to recieve our local IP
            .unwrap()
            .local_addr()
            .unwrap()
            .ip();
        println!("{}",adr);
        Self{
            elevator_number: elevator_id, 
        }
    }

    fn broadcast(
        etc_elevator_argument_rx: cbc::Receiver<(u8,ElevatorArgument)>,
        etc_network_elevator_argument_tx: cbc::Sender<(u8,ElevatorArgument)>,  
        etc_call_button_rx: cbc::Receiver<(CallButton, bool)>, 
        etc_network_call_button_tx: cbc::Sender<(CallButton, bool)>,
        hierarchy_position_rx: cbc::Receiver<u8>, 
        elevator_number: u8, 
        cte_hall_request_matrix_rx: cbc::Receiver<(u8,HallRequestMatrix)>, 
        cte_network_hall_request_matrix_tx: cbc::Sender<(u8,HallRequestMatrix)>,
        ctc_reconnecting_hall_requests_rx: cbc::Receiver<HallRequestMatrix>,
        ctc_inactive_state_rx: cbc::Receiver<(u8,bool)>,
        ctc_elevator_argument_rx: cbc::Receiver<(u8,ElevatorArgument)>,
        bcast_port: u16 
    ){
        let msg_port = bcast_port;//19735;
        let mut controller_hierarchy_position = elevator_number;

        let (bcast_tx, bcast_rx) = cbc::unbounded::<MessageWrapper>();

        thread::spawn(move || {
            if udpnet::bcast::tx(msg_port, bcast_rx).is_err() {
                // crash program if creating the socket fails (`bcast:tx` will always block if the initialization succeeds)
                process::exit(1);
            }
        });

        loop{
            if let Ok(a) = hierarchy_position_rx.try_recv(){
                controller_hierarchy_position = a;
            }
            while let Ok(elevator_argument) = etc_elevator_argument_rx.try_recv(){
                etc_network_elevator_argument_tx.send(elevator_argument).expect("etc_network_elevator_argument_tx");
                bcast_tx.send(
                    MessageWrapper::ETC(
                        ETCMessageWrapper::ElevatorArgument(
                            ElevatorArgumentDataType{
                                data: elevator_argument.1,
                                elevator_number: elevator_argument.0,
                            }))
                ).expect("ETCMessageWrapper::ElevatorArgument"); 
            }
            while let Ok(call_button) = etc_call_button_rx.try_recv(){
                etc_network_call_button_tx.send(call_button.clone()).expect("etc_network_call_button_tx");
                bcast_tx.send(
                    MessageWrapper::ETC(
                    ETCMessageWrapper::OrderState(
                        OrderStateDataType{
                            data: call_button,
                            elevator_number: 0, 
                        }))    
                ).expect("ETCMessageWrapper::");   
            }
            while let Ok(hall_request_matrix) = cte_hall_request_matrix_rx.try_recv(){
                if  controller_hierarchy_position != 0 {

                }else{ 

                    cte_network_hall_request_matrix_tx.send(hall_request_matrix).expect("cte_network_hall_request_matrix_tx");
                    bcast_tx.send(
                        MessageWrapper::CTE(
                            CTEMessageWrapper::HallRequest(
                                HallRequestDataType{
                                    data: hall_request_matrix.1,
                                    elevator_number: hall_request_matrix.0
                                }))
                    ).expect("MessageWrapper::CTE");
                }
            }
            while let Ok(hall_requests) = ctc_reconnecting_hall_requests_rx.try_recv(){
                bcast_tx.send(
                    MessageWrapper::CTC(
                        CTCMessageWrapper::HallRequest(
                            HallRequestDataType{
                                data: hall_requests,
                                elevator_number: 0
                            }))
                ).expect("MessageWrapper::CTC");
            }
            while let Ok((elevator_number, inactive_state)) = ctc_inactive_state_rx.try_recv(){
                bcast_tx.send(
                    MessageWrapper::CTC(
                        CTCMessageWrapper::InactiveState(
                            InactiveStateDataType{
                                elevator_number: elevator_number,
                                data: inactive_state
                            }))
                ).expect("MessageWrapper::CTC");
            }

            while let Ok(elevator_argument) =  ctc_elevator_argument_rx.try_recv(){
                bcast_tx.send(
                    MessageWrapper::CTC(
                        CTCMessageWrapper::ElevatorArgument(
                            ElevatorArgumentDataType{
                                elevator_number: 0,
                                data: elevator_argument.1
                            }))
                ).expect("MessageWrapper::CTC");
            }
            thread::sleep(time::Duration::from_millis(10));
        }
    }

    fn reciever(
        etc_network_controller_order_state_tx: cbc::Sender<(CallButton,bool)>,
        etc_network_controller_elevator_argument_tx: cbc::Sender<(u8,ElevatorArgument)>,
        unit_elevator_number: u8,
        cte_network_elevator_hall_request_tx: cbc::Sender<(u8, HallRequestMatrix)>,
        ctc_network_hall_requests_tx: cbc::Sender<HallRequestMatrix>,
        ctc_inactive_state_tx: cbc::Sender<(u8,bool)>,
        ctc_elevator_argument_tx: cbc::Sender<(u8,ElevatorArgument)>,
        receive_port: u16
    ){
        let msg_port = receive_port;//19736;
        let (reciever_tx, reciever_rx) = cbc::unbounded::<MessageWrapper>();

        thread::spawn(move || {
            if udpnet::bcast::rx(msg_port, reciever_tx).is_err() {
                // crash program if creating the socket fails (`bcast:rx` will always block if the
                // initialization succeeds)
                process::exit(1);
            }
        });
        loop {
            if let Ok(msg) = reciever_rx.recv(){

                match msg{
                    MessageWrapper::ETC(etc_msg) => match etc_msg{
                        ETCMessageWrapper::ElevatorArgument(elevator_argument) =>{
                            etc_network_controller_elevator_argument_tx.send((elevator_argument.elevator_number, elevator_argument.data)).expect("etc_network_controller_elevator_argument_tx");
                        }
                        ETCMessageWrapper::OrderState(hall_request) =>{
                            etc_network_controller_order_state_tx.send(hall_request.data).expect("etc_network_controller_order_state_tx");
                        }
                    }
                    MessageWrapper::CTE(cte_msg) => match  cte_msg{
                        CTEMessageWrapper::HallRequest(HallRequestDataType{ data, elevator_number}) =>{
                            if elevator_number == unit_elevator_number{
                                cte_network_elevator_hall_request_tx.send((elevator_number,data)).expect("cte_network_elevator_hall_request_tx");
                            }
                        }
                    }
                    MessageWrapper::CTC(ctc_msg) => match ctc_msg {
                        CTCMessageWrapper::HallRequest(hall_request) =>{
                            ctc_network_hall_requests_tx.send(hall_request.data).expect("ctc_network_hall_requests_tx");
                        }
                        CTCMessageWrapper::InactiveState(state_data_type) =>{
                            ctc_inactive_state_tx.send((state_data_type.elevator_number, state_data_type.data)).expect("ctc_inactive_state_tx");
                        } 
                        CTCMessageWrapper::ElevatorArgument(elevator_data_type) =>{
                            ctc_elevator_argument_tx.send((elevator_data_type.elevator_number, elevator_data_type.data)).expect("ctc_elevator_argument_tx");
                        }
                    }
                }
            }
            thread::sleep(time::Duration::from_millis(100));
        }
    }
}
