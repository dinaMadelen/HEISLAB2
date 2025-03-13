defmodule ButtonPoller do
  @moduledoc """
  This module is responsible for polling the order buttons and notify OrderManager when an order is
  registered.
  """
  alias ButtonPoller.Core, as: Core

  # Config constants ------------------------------------------------------------------------------
  @top_floor Application.compile_env!(:elevator, :top_floor)
  @bottom_floor Application.compile_env!(:elevator, :bottom_floor)
  @button_poll_rate Application.compile_env!(:elevator, :button_poll_rate)

  # Module init -----------------------------------------------------------------------------------
  @doc """
  Starts the polling process and links it to Supervisor.
  """
  @spec start_link() :: {:ok, pid}
  def start_link() do
    pid = spawn_link(fn -> init() end)
    {:ok, pid}
  end

  @doc """
  Child specification required by supervisor.
  """
  @spec child_spec(any) :: map
  def child_spec(_) do
    %{id: __MODULE__, start: {__MODULE__, :start_link, []}, restart: :permanent, type: :worker}
  end

  @doc """
  Initialises the button pollers and then goes into a permanent sleep so that the Supervisor can
  supervise the process and its linked processes.
  """
  @spec init() :: no_return
  def init() do
    Core.get_all_buttons(@bottom_floor, @top_floor)
    |> Enum.each(fn button ->
      spawn_link(fn -> poll_button_sensor(button.type, button.floor, 0) end)
    end)

    Process.sleep(:infinity)
  end

  # Internal functions ----------------------------------------------------------------------------
  @doc """
  Polls the button sensor and notifies OrderManager when an order is registered.
  """
  @spec poll_button_sensor(atom, integer, 0) :: no_return
  def poll_button_sensor(button_type, floor, 0) do
    :timer.sleep(@button_poll_rate)
    button_state = Driver.get_order_button_state(floor, button_type)

    case button_state do
      1 -> OrderManager.new_order(%Order{floor: floor, type: button_type}, Node.self())
      0 -> :ignore
    end

    poll_button_sensor(button_type, floor, button_state)
  end

  @spec poll_button_sensor(atom, integer, 1) :: no_return
  def poll_button_sensor(button_type, floor, 1) do
    :timer.sleep(@button_poll_rate)
    poll_button_sensor(button_type, floor, Driver.get_order_button_state(floor, button_type))
  end
end

defmodule ButtonPoller.Core do
  @moduledoc """
  Core functions for ButtonPoller, stateless.
  """
  # Types and structs  ----------------------------------------------------------------------------
  @type order :: Order.t()

  # API functions  --------------------------------------------------------------------------------
  @doc """
  Taken and modified from: https://github.com/jostlowe/kokeplata/blob/master/lib/buttonstuff.ex
  Returns all possible orders on a single elevator
  Returns a list of tuples on the form {button_type, floor}

  ## Examples
      iex> Kokeplata.get_all_buttons(0, 3)
      [
      %Order{floor: 0, type: :hall_up},
      %Order{floor: 1, type: :hall_up},
      %Order{floor: 2, type: :hall_up},
      %Order{floor: 1, type: :hall_down},
      %Order{floor: 2, type: :hall_down},
      %Order{floor: 3, type: :hall_down},
      %Order{floor: 0, type: :cab},
      %Order{floor: 1, type: :cab},
      %Order{floor: 2, type: :cab},
      %Order{floor: 3, type: :cab},
      ]
  """
  @spec get_all_buttons(integer, integer) :: [order]
  def get_all_buttons(bottom_floor, top_floor) do
    [:hall_up, :hall_down, :cab]
    |> Enum.map(fn button_type -> get_buttons_of_type(button_type, bottom_floor, top_floor) end)
    |> List.flatten()
  end

  @doc """
  Taken and modified from: https://github.com/jostlowe/kokeplata/blob/master/lib/buttonstuff.ex
  Returns all possible orders of a single button type, given the number of the top floor
  Returns a list of tuples on the form {button_type, floor}

  ## Examples
      iex> Kokeplata.get_buttons_of_type(:hall_up, 0, 3)
      [
      %Order{floor: 0, type: :hall_up},
      %Order{floor: 1, type: :hall_up},
      %Order{floor: 2, type: :hall_up},
      ]
  """
  @spec get_buttons_of_type(atom, integer, integer) :: [order]
  def get_buttons_of_type(button_type, bottom_floor, top_floor) do
    floor_list =
      case button_type do
        :hall_up -> bottom_floor..(top_floor - 1)
        :hall_down -> (bottom_floor + 1)..top_floor
        :cab -> bottom_floor..top_floor
      end

    floor_list |> Enum.map(fn floor -> %Order{floor: floor, type: button_type} end)
  end
end
