defmodule OrderSupervisor.CoreTest do
  @moduledoc """
  Tests for OrderSupervisor.Core functions
  """
  use ExUnit.Case

  @type supervision_map :: OrderSupervisor.t()

  setup do
    pid1 = spawn(fn -> :ok end)
    pid2 = spawn(fn -> :ok end)
    pid3 = spawn(fn -> :ok end)
    order1 = %Order{floor: 1, type: :cab}
    order2 = %Order{floor: 2, type: :hall_up}
    order3 = %Order{floor: 3, type: :hall_down}
    node1 = :node1
    node2 = :node2
    supervision_map = %{{order1, node1} => pid1, {order2, node1} => pid2, {order3, node2} => pid3}
    {:ok, supervision_map: supervision_map, pid1: pid1, pid2: pid2, pid3: pid3}
  end

  describe "get_pids_for_node/2" do
    test "get pids for node with mixed orders", %{
      supervision_map: supervision_map,
      pid1: pid1,
      pid2: pid2
    } do
      node = :node1
      assert OrderSupervisor.Core.get_pids_for_node(supervision_map, node) == [pid1, pid2]
    end

    test "get pids for node with single order", %{supervision_map: supervision_map, pid3: pid3} do
      node = :node2
      assert OrderSupervisor.Core.get_pids_for_node(supervision_map, node) == [pid3]
    end

    test "get pids for node not in map", %{supervision_map: supervision_map} do
      node = :node4
      assert OrderSupervisor.Core.get_pids_for_node(supervision_map, node) == []
    end
  end
end
