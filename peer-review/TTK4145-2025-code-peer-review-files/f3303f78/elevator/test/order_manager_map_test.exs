defmodule OrderManager.CoreMapTest do
  @moduledoc """
  Tests for OrderManager.Core functions for order_map manipulation
  """
  use ExUnit.Case

  setup do
    order1 = %Order{floor: 1, type: :cab}
    order2 = %Order{floor: 2, type: :hall_up}

    order_map = %{
      node1: [order1],
      node2: [order2, order1]
    }

    {:ok, order1: order1, order2: order2, order_map: order_map}
  end

  describe "add_order/3" do
    test "add_order/3 adds a new order to the order map", %{
      order1: order1,
      order2: order2,
      order_map: order_map
    } do
      new_order = %Order{floor: 1, type: :hall_down}
      updated_order_map = OrderManager.Core.add_order(order_map, new_order, :node1)
      assert updated_order_map[:node1] == [new_order, order1]
      assert updated_order_map[:node2] == [order2, order1]
    end

    test "add_order/3 adds a new order to a node that already has multiple order", %{
      order1: order1,
      order2: order2,
      order_map: order_map
    } do
      new_order = %Order{floor: 2, type: :hall_down}
      updated_order_map = OrderManager.Core.add_order(order_map, new_order, :node2)
      assert updated_order_map[:node1] == [order1]
      assert updated_order_map[:node2] == [new_order, order2, order1]
    end

    test "add_order/3 does not add an existing order to the order map", %{
      order1: order1,
      order2: order2,
      order_map: order_map
    } do
      updated_order_map = OrderManager.Core.add_order(order_map, order1, :node1)
      assert updated_order_map[:node1] == [order1]
      assert updated_order_map[:node2] == [order2, order1]
    end

    test "add_order/3 adds an order to node that wasn't in the map", %{
      order1: order1,
      order2: order2,
      order_map: order_map
    } do
      new_order = %Order{floor: 3, type: :hall_down}
      updated_order_map = OrderManager.Core.add_order(order_map, new_order, :node3)
      assert updated_order_map[:node3] == [new_order]
      assert updated_order_map[:node1] == [order1]
      assert updated_order_map[:node2] == [order2, order1]
    end
  end

  describe "remove_order/3" do
    test "remove_order/3 removes an order from the order map", %{
      order1: order1,
      order2: order2,
      order_map: order_map
    } do
      updated_order_map = OrderManager.Core.remove_order(order_map, order1, :node1)
      assert updated_order_map[:node1] == []
      assert updated_order_map[:node2] == [order2, order1]
    end

    test "remove_order/3 removes an order from the order map 2", %{
      order1: order1,
      order2: order2,
      order_map: order_map
    } do
      updated_order_map = OrderManager.Core.remove_order(order_map, order1, :node2)
      assert updated_order_map[:node1] == [order1]
      assert updated_order_map[:node2] == [order2]
    end

    test "remove_order/3 executes without error when removing a non-existent order", %{
      order1: order1,
      order2: order2,
      order_map: order_map
    } do
      non_existent_order = %Order{floor: 3, type: :hall_down}
      updated_order_map = OrderManager.Core.remove_order(order_map, non_existent_order, :node1)
      assert updated_order_map[:node1] == [order1]
      assert updated_order_map[:node2] == [order2, order1]
    end
  end

  describe "merge_orders/2" do
    test "Test two order_maps with no duplicates" do
      order_map1 = %{
        node1: [%Order{floor: 1, type: :cab}],
        node2: [%Order{floor: 2, type: :hall_up}]
      }

      order_map2 = %{
        node1: [%Order{floor: 3, type: :hall_down}],
        node2: [%Order{floor: 4, type: :cab}]
      }

      expected_order_map = %{
        node1: [%Order{floor: 1, type: :cab}, %Order{floor: 3, type: :hall_down}],
        node2: [%Order{floor: 2, type: :hall_up}, %Order{floor: 4, type: :cab}]
      }

      assert OrderManager.Core.merge_orders(order_map1, order_map2) == expected_order_map
    end

    test "Test two order_maps with duplicates" do
      order_map1 = %{
        node1: [%Order{floor: 1, type: :cab}],
        node2: [%Order{floor: 2, type: :hall_up}]
      }

      order_map2 = %{
        node1: [%Order{floor: 2, type: :hall_up}],
        node2: [%Order{floor: 2, type: :hall_up}]
      }

      expected_order_map = %{
        node1: [%Order{floor: 1, type: :cab}, %Order{floor: 2, type: :hall_up}],
        node2: [%Order{floor: 2, type: :hall_up}]
      }

      assert OrderManager.Core.merge_orders(order_map1, order_map2) == expected_order_map
    end

    test "Test with mismatched nodes in maps" do
      order_map1 = %{
        node1: [%Order{floor: 1, type: :cab}],
        node2: [%Order{floor: 2, type: :hall_up}]
      }

      order_map2 = %{
        node1: [%Order{floor: 2, type: :hall_up}],
        node3: [%Order{floor: 2, type: :hall_up}]
      }

      expected_order_map = %{
        node1: [%Order{floor: 1, type: :cab}, %Order{floor: 2, type: :hall_up}],
        node2: [%Order{floor: 2, type: :hall_up}],
        node3: [%Order{floor: 2, type: :hall_up}]
      }

      assert OrderManager.Core.merge_orders(order_map1, order_map2) == expected_order_map
    end

    test "Test with empty maps" do
      order_map1 = %{}
      order_map2 = %{}

      assert OrderManager.Core.merge_orders(order_map1, order_map2) == %{}
    end
  end

  describe "order_exists?/2" do
    test "order_exists?/2 returns true if the order exists in the order map", %{
      order1: order1,
      order_map: order_map
    } do
      assert OrderManager.Core.order_exists?(order_map, order1) == true
    end

    test "order_exists?/2 returns false if the order does not exist in the order map", %{
      order_map: order_map
    } do
      non_existent_order = %Order{floor: 3, type: :hall_down}
      assert OrderManager.Core.order_exists?(order_map, non_existent_order) == false
    end
  end
end
