defmodule ElevatorFloorPoller do
  @moduledoc """
  This module is responsible for polling the floor sensor and notifying the elevator when a new
  floor is detected.
  """
  # Config constants  ------------------------------------------------------------------------------
  @floor_detector_poll_rate Application.compile_env!(:elevator, :floor_detector_poll_rate)

  # Module init -----------------------------------------------------------------------------------
  @doc """
  Starts the polling process and links it to Supervisor.
  """
  @spec start_link() :: {:ok, pid} | {:error, any} | :ignore
  def start_link() do
    pid = spawn_link(fn -> poll_floor_sensor(0) end)
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
  Polls the floor sensor and notifies the elevator when a floor is registered.
  """
  @spec poll_floor_sensor(:between_floors) :: no_return
  def poll_floor_sensor(:between_floors) do
    :timer.sleep(@floor_detector_poll_rate)
    floor = Driver.get_floor_sensor_state()

    if is_integer(floor) do
      ElevatorFSM.arrived_floor(floor)
    end

    poll_floor_sensor(floor)
  end

  @spec poll_floor_sensor(integer) :: no_return
  def poll_floor_sensor(floor) when is_integer(floor) do
    :timer.sleep(@floor_detector_poll_rate)
    poll_floor_sensor(Driver.get_floor_sensor_state())
  end
end
