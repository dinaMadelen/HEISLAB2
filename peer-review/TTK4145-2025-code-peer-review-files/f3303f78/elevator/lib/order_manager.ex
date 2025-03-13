defmodule OrderManager do
  @moduledoc """
  This module is responsible for receiving orders from the poller/network, syncing and distribute
  orders.
  """
  use GenServer
  alias OrderManager.Core, as: Core

  # Types and structs  ----------------------------------------------------------------------------
  @type from :: {pid, any}
  @type order :: Order.t()
  @type order_map :: OrderMap.t()
  @type elevator_state :: ElevatorFSM.t()

  # Config constants  -----------------------------------------------------------------------------
  @top_floor Application.compile_env!(:elevator, :top_floor)
  @bottom_floor Application.compile_env!(:elevator, :bottom_floor)
  @call_duration Application.compile_env!(:elevator, :call_duration)
  @long_call_duration Application.compile_env!(:elevator, :long_call_duration)
  @response_timeout Application.compile_env!(:elevator, :response_timeout)

  # Module init  ----------------------------------------------------------------------------------
  @doc """
  Starts the OrderManager GenServer and links it to Supervisor.
  """
  @spec start_link(any) :: {:ok, pid} | {:error, any} | :ignore
  def start_link(_) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  @doc """
  Initializes the OrderManager GenServer with an empty map of orders.
  """
  @impl GenServer
  @spec init(any) :: {:ok, order_map}
  def init(_) do
    IO.puts("OrderManager started on node key: #{Node.self()}")
    {:ok, %{Node.self() => []}}
  end

  # API functions  --------------------------------------------------------------------------------
  @doc """
  Initiates the process of distributing a new order.
  """
  @spec new_order(order, node) :: :ok
  def new_order(order, origin) do
    case order.type do
      :cab -> GenServer.abcast(Network.all_nodes(), __MODULE__, {:insert_order, order, origin})
      _ -> GenServer.cast(__MODULE__, {:new_order, order})
    end
  end

  @doc """
  Determines if the elevator should stop at the given floor by checking for active orders.
  """
  @spec elevator_stop?(integer, atom) :: boolean
  def elevator_stop?(floor, direction) do
    try do
      GenServer.call(__MODULE__, {:elevator_stop?, floor, direction}, @response_timeout)
    catch
      :exit, {:timeout, _} -> floor in [@bottom_floor, @top_floor]
    end
  end

  @doc """
  Handles the elevator report of a completed order, this can mark several orders on the same floor
  as completed.
  """
  @spec order_done(integer, atom) :: %Task{}
  def order_done(order_floor, elevator_direction) do
    Task.async(fn ->
      GenServer.multi_call(
        Network.all_nodes(),
        __MODULE__,
        {:order_done, Node.self(), order_floor, elevator_direction},
        @long_call_duration
      )
    end)
  end

  @doc """
  Get the next floor (order) to visit for an elevator/node, implements timeout to avoid deadlocks.
  """
  @spec get_next_floor(node, integer, atom) :: integer | :no_orders
  def get_next_floor(node, floor, direction) do
    try do
      GenServer.call(__MODULE__, {:get_next_floor, node, floor, direction}, @response_timeout)
    catch
      :exit, {:timeout, _} -> :no_orders
    end
  end

  @doc """
  Sends a request to remove an order from the module order_state, returns the new order_map.
  """
  @spec remove_order(order, node) :: :ok
  def remove_order(order, node) do
    GenServer.call(__MODULE__, {:remove_order, order, node})
  end

  @doc """
  Sends a request to get the current order_map from the module order_state.
  """
  @spec get_orders() :: order_map
  def get_orders() do
    GenServer.call(__MODULE__, :get_orders)
  end

  @doc """
  Sends a request to nodes to merge their order map with the local order map, then receives those
  merged order maps to merge with their own local order map.
  """
  @spec sync_orders() :: {[{node, order_map}], [node]}
  def sync_orders() do
    {replies, _} = GenServer.multi_call(Node.list(), __MODULE__, :get_orders, @call_duration)
    local_orders = get_orders()

    new_orders =
      Enum.reduce(replies, local_orders, fn {_, external_orders}, new_orders ->
        Core.merge_orders(new_orders, external_orders)
      end)

    GenServer.multi_call(
      Network.all_nodes(),
      __MODULE__,
      {:merge_orders, new_orders},
      @call_duration
    )
  end

  # Calls -----------------------------------------------------------------------------------------
  @doc """
  Implementation of call handling for the GenServer/module.
  """
  @impl GenServer
  @spec handle_call({:elevator_stop?, integer, atom}, from, order_map) ::
          {:reply, boolean, order_map}
  def handle_call({:elevator_stop?, floor, direction}, _from, order_state) do
    next_order = Core.get_next_order(order_state, Node.self(), floor, direction)
    edge_floor = floor in [@bottom_floor, @top_floor]

    case next_order do
      :no_orders -> {:reply, edge_floor, order_state}
      _ -> {:reply, edge_floor or next_order.floor == floor, order_state}
    end
  end

  @impl GenServer
  @spec handle_call({:order_done, node, integer, atom}, from, order_map) ::
          {:reply, :ok, order_map}
  def handle_call({:order_done, node, floor, direction}, _from, order_state) do
    cab_order = %Order{floor: floor, type: :cab}
    new_order_state = Core.remove_order(order_state, cab_order, node)
    OrderSupervisor.terminate_order_supervision(cab_order, node)
    OrderLightController.set_light(cab_order, node, :off)
    next_order = Core.get_next_order(new_order_state, node, floor, direction)

    new_order_state =
      if next_order == :no_orders or next_order.floor != floor do
        new_order_state
      else
        OrderSupervisor.terminate_order_supervision(next_order, node)
        OrderLightController.set_light(next_order, node, :off)
        Core.remove_order(new_order_state, next_order, node)
      end

    {:reply, :ok, new_order_state}
  end

  @impl GenServer
  @spec handle_call({:get_next_floor, node, integer, atom}, from, order_map) ::
          {:reply, integer, order_map}
  def handle_call({:get_next_floor, node, floor, direction}, _from, order_state) do
    next_order = Core.get_next_order(order_state, node, floor, direction)

    next_order_floor =
      case next_order do
        :no_orders -> :no_orders
        _ -> next_order.floor
      end

    {:reply, next_order_floor, order_state}
  end

  @impl GenServer
  @spec handle_call({:remove_order, order, node}, from, order_map) :: {:reply, :ok, order_map}
  def handle_call({:remove_order, order, node}, _from, order_state) do
    new_order_state = Core.remove_order(order_state, order, node)
    {:reply, :ok, new_order_state}
  end

  @impl GenServer
  @spec handle_call(:get_orders, from, order_map) :: {:reply, order_map, order_map}
  def handle_call(:get_orders, _from, order_state) do
    {:reply, order_state, order_state}
  end

  @impl GenServer
  @spec handle_call({:merge_orders, order}, from, order_map) :: {:reply, order_map, order_map}
  def handle_call({:merge_orders, orders}, _from, order_state) do
    new_order_state = Core.merge_orders(order_state, orders)
    OrderLightController.update_lights(new_order_state)
    {:reply, new_order_state, new_order_state}
  end

  # Casts -----------------------------------------------------------------------------------------
  @doc """
  Implementation of cast handling for the GenServer/module.
  """
  @impl GenServer
  @spec handle_cast({:new_order, order}, any) :: {:noreply, order_map}
  def handle_cast({:new_order, order}, order_state) do
    if Core.order_exists?(order_state, order) do
      {:noreply, order_state}
    else
      {replies, _bad_nodes} =
        :rpc.multicall(Network.all_nodes(), ElevatorFSM, :get_state, [], @call_duration)

      node = OrderCost.Core.get_cheapest_node(replies, order_state, order, Node.self())
      GenServer.abcast(Network.all_nodes(), __MODULE__, {:insert_order, order, node})
      {:noreply, order_state}
    end
  end

  @impl GenServer
  @spec handle_cast({:insert_order, order, node}, order_map) :: {:reply, :ok, order_map}
  def handle_cast({:insert_order, order, node}, order_state) do
    new_order_state = Core.add_order(order_state, order, node)
    OrderSupervisor.supervise_order(order, node)
    OrderLightController.set_light(order, node, :on)
    {:noreply, new_order_state}
  end
end

defmodule OrderManager.Core do
  @moduledoc """
  This module is responsible for handling the core functionality of the OrderManager module,
  stateless.
  """
  # Types and structs  ----------------------------------------------------------------------------
  @type order :: Order.t()
  @type order_map :: OrderMap.t()

  # API functions  --------------------------------------------------------------------------------
  @doc """
  Adds an order to the order map, ignores duplicates
  """
  @spec add_order(order_map, order, node) :: order_map
  def add_order(orders, order, node) do
    cond do
      node not in Map.keys(orders) ->
        Map.put(orders, node, [order])

      order not in orders[node] ->
        Map.update(orders, node, [order], fn node_orders -> [order | node_orders] end)

      true ->
        orders
    end
  end

  @doc """
  Removes an order from the order map
  """
  @spec remove_order(order_map, order, node) :: order_map
  def remove_order(orders, order, node) do
    Map.update(orders, node, [], fn node_orders -> List.delete(node_orders, order) end)
  end

  @doc """
  Merges two order maps together
  """
  @spec merge_orders(order_map, order_map) :: order_map
  def merge_orders(orders_1, orders_2) do
    Map.merge(orders_1, orders_2, fn _, order_list_1, order_list_2 ->
      (order_list_1 ++ order_list_2) |> Enum.uniq()
    end)
  end

  @doc """
  Checks if an order exists in the order map
  """
  @spec order_exists?(order_map, order) :: boolean
  def order_exists?(orders, order) do
    orders |> Enum.any?(fn {_, node_orders} -> order in node_orders end)
  end

  @doc """
  Calculates the next floor (order) to handle based on the current floor and direction of the elevator.
  """
  @spec get_next_order(order_map, node, integer, atom) :: order | :no_orders
  def get_next_order(order_map, node, floor, direction) do
    direction_comparator = if direction == :up, do: &>=/2, else: &<=/2
    order_direction = if direction == :up, do: :hall_up, else: :hall_down

    in_path_orders =
      order_map[node]
      |> Enum.filter(fn order -> direction_comparator.(order.floor, floor) end)

    not_in_path_orders =
      order_map[node]
      |> Enum.filter(fn order -> not direction_comparator.(order.floor, floor) end)

    cond do
      length(in_path_orders) > 0 ->
        optimal_destination(in_path_orders, order_direction)

      length(not_in_path_orders) > 0 ->
        reverse_order_direction = if direction == :up, do: :hall_down, else: :hall_up
        optimal_destination(not_in_path_orders, reverse_order_direction)

      true ->
        :no_orders
    end
  end

  # Internal functions  --------------------------------------------------------------------------
  @doc """
  Helper function for get_next_order/4, determines the optimal order based on a pre-processed list
  from get_next_order/4.
  """
  @spec optimal_destination([order], atom) :: integer
  def optimal_destination(order_list, direction) do
    same_direction_orders =
      order_list |> Enum.filter(fn order -> order.type in [:cab, direction] end)

    reverse_direction_orders =
      order_list |> Enum.filter(fn order -> order.type not in [:cab, direction] end)

    cond do
      length(same_direction_orders) > 0 ->
        if direction == :hall_up do
          same_direction_orders |> Enum.min_by(fn order -> order.floor end)
        else
          same_direction_orders |> Enum.max_by(fn order -> order.floor end)
        end

      direction == :hall_up ->
        reverse_direction_orders |> Enum.max_by(fn order -> order.floor end)

      direction == :hall_down ->
        reverse_direction_orders |> Enum.min_by(fn order -> order.floor end)
    end
  end
end
