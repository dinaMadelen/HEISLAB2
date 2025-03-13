defmodule Requests do 
  use GenServer
  require Logger

  def start_link(_state) do GenServer.start_link(__MODULE__, [], name: __MODULE__) end

  def init(_state) do
    {:ok, nil}
  end

  def handle_call({:compute, state} , _from, _state) do

    valid_nodes = Enum.filter([Node.self() | Node.list()], fn node ->
      node_fsm = Map.get(state, node)
      if node_fsm do Map.get(node_fsm, :behaviour) != :init else false end
    end)
    # IO.inspect(valid_nodes)

    tmp = %{}
    tmp = Enum.reduce(valid_nodes, tmp, fn node, accumulator ->
      fsm = Map.get(state, node)
      Map.put(accumulator, node, %{
        "behaviour" => Atom.to_string(Map.get(fsm, :behaviour)),
        "floor" => Map.get(fsm, :floor_integer),
        "direction" => Atom.to_string(Map.get(fsm, :direction)),
        "cabRequests" => Map.get(fsm, :cab_requests),
        })
    end)

    
    input = %{
        "hallRequests" => Map.get(state, :global_hall_requests),
        "states" =>  tmp
    }
    # IO.inspect(input)
    elev_state = Jason.encode!(input)
    response = System.cmd("hall_request_assigner", ["--input", elev_state])
    string = elem(response, 0)
    map = Jason.decode!(string)

    {:reply, map, nil}
  end
end
