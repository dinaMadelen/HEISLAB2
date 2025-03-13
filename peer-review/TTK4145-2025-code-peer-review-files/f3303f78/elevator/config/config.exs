import Config

config :elevator,
  # Application configuration
  env: Mix.env(),

  # Elevator configuration
  elevator_name: :elevator_fsm,
  top_floor: 3,
  bottom_floor: 0,
  door_open_time: 3000,
  between_floor_time: 4000,
  order_pool_rate: 1250,
  response_timeout: 200,

  # Network configuration
  node_name: :g99elevator,
  cookie: :g99cookie,
  broadcast_port: 20099,
  node_tick_time: 60000,
  broadcast_interval: 1000,

  # OrderManager/OrderSupervisor configuration
  call_duration: 1000,
  long_call_duration: 3000,

  # Floor poller configuration
  floor_detector_poll_rate: 100,

  # Button poller configuration
  button_poll_rate: 100
