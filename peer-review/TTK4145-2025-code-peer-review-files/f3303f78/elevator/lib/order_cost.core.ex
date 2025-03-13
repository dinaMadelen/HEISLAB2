defmodule OrderCost.Core do
  @moduledoc """
  This module contains the core functionality for calculating the cost of an order, stateless.
  """
  # Types and structs  ----------------------------------------------------------------------------
  @type order :: Order.t()
  @type order_map :: OrderMap.t()
  @type elevator_state :: ElevatorFSM.t()

  # Config constants  -----------------------------------------------------------------------------
  @door_open_time Application.compile_env!(:elevator, :door_open_time)
  @between_floor_time Application.compile_env!(:elevator, :between_floor_time)

  # API functions  --------------------------------------------------------------------------------
  @doc """
  Returns the node they node that will most time effectively handle the new order
  """
  @spec get_cheapest_node([{node, elevator_state}], order_map, order, node) :: node
  def get_cheapest_node(synced_states, orders, order, fallback_node) do
    available_elevator_states = Enum.filter(synced_states, fn {_, state} -> state.available end)
    floor_adjusted_elevator_states = adjust_elevator_floors(available_elevator_states)

    order_added_map =
      Map.keys(orders)
      |> Enum.reduce(orders, fn node, new_map ->
        OrderManager.Core.add_order(new_map, order, node)
      end)

    current_costs = get_order_costs(floor_adjusted_elevator_states, orders)
    added_costs = get_order_costs(floor_adjusted_elevator_states, order_added_map)
    cost_diff = Enum.map(added_costs, fn {node, cost} -> {node, cost - current_costs[node]} end)

    # TODO: Discuss with Sverre what qualifies as most efficient serving of orders, minimising
    #       total elevator time, minimising order service time, etc...
    _cheapest_node =
      cost_diff
      # Get cheapest nodes based on added cost of the new order
      |> get_cheapest_nodes(fallback_node)
      |> Enum.map(fn {node, _} -> {node, added_costs[node]} end)
      # Get the cheapest of the filtered nodes based on total cost of the nodes
      |> get_cheapest_nodes(fallback_node)
      # If there a multiple options, pick the node with the lowest atom value
      |> Enum.sort(fn {node1, _}, {node2, _} -> node1 < node2 end)
      |> List.first()
      |> elem(0)
  end

  @doc """
  Filters a list of node costs and returns a list of the tuples with the lowest cost
  """
  @spec get_cheapest_nodes([{node, integer}], node) :: [{node, integer}]
  def get_cheapest_nodes(node_list, fallback_node) do
    if Enum.empty?(node_list) do
      [{fallback_node, 0}]
    else
      node_list
      |> Enum.group_by(fn {_, cost} -> cost end)
      |> Enum.min_by(fn {cost, _} -> cost end)
      |> elem(1)
    end
  end

  @doc """
  Returns a list of tuples with the total cost of all the orders each node has to service
  """
  @spec get_order_costs([{node, elevator_state}], order_map) :: [{node, integer}]
  def get_order_costs(synced_states, orders) do
    synced_states
    |> Enum.map(fn {node, %{reported_floor: floor, direction: direction}} ->
      {node, get_node_order_cost(orders, node, floor, direction, 0)}
    end)
  end

  @doc """
  Recursively consume orders for a given node and returns the total cost of all the orders
  """
  @spec get_node_order_cost(order_map, node, integer, atom, integer) :: integer
  def get_node_order_cost(orders, node, floor, direction, total_cost) do
    if Enum.empty?(Map.get(orders, node, [])) do
      total_cost
    else
      new_order = OrderManager.Core.get_next_order(orders, node, floor, direction)
      new_direction = order_to_direction(new_order.type, direction)
      new_orders = OrderManager.Core.remove_order(orders, new_order, node)

      {new_orders, new_direction} =
        skip_same_floor_cab_order(new_orders, new_order, node, new_direction)

      cost = order_cost(new_order.floor, floor)
      get_node_order_cost(new_orders, node, new_order.floor, new_direction, total_cost + cost)
    end
  end

  @doc """
  Removes the next order if the next order is on the same floor as the previous floor, and one of
  the orders was a :cab order.
  """
  @spec skip_same_floor_cab_order(order_map, order, node, atom) :: {order_map, atom}
  def skip_same_floor_cab_order(orders, current_order, node, direction) do
    next_order = OrderManager.Core.get_next_order(orders, node, current_order.floor, direction)

    cond do
      :no_orders in [current_order, next_order] ->
        {orders, direction}

      next_order.floor == current_order.floor and :cab in [next_order.type, current_order.type] ->
        new_direction = order_to_direction(next_order.type, direction)
        new_orders = OrderManager.Core.remove_order(orders, next_order, node)
        {new_orders, new_direction}

      true ->
        {orders, direction}
    end
  end

  @doc """
  Converts an order direction type to the equivalent elevator direction
  """
  @spec order_to_direction(atom, atom) :: atom
  def order_to_direction(order_direction, current_direction) do
    case order_direction do
      :hall_up -> :up
      :hall_down -> :down
      :cab -> current_direction
    end
  end

  @doc """
  Returns the cost of an order based on floor distance and the constant door open time
  """
  @spec order_cost(integer, integer) :: integer
  def order_cost(order_floor, floor) do
    abs(order_floor - floor) * @between_floor_time + @door_open_time
  end

  @doc """
  Updates the reported_floor states to the next floor the elevators can service if moving.
  """
  @spec adjust_elevator_floors([{node, elevator_state}]) :: [{node, elevator_state}]
  def adjust_elevator_floors(synced_states) do
    synced_states
    |> Enum.map(fn {node, state} -> {node, %{state | reported_floor: update_floor(state)}} end)
  end

  @doc """
  Updates the floor in the elevator state to next floor in elevator's path if the elevator is
  moving. Support function for adjust_elevator_floors/1.
  """
  @spec update_floor(elevator_state) :: integer
  def update_floor(state) do
    cond do
      Map.fetch!(state, :state) != :moving -> Map.fetch!(state, :reported_floor)
      Map.fetch!(state, :direction) == :up -> Map.fetch!(state, :reported_floor) + 1
      Map.fetch!(state, :direction) == :down -> Map.fetch!(state, :reported_floor) - 1
    end
  end
end
