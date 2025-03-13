defmodule Main do
  @moduledoc """
  Main module for the elevator project, starts the Application Supervisor.
  """
  use Application

  # Config constants  -----------------------------------------------------------------------------
  @application_environment Application.compile_env!(:elevator, :env)

  def start(_type, _args) do
    children =
      case @application_environment do
        :test ->
          []

        _ ->
          [
            {Driver, []},
            {Network, []},
            {ButtonPoller, []},
            {ElevatorFloorPoller, []},
            {OrderManager, []},
            {OrderSupervisor, []},
            {ElevatorFSM, []}
          ]
      end

    opts = [
      strategy: :one_for_all,
      name: Elevator.Supervisor,
      max_restarts: 10,
      max_seconds: 1,
      auto_shutdown: :never
    ]

    Supervisor.start_link(children, opts)
  end
end
