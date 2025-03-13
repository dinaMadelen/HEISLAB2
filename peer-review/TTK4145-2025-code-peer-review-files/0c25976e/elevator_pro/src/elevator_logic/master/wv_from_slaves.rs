use crate::world_view::world_view::{self, ElevatorContainer};
use crate::elevator_logic::master::wv_from_slaves::world_view::TaskStatus;
use std::collections::HashSet;

/// ### Oppdatere statuser til slave-heis basert på melding fra TCP
pub async fn update_statuses(deser_wv: &mut world_view::WorldView, container: &ElevatorContainer, i: usize) {
    //Setter alle 'enkle' statuser likt som slaven har 
    deser_wv.elevator_containers[i].door_open = container.door_open;
    deser_wv.elevator_containers[i].last_floor_sensor = container.last_floor_sensor;
    deser_wv.elevator_containers[i].obstruction = container.obstruction;
    deser_wv.elevator_containers[i].motor_dir = container.motor_dir;
    deser_wv.elevator_containers[i].calls = container.calls.clone(); 
    deser_wv.elevator_containers[i].tasks_status = container.tasks_status.clone();

    // Finner ID til tasks slaven er ferdig med
    let completed_tasks_ids: HashSet<u16> = container
        .tasks_status
        .iter()
        .filter(|t| t.status == TaskStatus::DONE)
        .map(|t| t.id)
        .collect();

    /*_____ Fjern Tasks som er markert som ferdig av slaven _____ */
    deser_wv.elevator_containers[i].tasks.retain(|t| !completed_tasks_ids.contains(&t.id));
}

/// ### Oppdaterer globale call_buttons fra slaven sine lokale call_buttons
pub async fn update_call_buttons(deser_wv: &mut world_view::WorldView, container: &ElevatorContainer, i: usize) {
    // Sett opp et HashSet for å sjekke for duplikater
    let mut seen = HashSet::new();
    
    // Legg til eksisterende elementer i HashSet
    for &elem in &deser_wv.outside_button.clone() {
        seen.insert(elem);
    }

    // Utvid outside_button med elementer som ikke er i HashSet
    //println!("Callbtwns hos slave {}: {:?}", container.elevator_id, container.calls);
    for &call in &container.calls {
        if !seen.contains(&call) {
            deser_wv.outside_button.push(call);
            seen.insert(call.clone());
        }
    }
}

/// Kommende funksjon
pub async fn update_tasks() {

}