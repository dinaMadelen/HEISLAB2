defmodule FSM do
  import Util

  @upwards 0

  def should_stop?(fsm, polled_floor) do
    if polled_floor == :between_floors do false else
      if request_in_current_direction?(fsm) do true else
        if last_request_in_direction?(fsm) do true else
          false
    end end end
  end

  def consume_order_from_idle(fsm) do
    case {fsm.last_direction, request_above?(fsm), request_below?(fsm), request_this_floor?(fsm)} do
      {:up  , true, _   , _   } -> fsm_set_direction(fsm, :up)         # continue in direction
      {:down, _   , true, _   } -> fsm_set_direction(fsm, :down)       # continue in direction
      {:up  , _   , true, _   } -> fsm_set_direction(fsm, :down)       # change direction
      {:down, true, _   , _   } -> fsm_set_direction(fsm, :up)         # change direction
      {_    , _   , _   , true} -> arrive_at_floor(fsm, go_idle=false) # more requests at this floor
      {_    , _   , _   , _   } -> fsm
    end
  end

  def update_fsm_with_polled_info(state, polled_floor, polled_hall_requests, polled_cab_requests) do
    fsm = Map.get(state, Node.self())
    fsm = if is_atom(polled_floor) do fsm else Map.replace!(fsm, :floor_integer, polled_floor) end
    fsm = Map.replace!(fsm, :cab_requests, boolean_array_or(polled_cab_requests, fsm.cab_requests))
    state
      |> Map.replace!(:global_hall_requests, boolean_array_or(polled_hall_requests, state.global_hall_requests))
      |> Map.replace!(Node.self(), fsm)
  end

  def arrive_at_floor(fsm, go_idle) do
    GenServer.cast(Driver, {:set_motor_direction, :stop})
    GenServer.cast(Driver, {:set_door_open_light, if(go_idle, do: :off, else: :on)})
    GenServer.cast(Driver, {:set_floor_indicator, fsm.floor_integer})
    fsm
      |> Map.replace!(:behaviour, if(go_idle, do: :idle, else: :doorOpen))
      |> Map.replace!(:direction, :stop)
      |> Map.replace!(:last_time, if(go_idle, do: fsm.last_time, else: :erlang.monotonic_time(:millisecond)))
      |> clear_requests_at_current_floor()
  end

  def number_of_orders(fsm) do
    Enum.count(fsm.cab_requests ++ List.flatten(fsm.assigned_hall_requests), fn x ->x == true end)
  end

  def clear_requests_at_current_floor(fsm) do
    hall_requests_at_floor = Enum.at(fsm.assigned_hall_requests, fsm.floor_integer)
    direction_index = case fsm.last_direction do
      :up -> 0
      :down -> 1
    end
    opposite_direction_index = case fsm.last_direction do
      :up -> 1
      :down -> 0
    end
    for node <- [Node.self() | Node.list()] do
      GenServer.cast({Controller, node}, {:clear_hall_request, fsm.floor_integer, direction_index})
    end

    hall_requests_at_floor = List.replace_at(hall_requests_at_floor, direction_index, false)

    hall_requests_at_floor = if((number_of_orders(fsm) == 1)
      and Enum.at(hall_requests_at_floor, opposite_direction_index)) do
        for node <- [Node.self() | Node.list()] do
            GenServer.cast({Controller, node}, {:clear_hall_request, fsm.floor_integer, opposite_direction_index})
        end
        [false, false]
    else
        hall_requests_at_floor
    end

    %{fsm | assigned_hall_requests: List.replace_at(fsm.assigned_hall_requests, fsm.floor_integer, hall_requests_at_floor),
            cab_requests: List.replace_at(fsm.cab_requests, fsm.floor_integer, false)}
  end


  def fsm_set_direction(fsm, direction) do
    GenServer.cast(Driver, {:set_motor_direction, direction})
    fsm
      |> Map.replace!(:direction, direction)
      |> Map.replace!(:last_direction, direction)
      |> Map.replace!(:behaviour, :moving)
  end

  def update_lights(state) do
    Enum.each(Enum.with_index(state.global_hall_requests), fn {up_down_pair, index} ->
      GenServer.cast(Driver, {:set_order_button_light, :hall_up, index, if(Enum.at(up_down_pair, 0), do: :on, else: :off)})
      GenServer.cast(Driver, {:set_order_button_light, :hall_down, index, if(Enum.at(up_down_pair, 1), do: :on, else: :off)}) end)

    fsm = Map.get(state, Node.self())
    Enum.each(Enum.with_index(fsm.cab_requests), fn {cab_request, index} ->
      GenServer.cast(Driver, {:set_order_button_light, :cab, index, if(cab_request, do: :on, else: :off)}) end)
    fsm
  end
end
