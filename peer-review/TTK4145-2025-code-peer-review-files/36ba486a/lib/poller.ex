defmodule Poller do
  use GenServer

  def start_link(_state) do GenServer.start_link(__MODULE__, [], name: __MODULE__) end

  def init(_state) do
    loop()
    {:ok, nil}
  end

  def handle_info(:poll, state) do
    m_floors = Application.fetch_env!(:elevator, :m_floors)
    hall_requests = for floor <- 0..m_floors do [
      GenServer.call(Driver, {:get_order_button_state, floor, :hall_up}) == 1,
      GenServer.call(Driver, {:get_order_button_state, floor, :hall_down}) == 1] end

    cab_requests = for floor <- 0..m_floors do
      GenServer.call(Driver, {:get_order_button_state, floor, :cab}) == 1 end

    floor = GenServer.call(Driver, :get_floor_sensor_state)
    stop = GenServer.call(Driver, :get_stop_button_state)
    obstruct = GenServer.call(Driver, :get_obstruction_switch_state)
    GenServer.cast(Controller, {:send, [floor, hall_requests, cab_requests, stop, obstruct]})

    loop()
    {:noreply, state}
  end

  defp loop() do
    Process.send_after(self(), :poll, Application.fetch_env!(:elevator, :poller_timeout_ms))
  end
end
