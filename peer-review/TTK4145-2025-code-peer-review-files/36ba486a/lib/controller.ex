defmodule Controller do
  use GenServer
  import FSM
  import Util
  require Logger

  def start_link(_fsm) do GenServer.start_link(__MODULE__, [], name: __MODULE__) end

  def init(_fsm) do
    GenServer.cast(Driver, {:set_motor_direction, :down})
    initial_state = %{
      :global_hall_requests => [[false,false],[false,false],[false,false],[false,false]],
      Node.self() => %{
        behaviour: :init, # moves elevator to known floor
        floor_integer: nil, # always an int after :init has finished
        direction: :down,
        last_direction: :down,
        assigned_hall_requests: [[false,false],[false,false],[false,false],[false,false]],
        cab_requests: [false,false,false,false],
        last_time: :erlang.monotonic_time(:millisecond),
        door_timeout_ms: Application.fetch_env!(:elevator, :door_timeout_ms)
      }
    }
    {:ok, initial_state}
  end


  def handle_cast({:propagate_state, other_state}, state) do
    node = elem(other_state, 0)
    node_fsm = elem(other_state, 1)
    other_global_hall_requests = elem(other_state, 2)
    
    #Puts the given value under key in map.
    state = Map.replace!(state, :global_hall_requests,
      boolean_array_or(Map.get(state, :global_hall_requests), other_global_hall_requests))
    state = Map.put(state, node, node_fsm)

    {:noreply, state}
  end

  def handle_cast({:clear_hall_request, floor, direction}, state) do
    updated_global_hall_orders = List.update_at(state.global_hall_requests, floor, fn row ->
      List.update_at(row, direction, fn _ -> false end)
    end)
    
    state = Map.replace!(state, :global_hall_requests, updated_global_hall_orders)
    {:noreply, state}
  end
  
  def handle_cast({:assign_requests, requests}, state) do
    fsm = Map.get(state, Node.self())
    fsm = Map.replace!(fsm, :assigned_hall_requests, requests)
    state = Map.replace!(state, Node.self(), fsm)
    {:noreply, state}
  end
  
  def handle_cast({:send, [polled_floor, polled_hall_requests, polled_cab_requests, _polled_stop, _polled_obstruct]}, state) do 
      for node <- Node.list() do
        try do
          GenServer.cast({Controller, node}, {:propagate_state,
            {Node.self(), Map.get(state, Node.self()), Map.get(state, :global_hall_requests)}
          })

        catch
          :exit, reason -> 
          IO.puts("Failed to call #{node}: #{inspect(reason)}")
          :error
        end
      end

      if leader?() do
        IO.puts("i am leader")
        fsm = Map.get(state, Node.self())
        if(fsm.behaviour != :init) do 
          computed_requests = GenServer.call(Requests, {:compute, state})
          for {node, hall_requests} <- computed_requests do
            # IO.inspect(hall_requests)
            # IO.inspect(node)
            GenServer.cast({Controller, String.to_atom(node)}, {:assign_requests, hall_requests})
          end
          IO.inspect(computed_requests)
          IO.inspect(Map.get(state, :global_hall_requests))
        end

      else IO.puts("worker") end

      state = update_fsm_with_polled_info(state, polled_floor, polled_hall_requests, polled_cab_requests)
      fsm = Map.get(state, Node.self())
      fsm = case {
        fsm.behaviour,
        is_integer(polled_floor),
        :erlang.monotonic_time(:millisecond) - fsm.last_time > fsm.door_timeout_ms,
        should_stop?(fsm, polled_floor)} do

        {:init    , true, _   , _   } -> Logger.info("finish init")
          arrive_at_floor(fsm, go_idle=true)
        {:idle    , true, _   , _   } -> Logger.info("idle")
          consume_order_from_idle(fsm)
        {:moving  , true, _   , true} -> Logger.info("open door")
          arrive_at_floor(fsm, go_idle=false)
        {:doorOpen, true, true, _   } -> Logger.info("closing door")
          arrive_at_floor(fsm, go_idle=true) 
        _ -> fsm
      end

      update_lights(state)
      state = Map.replace(state, Node.self(), fsm)
      # IO.inspect(state)

    {:noreply, state}
  end

  def leader? do
    nodes = [Node.self() | Node.list()]
    
    nodes
    |> Enum.map(&extract_ip/1)
    |> Enum.min()
    |> Kernel.==(extract_ip(Node.self()))
  end

  defp extract_ip(node) do
      node
      |> Atom.to_string()
      |> String.split("@")
      |> List.last()
      |> String.split(".")
      |> Enum.map(&String.to_integer/1)
      |> List.to_tuple()
  end
end
