use crate::elevator;
use std::collections::HashMap;
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TcpLiftOrderT {
    pub button_type: elevator::ButtonType,
    pub floor: u32,
    pub elevator_id: u32,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum tcp_message {
    set_order { order: TcpLiftOrderT },   // a single order
    clear_order { order: TcpLiftOrderT }, // a single order is to be cleared.
    NOP { elevator_id: u32 },             //do nothing
}


#[derive(Debug, Clone, PartialEq)]
pub struct CabButtonsT{
    button_state : [bool ; elevator::N_FLOORS],
    id : u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElevatorDB {
    orders_as_id: [[u32; elevator::N_SHARED_BUTTONS]; elevator::N_FLOORS], // store order id
    cab_buttons: Vec<CabButtonsT>,
}

impl ElevatorDB {
    pub fn new() -> Self {
        ElevatorDB {
            orders_as_id: [[0; elevator::N_SHARED_BUTTONS]; elevator::N_FLOORS],
            cab_buttons : Vec::new(),
        }
    }
    // give the order to the id which fits best. return None if a lift is handeling the order
    pub fn allocate_to_best_id(&mut self, lift_order: TcpLiftOrderT) -> Option<TcpLiftOrderT> {
        ////dbg!("button: {:?} ({:?}) floor: {:?} ({:?})", lift_order.button_type.to_u8(),elevator::N_SHARED_BUTTONS ,lift_order.floor,elevator::N_FLOORS);
        if (lift_order.button_type ==elevator::ButtonType::Cab ){ 
            match (self.cab_buttons.iter().position(|cab| cab.id == lift_order.elevator_id)){
                Some(index) => {
                    self.cab_buttons[index].button_state[lift_order.floor as usize] = true; 
                },
                None =>{ 
                    let mut _order : CabButtonsT = CabButtonsT { button_state: [true;elevator::N_FLOORS], id: lift_order.elevator_id};
                    _order.button_state[lift_order.floor as usize] = true; 
                    self.cab_buttons.push(_order);
                }
            }

         return Some(lift_order);   
        }else if (self.orders_as_id[lift_order.floor as usize][lift_order.button_type.to_u8() as usize])
            != 0
        {
            return None; // the order is already handled by a elevator.
        }
        let result = self.find_least_orders(lift_order.elevator_id);
        let mut _order = lift_order.clone();
        _order.elevator_id = result.unwrap();
        self.set(_order);
        return Some(_order);
    }
    // get all relevant orders of specified id
    // TODO: rename or remove one of them.
    pub fn get_hallbuttons_as_vec(&self, id: u32) -> Vec<TcpLiftOrderT> {
            [[false; elevator::N_SHARED_BUTTONS]; elevator::N_FLOORS];
        let mut output_orders :Vec<TcpLiftOrderT>= Vec::new();
        for (row_index, row) in self.orders_as_id.iter().enumerate() {
            for (col_index, &value) in row.iter().enumerate() {
                if value == id {
                    let order_definition: TcpLiftOrderT = TcpLiftOrderT{button_type:elevator::ButtonType::from_u8(row_index as u8 ),floor:col_index as u32, elevator_id : id }; 
                    output_orders.push(order_definition);
                }
            }
        }
        return output_orders;
    }

    pub fn get_hallbuttons_as_array(&self, id: u32) -> [[bool; elevator::N_SHARED_BUTTONS]; elevator::N_FLOORS] {
        let mut output: [[bool; elevator::N_SHARED_BUTTONS]; elevator::N_FLOORS] =
            [[false; elevator::N_SHARED_BUTTONS]; elevator::N_FLOORS];
        for (row_index, row) in self.orders_as_id.iter().enumerate() {
            for (col_index, &value) in row.iter().enumerate() {
                if value == id {
                    output[row_index][col_index] = true;
                }
            }
        }
        return output;
    }

    pub fn get_all_buttons_as_array(&self, id: u32) -> [[bool; elevator::N_BUTTONS]; elevator::N_FLOORS] {
        let mut output: [[bool; elevator::N_BUTTONS]; elevator::N_FLOORS] =
            [[false; elevator::N_BUTTONS]; elevator::N_FLOORS];
        for (row_index, row) in self.orders_as_id.iter().enumerate() {
            for (col_index, &value) in row.iter().enumerate() {
                if value == id {
                    output[row_index][col_index] = true;
                }
            }
        }
        return output;
    }

    pub fn get_all_buttons_as_vec(&self, id: u32) -> Vec<TcpLiftOrderT> {
        [[false; elevator::N_SHARED_BUTTONS]; elevator::N_FLOORS];
        let mut output_orders :Vec<TcpLiftOrderT>= Vec::new();
        // add all hall orders
        for (row_index, row) in self.orders_as_id.iter().enumerate() {
            for (col_index, &value) in row.iter().enumerate() {
                if value == id {
                    let order_definition: TcpLiftOrderT = TcpLiftOrderT{button_type:elevator::ButtonType::from_u8(row_index as u8 ),floor:col_index as u32, elevator_id : id }; 
                    output_orders.push(order_definition);
                }
            }
        }
        // add all cab buttons
        match (self.cab_buttons.iter().position(|cab| cab.id == id)){
            Some(index) => {
               for (col_index,col) in self.cab_buttons[index].button_state.iter().enumerate(){
                if col == &false{
                  let order_definition: TcpLiftOrderT=TcpLiftOrderT{button_type:elevator::ButtonType::Cab ,floor:col_index as u32, elevator_id : id }; 
                  output_orders.push(order_definition);
            }}},
            None =>{ 
            }
        }

        return output_orders;
    
    }
    // force index to match order
    pub fn set(&mut self, lift_order: TcpLiftOrderT) -> Result<(), u32> {
        if lift_order.elevator_id != 0 {
        if (lift_order.button_type ==elevator::ButtonType::Cab ){ 
            match (self.cab_buttons.iter().position(|cab| cab.id == lift_order.elevator_id)){
                Some(index) => {
                    self.cab_buttons[index].button_state[lift_order.floor as usize] = true; 
                },
                None =>{ 
                    let mut _order : CabButtonsT = CabButtonsT { button_state: [true;elevator::N_FLOORS], id: lift_order.elevator_id};
                    _order.button_state[lift_order.floor as usize] = true; 
                    self.cab_buttons.push(_order);
                }
            }

         return Ok(());   
        }else {
            self.orders_as_id[lift_order.floor as usize][lift_order.button_type.to_u8() as usize] =
                lift_order.elevator_id;
            return Ok(());
    }
    } else {
        return Err(1);
    }




    }
    pub fn clear(&mut self, lift_order: TcpLiftOrderT) -> Result<(), u32> {
        if lift_order.elevator_id != 0 {
        if (lift_order.button_type ==elevator::ButtonType::Cab ){ 
            match (self.cab_buttons.iter().position(|cab| cab.id == lift_order.elevator_id)){
                Some(index) => {
                    self.cab_buttons[index].button_state[lift_order.floor as usize] = false; 
                },
                None =>{ 
                    return Err((1));
                }
            }

         return Ok(());   
        }else {
            self.orders_as_id[lift_order.floor as usize][lift_order.button_type.to_u8() as usize] =
                0;
            return Ok(());
    }
    } else {
        return Err(1);
    }
}

    pub fn print(&self) {
        //dbg!(&self.orders_as_id); // dbg the database
    }

    pub fn remove_and_recalculate(&mut self, id: u32) -> Vec<TcpLiftOrderT>{
        let orders_to_remove = self.get_all_buttons_as_vec(id);
        let mut output: Vec<TcpLiftOrderT> = Vec::new();
        let db = self.clone();
        for ( order) in orders_to_remove.iter() {
            self.clear(*order);
        }
        for ( order) in orders_to_remove.iter() {
            let mut clear_order=order.clone();
             clear_order.elevator_id = 0;
            match self.allocate_to_best_id(*order){
                Some(order) =>{
                    output.push(order);
                },
                None => {}
            }
        }
        
        return output;
    }

    // this implementation can give problems if a "ghost" elevator appears
    pub fn find_least_orders(&self, id: u32) -> Option<u32> {
        let mut counts = HashMap::new();
        // insert the id of the sender, since it might be missing.
        *counts.entry(id).or_insert(0);
        // Iterate over each row in the 2D array
        for row in self.orders_as_id.iter() {
            for &num in row.iter() {
                if num != 0 {
                    // Count occurrences of each number, ignoring 0
                    *counts.entry(num).or_insert(0) += 1;
                }
            }
        }

        // Find the number with the fewest occurrences
        counts
            .into_iter()
            .min_by_key(|&(_, count)| count)
            .map(|(num, _)| num)
    }
    // removes all occurences of a id and replaces it with another.
}
