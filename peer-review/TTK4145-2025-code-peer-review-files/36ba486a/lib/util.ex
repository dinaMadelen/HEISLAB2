defmodule Util do
  @up 0
  @down 1

  def up_this_floor?(fsm) do Enum.at(fsm.assigned_hall_requests, fsm.floor_integer) |> Enum.at(@up) end

  def down_this_floor?(fsm) do Enum.at(fsm.assigned_hall_requests, fsm.floor_integer) |> Enum.at(@down) end

  def cab_this_floor?(fsm) do Enum.at(fsm.cab_requests, fsm.floor_integer) end

  def request_this_floor?(fsm) do up_this_floor?(fsm) or down_this_floor?(fsm) or cab_this_floor?(fsm) end

  def request_above?(fsm) do
    m_floors = Application.fetch_env!(:elevator, :m_floors)
    if(fsm.floor_integer == m_floors) do false else # edge case
      range = fsm.floor_integer+1..m_floors
      request_at_floors = boolean_array_or(hall_and_cab_combined_1d(fsm.assigned_hall_requests), fsm.cab_requests)
      Enum.any?(Enum.slice(request_at_floors, range)) end
  end

  def request_below?(fsm) do
    if(fsm.floor_integer == 0) do false else # edge case
      range = 0..fsm.floor_integer-1
      request_at_floors = boolean_array_or(hall_and_cab_combined_1d(fsm.assigned_hall_requests), fsm.cab_requests)
      Enum.any?(Enum.slice(request_at_floors, range)) end
  end

  def last_request_in_direction?(fsm) do
    case {fsm.direction, up_this_floor?(fsm), down_this_floor?(fsm), request_above?(fsm), request_below?(fsm)} do
      {:up  , _   , true, false, _    } -> true  # upwards last
      {:down, true, _   , _    , false} -> true  # downwards last
      {:up  , _   , true, true , _    } -> false # more requests in direction of travel
      {:down, true, _   , _    , true } -> false # more requests in direction of travel
      {_    , _   , _   , _    , _    } -> false # empty order catch
    end
  end

  def request_in_current_direction?(fsm) do
    (case fsm.last_direction do
      :up -> up_this_floor?(fsm)
      :down -> down_this_floor?(fsm)
    end or cab_this_floor?(fsm))
  end

  def hall_and_cab_combined_1d(hall_requests) do
    Enum.map(hall_requests, fn per_floor_pair ->
        Enum.any?(per_floor_pair, fn val -> val end) end)
  end

  def boolean_array_or(left, right) do
    cond do
      is_list(hd(left)) and is_list(hd(right)) -> # Handle 2D arrays
        Enum.zip(left, right) |> Enum.map(fn {row1, row2} ->
          Enum.zip(row1, row2) |> Enum.map(fn {a, b} -> a or b end) end)

      is_list(left) and is_list(right) -> # Handle 1D arrays
        Enum.zip(left, right) |> Enum.map(fn {a, b} -> a or b end)

      true ->
        raise ArgumentError, "Inputs must both be either 1D or 2D boolean arrays"
    end
  end
end
