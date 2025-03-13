defmodule ElevatorFSM do
  @moduledoc """
  Implementation of the elevator state machine, using gen_statem behaviour.
  """
  @behaviour :gen_statem
  alias ElevatorFSM.Core, as: Core
  # alias Simul.Driver, as: Driver
  # alias Simul.OrderManager, as: OrderManager

  # Types and structs  ----------------------------------------------------------------------------
  @enforce_keys [:reported_floor, :direction, :available]
  defstruct [:reported_floor, :direction, :available]
  @type t :: %{reported_floor: integer, direction: atom, available: boolean, state: atom}
  @type from :: {pid, any}

  # Config constants  -----------------------------------------------------------------------------
  @elevator_name Application.compile_env!(:elevator, :elevator_name)
  @door_open_time Application.compile_env!(:elevator, :door_open_time)
  @order_pool_rate Application.compile_env!(:elevator, :order_pool_rate)
  @motor_failure Application.compile_env!(:elevator, :between_floor_time)

  # Module init  ----------------------------------------------------------------------------------
  @doc """
  Set the state machine to act on a per-event basis.
  """
  @impl :gen_statem
  @spec callback_mode() :: [atom]
  def callback_mode() do
    [:handle_event_function]
  end

  @doc """
  Starts the elevator state machine server and links the process to the calling process.
  """
  @spec start_link() :: {:ok, pid} | {:error, any} | :ignore
  def start_link() do
    server_name = {:local, @elevator_name}
    :gen_statem.start_link(server_name, __MODULE__, [], [])
  end

  @doc """
  Child specification required by supervisor.
  """
  @spec child_spec(any) :: map
  def child_spec(_) do
    %{id: __MODULE__, start: {__MODULE__, :start_link, []}, restart: :permanent, type: :worker}
  end

  @doc """
  Initialises the elevator state machine with the initial state and data.
  """
  @impl :gen_statem
  @spec init(any) :: {:ok, atom, %__MODULE__{}, tuple}
  def init(_) do
    IO.puts("ElevatorFSM starting up")
    OrderLightController.update_lights(OrderManager.get_orders())
    Driver.set_door_open_light(:off)

    case Driver.get_floor_sensor_state() do
      :between_floors ->
        fsm_data = %__MODULE__{reported_floor: 0, direction: :down, available: true}
        {:ok, :moving, fsm_data, {:next_event, :internal, :start_motor}}

      reported_floor ->
        Driver.set_motor_direction(:stop)
        fsm_data = %__MODULE__{reported_floor: reported_floor, direction: :down, available: true}
        {:ok, :idle, fsm_data, {:state_timeout, @order_pool_rate, :check_order}}
    end
  end

  # Termination handling  -------------------------------------------------------------------------
  @doc """
  Called upon termination of the state machine, make sure the elevator stops and turn off lights.
  """
  @impl :gen_statem
  @spec terminate(any, atom, %__MODULE__{}) :: :ok
  def terminate(reason, _state, _fsm_data) do
    IO.puts("ElevatorFSM terminating due to: #{inspect(reason)}")
    Driver.set_motor_direction(:stop)
    OrderLightController.update_lights(%{})
  end

  # API functions  --------------------------------------------------------------------------------
  @doc """
  Get the current state of the elevator.
  """
  @spec get_state() :: {node, ElevatorFSM.t()}
  def get_state() do
    {Node.self(), :gen_statem.call(@elevator_name, :get_state, 2000)}
  end

  @doc """
  Creates a floor arrival event, notifying the elevator that it has arrived at a floor.
  """
  @spec arrived_floor(integer) :: :ok
  def arrived_floor(floor) do
    :gen_statem.cast(@elevator_name, {:arrived_floor, floor})
  end

  # Event handlers  -------------------------------------------------------------------------------
  @doc """
  Event handler for the elevator state machine.
  """
  @impl :gen_statem
  def handle_event(:cast, {:arrived_floor, floor}, state, fsm_data) do
    Driver.set_floor_indicator(floor)
    new_fsm_data = %__MODULE__{fsm_data | reported_floor: floor}
    OrderSupervisor.reset_timers(Node.self())

    case state do
      :moving ->
        if OrderManager.elevator_stop?(floor, fsm_data.direction) do
          #Bell.play()
          actions = [{:state_timeout, :cancel}, {:next_event, :internal, :floor_arrival}]
          {:keep_state, new_fsm_data, actions}
        else
          {:keep_state, new_fsm_data, {:state_timeout, @motor_failure, :motor_failure}}
        end

      _ ->
        {:keep_state, new_fsm_data, {:state_timeout, :cancel}}
    end
  end

  @impl :gen_statem
  def handle_event(:internal, :floor_arrival, _state, fsm_data) do
    IO.puts("Elevator has arrived at floor #{fsm_data.reported_floor}")
    Driver.set_motor_direction(:stop)
    Driver.set_door_open_light(:on)
    OrderManager.order_done(fsm_data.reported_floor, fsm_data.direction)
    actions = {:state_timeout, @door_open_time, :door_timeout}
    {:next_state, :open_door, %__MODULE__{fsm_data | available: true}, actions}
  end

  @impl :gen_statem
  def handle_event(:state_timeout, :door_timeout, _state, fsm_data) do
    case Driver.get_obstruction_switch_state() do
      :active ->
        IO.puts("Obstruction detected, waiting for obstruction to be removed")
        actions = {:state_timeout, @door_open_time, :door_timeout}
        {:keep_state, %__MODULE__{fsm_data | available: false}, actions}

      :inactive ->
        Driver.set_door_open_light(:off)
        actions = {:state_timeout, 0, :check_order}
        {:next_state, :idle, %__MODULE__{fsm_data | available: true}, actions}
    end
  end

  @impl :gen_statem
  def handle_event(:state_timeout, :check_order, _state, fsm_data) do
    case OrderManager.get_next_floor(Node.self(), fsm_data.reported_floor, fsm_data.direction) do
      :no_orders ->
        {:next_state, :idle, fsm_data, {:state_timeout, @order_pool_rate, :check_order}}

      order_floor ->
        IO.puts("New order at floor #{order_floor}")

        case Core.get_new_direction(fsm_data.reported_floor, order_floor) do
          :at_floor ->
            {:keep_state_and_data, {:next_event, :internal, :floor_arrival}}

          dir ->
            actions = {:next_event, :internal, :start_motor}
            {:keep_state, %__MODULE__{fsm_data | direction: dir}, actions}
        end
    end
  end

  @impl :gen_statem
  def handle_event(:internal, :start_motor, _state, fsm_data) do
    Driver.set_motor_direction(fsm_data.direction)
    {:next_state, :moving, fsm_data, {:state_timeout, @motor_failure, :motor_failure}}
  end

  @impl :gen_statem
  def handle_event(:state_timeout, :motor_failure, _state, fsm_data) do
    IO.puts("Motor failure detected, elevator is now unavailable")
    {:keep_state, %__MODULE__{fsm_data | available: false}}
  end

  @impl :gen_statem
  def handle_event({:call, from}, :get_state, state, fsm_data) do
    :gen_statem.reply(from, Map.merge(%{state: state}, fsm_data))
    {:keep_state_and_data, []}
  end

  @impl :gen_statem
  def handle_event(:info, _info, _state, _fsm_data) do
    {:keep_state_and_data, []}
  end
end

defmodule ElevatorFSM.Core do
  @moduledoc """
  Core functions for the elevator state machine, stateless.
  """
  # API functions  --------------------------------------------------------------------------------
  @doc """
  Get the direction of the elevator based on the current floor and the destination floor.
  """
  @spec get_new_direction(integer, integer) :: atom
  def get_new_direction(current_floor, destination_floor) do
    cond do
      current_floor < destination_floor -> :up
      current_floor > destination_floor -> :down
      true -> :at_floor
    end
  end
end
