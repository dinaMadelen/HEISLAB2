defmodule OrderCost.CoreTest do
  @moduledoc """
  Tests for OrderCost.Core functions
  """
  use ExUnit.Case

  @door_open_time Application.compile_env!(:elevator, :door_open_time)
  @between_floor_time Application.compile_env!(:elevator, :between_floor_time)

  setup do
    order1 = %Order{floor: 0, type: :cab}
    order2 = %Order{floor: 2, type: :hall_up}
    order3 = %Order{floor: 2, type: :hall_down}
    order4 = %Order{floor: 3, type: :cab}
    order5 = %Order{floor: 1, type: :hall_down}

    orders = %{
      node1: [order1, order2],
      node2: [order3],
      node3: [order4],
      node4: [order5]
    }

    synced_states = [
      {:node1, %{available: true, reported_floor: 0, direction: :up, state: :idle}},
      {:node2, %{available: false, reported_floor: 1, direction: :down, state: :idle}},
      {:node3, %{available: true, reported_floor: 2, direction: :up, state: :idle}},
      {:node4, %{available: true, reported_floor: 3, direction: :down, state: :idle}}
    ]

    {:ok, orders: orders, synced_states: synced_states}
  end

  describe "get_cheapest_node/4" do
    test "get_cheapest_node with mixed orders", %{orders: orders, synced_states: synced_states} do
      new_order = %Order{floor: 1, type: :hall_down}
      assert OrderCost.Core.get_cheapest_node(synced_states, orders, new_order, :node7) == :node4
    end

    test "get_cheapest_node with mixed orders 2", %{orders: orders, synced_states: synced_states} do
      new_order = %Order{floor: 3, type: :hall_up}
      assert OrderCost.Core.get_cheapest_node(synced_states, orders, new_order, :node7) == :node3
    end

    test "get_cheapest_node with all nodes having empty orders", %{synced_states: synced_states} do
      orders = %{
        node1: [],
        node2: [],
        node3: [],
        node4: []
      }

      new_order = %Order{floor: 1, type: :hall_down}
      assert OrderCost.Core.get_cheapest_node(synced_states, orders, new_order, :node7) == :node1
    end

    test "get_cheapest_node with all nodes having empty orders, 2" do
      orders = %{
        node1: [],
        node2: [],
        node3: [],
        node4: []
      }

      new_order = %Order{floor: 1, type: :hall_up}

      synced_states = [
        {:node2, %{available: true, reported_floor: 1, direction: :down, state: :idle}},
        {:node3, %{available: true, reported_floor: 2, direction: :up, state: :idle}},
        {:node4, %{available: true, reported_floor: 3, direction: :down, state: :idle}},
        {:node1, %{available: true, reported_floor: 0, direction: :up, state: :idle}}
      ]

      assert OrderCost.Core.get_cheapest_node(synced_states, orders, new_order, :node7) == :node2
    end

    test "get_cheapest_node with all nodes having same orders", %{synced_states: synced_states} do
      orders = %{
        node1: [%Order{floor: 3, type: :hall_down}],
        node2: [%Order{floor: 3, type: :hall_down}],
        node3: [%Order{floor: 3, type: :hall_down}],
        node4: [%Order{floor: 3, type: :hall_down}]
      }

      new_order = %Order{floor: 2, type: :hall_down}
      assert OrderCost.Core.get_cheapest_node(synced_states, orders, new_order, :node7) == :node4
    end

    test "get_cheapest_node with no available nodes", %{orders: orders} do
      synced_states = [
        {:node1, %{available: false, reported_floor: 0, direction: :up, state: :idle}},
        {:node2, %{available: false, reported_floor: 1, direction: :down, state: :idle}},
        {:node3, %{available: false, reported_floor: 2, direction: :up, state: :idle}},
        {:node4, %{available: false, reported_floor: 3, direction: :down, state: :idle}}
      ]

      new_order = %Order{floor: 2, type: :hall_down}

      assert OrderCost.Core.get_cheapest_node(synced_states, orders, new_order, :node2) == :node2
    end
  end

  describe "get_cheapest_nodes/2" do
    test "get fallback node when empty list" do
      assert OrderCost.Core.get_cheapest_nodes([], :node1) == [{:node1, 0}]
    end

    test "get cheapest node from mixed list" do
      node_list = [
        {:node1, 1},
        {:node2, 2},
        {:node3, 1},
        {:node4, 3}
      ]

      assert OrderCost.Core.get_cheapest_nodes(node_list, :node7) == [{:node1, 1}, {:node3, 1}]
    end
  end

  describe "order_cost/2" do
    test "caluclates correctly order cost with floor differential" do
      assert OrderCost.Core.order_cost(2, -1) == @door_open_time + @between_floor_time * 3
    end

    test "caluclates correctly order cost with no floor differential" do
      assert OrderCost.Core.order_cost(1, 1) == @door_open_time
    end
  end

  describe "get_order_costs/2" do
    test "test with all nodes having one order each", %{
      orders: orders,
      synced_states: synced_states
    } do
      expected_costs = [
        {:node1, @door_open_time * 2 + @between_floor_time * 2},
        {:node2, @door_open_time + @between_floor_time},
        {:node3, @door_open_time + @between_floor_time},
        {:node4, @door_open_time + @between_floor_time * 2}
      ]

      assert OrderCost.Core.get_order_costs(synced_states, orders) == expected_costs
    end

    test "test with all nodes having empty orders", %{synced_states: synced_states} do
      orders = %{
        node1: [],
        node2: [],
        node3: [],
        node4: []
      }

      expected_costs = [
        {:node1, 0},
        {:node2, 0},
        {:node3, 0},
        {:node4, 0}
      ]

      assert OrderCost.Core.get_order_costs(synced_states, orders) == expected_costs
    end

    test "test with nodes missing from order map", %{synced_states: synced_states} do
      orders = %{
        node1: [],
        node2: [],
        node3: []
      }

      expected_costs = [
        {:node1, 0},
        {:node2, 0},
        {:node3, 0},
        {:node4, 0}
      ]

      assert OrderCost.Core.get_order_costs(synced_states, orders) == expected_costs
    end

    test "test with node missing from synced states", %{orders: orders} do
      synced_states = [
        {:node1, %{available: true, reported_floor: 0, direction: :up, state: :idle}},
        {:node2, %{available: true, reported_floor: 1, direction: :down, state: :idle}},
        {:node3, %{available: true, reported_floor: 2, direction: :up, state: :idle}}
      ]

      expected_costs = [
        {:node1, @door_open_time * 2 + @between_floor_time * 2},
        {:node2, @door_open_time + @between_floor_time},
        {:node3, @door_open_time + @between_floor_time}
      ]

      assert OrderCost.Core.get_order_costs(synced_states, orders) == expected_costs
    end
  end

  describe "get_node_order_cost/4" do
    test "test with orders at different floors" do
      orders = %{
        node1: [
          %Order{floor: 0, type: :cab},
          %Order{floor: 2, type: :hall_up},
          %Order{floor: 1, type: :hall_down}
        ]
      }

      expected_door_open_num = 3
      expected_floors_traveled = 3

      assert OrderCost.Core.get_node_order_cost(orders, :node1, 0, :up, 0) ==
               @door_open_time * expected_door_open_num +
                 @between_floor_time * expected_floors_traveled
    end

    test "test with duplicate cab order on floor" do
      orders = %{
        node1: [
          %Order{floor: 0, type: :cab},
          %Order{floor: 2, type: :hall_up},
          %Order{floor: 2, type: :cab}
        ]
      }

      expected_door_open_num = 2
      expected_floors_traveled = 2

      assert OrderCost.Core.get_node_order_cost(orders, :node1, 0, :up, 0) ==
               @door_open_time * expected_door_open_num +
                 @between_floor_time * expected_floors_traveled
    end

    test "test with triple duplicate order on floor" do
      orders = %{
        node1: [
          %Order{floor: 0, type: :cab},
          %Order{floor: 2, type: :hall_up},
          %Order{floor: 2, type: :cab},
          %Order{floor: 2, type: :hall_down}
        ]
      }

      expected_door_open_num = 3
      expected_floors_traveled = 2

      assert OrderCost.Core.get_node_order_cost(orders, :node1, 0, :up, 0) ==
               @door_open_time * expected_door_open_num +
                 @between_floor_time * expected_floors_traveled
    end

    test "test with triple duplicate order on floor, and order on floor above" do
      orders = %{
        node1: [
          %Order{floor: 0, type: :cab},
          %Order{floor: 2, type: :hall_up},
          %Order{floor: 2, type: :cab},
          %Order{floor: 2, type: :hall_down},
          %Order{floor: 3, type: :cab}
        ]
      }

      expected_door_open_num = 4
      expected_floors_traveled = 4

      assert OrderCost.Core.get_node_order_cost(orders, :node1, 0, :up, 0) ==
               @door_open_time * expected_door_open_num +
                 @between_floor_time * expected_floors_traveled
    end

    test "test with node missing from from order map" do
      orders = %{
        node2: [
          %Order{floor: 0, type: :cab}
        ]
      }

      expected_door_open_num = 0
      expected_floors_traveled = 0

      assert OrderCost.Core.get_node_order_cost(orders, :node1, 0, :up, 0) ==
               @door_open_time * expected_door_open_num +
                 @between_floor_time * expected_floors_traveled
    end
  end

  describe "skip_same_floor_cab_order/4" do
    test "Check the function doesn't skip floor when it shouldn't" do
      order0 = %Order{floor: 0, type: :hall_up}
      order1 = %Order{floor: 1, type: :hall_up}
      order2 = %Order{floor: 1, type: :cab}
      order3 = %Order{floor: 2, type: :hall_down}

      orders = %{
        node1: [order1, order2, order3]
      }

      direction = :up

      {new_orders, new_direction} =
        OrderCost.Core.skip_same_floor_cab_order(orders, order0, :node1, direction)

      assert orders == new_orders
      assert direction == new_direction
    end

    test "Check the function skips hall_up order when it should" do
      order0 = %Order{floor: 1, type: :hall_up}
      order1 = %Order{floor: 1, type: :cab}
      order2 = %Order{floor: 2, type: :hall_down}
      orders = %{node1: [order1, order2]}
      direction = :up
      expected_orders = %{node1: [order2]}
      expected_direction = :up

      {new_orders, new_direction} =
        OrderCost.Core.skip_same_floor_cab_order(orders, order0, :node1, direction)

      assert expected_orders == new_orders
      assert expected_direction == new_direction
    end

    test "Check the function skips cab order when it should" do
      order0 = %Order{floor: 2, type: :cab}
      order1 = %Order{floor: 0, type: :hall_up}
      order2 = %Order{floor: 2, type: :hall_down}
      orders = %{node1: [order1, order2]}
      direction = :up
      expected_orders = %{node1: [order1]}
      expected_direction = :down

      {new_orders, new_direction} =
        OrderCost.Core.skip_same_floor_cab_order(orders, order0, :node1, direction)

      assert expected_orders == new_orders
      assert expected_direction == new_direction
    end

    test "Skip order on same floor when triple orders on same floor." do
      order0 = %Order{floor: 1, type: :hall_up}
      order1 = %Order{floor: 1, type: :hall_down}
      order2 = %Order{floor: 1, type: :cab}
      orders = %{node1: [order1, order2]}
      direction = :up
      expected_orders = %{node1: [order1]}
      expected_direction = :up

      {new_orders, new_direction} =
        OrderCost.Core.skip_same_floor_cab_order(orders, order0, :node1, direction)

      assert expected_orders == new_orders
      assert expected_direction == new_direction
    end
  end

  describe "order_to_direction/2" do
    test ":hall_up" do
      assert OrderCost.Core.order_to_direction(:hall_up, :ignore) == :up
    end

    test ":hall_down" do
      assert OrderCost.Core.order_to_direction(:hall_down, :ignore) == :down
    end

    test ":cab up" do
      assert OrderCost.Core.order_to_direction(:cab, :up) == :up
    end

    test ":cab down" do
      assert OrderCost.Core.order_to_direction(:cab, :down) == :down
    end
  end

  describe "update_floor/1" do
    test "test with state being :idle", %{synced_states: synced_states} do
      synced_states
      |> Enum.each(fn {_, state} ->
        assert OrderCost.Core.update_floor(state) == state.reported_floor
      end)
    end

    test "test with state being :moving" do
      negative_up = %{available: true, reported_floor: -2, direction: :up, state: :moving}
      positive_up = %{available: true, reported_floor: 2, direction: :up, state: :moving}
      negative_down = %{available: true, reported_floor: -2, direction: :down, state: :moving}
      positive_down = %{available: true, reported_floor: 2, direction: :down, state: :moving}
      assert OrderCost.Core.update_floor(negative_up) == -1
      assert OrderCost.Core.update_floor(positive_up) == 3
      assert OrderCost.Core.update_floor(negative_down) == -3
      assert OrderCost.Core.update_floor(positive_down) == 1
    end
  end

  describe "adjust_elevator_floors/1" do
    test "test with all nodes being idle", %{synced_states: synced_states} do
      updated_states = OrderCost.Core.adjust_elevator_floors(synced_states)
      assert synced_states == updated_states
    end

    test "test with all nodes being :moving" do
      synced_states = [
        {:node, %{available: true, reported_floor: -2, direction: :up, state: :moving}},
        {:node, %{available: true, reported_floor: 2, direction: :up, state: :moving}},
        {:node, %{available: true, reported_floor: -2, direction: :down, state: :moving}},
        {:node, %{available: true, reported_floor: 2, direction: :down, state: :moving}}
      ]

      updated_states = OrderCost.Core.adjust_elevator_floors(synced_states)

      assert elem(Enum.fetch!(updated_states, 0), 1).reported_floor == -1
      assert elem(Enum.fetch!(updated_states, 1), 1).reported_floor == 3
      assert elem(Enum.fetch!(updated_states, 2), 1).reported_floor == -3
      assert elem(Enum.fetch!(updated_states, 3), 1).reported_floor == 1
    end
  end
end
