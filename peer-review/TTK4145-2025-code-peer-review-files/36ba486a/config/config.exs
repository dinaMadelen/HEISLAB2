import Config

config :logger, :console,
 format: "[$level] [$metadata] $message\n",
 metadata: [:error_code, :mfa]

config :elevator,
  n_elevators: 1,
  m_floors: 3,
  poller_timeout_ms: 150,
  door_timeout_ms: 3000
