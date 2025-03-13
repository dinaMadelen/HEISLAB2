defmodule OrderSupervisor do
  @moduledoc """
  This module is responsible for supervising orders, re-inserting orders that have not been
  completed within an expected time frame.
  """
  use GenServer
  alias OrderSupervisor.Core, as: Core

  # Types and structs  ----------------------------------------------------------------------------
  @type from :: {pid, any}
  @type order :: Order.t()
  @type t :: %{{order, node} => pid}

  # Config constants  -----------------------------------------------------------------------------
  @long_call_duration Application.compile_env!(:elevator, :long_call_duration)
  @elevator_timeout Application.compile_env!(:elevator, :between_floor_time) * 2

  # Module init  ----------------------------------------------------------------------------------
  @doc """
  Starts the OrderSupervisor GenServer and links it to Supervisor.
  """
  @spec start_link(any) :: {:ok, pid} | {:error, any} | :ignore
  def start_link(_) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  @doc """
  Initializes the OrderSupervisor GenServer.
  """
  @impl GenServer
  @spec init(any) :: {:ok, OrderSupervisor.t()}
  def init(_) do
    {:ok, %{}}
  end

  # API functions  --------------------------------------------------------------------------------
  @doc """
  Starts supervision of an order
  """
  @spec supervise_order(order, node) :: :ok
  def supervise_order(order, node) do
    GenServer.call(__MODULE__, {:supervise_order, order, node})
  end

  @doc """
  Terminates the supervision of an order
  """
  @spec terminate_order_supervision(order, node) :: :ok
  def terminate_order_supervision(order, node) do
    GenServer.call(__MODULE__, {:terminate_order_supervision, order, node})
  end

  @doc """
  Resets all timers for a given node
  """
  @spec reset_timers(node) :: %Task{}
  def reset_timers(node) do
    Task.async(fn ->
      GenServer.multi_call(
        Network.all_nodes(),
        __MODULE__,
        {:reset_timers, node},
        @long_call_duration
      )
    end)
  end

  # Internal functions  ---------------------------------------------------------------------------
  @doc """
  Supervises an order. Resets or terminates supervision based on received messages. Simulates new order if timeout.
  """
  def supervise(order, node) do
    receive do
      :terminate -> :ok
      :reset_timer -> supervise(order, node)
    after
      @elevator_timeout ->
        OrderManager.remove_order(order, node)
        OrderManager.new_order(order, node)
        terminate_order_supervision(order, node)
    end
  end

  # Casts -----------------------------------------------------------------------------------------
  @doc """
  Implementation of call handling for the GenServer/module.
  """
  @impl GenServer
  @spec handle_call({:supervise_order, order, node}, from, OrderSupervisor.t()) ::
          {:reply, :ok, OrderSupervisor.t()}
  def handle_call({:supervise_order, order, node}, _from, supervision_state) do
    case Map.has_key?(supervision_state, {order, node}) do
      true ->
        {:reply, :ok, supervision_state}

      false ->
        pid = spawn_link(fn -> supervise(order, node) end)
        {:reply, :ok, Map.put(supervision_state, {order, node}, pid)}
    end
  end

  @impl GenServer
  @spec handle_call({:terminate_order_supervision, order, node}, from, OrderSupervisor.t()) ::
          {:reply, :ok, OrderSupervisor.t()}
  def handle_call({:terminate_order_supervision, order, node}, _from, supervision_state) do
    case Map.get(supervision_state, {order, node}) do
      nil -> :ok
      pid -> send(pid, :terminate)
    end

    {:reply, :ok, Map.delete(supervision_state, {order, node})}
  end

  @impl GenServer
  @spec handle_call({:reset_timers, node}, from, OrderSupervisor.t()) ::
          {:reply, :ok, OrderSupervisor.t()}
  def handle_call({:reset_timers, node}, _from, supervision_state) do
    Core.get_pids_for_node(supervision_state, node)
    |> Enum.each(fn pid -> send(pid, :reset_timer) end)

    {:reply, :ok, supervision_state}
  end
end

defmodule OrderSupervisor.Core do
  @moduledoc """
  This module contains core functions for the OrderSupervisor module, stateless.
  """
  # Types and structs  ----------------------------------------------------------------------------
  @type supervision_map :: OrderSupervisor.t()

  # API functions  --------------------------------------------------------------------------------
  @doc """
  Returns a list of all pids for a given node in the supervision map.
  """
  @spec get_pids_for_node(supervision_map, node) :: [pid]
  def get_pids_for_node(supervision_map, target_node) do
    supervision_map
    |> Enum.filter(fn {{_, node}, _} -> node == target_node end)
    |> Enum.map(fn {_, pid} -> pid end)
  end
end
