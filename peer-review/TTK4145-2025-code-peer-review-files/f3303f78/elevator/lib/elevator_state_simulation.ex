defmodule ElevatorSimul do
  @moduledoc """
  Simulation of the elevator system.
  """
  use Application

  def start(_type, _args) do
    children = [
      {Simul.Driver, []},
      {Simul.OrderManager, []},
      {ElevatorFSM, [0]}
    ]

    opts = [
      strategy: :one_for_all,
      name: ElevatorSimul.Supervisor,
      max_restarts: 2,
      max_seconds: 1,
      shutdown: :never
    ]

    Supervisor.start_link(children, opts)
  end
end

defmodule Simul.Driver do
  @moduledoc """
  Rudimentary simulation of the driver provided by the course.
  """

  use GenServer

  defstruct [:floor, :direction]

  def start_link([]) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  def init([]) do
    driver_state = %__MODULE__{floor: 1, direction: :stop}
    {:ok, driver_state}
  end

  def get_floor_sensor_state() do
    GenServer.call(__MODULE__, :get_floor_sensor_state)
  end

  def set_motor_direction(direction) do
    GenServer.call(__MODULE__, {:set_motor_direction, direction})
  end

  def set_door_open_light(state) do
    IO.puts("Door open light set to #{state}")
  end

  def get_obstruction_switch_state() do
    GenServer.call(__MODULE__, :get_obstruction_switch_state)
  end

  # def new_floor_event do
  #  GenServer.cast(__MODULE__, :new_floor_event)
  # end

  # Calls
  def handle_call(:get_floor_sensor_state, _from, state) do
    case :rand.uniform(2) do
      1 -> {:reply, :between_floors, state}
      _ -> {:reply, state.floor, state}
    end
  end

  def handle_call({:set_motor_direction, direction}, _from, state) do
    IO.puts("Motor direction set to #{direction}")

    new_floor =
      case direction do
        :up -> state.floor + 1
        :down -> state.floor - 1
        :stop -> state.floor
      end

    new_state = %{state | floor: new_floor, direction: direction}

    if new_state.direction != :stop do
      IO.puts("Setting up new floor event")
      spawn_link(fn -> Simul.Driver.new_floor_event(new_state.floor) end)
    end

    {:reply, :ok, new_state}
  end

  def handle_call(:get_obstruction_switch_state, _from, state) do
    {:reply, Enum.random([:active, :inactive]), state}
  end

  # Casts
  def new_floor_event(floor) do
    IO.puts("New floor event called")
    :timer.sleep(1000)
    IO.puts("New floor event starting for floor #{floor}")
    ElevatorFSM.arrived_floor(floor)
  end
end

defmodule Simul.OrderManager do
  @moduledoc """
  Rudimentary simulation of the order manager provided by the course.
  """

  use GenServer

  @top_floor Application.compile_env(:elevator, :top_floor, 3)

  defstruct [:orders]

  def start_link([]) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  def init([]) do
    {:ok, %__MODULE__{orders: []}}
  end

  def elevator_stop?(floor, direction) do
    GenServer.call(__MODULE__, {:elevator_stop?, floor, direction})
  end

  def order_done(floor) do
    GenServer.cast(__MODULE__, {:order_done, floor})
  end

  def get_next_floor(_node, _floor, _direction) do
    GenServer.call(__MODULE__, :get_next_floor)
  end

  # Calls
  def handle_call({:elevator_stop?, floor, _direction}, _from, state) do
    answer =
      Enum.member?(state.orders, floor) or
        Enum.member?([0, @top_floor], floor)

    {:reply, answer, state}
  end

  def handle_call(:get_next_floor, _from, state) do
    case :rand.uniform(8) do
      4 ->
        state = %{state | orders: state.orders ++ [0]}
        {:reply, 0, state}

      1 ->
        state = %{state | orders: state.orders ++ [1]}
        {:reply, 1, state}

      2 ->
        state = %{state | orders: state.orders ++ [2]}
        {:reply, 2, state}

      3 ->
        state = %{state | orders: state.orders ++ [3]}
        {:reply, 3, state}

      _ ->
        {:reply, :no_orders, state}
    end
  end

  # Casts
  def handle_cast({:order_done, floor}, state) do
    state = %{state | orders: List.delete(state.orders, floor)}
    {:noreply, state}
  end
end
