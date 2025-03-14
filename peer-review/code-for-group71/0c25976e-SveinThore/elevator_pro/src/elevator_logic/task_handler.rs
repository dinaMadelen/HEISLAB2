use std::thread::sleep;
use std::time::Duration;

use crate::network::local_network;
use crate::utils::update_wv;
use crate::world_view::world_view::{ElevatorContainer, TaskStatus};
use crate::elevio::elev;
use crate::{utils, world_view::world_view};


pub async fn execute_tasks(chs: local_network::LocalChannels, elevator: elev::Elevator){
    let mut wv = utils::get_wv(chs.clone());    

    // loop{
    //     let wv = utils::get_wv(chs.clone());
    //     let wv_deser = world_view::deserialize_worldview(&wv);
    //     world_view::print_wv(wv);

    // }
    let mut container: ElevatorContainer;
    update_wv(chs.clone(), &mut wv).await;
    container = utils::extract_self_elevator_container(wv.clone());update_wv(chs.clone(), &mut wv).await;
    container = utils::extract_self_elevator_container(wv.clone());
    elevator.motor_direction(elev::DIRN_DOWN);
    
    loop {
        // let tasks_from_udp = utils::get_elev_tasks(chs.clone());
        update_wv(chs.clone(), &mut wv).await;
        container = utils::extract_self_elevator_container(wv.clone());
        let tasks_from_udp = container.tasks;
        // utils::print_err(format!("last_floor: {}", container.last_floor_sensor));
        if !tasks_from_udp.is_empty() {
            //utils::print_err(format!("TODO: {}, last_floor: {}", 0, container.last_floor_sensor));
            if tasks_from_udp[0].to_do < container.last_floor_sensor {
                elevator.motor_direction(elev::DIRN_DOWN);
            }
            else if tasks_from_udp[0].to_do > container.last_floor_sensor {
                elevator.motor_direction(elev::DIRN_UP);
            }
            else {
                elevator.motor_direction(elev::DIRN_STOP);
                // Si fra at første task er ferdig
                let _ = chs.mpscs.txs.update_task_status.send((tasks_from_udp[0].id, TaskStatus::DONE)).await;
                // open_door_protocol().await;
                sleep(Duration::from_millis(3000));
            }
        }
    }
}
