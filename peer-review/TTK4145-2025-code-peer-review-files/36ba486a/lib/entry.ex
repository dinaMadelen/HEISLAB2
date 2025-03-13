defmodule Entry do
  use Application
  require Logger

  def start(_type, _args) do
    n_elevators = Application.fetch_env!(:elevator, :n_elevators)
    m_floors = Application.fetch_env!(:elevator, :m_floors)
    Logger.notice("Starting #{n_elevators} elevators for #{m_floors} floors")

    topologies = [
      example: [
        strategy: Cluster.Strategy.Epmd,
        config: [hosts: [:"node@10.100.23.11", :"node@10.100.23.12", :"node@10.100.23.18"]],
      ]
    ]
    children = [
      {Cluster.Supervisor, [topologies, [name: ClusterSupervisor]]},
      {Driver, [address: {127, 0, 0, 1}, port: 1234]},
      {Poller, []},
      {Requests, []},
      {Controller, []}
    ]
    Supervisor.start_link(children, strategy: :one_for_one, name: Supervisor)
  end
end
