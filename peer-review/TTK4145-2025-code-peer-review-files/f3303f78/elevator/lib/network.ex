defmodule Network do
  @moduledoc """
  This module contains the shell for the network module. It is responsible for booting the local
  Node, starting the broadcast and discovery of nodes in the network.
  """
  alias Network.Core, as: Core

  # Types and structs  ----------------------------------------------------------------------------
  @type ip4_address :: {0..255, 0..255, 0..255, 0..255}

  # Config constants  -----------------------------------------------------------------------------
  @cookie Application.compile_env!(:elevator, :cookie)
  @node_name Application.compile_env!(:elevator, :node_name)
  @broadcast_port Application.compile_env!(:elevator, :broadcast_port)
  @node_tick_time Application.compile_env!(:elevator, :node_tick_time)
  @broadcast_interval Application.compile_env!(:elevator, :broadcast_interval)

  # Module init  ----------------------------------------------------------------------------------
  @doc """
  Starts the network module links it to Supervisor.
  """
  @spec start_link() :: {:ok, pid}
  def start_link() do
    case start_epmd() do
      {:enoent, reason} ->
        IO.puts("\e[31m#{reason}\e[0m")
        {:error, {:shutdown, reason}}

      :ok ->
        {:ok, self_node_name} = boot_node(@node_name, @node_tick_time)
        pid = spawn_link(fn -> create_node_network(self_node_name) end)
        {:ok, pid}
    end
  end

  @doc """
  Child specification required by supervisor.
  """
  @spec child_spec(any) :: map
  def child_spec(_) do
    %{id: __MODULE__, start: {__MODULE__, :start_link, []}, restart: :permanent, type: :worker}
  end

  # Internal functions  ---------------------------------------------------------------------------
  @doc """
  This function is responsible for opening a UDP socket, spawning the broadcast_node function and starting to listen
  for messages sent to the socket.
  """
  @spec create_node_network(String.t()) :: no_return
  def create_node_network(self_node_name) do
    {:ok, socket} = :gen_udp.open(@broadcast_port, [:binary, active: true, broadcast: true])
    spawn_link(fn -> broadcast_node(socket, self_node_name) end)
    my_ip = get_my_ip()
    receive_nodes(socket, my_ip)
  end

  @doc """
  This function listens to messages received and initialise a connection to the node messages received.
  """
  @spec receive_nodes(:inet.socket(), ip4_address) :: no_return
  def receive_nodes(socket, my_ip) do
    receive do
      {:udp, _socket, ^my_ip, _port, _node_name} ->
        :ok

      {:udp, _socket, ip, _port, node_name} ->
        if not Core.node_connected?(node_name, Node.list()) do
          case Node.connect(String.to_atom(node_name)) do
            true ->
              IO.puts("Connected to node: #{node_name} from #{Core.ip_to_string(ip)}")
              OrderManager.sync_orders()

            false ->
              IO.puts("Failed to connect to node: #{node_name}")

              # Node.connect can also return :ignore if local Node is not alive, we want this failure to propagate
          end
        end

      _ ->
        :ignore
    end

    receive_nodes(socket, my_ip)
  end

  @doc """
  This function is responsible for broadcasting the node name to the local network.
  """
  @spec broadcast_node(:inet.socket(), String.t()) :: no_return
  def broadcast_node(socket, node_name) do
    case :gen_udp.send(socket, {255, 255, 255, 255}, @broadcast_port, node_name) do
      :ok -> :ok
      {:error, reason} -> IO.puts("Error sending broadcast: #{reason}")
    end

    :timer.sleep(@broadcast_interval)
    broadcast_node(socket, node_name)
  end

  @doc """
  Starts the Erlang Port Mapper Daemon (epmd) if it is not already running.
  """
  @spec start_epmd() :: :ok | {:enoent, String.t()}
  def start_epmd() do
    try do
      System.cmd("epmd", ["-daemon"])
      :ok
    catch
      :error, :enoent ->
        {:enoent, "epmd not found, make sure Erlang is installed and configured correctly."}
    end
  end

  @doc """
  Taken from: https://github.com/jostlowe/kokeplata/blob/master/lib/networkstuff.ex
  Returns (hopefully) the ip address of your network interface.
  """
  @spec get_my_ip() :: ip4_address | {:error, atom}
  def get_my_ip() do
    {:ok, socket} = :gen_udp.open(6789, active: false, broadcast: true)
    :ok = :gen_udp.send(socket, {255, 255, 255, 255}, 6789, "test packet")

    ip =
      case :gen_udp.recv(socket, 100, 1000) do
        {:ok, {ip, _port, _data}} -> ip
        {:error, _} -> {:error, :could_not_get_ip}
      end

    :gen_udp.close(socket)
    ip
  end

  @doc """
  Taken and modified from: https://github.com/jostlowe/kokeplata/blob/master/lib/networkstuff.ex
  Boots a node with a specified tick time. node_name sets the node name before @. The IP-address is
  automatically imported.
  """
  @spec boot_node(atom, integer) :: {:ok, String.t()}
  def boot_node(node_name, tick_time \\ 15000) do
    ip = get_my_ip() |> Core.ip_to_string()
    full_name = Atom.to_string(node_name) <> "@" <> ip
    IO.puts("Booting node: #{full_name}")

    case Node.start(String.to_atom(full_name), :longnames, tick_time) do
      {:ok, _} ->
        IO.puts("Node successfully booted: #{full_name}")
        Node.set_cookie(@cookie)
        {:ok, full_name}

      {:error, {:already_started, _}} ->
        IO.puts("Node already booted: #{Node.self()}")
        {:ok, full_name}
    end
  end

  @doc """
  Taken and modified from: https://github.com/jostlowe/kokeplata/blob/master/lib/networkstuff.ex
  Returns all nodes in the current cluster.
  """
  @spec all_nodes() :: [atom]
  def all_nodes() do
    [Node.self() | Node.list()]
  end
end

defmodule Network.Core do
  @moduledoc """
  This module contains the core functionality of the network module, stateless.
  """
  # Types and structs  ----------------------------------------------------------------------------
  @type ip4_address :: {0..255, 0..255, 0..255, 0..255}
  @type ip6_address ::
          {0..65535, 0..65535, 0..65535, 0..65535, 0..65535, 0..65535, 0..65535, 0..65535}
  @type ip_address :: ip4_address | ip6_address

  # API functions  --------------------------------------------------------------------------------
  @doc """
  Taken from: https://github.com/jostlowe/kokeplata/blob/master/lib/networkstuff.ex
  Formats an ip address on tuple format to a bytestring
  """
  @spec ip_to_string(ip_address) :: String.t()
  def ip_to_string(ip) do
    :inet.ntoa(ip) |> to_string()
  end

  @doc """
  Checks if a node name is already in a list of nodes
  """
  @spec node_connected?(String.t(), [atom]) :: boolean
  def node_connected?(node_name, node_list) do
    node_name in Enum.map(node_list, &Atom.to_string/1)
  end
end
